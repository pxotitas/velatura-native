//! Niri event-stream process supervisor.
//!
//! Starts immediately and reconnects after exactly 3000ms only when the
//! child exits non-zero, matching the lifecycle policy in NiriService.qml.

use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

const RECONNECT_DELAY: Duration = Duration::from_millis(3000);
const POLL_INTERVAL: Duration = Duration::from_millis(40);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NiriProcessEvent {
    StdoutLine(String),
    Stderr(String),
    Exited { code: Option<i32>, success: bool },
    SpawnFailed(String),
}

pub struct NiriEventStream {
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl NiriEventStream {
    pub fn spawn() -> (Self, Receiver<NiriProcessEvent>) {
        Self::with_program("niri")
    }

    pub fn with_program(program: impl Into<String>) -> (Self, Receiver<NiriProcessEvent>) {
        let program = program.into();
        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop);
        let (sender, receiver) = mpsc::channel();
        let worker = thread::Builder::new()
            .name("velatura-niri-event-stream".into())
            .spawn(move || supervise(program, sender, worker_stop))
            .expect("failed to start Niri event stream supervisor");
        (
            Self {
                stop,
                worker: Some(worker),
            },
            receiver,
        )
    }

    pub fn stop(&self) {
        self.stop.store(true, Ordering::Release);
    }

    pub fn is_stopping(&self) -> bool {
        self.stop.load(Ordering::Acquire)
    }
}

