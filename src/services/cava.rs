//! On-demand Cava spectrum worker, matching the original 64-band raw stream.

use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader};
use std::os::unix::fs::OpenOptionsExt;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

const CONFIG: &str = include_str!("../../resources/cava.conf");

fn parse_line(line: &str) -> Option<[f32; 8]> {
    let mut bands = [0.0_f32; 64];
    let mut count = 0;
    for field in line.trim().split(';') {
        if count == bands.len() {
            break;
        }
        bands[count] = field.parse::<f32>().ok()?.clamp(0.0, 100.0) / 100.0;
        count += 1;
    }
    if count != bands.len() {
        return None;
    }

    let mut low_to_high = [0.0_f32; 4];
    for (group, value) in low_to_high.iter_mut().enumerate() {
        let start = group * 16;
        *value = bands[start..start + 16].iter().sum::<f32>() / 16.0;
    }
    Some([
        low_to_high[3],
        low_to_high[2],
        low_to_high[1],
        low_to_high[0],
        low_to_high[0],
        low_to_high[1],
        low_to_high[2],
        low_to_high[3],
    ])
}

pub struct CavaWorker {
    child: Child,
    reader: Option<JoinHandle<()>>,
    values: Arc<Mutex<[f32; 8]>>,
    config_path: std::path::PathBuf,
}

impl CavaWorker {
    pub fn start() -> Result<Self, String> {
        let config_path =
            std::env::temp_dir().join(format!("velatura-cava-{}.conf", std::process::id()));
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&config_path)
            .map_err(|error| error.to_string())?;
        std::io::Write::write_all(&mut file, CONFIG.as_bytes())
            .map_err(|error| error.to_string())?;

        let mut child = Command::new("cava")
            .args(["-p", config_path.to_string_lossy().as_ref()])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| {
                let _ = fs::remove_file(&config_path);
                error.to_string()
            })?;
        let stdout = child.stdout.take().ok_or("Cava stdout unavailable")?;
        let values = Arc::new(Mutex::new([0.0; 8]));
        let shared = Arc::clone(&values);
        let reader = thread::Builder::new()
            .name("velatura-cava-reader".into())
            .spawn(move || {
                for line in BufReader::new(stdout).lines() {
                    let Ok(line) = line else { break };
                    if let Some(next) = parse_line(&line) {
                        *shared.lock().unwrap() = next;
                    }
                }
            })
            .map_err(|error| {
                let _ = child.kill();
                let _ = child.wait();
                let _ = fs::remove_file(&config_path);
                error.to_string()
            })?;

        Ok(Self {
            child,
            reader: Some(reader),
            values,
            config_path,
        })
    }

    pub fn values(&self) -> [f32; 8] {
        *self.values.lock().unwrap()
    }

    pub fn alive(&mut self) -> bool {
        self.child
            .try_wait()
            .map(|status| status.is_none())
            .unwrap_or(false)
    }
}

impl Drop for CavaWorker {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
        let _ = fs::remove_file(&self.config_path);
    }
}

#[cfg(test)]
mod tests {
    use super::parse_line;

    #[test]
    fn mirrors_four_low_to_high_groups_into_eight_bars() {
        let line = (0..64)
            .map(|index| index.to_string())
            .collect::<Vec<_>>()
            .join(";");
        let bars = parse_line(&line).unwrap();
        assert_eq!(bars.len(), 8);
        assert_eq!(bars[0], bars[7]);
        assert_eq!(bars[1], bars[6]);
        assert_eq!(bars[2], bars[5]);
        assert_eq!(bars[3], bars[4]);
        assert!(bars[0] > bars[1]);
    }

    #[test]
    fn clamps_values_and_rejects_incomplete_frames() {
        assert!(parse_line("0;1;2").is_none());
        let line = std::iter::repeat_n("200", 64).collect::<Vec<_>>().join(";");
        assert_eq!(parse_line(&line).unwrap(), [1.0; 8]);
    }
}
