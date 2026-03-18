use std::path::Path;
use std::process::Command;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum FfmpegError {
    #[error("Failed to launch ffmpeg: {0}")]
    Io(#[from] std::io::Error),
    #[error("ffmpeg exited with status {status}: {stderr}")]
    Failed { status: i32, stderr: String },
}

pub fn download_to_mp4(input_url: &str, output_path: &Path) -> Result<(), FfmpegError> {
    let output = Command::new("ffmpeg")
        .arg("-nostdin")
        .arg("-y")
        .arg("-i")
        .arg(input_url)
        .arg("-c")
        .arg("copy")
        .arg("-bsf:a")
        .arg("aac_adtstoasc")
        .arg("-movflags")
        .arg("+faststart")
        .arg(output_path)
        .output()?;

    if output.status.success() {
        return Ok(());
    }

    Err(FfmpegError::Failed {
        status: output.status.code().unwrap_or(-1),
        stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
    })
}
