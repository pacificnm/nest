//! FFprobe subprocess runner.

use std::io;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use tracing::{debug, instrument};

use crate::config::TranscodeConfig;
use crate::dto::FfprobeOutput;
use crate::error::{TranscodeError, TranscodeResult};

/// Executes ffprobe and parses JSON output.
#[derive(Debug, Clone)]
pub struct FfprobeRunner {
    config: TranscodeConfig,
}

impl FfprobeRunner {
    /// Creates a runner from configuration.
    pub fn new(config: TranscodeConfig) -> TranscodeResult<Self> {
        Ok(Self { config })
    }

    /// Returns the runner configuration.
    pub fn config(&self) -> &TranscodeConfig {
        &self.config
    }

    /// Probes one media file synchronously.
    #[instrument(skip(self), fields(path = %absolute_path.display()))]
    pub(crate) fn probe_file(&self, absolute_path: &Path) -> TranscodeResult<FfprobeOutput> {
        if !absolute_path.is_file() {
            return Err(TranscodeError::probe(format!(
                "media file not found: {}",
                absolute_path.display()
            )));
        }

        let mut command = Command::new(&self.config.ffprobe_path);
        command
            .arg("-v")
            .arg("quiet")
            .arg("-print_format")
            .arg("json")
            .arg("-show_format")
            .arg("-show_streams");

        for arg in &self.config.extra_ffprobe_args {
            command.arg(arg);
        }

        command.arg(absolute_path);
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        debug!("spawning ffprobe");
        let mut child = command.spawn().map_err(map_spawn_error)?;

        let timeout = Duration::from_secs(self.config.timeout_seconds as u64);
        let started = Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(_status)) => break,
                Ok(None) if started.elapsed() >= timeout => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(TranscodeError::timeout(format!(
                        "ffprobe timed out after {} seconds",
                        self.config.timeout_seconds
                    )));
                }
                Ok(None) => thread::sleep(Duration::from_millis(50)),
                Err(error) => {
                    return Err(TranscodeError::probe(format!(
                        "failed waiting for ffprobe: {error}"
                    ))
                    .with_source(error));
                }
            }
        }

        let output = child
            .wait_with_output()
            .map_err(|error| TranscodeError::probe(format!("ffprobe failed: {error}")).with_source(error))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(TranscodeError::probe(format!(
                "ffprobe exited with {}: {}",
                output.status,
                stderr.trim()
            )));
        }

        serde_json::from_slice(&output.stdout).map_err(|error| {
            TranscodeError::parse(format!("failed to parse ffprobe JSON: {error}"))
        })
    }
}

fn map_spawn_error(error: io::Error) -> TranscodeError {
    if error.kind() == io::ErrorKind::NotFound {
        TranscodeError::binary_not_found(format!(
            "ffprobe binary not found: ensure ffmpeg is installed"
        ))
        .with_source(error)
    } else {
        TranscodeError::probe(format!("failed to spawn ffprobe: {error}")).with_source(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn probe_missing_file_fails() {
        let runner = FfprobeRunner::new(TranscodeConfig::builder().build().unwrap()).unwrap();
        let error = runner
            .probe_file(Path::new("/no/such/file.mkv"))
            .unwrap_err();
        assert_eq!(error.kind(), crate::error::TranscodeErrorKind::Probe);
    }

    #[test]
    #[ignore = "requires ffprobe installed; set NEST_FFPROBE_TEST=1 to run"]
    fn probe_real_file_when_enabled() {
        if std::env::var("NEST_FFPROBE_TEST").ok().as_deref() != Some("1") {
            return;
        }

        let dir = tempdir().unwrap();
        let path = dir.path().join("empty.mkv");
        fs::write(&path, b"").unwrap();

        let runner = FfprobeRunner::new(TranscodeConfig::builder().build().unwrap()).unwrap();
        let _ = runner.probe_file(&path);
    }
}
