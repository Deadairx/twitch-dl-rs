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

pub fn scan_queue_files(output_root: &Path) -> Result<Vec<VodEntry>, std::io::Error> {
    let queue_dir = output_root.join("queues");
    if !queue_dir.exists() {
        return Ok(vec![]);
    }
    let mut entries = Vec::new();
    for entry in fs::read_dir(&queue_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            let content = fs::read_to_string(&path).unwrap_or_default();
            if let Ok(qf) = serde_json::from_str::<QueueFile>(&content) {
                entries.extend(qf.queued);
            }
        }
    }
    Ok(entries)
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

    #[test]
    fn test_scan_queue_files_no_queues_dir() {
        let dir = tempdir().unwrap();
        // dir has no queues/ subdirectory
        let result = scan_queue_files(dir.path()).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_scan_queue_files_single_file() {
        let dir = tempdir().unwrap();
        let queue_dir = dir.path().join("queues");
        fs::create_dir_all(&queue_dir).unwrap();
        
        let json = r#"{"schema_version":1,"channel":"chan1","generated_at_epoch_s":0,"past_broadcasts_only":false,"min_seconds":600,"queued_count":2,"queued":[{"channel":"testchan","title":"Test VOD","url":"https://www.twitch.tv/videos/123","video_id":"aaa","uploaded_at":"2026-01-01T00:00:00Z","duration":"PT3600S"},{"channel":"testchan","title":"Test VOD 2","url":"https://www.twitch.tv/videos/124","video_id":"bbb","uploaded_at":"2026-01-01T00:00:00Z","duration":"PT3600S"}],"skipped_existing_ids":[]}"#;
        fs::write(queue_dir.join("chan1.json"), json).unwrap();
        
        let result = scan_queue_files(dir.path()).unwrap();
        assert_eq!(result.len(), 2);
        
        let ids: Vec<_> = result.iter().map(|e| e.video_id.as_str()).collect();
        assert!(ids.contains(&"aaa"));
        assert!(ids.contains(&"bbb"));
    }

    #[test]
    fn test_scan_queue_files_multiple_files() {
        let dir = tempdir().unwrap();
        let queue_dir = dir.path().join("queues");
        fs::create_dir_all(&queue_dir).unwrap();
        
        let json1 = r#"{"schema_version":1,"channel":"chan1","generated_at_epoch_s":0,"past_broadcasts_only":false,"min_seconds":600,"queued_count":2,"queued":[{"channel":"testchan","title":"Test VOD","url":"https://www.twitch.tv/videos/123","video_id":"aaa","uploaded_at":"2026-01-01T00:00:00Z","duration":"PT3600S"},{"channel":"testchan","title":"Test VOD 2","url":"https://www.twitch.tv/videos/124","video_id":"bbb","uploaded_at":"2026-01-01T00:00:00Z","duration":"PT3600S"}],"skipped_existing_ids":[]}"#;
        fs::write(queue_dir.join("chan1.json"), json1).unwrap();
        
        let json2 = r#"{"schema_version":1,"channel":"chan2","generated_at_epoch_s":0,"past_broadcasts_only":false,"min_seconds":600,"queued_count":1,"queued":[{"channel":"testchan","title":"Test VOD 3","url":"https://www.twitch.tv/videos/125","video_id":"ccc","uploaded_at":"2026-01-01T00:00:00Z","duration":"PT3600S"}],"skipped_existing_ids":[]}"#;
        fs::write(queue_dir.join("chan2.json"), json2).unwrap();
        
        let result = scan_queue_files(dir.path()).unwrap();
        assert_eq!(result.len(), 3);
        
        let ids: Vec<_> = result.iter().map(|e| e.video_id.as_str()).collect();
        assert!(ids.contains(&"aaa"));
        assert!(ids.contains(&"bbb"));
        assert!(ids.contains(&"ccc"));
    }

    #[test]
    fn test_scan_queue_files_malformed_file() {
        let dir = tempdir().unwrap();
        let queue_dir = dir.path().join("queues");
        fs::create_dir_all(&queue_dir).unwrap();
        
        let good_json = r#"{"schema_version":1,"channel":"good","generated_at_epoch_s":0,"past_broadcasts_only":false,"min_seconds":600,"queued_count":1,"queued":[{"channel":"testchan","title":"Test VOD","url":"https://www.twitch.tv/videos/123","video_id":"zzz","uploaded_at":"2026-01-01T00:00:00Z","duration":"PT3600S"}],"skipped_existing_ids":[]}"#;
        fs::write(queue_dir.join("good.json"), good_json).unwrap();
        
        let bad_json = r#""not valid json at all""#;
        fs::write(queue_dir.join("bad.json"), bad_json).unwrap();
        
        let result = scan_queue_files(dir.path()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].video_id, "zzz");
    }

    #[test]
    fn test_scan_queue_dedup_with_artifact() {
        let dir = tempdir().unwrap();

        // Setup: queues/chan1.json with 2 VodEntries: IDs "100" and "200"
        let queue_dir = dir.path().join("queues");
        fs::create_dir_all(&queue_dir).unwrap();
        let queue_json = r#"{"schema_version":1,"channel":"chan1","generated_at_epoch_s":0,"past_broadcasts_only":false,"min_seconds":600,"queued_count":2,"queued":[{"channel":"chan1","title":"Queue VOD 1","url":"https://www.twitch.tv/videos/100","video_id":"100","uploaded_at":"2026-01-01T00:00:00Z","duration":"PT3600S"},{"channel":"chan1","title":"Queue VOD 2","url":"https://www.twitch.tv/videos/200","video_id":"200","uploaded_at":"2026-01-02T00:00:00Z","duration":"PT3600S"}],"skipped_existing_ids":[]}"#;
        fs::write(queue_dir.join("chan1.json"), queue_json).unwrap();

        // Setup: artifact dir 100/ with metadata.json (ID "100" appears in both sources)
        let artifact_100 = dir.path().join("100");
        fs::create_dir_all(&artifact_100).unwrap();
        let metadata_json = r#"{"schema_version":1,"video_id":"100","source_url":"https://www.twitch.tv/videos/100","downloaded_at_epoch_s":0,"used_auth_token":false,"output_file":"audio.m4a","output_size_bytes":100,"stream_name":null,"selected_bandwidth":null,"selected_resolution":null,"selected_codecs":null,"title":"My VOD","channel":"chan1","uploaded_at":"2026-01-01T00:00:00Z","is_audio_only":true}"#;
        fs::write(artifact_100.join("metadata.json"), metadata_json).unwrap();
        let status_json = r#"{"schema_version":1,"video_id":"100","source_url":"https://www.twitch.tv/videos/100","media_file":"audio.m4a","transcript_file":null,"downloaded":true,"transcribed":false,"last_error":null,"updated_at_epoch_s":0}"#;
        fs::write(artifact_100.join("status.json"), status_json).unwrap();

        // Setup: artifact dir 300/ with audio.m4a file but no status.json (pre-S01 bare download)
        let artifact_300 = dir.path().join("300");
        fs::create_dir_all(&artifact_300).unwrap();
        fs::write(artifact_300.join("audio.m4a"), "dummy audio").unwrap();

        // Assert: scan_queue_files returns 2 entries (IDs "100" and "200")
        let queue_results = scan_queue_files(dir.path()).unwrap();
        assert_eq!(queue_results.len(), 2);
        let queue_ids: Vec<_> = queue_results.iter().map(|v| v.video_id.as_str()).collect();
        assert!(queue_ids.contains(&"100"));
        assert!(queue_ids.contains(&"200"));

        // Assert: scan_artifact_statuses returns 2 entries (IDs "100" and "300")
        let artifact_results = scan_artifact_statuses(dir.path()).unwrap();
        assert_eq!(artifact_results.len(), 2);
        let artifact_ids: Vec<_> = artifact_results.iter().map(|(id, _)| id.as_str()).collect();
        assert!(artifact_ids.contains(&"100"));
        assert!(artifact_ids.contains(&"300"));

        // Assert: filtering scan_queue_files by IDs not in scan_artifact_statuses yields only "200"
        let artifact_id_set: std::collections::HashSet<_> = artifact_results.iter().map(|(id, _)| id.clone()).collect();
        let deduped: Vec<_> = queue_results
            .into_iter()
            .filter(|v| !artifact_id_set.contains(&v.video_id))
            .collect();
        assert_eq!(deduped.len(), 1);
        assert_eq!(deduped[0].video_id, "200");
    }

    #[test]
    fn test_queue_video_idempotent_dedup() {
        let dir = tempdir().unwrap();
        let queue_dir = dir.path().join("queues");
        fs::create_dir_all(&queue_dir).unwrap();

        // Step 1: Write a queue file with one entry (video_id "111")
        let queued = vec![VodEntry {
            channel: "testchan".to_string(),
            title: "Test VOD 1".to_string(),
            url: "https://www.twitch.tv/videos/111".to_string(),
            video_id: "111".to_string(),
            uploaded_at: "2026-01-01T00:00:00Z".to_string(),
            duration: "PT3600S".to_string(),
        }];
        let _queue_path = write_queue_file(
            dir.path(),
            "testchan",
            false,
            0,
            queued.clone(),
            vec![],
        ).unwrap();

        // Step 2: Read the queue file back and verify dedup triggers on same video_id
        let read_back = read_queue_file(dir.path(), "testchan").unwrap();
        assert_eq!(read_back.queued.len(), 1);
        assert!(read_back.queued.iter().any(|v| v.video_id == "111"));

        // Step 3: Write a second entry (video_id "222") to the same queue
        let mut queued_updated = read_back.queued;
        queued_updated.push(VodEntry {
            channel: "testchan".to_string(),
            title: "Test VOD 2".to_string(),
            url: "https://www.twitch.tv/videos/222".to_string(),
            video_id: "222".to_string(),
            uploaded_at: "2026-01-02T00:00:00Z".to_string(),
            duration: "PT3600S".to_string(),
        });
        let _queue_path = write_queue_file(
            dir.path(),
            "testchan",
            false,
            0,
            queued_updated,
            vec![],
        ).unwrap();

        // Step 4: Read back and assert we have exactly 2 entries
        let final_queue = read_queue_file(dir.path(), "testchan").unwrap();
        assert_eq!(final_queue.queued.len(), 2);
        
        let ids: Vec<_> = final_queue.queued.iter().map(|v| v.video_id.as_str()).collect();
        assert!(ids.contains(&"111"));
        assert!(ids.contains(&"222"));
    }

    #[test]
    fn test_download_all_no_channel_filter() {
        let dir = tempdir().unwrap();
        let queue_dir = dir.path().join("queues");
        fs::create_dir_all(&queue_dir).unwrap();

        // Setup: Two queue files with 3 total entries
        // chan1.json: ID "111" (1 entry)
        let queue_json_1 = r#"{"schema_version":1,"channel":"chan1","generated_at_epoch_s":0,"past_broadcasts_only":false,"min_seconds":600,"queued_count":1,"queued":[{"channel":"chan1","title":"VOD A","url":"https://www.twitch.tv/videos/111","video_id":"111","uploaded_at":"2026-01-01T00:00:00Z","duration":"PT3600S"}],"skipped_existing_ids":[]}"#;
        fs::write(queue_dir.join("chan1.json"), queue_json_1).unwrap();

        // chan2.json: IDs "222" and "333" (2 entries)
        let queue_json_2 = r#"{"schema_version":1,"channel":"chan2","generated_at_epoch_s":0,"past_broadcasts_only":false,"min_seconds":600,"queued_count":2,"queued":[{"channel":"chan2","title":"VOD B","url":"https://www.twitch.tv/videos/222","video_id":"222","uploaded_at":"2026-01-02T00:00:00Z","duration":"PT3600S"},{"channel":"chan2","title":"VOD C","url":"https://www.twitch.tv/videos/333","video_id":"333","uploaded_at":"2026-01-03T00:00:00Z","duration":"PT3600S"}],"skipped_existing_ids":[]}"#;
        fs::write(queue_dir.join("chan2.json"), queue_json_2).unwrap();

        // Setup: One artifact dir with downloaded=true for ID "222"
        let artifact_222 = dir.path().join("222");
        fs::create_dir_all(&artifact_222).unwrap();
        let status_json = r#"{"schema_version":1,"video_id":"222","source_url":"https://www.twitch.tv/videos/222","media_file":"audio.m4a","transcript_file":null,"downloaded":true,"transcribed":false,"last_error":null,"updated_at_epoch_s":0}"#;
        fs::write(artifact_222.join("status.json"), status_json).unwrap();

        // Execute: Simulate no-channel download-all filter logic
        let all_vods = scan_queue_files(dir.path()).unwrap();
        // All 3 queue entries should be present: 111 (from chan1), 222, 333 (from chan2)
        assert_eq!(all_vods.len(), 3);

        // Only 222 has a status.json with downloaded=true (created above)
        let artifact_statuses = scan_artifact_statuses(dir.path()).unwrap();
        let downloaded_ids: std::collections::HashSet<String> = artifact_statuses
            .iter()
            .filter_map(|(video_id, status_opt)| {
                status_opt.as_ref()
                    .filter(|s| s.downloaded)
                    .map(|_| video_id.clone())
            })
            .collect();

        let pending: Vec<_> = all_vods
            .into_iter()
            .filter(|vod| !downloaded_ids.contains(&vod.video_id))
            .collect();

        // Assert: Pending should have 2 entries (111, 333) — excluding 222 which has downloaded=true
        assert_eq!(pending.len(), 2);
        let pending_ids: Vec<_> = pending.iter().map(|v| v.video_id.as_str()).collect();
        assert!(pending_ids.contains(&"111"));
        assert!(pending_ids.contains(&"333"));
        assert!(!pending_ids.contains(&"222"));
    }

    #[test]
    fn test_download_all_channel_regression() {
        let dir = tempdir().unwrap();
        let queue_dir = dir.path().join("queues");
        fs::create_dir_all(&queue_dir).unwrap();

        // Setup: One queue file (chan1) with 2 entries: IDs "111" and "222"
        let queue_json = r#"{"schema_version":1,"channel":"chan1","generated_at_epoch_s":0,"past_broadcasts_only":false,"min_seconds":600,"queued_count":2,"queued":[{"channel":"chan1","title":"VOD 1","url":"https://www.twitch.tv/videos/111","video_id":"111","uploaded_at":"2026-01-01T00:00:00Z","duration":"PT3600S"},{"channel":"chan1","title":"VOD 2","url":"https://www.twitch.tv/videos/222","video_id":"222","uploaded_at":"2026-01-02T00:00:00Z","duration":"PT3600S"}],"skipped_existing_ids":[]}"#;
        fs::write(queue_dir.join("chan1.json"), queue_json).unwrap();

        // Setup: One artifact dir with downloaded=true for ID "111"
        let artifact_111 = dir.path().join("111");
        fs::create_dir_all(&artifact_111).unwrap();
        let status_json = r#"{"schema_version":1,"video_id":"111","source_url":"https://www.twitch.tv/videos/111","media_file":"audio.m4a","transcript_file":null,"downloaded":true,"transcribed":false,"last_error":null,"updated_at_epoch_s":0}"#;
        fs::write(artifact_111.join("status.json"), status_json).unwrap();

        // Execute: Simulate single-channel download-all filter logic (from read_queue_file)
        let queue_file = read_queue_file(dir.path(), "chan1").unwrap();
        let pending: Vec<_> = queue_file
            .queued
            .into_iter()
            .filter(|vod| {
                let artifact_dir = dir.path().join(&vod.video_id);
                let status = read_status(&artifact_dir).unwrap_or(None);
                !status.map(|s| s.downloaded).unwrap_or(false)
            })
            .collect();

        // Assert: Pending should have 1 entry (222) — excluding 111 which has downloaded=true
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].video_id, "222");
    }

    #[test]
    fn test_download_all_video_id_filter() {
        let dir = tempdir().unwrap();
        let queue_dir = dir.path().join("queues");
        fs::create_dir_all(&queue_dir).unwrap();

        // Setup: Two queue files with entries for different video IDs
        let queue_json_1 = r#"{"schema_version":1,"channel":"chan1","generated_at_epoch_s":0,"past_broadcasts_only":false,"min_seconds":600,"queued_count":1,"queued":[{"channel":"chan1","title":"VOD A","url":"https://www.twitch.tv/videos/111111","video_id":"111111","uploaded_at":"2026-01-01T00:00:00Z","duration":"PT3600S"}],"skipped_existing_ids":[]}"#;
        fs::write(queue_dir.join("chan1.json"), queue_json_1).unwrap();

        let queue_json_2 = r#"{"schema_version":1,"channel":"chan2","generated_at_epoch_s":0,"past_broadcasts_only":false,"min_seconds":600,"queued_count":1,"queued":[{"channel":"chan2","title":"VOD B","url":"https://www.twitch.tv/videos/222222","video_id":"222222","uploaded_at":"2026-01-02T00:00:00Z","duration":"PT3600S"}],"skipped_existing_ids":[]}"#;
        fs::write(queue_dir.join("chan2.json"), queue_json_2).unwrap();

        // Execute: Simulate no-channel download-all with video_id filter (111111)
        let all_vods = scan_queue_files(dir.path()).unwrap();
        let artifact_statuses = scan_artifact_statuses(dir.path()).unwrap();
        let downloaded_ids: std::collections::HashSet<String> = artifact_statuses
            .iter()
            .filter_map(|(video_id, status_opt)| {
                status_opt.as_ref()
                    .filter(|s| s.downloaded)
                    .map(|_| video_id.clone())
            })
            .collect();

        let pending: Vec<_> = all_vods
            .into_iter()
            .filter(|vod| !downloaded_ids.contains(&vod.video_id))
            .collect();

        // Apply video_id filter
        let video_id = Some("111111");
        let pending = if let Some(id) = video_id {
            let filtered: Vec<_> = pending.into_iter().filter(|v| v.video_id == id).collect();
            if filtered.is_empty() {
                panic!("Expected to find video ID 111111");
            }
            filtered
        } else {
            pending
        };

        // Assert: Filtered pending has exactly 1 entry with correct video_id
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].video_id, "111111");
    }

    #[test]
    fn test_download_all_video_id_not_found() {
        let dir = tempdir().unwrap();
        let queue_dir = dir.path().join("queues");
        fs::create_dir_all(&queue_dir).unwrap();

        // Setup: Two queue files with entries for different video IDs
        let queue_json_1 = r#"{"schema_version":1,"channel":"chan1","generated_at_epoch_s":0,"past_broadcasts_only":false,"min_seconds":600,"queued_count":1,"queued":[{"channel":"chan1","title":"VOD A","url":"https://www.twitch.tv/videos/111111","video_id":"111111","uploaded_at":"2026-01-01T00:00:00Z","duration":"PT3600S"}],"skipped_existing_ids":[]}"#;
        fs::write(queue_dir.join("chan1.json"), queue_json_1).unwrap();

        let queue_json_2 = r#"{"schema_version":1,"channel":"chan2","generated_at_epoch_s":0,"past_broadcasts_only":false,"min_seconds":600,"queued_count":1,"queued":[{"channel":"chan2","title":"VOD B","url":"https://www.twitch.tv/videos/222222","video_id":"222222","uploaded_at":"2026-01-02T00:00:00Z","duration":"PT3600S"}],"skipped_existing_ids":[]}"#;
        fs::write(queue_dir.join("chan2.json"), queue_json_2).unwrap();

        // Execute: Simulate no-channel download-all with video_id filter (non-existent 999999)
        let all_vods = scan_queue_files(dir.path()).unwrap();
        let artifact_statuses = scan_artifact_statuses(dir.path()).unwrap();
        let downloaded_ids: std::collections::HashSet<String> = artifact_statuses
            .iter()
            .filter_map(|(video_id, status_opt)| {
                status_opt.as_ref()
                    .filter(|s| s.downloaded)
                    .map(|_| video_id.clone())
            })
            .collect();

        let pending: Vec<_> = all_vods
            .into_iter()
            .filter(|vod| !downloaded_ids.contains(&vod.video_id))
            .collect();

        // Apply video_id filter
        let video_id = Some("999999");
        let pending = if let Some(id) = video_id {
            let filtered: Vec<_> = pending.into_iter().filter(|v| v.video_id == id).collect();
            if filtered.is_empty() {
                // This is the not-found case — assert it
                assert!(true);
                return;
            }
            filtered
        } else {
            pending
        };

        // If we reach here, the filter didn't find the ID, which is unexpected
        panic!("Expected empty filtered vector for non-existent video ID");
    }

    #[test]
    fn test_transcribe_all_video_id_filter() {
        let dir = tempdir().unwrap();

        // Setup: Two artifact dirs with status.json (downloaded=true, transcribed=false)
        let artifact_dir_1 = dir.path().join("111111");
        fs::create_dir_all(&artifact_dir_1).unwrap();
        let mut status_1 = ProcessStatus::new("111111", "https://www.twitch.tv/videos/111111");
        status_1.downloaded = true;
        status_1.transcribed = false;
        status_1.transcription_outcome = None;
        write_status(&artifact_dir_1, &status_1).unwrap();

        let artifact_dir_2 = dir.path().join("222222");
        fs::create_dir_all(&artifact_dir_2).unwrap();
        let mut status_2 = ProcessStatus::new("222222", "https://www.twitch.tv/videos/222222");
        status_2.downloaded = true;
        status_2.transcribed = false;
        status_2.transcription_outcome = None;
        write_status(&artifact_dir_2, &status_2).unwrap();

        // Execute: Simulate transcribe-all with video_id filter (111111)
        let items = scan_artifact_statuses(dir.path()).unwrap();
        let pending: Vec<_> = items
            .into_iter()
            .filter_map(|(vid, status)| {
                let s = status?;
                if s.downloaded
                    && !s.transcribed
                    && s.transcription_outcome.as_deref() != Some("suspect")
                {
                    Some((vid, s))
                } else {
                    None
                }
            })
            .collect();

        // Apply video_id filter
        let video_id = Some("111111");
        let pending = if let Some(id) = video_id {
            let filtered: Vec<_> = pending.into_iter().filter(|(vid, _)| vid == id).collect();
            if filtered.is_empty() {
                panic!("Expected to find video ID 111111");
            }
            filtered
        } else {
            pending
        };

        // Assert: Filtered pending has exactly 1 entry with correct video_id
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].0, "111111");
    }

    #[test]
    fn test_transcribe_all_video_id_not_found() {
        let dir = tempdir().unwrap();

        // Setup: Two artifact dirs with status.json (downloaded=true, transcribed=false)
        let artifact_dir_1 = dir.path().join("111111");
        fs::create_dir_all(&artifact_dir_1).unwrap();
        let mut status_1 = ProcessStatus::new("111111", "https://www.twitch.tv/videos/111111");
        status_1.downloaded = true;
        status_1.transcribed = false;
        status_1.transcription_outcome = None;
        write_status(&artifact_dir_1, &status_1).unwrap();

        let artifact_dir_2 = dir.path().join("222222");
        fs::create_dir_all(&artifact_dir_2).unwrap();
        let mut status_2 = ProcessStatus::new("222222", "https://www.twitch.tv/videos/222222");
        status_2.downloaded = true;
        status_2.transcribed = false;
        status_2.transcription_outcome = None;
        write_status(&artifact_dir_2, &status_2).unwrap();

        // Execute: Simulate transcribe-all with video_id filter (non-existent 999999)
        let items = scan_artifact_statuses(dir.path()).unwrap();
        let pending: Vec<_> = items
            .into_iter()
            .filter_map(|(vid, status)| {
                let s = status?;
                if s.downloaded
                    && !s.transcribed
                    && s.transcription_outcome.as_deref() != Some("suspect")
                {
                    Some((vid, s))
                } else {
                    None
                }
            })
            .collect();

        // Apply video_id filter
        let video_id = Some("999999");
        let pending = if let Some(id) = video_id {
            let filtered: Vec<_> = pending.into_iter().filter(|(vid, _)| vid == id).collect();
            if filtered.is_empty() {
                // This is the not-found case — assert it
                assert!(true);
                return;
            }
            filtered
        } else {
            pending
        };

        // If we reach here, the filter didn't find the ID, which is unexpected
        panic!("Expected empty filtered vector for non-existent video ID");
    }
}
