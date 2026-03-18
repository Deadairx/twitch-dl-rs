use std::path::{Path, PathBuf};
use std::process::Command;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum TranscriptionError {
    #[error("No mlx-whisper executable found (tried `mlx-whisper` and `mlx_whisper`)")]
    MissingExecutable,
    #[error("Failed to launch transcription command: {0}")]
    Io(#[from] std::io::Error),
    #[error("Transcription command exited with status {status}: {stderr}")]
    Failed { status: i32, stderr: String },
}

pub fn transcribe_to_txt(
    media_path: &Path,
    artifact_dir: &Path,
) -> Result<PathBuf, TranscriptionError> {
    let command_name = available_command().ok_or(TranscriptionError::MissingExecutable)?;
    let output = Command::new(command_name)
        .arg(media_path)
        .arg("--output-dir")
        .arg(artifact_dir)
        .arg("--output-name")
        .arg("transcript")
        .arg("--output-format")
        .arg("txt")
        .output()?;

    if output.status.success() {
        return Ok(artifact_dir.join("transcript.txt"));
    }

    Err(TranscriptionError::Failed {
        status: output.status.code().unwrap_or(-1),
        stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
    })
}

fn available_command() -> Option<&'static str> {
    ["mlx-whisper", "mlx_whisper"]
        .into_iter()
        .find(|command| Command::new(command).arg("--help").output().is_ok())
}
