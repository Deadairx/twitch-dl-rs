use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::downloader::StreamInfo;
use crate::twitch::VodEntry;

#[derive(Debug, Serialize, Deserialize)]
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uploaded_at: Option<String>,
    is_audio_only: bool,
}

impl ArtifactMetadata {
    pub fn from_download(
        video_id: &str,
        source_url: &str,
        output_file: &Path,
        stream: &StreamInfo,
        used_auth_token: bool,
        vod_context: Option<(&str, &str, &str)>,
    ) -> Result<Self, std::io::Error> {
        let output_size_bytes = fs::metadata(output_file)?.len();
        let downloaded_at_epoch_s = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let (title, channel, uploaded_at) = match vod_context {
            Some((t, c, u)) => (Some(t.to_string()), Some(c.to_string()), Some(u.to_string())),
            None => (None, None, None),
        };

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
            title,
            channel,
            uploaded_at,
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

pub fn read_metadata(
    artifact_dir: &Path,
) -> Result<Option<ArtifactMetadata>, Box<dyn std::error::Error>> {
    let metadata_path = artifact_dir.join("metadata.json");
    if !metadata_path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(metadata_path)?;
    let metadata = serde_json::from_str(&content)?;
    Ok(Some(metadata))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueueFile {
    schema_version: u32,
    channel: String,
    generated_at_epoch_s: u64,
    past_broadcasts_only: bool,
    min_seconds: u64,
    queued_count: usize,
    pub queued: Vec<VodEntry>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transcription_outcome: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transcription_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transcript_word_count: Option<u64>,
    #[serde(default)]
    pub ready_for_notes: bool,
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
            transcription_outcome: None,
            transcription_reason: None,
            transcript_word_count: None,
            ready_for_notes: false,
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

pub fn read_queue_file(
    output_root: &Path,
    channel: &str,
) -> Result<QueueFile, Box<dyn std::error::Error>> {
    let queue_path = output_root.join("queues").join(format!("{}.json", channel.to_ascii_lowercase()));
    if !queue_path.exists() {
        return Err(format!("No queue file found for channel '{}'. Run 'queue {}' first.", channel, channel).into());
    }
    let content = fs::read_to_string(&queue_path)?;
    Ok(serde_json::from_str(&content)?)
}

pub fn scan_artifact_statuses(
    output_root: &Path,
) -> Result<Vec<(String, Option<ProcessStatus>)>, std::io::Error> {
    let mut results = Vec::new();
    let ids = existing_artifact_ids(output_root)?;
    for video_id in ids {
        let artifact_dir = output_root.join(&video_id);
        let status = read_status(&artifact_dir).unwrap_or(None);
        results.push((video_id, status));
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_read_queue_file_roundtrip() {
        let dir = tempdir().unwrap();
        // write a minimal QueueFile JSON, then read it back
        let queue_dir = dir.path().join("queues");
        fs::create_dir_all(&queue_dir).unwrap();
        let json = r#"{"schema_version":1,"channel":"testchan","generated_at_epoch_s":0,"past_broadcasts_only":false,"min_seconds":600,"queued_count":1,"queued":[{"channel":"testchan","title":"Test VOD","url":"https://www.twitch.tv/videos/123","video_id":"123","uploaded_at":"2026-01-01T00:00:00Z","duration":"PT3600S"}],"skipped_existing_ids":[]}"#;
        fs::write(queue_dir.join("testchan.json"), json).unwrap();
        let qf = read_queue_file(dir.path(), "testchan").unwrap();
        assert_eq!(qf.queued.len(), 1);
        assert_eq!(qf.queued[0].video_id, "123");
    }

    #[test]
    fn test_scan_artifact_statuses_empty() {
        let dir = tempdir().unwrap();
        let results = scan_artifact_statuses(dir.path()).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_status_roundtrip() {
        let dir = tempdir().unwrap();
        let artifact_dir = dir.path().join("123456");
        fs::create_dir_all(&artifact_dir).unwrap();
        let mut status = ProcessStatus::new("123456", "https://www.twitch.tv/videos/123456");
        status.downloaded = true;
        write_status(&artifact_dir, &status).unwrap();
        let read_back = read_status(&artifact_dir).unwrap().unwrap();
        assert_eq!(read_back.downloaded, true);
        assert_eq!(read_back.video_id, "123456");
    }

    #[test]
    fn test_process_status_backward_compat() {
        // Deserialize old schema without new transcription fields
        let old_json = r#"{"schema_version":1,"video_id":"abc","source_url":"https://twitch.tv/videos/abc","media_file":null,"transcript_file":null,"downloaded":true,"transcribed":false,"last_error":null,"updated_at_epoch_s":0}"#;
        let status: ProcessStatus = serde_json::from_str(old_json).unwrap();
        assert_eq!(status.video_id, "abc");
        assert_eq!(status.downloaded, true);
        assert_eq!(status.transcribed, false);
        // All new fields should default to None
        assert_eq!(status.transcription_outcome, None);
        assert_eq!(status.transcription_reason, None);
        assert_eq!(status.transcript_word_count, None);
    }

    #[test]
    fn test_ready_for_notes_backward_compat() {
        // Old status.json without ready_for_notes field should deserialize as false
        let old_json = r#"{"schema_version":1,"video_id":"123","source_url":"https://twitch.tv/videos/123","media_file":null,"transcript_file":null,"downloaded":true,"transcribed":true,"last_error":null,"updated_at_epoch_s":0,"transcription_outcome":"completed"}"#;
        let status: ProcessStatus = serde_json::from_str(old_json).unwrap();
        assert_eq!(status.ready_for_notes, false);
    }

    #[test]
    fn test_ready_for_notes_roundtrip() {
        let dir = tempdir().unwrap();
        let artifact_dir = dir.path().join("999");
        fs::create_dir_all(&artifact_dir).unwrap();
        let mut status = ProcessStatus::new("999", "https://www.twitch.tv/videos/999");
        status.downloaded = true;
        status.transcribed = true;
        status.ready_for_notes = true;
        status.transcription_outcome = Some("completed".to_string());
        write_status(&artifact_dir, &status).unwrap();
        let read_back = read_status(&artifact_dir).unwrap().unwrap();
        assert_eq!(read_back.ready_for_notes, true);
        assert_eq!(read_back.transcribed, true);
        assert_eq!(read_back.transcription_outcome, Some("completed".to_string()));
    }

    #[test]
    fn test_cleanup_candidate_filtering() {
        // Verify that scan_artifact_statuses correctly identifies cleanup candidates
        let dir = tempdir().unwrap();

        // Create a ready-for-notes candidate with completed outcome
        let artifact_dir_1 = dir.path().join("111111");
        fs::create_dir_all(&artifact_dir_1).unwrap();
        let mut status_1 = ProcessStatus::new("111111", "https://www.twitch.tv/videos/111111");
        status_1.downloaded = true;
        status_1.transcribed = true;
        status_1.ready_for_notes = true;
        status_1.transcription_outcome = Some("completed".to_string());
        write_status(&artifact_dir_1, &status_1).unwrap();
        fs::write(artifact_dir_1.join("audio.m4a"), "test").unwrap();
        fs::write(artifact_dir_1.join("transcript.srt"), "test").unwrap();

        // Create a ready-for-notes candidate with suspect outcome (should NOT be candidate)
        let artifact_dir_2 = dir.path().join("222222");
        fs::create_dir_all(&artifact_dir_2).unwrap();
        let mut status_2 = ProcessStatus::new("222222", "https://www.twitch.tv/videos/222222");
        status_2.downloaded = true;
        status_2.transcribed = true;
        status_2.ready_for_notes = true;
        status_2.transcription_outcome = Some("suspect".to_string());
        write_status(&artifact_dir_2, &status_2).unwrap();
        fs::write(artifact_dir_2.join("audio.m4a"), "test").unwrap();

        // Create a NOT ready-for-notes artifact with completed outcome (should NOT be candidate)
        let artifact_dir_3 = dir.path().join("333333");
        fs::create_dir_all(&artifact_dir_3).unwrap();
        let mut status_3 = ProcessStatus::new("333333", "https://www.twitch.tv/videos/333333");
        status_3.downloaded = true;
        status_3.transcribed = true;
        status_3.ready_for_notes = false;
        status_3.transcription_outcome = Some("completed".to_string());
        write_status(&artifact_dir_3, &status_3).unwrap();
        fs::write(artifact_dir_3.join("audio.m4a"), "test").unwrap();

        // Scan all artifacts
        let results = scan_artifact_statuses(dir.path()).unwrap();
        assert_eq!(results.len(), 3);

        // Only 111111 should be a valid cleanup candidate
        // (ready_for_notes=true AND transcription_outcome="completed")
        let candidates: Vec<_> = results
            .iter()
            .filter(|(_, status_opt)| {
                if let Some(status) = status_opt {
                    status.ready_for_notes
                        && status.transcription_outcome.as_deref() == Some("completed")
                } else {
                    false
                }
            })
            .collect();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].0, "111111");
    }

    #[test]
    fn test_metadata_backward_compat() {
        // Deserialize old schema without title, channel, uploaded_at fields
        let old_json = r#"{"schema_version":1,"video_id":"123","source_url":"https://twitch.tv/videos/123","downloaded_at_epoch_s":0,"used_auth_token":false,"output_file":"audio.m4a","output_size_bytes":100,"stream_name":null,"selected_bandwidth":null,"selected_resolution":null,"selected_codecs":null,"is_audio_only":true}"#;
        let metadata: ArtifactMetadata = serde_json::from_str(old_json).unwrap();
        assert_eq!(metadata.video_id, "123");
        assert_eq!(metadata.title, None);
        assert_eq!(metadata.channel, None);
        assert_eq!(metadata.uploaded_at, None);
        assert_eq!(metadata.is_audio_only, true);
    }

    #[test]
    fn test_metadata_roundtrip() {
        let dir = tempdir().unwrap();
        let artifact_dir = dir.path().join("123456");
        fs::create_dir_all(&artifact_dir).unwrap();

        // Create a mock StreamInfo for from_download
        use crate::downloader::StreamInfo;
        let stream = StreamInfo {
            playlist_url: "https://example.com/playlist.m3u8".to_string(),
            name: Some("test_stream".to_string()),
            bandwidth: Some(5000000),
            resolution: Some("1920x1080".to_string()),
            codecs: Some("avc1.640028,mp4a.40.2".to_string()),
            is_audio_only: false,
        };

        // Create a dummy video file so from_download can stat it
        let video_path = artifact_dir.join("video.mp4");
        fs::write(&video_path, b"dummy video content").unwrap();

        // Create metadata with vod_context
        let metadata = ArtifactMetadata::from_download(
            "123456",
            "https://www.twitch.tv/videos/123456",
            &video_path,
            &stream,
            false,
            Some(("Test VOD", "testchan", "2026-01-01T00:00:00Z")),
        ).unwrap();

        // Serialize to JSON
        let json = serde_json::to_string_pretty(&metadata).unwrap();
        
        // Deserialize back
        let deserialized: ArtifactMetadata = serde_json::from_str(&json).unwrap();
        
        // Verify new fields are populated
        assert_eq!(deserialized.title, Some("Test VOD".to_string()));
        assert_eq!(deserialized.channel, Some("testchan".to_string()));
        assert_eq!(deserialized.uploaded_at, Some("2026-01-01T00:00:00Z".to_string()));
        assert_eq!(deserialized.video_id, "123456");
    }
}