impl Drop for NiriEventStream {
    fn drop(&mut self) {
        self.stop();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn supervise(program: String, sender: Sender<NiriProcessEvent>, stop: Arc<AtomicBool>) {
    while !stop.load(Ordering::Acquire) {
        let mut child = match Command::new(&program)
            .args(["msg", "--json", "event-stream"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(error) => {
                let _ = sender.send(NiriProcessEvent::SpawnFailed(error.to_string()));
                break;
            }
        };

        let stdout = child.stdout.take().expect("piped stdout");
        let stderr = child.stderr.take().expect("piped stderr");
        let (reader_tx, reader_rx) = mpsc::channel();
        let out_tx = reader_tx.clone();
        let stdout_reader = thread::Builder::new()
            .name("velatura-niri-stdout".into())
            .spawn(move || {
                for line in BufReader::new(stdout).lines() {
                    match line {
                        Ok(line) => {
                            if out_tx.send(ReaderMessage::Stdout(line)).is_err() {
                                break;
                            }
                        }
                        Err(error) => {
                            let _ = out_tx.send(ReaderMessage::ReadError(error.to_string()));
                            break;
                        }
                    }
                }
                let _ = out_tx.send(ReaderMessage::StdoutClosed);
            });
        let err_tx = reader_tx.clone();
        let stderr_reader = thread::Builder::new()
            .name("velatura-niri-stderr".into())
            .spawn(move || {
                let mut reader = BufReader::new(stderr);
                let mut line = String::new();
                loop {
                    line.clear();
                    match reader.read_line(&mut line) {
                        Ok(0) => break,
                        Ok(_) => {
                            let text = line.trim_end().to_owned();
                            if !text.is_empty() && err_tx.send(ReaderMessage::Stderr(text)).is_err()
                            {
                                break;
                            }
                        }
                        Err(error) => {
                            let _ = err_tx.send(ReaderMessage::ReadError(error.to_string()));
                            break;
                        }
                    }
                }
                let _ = err_tx.send(ReaderMessage::StderrClosed);
            });
        drop(reader_tx);

        let mut stdout_closed = false;
        let mut stderr_closed = false;
        let mut exit_status = None;
        while !stop.load(Ordering::Acquire) {
            match child.try_wait() {
                Ok(Some(status)) => {
                    exit_status = Some(status);
                    break;
                }
                Ok(None) => {}
                Err(error) => {
                    let _ = sender.send(NiriProcessEvent::Stderr(error.to_string()));
                }
            }
            match reader_rx.recv_timeout(POLL_INTERVAL) {
                Ok(ReaderMessage::Stdout(line)) => {
                    if sender.send(NiriProcessEvent::StdoutLine(line)).is_err() {
                        stop.store(true, Ordering::Release);
                        break;
                    }
                }
                Ok(ReaderMessage::Stderr(text)) => {
                    if sender.send(NiriProcessEvent::Stderr(text)).is_err() {
                        stop.store(true, Ordering::Release);
                        break;
                    }
                }
                Ok(ReaderMessage::ReadError(error)) => {
                    let _ = sender.send(NiriProcessEvent::Stderr(error));
                }
                Ok(ReaderMessage::StdoutClosed) => stdout_closed = true,
                Ok(ReaderMessage::StderrClosed) => stderr_closed = true,
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => break,
            }
            if stdout_closed && stderr_closed {
                if let Ok(Some(status)) = child.try_wait() {
                    exit_status = Some(status);
                    break;
                }
            }
        }

        if stop.load(Ordering::Acquire) || exit_status.is_none() {
            let _ = child.kill();
            exit_status = child.wait().ok();
        }

        // Drain both pipes before publishing exit/reconnect. This preserves
        // trailing diagnostics and avoids leaving reader threads behind.
        if let Ok(reader) = stdout_reader {
            let _ = reader.join();
        }
        if let Ok(reader) = stderr_reader {
            let _ = reader.join();
        }
        while let Ok(message) = reader_rx.try_recv() {
            match message {
                ReaderMessage::Stdout(line) => {
                    if sender.send(NiriProcessEvent::StdoutLine(line)).is_err() {
                        return;
                    }
                }
                ReaderMessage::Stderr(text) | ReaderMessage::ReadError(text) => {
                    if sender.send(NiriProcessEvent::Stderr(text)).is_err() {
                        return;
                    }
                }
                ReaderMessage::StdoutClosed | ReaderMessage::StderrClosed => {}
            }
        }

        if let Some(status) = exit_status {
            let success = status.success();
            if sender
                .send(NiriProcessEvent::Exited {
                    code: status.code(),
                    success,
                })
                .is_err()
            {
                break;
            }
            if success || stop.load(Ordering::Acquire) || !wait_or_stop(&stop, RECONNECT_DELAY) {
                break;
            }
        } else {
            break;
        }
    }
}

fn wait_or_stop(stop: &AtomicBool, duration: Duration) -> bool {
    let slices = (duration.as_millis() / 40).max(1) as usize;
    for _ in 0..slices {
        if stop.load(Ordering::Acquire) {
            return false;
        }
        thread::sleep(POLL_INTERVAL);
    }
    !stop.load(Ordering::Acquire)
}

enum ReaderMessage {
    Stdout(String),
    Stderr(String),
    ReadError(String),
    StdoutClosed,
    StderrClosed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TestProgram(std::path::PathBuf);
    impl TestProgram {
        fn new(body: &str) -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let dir = std::env::temp_dir().join(format!("velatura-niri-stream-{nonce}"));
            fs::create_dir_all(&dir).unwrap();
            let path = dir.join("niri");
            fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
            let mut permissions = fs::metadata(&path).unwrap().permissions();
            permissions.set_mode(0o700);
            fs::set_permissions(&path, permissions).unwrap();
            Self(path)
        }
    }
    impl Drop for TestProgram {
        fn drop(&mut self) {
            if let Some(parent) = self.0.parent() {
                let _ = fs::remove_dir_all(parent);
            }
        }
    }

    #[test]
    fn forwards_stdout_stderr_and_exit() {
        let program = TestProgram::new("echo stream-line\necho diagnostic >&2\nexit 0");
        let (stream, events) = NiriEventStream::with_program(program.0.to_string_lossy());
        let mut line = false;
        let mut stderr = false;
        let mut exited = false;
        for _ in 0..3 {
            match events.recv_timeout(Duration::from_secs(2)).unwrap() {
                NiriProcessEvent::StdoutLine(value) => {
                    assert!(value.contains("stream-line"));
                    line = true;
                }
                NiriProcessEvent::Stderr(value) => {
                    assert!(value.contains("diagnostic"));
                    stderr = true;
                }
                NiriProcessEvent::Exited {
                    code: Some(0),
                    success: true,
                } => exited = true,
                other => panic!("unexpected event: {other:?}"),
            }
        }
        assert!(line && stderr && exited);
        drop(stream);
    }

    #[test]
    fn clean_exit_does_not_reconnect() {
        let program = TestProgram::new("exit 0");
        let (stream, events) = NiriEventStream::with_program(program.0.to_string_lossy());
        assert_eq!(
            events.recv_timeout(Duration::from_secs(2)).unwrap(),
            NiriProcessEvent::Exited {
                code: Some(0),
                success: true
            }
        );
        assert!(events.recv_timeout(Duration::from_millis(200)).is_err());
        drop(stream);
    }

    #[test]
    fn nonzero_exit_reconnects_after_the_configured_delay() {
        use std::time::Instant;

        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let marker = std::env::temp_dir().join(format!("velatura-reconnect-{nonce}"));
        let marker_text = marker.display().to_string();
        let body = format!(
            "if [ ! -e '{marker_text}' ]; then touch '{marker_text}'; exit 9; fi\necho restarted\nexit 0"
        );
        let program = TestProgram::new(&body);
        let started = Instant::now();
        let (stream, events) = NiriEventStream::with_program(program.0.to_string_lossy());

        assert_eq!(
            events.recv_timeout(Duration::from_secs(2)).unwrap(),
            NiriProcessEvent::Exited {
                code: Some(9),
                success: false
            }
        );
        let mut saw_restart = false;
        loop {
            match events.recv_timeout(Duration::from_secs(5)).unwrap() {
                NiriProcessEvent::StdoutLine(line) if line == "restarted" => saw_restart = true,
                NiriProcessEvent::Exited {
                    code: Some(0),
                    success: true,
                } => break,
                other => panic!("unexpected event during reconnect: {other:?}"),
            }
        }
        assert!(saw_restart);
        assert!(started.elapsed() >= RECONNECT_DELAY);
        drop(stream);
        let _ = fs::remove_file(marker);
    }

    #[test]
    fn stop_terminates_a_running_child() {
        let program = TestProgram::new("echo ready\nexec sleep 30");
        let (stream, events) = NiriEventStream::with_program(program.0.to_string_lossy());
        assert_eq!(
            events.recv_timeout(Duration::from_secs(2)).unwrap(),
            NiriProcessEvent::StdoutLine("ready".into())
        );
        stream.stop();
        drop(stream);
        let _ = events.try_iter().count();
    }
}
