//! Serialized Niri action execution, preserving argv boundaries and FIFO order.

use std::io;
use std::process::{Command, Output};
use std::sync::mpsc::{self, Sender};
use std::thread::{self, JoinHandle};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NiriActionResult {
    pub args: Vec<String>,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub spawn_error: Option<String>,
}
impl NiriActionResult {
    pub fn succeeded(&self) -> bool {
        self.spawn_error.is_none() && self.exit_code == Some(0)
    }
}
enum QueueMessage {
    Run(Vec<String>),
}

pub struct NiriActionQueue {
    sender: Option<Sender<QueueMessage>>,
    worker: Option<JoinHandle<()>>,
}
impl NiriActionQueue {
    pub fn new() -> (Self, mpsc::Receiver<NiriActionResult>) {
        Self::with_program("niri")
    }
    pub fn with_program(program: impl Into<String>) -> (Self, mpsc::Receiver<NiriActionResult>) {
        let program = program.into();
        let (sender, receiver) = mpsc::channel::<QueueMessage>();
        let (result_sender, result_receiver) = mpsc::channel();
        let worker = thread::Builder::new()
            .name("velatura-niri-actions".into())
            .spawn(move || {
                while let Ok(QueueMessage::Run(args)) = receiver.recv() {
                    let result = execute_action(&program, &args);
                    if result_sender.send(result).is_err() {
                        break;
                    }
                }
            })
            .expect("failed to start Niri action queue worker");
        (
            Self {
                sender: Some(sender),
                worker: Some(worker),
            },
            result_receiver,
        )
    }
    pub fn enqueue<I, S>(&self, args: I) -> Result<(), mpsc::SendError<()>>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let args: Vec<String> = args.into_iter().map(Into::into).collect();
        if args.is_empty() {
            return Ok(());
        }
        self.sender
            .as_ref()
            .expect("queue already shut down")
            .send(QueueMessage::Run(args))
            .map_err(|_| mpsc::SendError(()))
    }
    pub fn enqueue_many<I, A, S>(&self, actions: I) -> Result<(), mpsc::SendError<()>>
    where
        I: IntoIterator<Item = A>,
        A: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for action in actions {
            self.enqueue(action)?;
        }
        Ok(())
    }
}
impl Drop for NiriActionQueue {
    fn drop(&mut self) {
        self.sender.take();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
fn execute_action(program: &str, args: &[String]) -> NiriActionResult {
    match Command::new(program)
        .arg("msg")
        .arg("action")
        .args(args)
        .output()
    {
        Ok(Output {
            status,
            stdout,
            stderr,
        }) => NiriActionResult {
            args: args.to_vec(),
            exit_code: status.code(),
            stdout: String::from_utf8_lossy(&stdout).into_owned(),
            stderr: String::from_utf8_lossy(&stderr).into_owned(),
            spawn_error: None,
        },
        Err(error) => NiriActionResult {
            args: args.to_vec(),
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            spawn_error: Some(io::Error::to_string(&error)),
        },
    }
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
            let dir = std::env::temp_dir().join(format!("velatura-niri-actions-{nonce}"));
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
    fn actions_run_fifo_and_preserve_arguments() {
        let program = TestProgram::new("echo action-ok");
        let (queue, results) = NiriActionQueue::with_program(program.0.to_string_lossy());
        queue.enqueue(["focus-workspace", "2"]).unwrap();
        queue.enqueue(["focus-monitor", "DP-1"]).unwrap();
        drop(queue);
        let first = results.recv().unwrap();
        let second = results.recv().unwrap();
        assert!(first.succeeded() && second.succeeded());
        assert_eq!(first.args, ["focus-workspace", "2"]);
        assert_eq!(second.args, ["focus-monitor", "DP-1"]);
        assert_eq!(first.stdout.trim(), "action-ok");
    }
    #[test]
    fn nonzero_exit_is_reported_and_does_not_stop_queue() {
        let program = TestProgram::new("exit 7");
        let (queue, results) = NiriActionQueue::with_program(program.0.to_string_lossy());
        queue.enqueue(["first"]).unwrap();
        queue.enqueue(["second"]).unwrap();
        drop(queue);
        assert_eq!(results.recv().unwrap().exit_code, Some(7));
        assert_eq!(results.recv().unwrap().exit_code, Some(7));
    }
}
