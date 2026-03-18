use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::downloader::StreamInfo;
use crate::twitch::VodEntry;

#[derive(Debug, Serialize)]
pub struct ArtifactMetadata {
    schema_version: u32,
    video_id: String,
    source_url: String,
    downloaded_at_epoch_s: u64,
    used_auth_token: bool,
    output_file: String,
    output_size_bytes: u64,
    stream_name: Option<String>,
    selected_bandwidth: Option<u64>,
    selected_resolution: Option<String>,
    selected_codecs: Option<String>,
    is_audio_only: bool,
}

impl ArtifactMetadata {
    pub fn from_download(
        video_id: &str,
        source_url: &str,
        output_file: &Path,
        stream: &StreamInfo,
        used_auth_token: bool,
    ) -> Result<Self, std::io::Error> {
        let output_size_bytes = fs::metadata(output_file)?.len();
        let downloaded_at_epoch_s = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Ok(Self {
            schema_version: 1,
            video_id: video_id.to_string(),
            source_url: source_url.to_string(),
            downloaded_at_epoch_s,
            used_auth_token,
            output_file: output_file
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            output_size_bytes,
            stream_name: stream.name.clone(),
            selected_bandwidth: stream.bandwidth,
            selected_resolution: stream.resolution.clone(),
            selected_codecs: stream.codecs.clone(),
            is_audio_only: stream.is_audio_only,
        })
    }
}

pub fn prepare_artifact_dir(artifact_dir: &Path) -> Result<(), std::io::Error> {
    fs::create_dir_all(artifact_dir)
}

pub fn write_source_url(artifact_dir: &Path, source_url: &str) -> Result<(), std::io::Error> {
    fs::write(
        artifact_dir.join("source_url.txt"),
        format!("{source_url}\n"),
    )
}

pub fn write_metadata(
    artifact_dir: &Path,
    metadata: &ArtifactMetadata,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let metadata_path = artifact_dir.join("metadata.json");
    let json = serde_json::to_string_pretty(metadata)?;
    fs::write(&metadata_path, format!("{json}\n"))?;
    Ok(metadata_path)
}

#[derive(Debug, Serialize)]
pub struct QueueFile {
    schema_version: u32,
    channel: String,
    generated_at_epoch_s: u64,
    past_broadcasts_only: bool,
    min_seconds: u64,
    queued_count: usize,
    queued: Vec<VodEntry>,
    skipped_existing_ids: Vec<String>,
}

pub fn existing_artifact_ids(output_root: &Path) -> Result<Vec<String>, std::io::Error> {
    let mut ids = Vec::new();

    if !output_root.exists() {
        return Ok(ids);
    }

    for entry in fs::read_dir(output_root)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.chars().all(|char| char.is_ascii_digit()) {
                ids.push(name.to_string());
            }
        }
    }

    ids.sort();
    Ok(ids)
}

pub fn write_queue_file(
    output_root: &Path,
    channel: &str,
    past_broadcasts_only: bool,
    min_seconds: u64,
    queued: Vec<VodEntry>,
    skipped_existing_ids: Vec<String>,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let queue_dir = output_root.join("queues");
    fs::create_dir_all(&queue_dir)?;

    let queue_file = QueueFile {
        schema_version: 1,
        channel: channel.to_ascii_lowercase(),
        generated_at_epoch_s: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        past_broadcasts_only,
        min_seconds,
        queued_count: queued.len(),
        queued,
        skipped_existing_ids,
    };

    let queue_path = queue_dir.join(format!("{}.json", channel.to_ascii_lowercase()));
    let json = serde_json::to_string_pretty(&queue_file)?;
    fs::write(&queue_path, format!("{json}\n"))?;
    Ok(queue_path)
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ProcessStatus {
    pub schema_version: u32,
    pub video_id: String,
    pub source_url: String,
    pub media_file: Option<String>,
    pub transcript_file: Option<String>,
    pub downloaded: bool,
    pub transcribed: bool,
    pub last_error: Option<String>,
    pub updated_at_epoch_s: u64,
}

impl ProcessStatus {
    pub fn new(video_id: &str, source_url: &str) -> Self {
        Self {
            schema_version: 1,
            video_id: video_id.to_string(),
            source_url: source_url.to_string(),
            media_file: None,
            transcript_file: None,
            downloaded: false,
            transcribed: false,
            last_error: None,
            updated_at_epoch_s: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}

pub fn read_status(
    artifact_dir: &Path,
) -> Result<Option<ProcessStatus>, Box<dyn std::error::Error>> {
    let status_path = artifact_dir.join("status.json");
    if !status_path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(status_path)?;
    let status = serde_json::from_str(&content)?;
    Ok(Some(status))
}

pub fn write_status(
    artifact_dir: &Path,
    status: &ProcessStatus,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let status_path = artifact_dir.join("status.json");
    let json = serde_json::to_string_pretty(status)?;
    fs::write(&status_path, format!("{json}\n"))?;
    Ok(status_path)
}

pub fn find_media_file(artifact_dir: &Path) -> Option<PathBuf> {
    ["audio.m4a", "video.mp4"]
        .into_iter()
        .map(|name| artifact_dir.join(name))
        .find(|path| path.exists())
}
