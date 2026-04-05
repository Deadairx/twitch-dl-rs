# S02: Decoupled staged processing

**Goal:** Add three new CLI commands — `status`, `download-all`, and `transcribe-all` — that decouple download progress from transcription progress. Status shows current artifact states; download-all reads the persisted queue and downloads pending items; transcribe-all scans for downloaded-but-not-transcribed artifacts and processes them. Interrupted work can be re-run safely because each command checks status.json before doing work.
**Demo:** After this: Downloads can continue making progress while transcription work remains pending, running, or failed, and interrupted work can be resumed.

## Tasks
- [x] **T01: Add QueueFile deserialization, read_queue_file helper, read_status wrapper, and status CLI command** — Two changes with zero risk of breaking existing behavior:

1. Make QueueFile and VodEntry deserializable so the persisted queue can be read back from disk. Both are currently Serialize-only. VodEntry is in src/twitch.rs; QueueFile is in src/artifact.rs.

2. Add read_queue_file() to src/artifact.rs — reads queues/<channel>.json, deserializes it, returns the VodEntry list.

3. Add status command to cli.rs and main.rs — scans all numeric artifact dirs under output_root, reads status.json for each, prints a human-readable table.

4. Add unit tests in src/artifact.rs for read_queue_file deserialization and for the status-scanning logic.

## Steps

1. In src/twitch.rs, add `Deserialize` to VodEntry's derive list: `#[derive(Debug, Clone, Serialize, Deserialize)]`.

2. In src/artifact.rs, add `Deserialize` to QueueFile's derive list: `#[derive(Debug, Serialize, Deserialize)]`. Add a pub read_queue_file() fn:
```rust
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
```

3. Add a pub scan_artifact_statuses() fn to src/artifact.rs that reads all numeric artifact dirs and their status.json files:
```rust
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
```

4. Add CliCommand::Status { output_root: PathBuf } to src/cli.rs.
   Add the 'status' subcommand to the clap tree with --output-root (default: 'artifacts').
   Add the match arm in parse_args().

5. Add the status handler in src/main.rs:
```rust
async fn show_status(output_root: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let items = artifact::scan_artifact_statuses(output_root)?;
    if items.is_empty() {
        println!("No artifacts found in {}", output_root.display());
        return Ok(());
    }
    println!("{:<15} {:<12} {:<12} {}", "VIDEO_ID", "DOWNLOADED", "TRANSCRIBED", "LAST_ERROR");
    println!("{}", "-".repeat(70));
    for (video_id, status) in &items {
        match status {
            Some(s) => {
                let last_err = s.last_error.as_deref().unwrap_or("-");
                let truncated = if last_err.len() > 40 { &last_err[..40] } else { last_err };
                println!("{:<15} {:<12} {:<12} {}", video_id, s.downloaded, s.transcribed, truncated);
            }
            None => println!("{:<15} {:<12} {:<12} {}", video_id, "(no status)", "-", "-"),
        }
    }
    println!("\n{} artifact(s) total", items.len());
    Ok(())
}
```
   Wire it into the match in main(): `cli::CliCommand::Status { output_root } => { if let Err(e) = show_status(&output_root).await { ... } }`

6. Add unit tests in src/artifact.rs:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
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
}
```
Note: add `tempfile` to Cargo.toml dev-dependencies if not present.
  - Estimate: 45m
  - Files: src/twitch.rs, src/artifact.rs, src/cli.rs, src/main.rs, Cargo.toml
  - Verify: cargo test artifact::tests && cargo build 2>&1 | grep -v 'warning' | head -20
- [ ] **T02: Extract download/transcribe helpers and add download-all and transcribe-all commands** — Refactor process_vod() into two composable stage helpers, then add download-all and transcribe-all commands that call them. The existing process command must continue to work.

## Steps

1. In src/main.rs, extract the download stage from process_vod() into a standalone async fn:
```rust
async fn download_vod_to_artifact(
    vod: &twitch::VodEntry,
    output_root: &std::path::Path,
    quality: cli::QualityPreference,
    status: &mut artifact::ProcessStatus,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let artifact_dir = output_root.join(&vod.video_id);
    if let Some(existing) = artifact::find_media_file(&artifact_dir) {
        println!("Reusing existing media for {}", vod.video_id);
        status.downloaded = true;
        status.media_file = existing.file_name().map(|n| n.to_string_lossy().to_string());
        return Ok(existing);
    }
    println!("Downloading {} | {}", vod.video_id, vod.title);
    let downloaded_dir = download_vod(&vod.url, None, output_root, quality).await?;
    let media_path = artifact::find_media_file(&downloaded_dir)
        .ok_or_else(|| format!("missing media file after download for {}", vod.video_id))?;
    status.downloaded = true;
    status.media_file = media_path.file_name().map(|n| n.to_string_lossy().to_string());
    status.last_error = None;
    status.updated_at_epoch_s = now_epoch_s();
    artifact::write_status(&downloaded_dir, status)?;
    Ok(media_path)
}
```

2. Extract the transcription stage into:
```rust
fn transcribe_artifact(
    video_id: &str,
    artifact_dir: &std::path::Path,
    media_path: &std::path::Path,
    status: &mut artifact::ProcessStatus,
) -> Result<(), Box<dyn std::error::Error>> {
    let transcript_path = artifact_dir.join("transcript.txt");
    if transcript_path.exists() {
        println!("Reusing existing transcript for {}", video_id);
        status.transcribed = true;
        status.transcript_file = Some("transcript.txt".to_string());
        status.last_error = None;
        status.updated_at_epoch_s = now_epoch_s();
        artifact::write_status(artifact_dir, status)?;
        return Ok(());
    }
    println!("Transcribing {} with mlx-whisper...", video_id);
    match transcribe::transcribe_to_txt(media_path, artifact_dir) {
        Ok(path) => {
            status.transcribed = true;
            status.transcript_file = path.file_name().map(|n| n.to_string_lossy().to_string());
            status.last_error = None;
        }
        Err(error) => {
            status.last_error = Some(error.to_string());
            status.updated_at_epoch_s = now_epoch_s();
            artifact::write_status(artifact_dir, status)?;
            return Err(Box::new(error));
        }
    }
    status.updated_at_epoch_s = now_epoch_s();
    artifact::write_status(artifact_dir, status)?;
    Ok(())
}
```

3. Rewrite process_vod() to delegate to the two helpers:
```rust
async fn process_vod(
    vod: &twitch::VodEntry,
    output_root: &std::path::Path,
    quality: cli::QualityPreference,
) -> Result<(), Box<dyn std::error::Error>> {
    let artifact_dir = output_root.join(&vod.video_id);
    artifact::prepare_artifact_dir(&artifact_dir)?;
    let mut status = artifact::read_status(&artifact_dir)?
        .unwrap_or_else(|| artifact::ProcessStatus::new(&vod.video_id, &vod.url));
    let media_path = download_vod_to_artifact(vod, output_root, quality, &mut status).await?;
    transcribe_artifact(&vod.video_id, &artifact_dir, &media_path, &mut status)?;
    Ok(())
}
```
Verify that process_channel still compiles and the behavior is unchanged — process_vod() now delegates rather than reimplementing.

4. Add download_all() top-level fn:
```rust
async fn download_all(
    channel: &str,
    output_root: &std::path::Path,
    quality: cli::QualityPreference,
    continue_on_error: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let queue_file = artifact::read_queue_file(output_root, channel)?;
    let pending: Vec<_> = queue_file.queued.into_iter().filter(|vod| {
        let artifact_dir = output_root.join(&vod.video_id);
        let status = artifact::read_status(&artifact_dir).unwrap_or(None);
        !status.map(|s| s.downloaded).unwrap_or(false)
    }).collect();

    if pending.is_empty() {
        println!("All queued VODs already downloaded.");
        return Ok(());
    }
    println!("Downloading {} pending VOD(s) for {channel}...", pending.len());
    for vod in pending {
        let artifact_dir = output_root.join(&vod.video_id);
        artifact::prepare_artifact_dir(&artifact_dir)?;
        let mut status = artifact::read_status(&artifact_dir)?
            .unwrap_or_else(|| artifact::ProcessStatus::new(&vod.video_id, &vod.url));
        match download_vod_to_artifact(&vod, output_root, quality, &mut status).await {
            Ok(_) => println!("Downloaded {}", vod.video_id),
            Err(e) => {
                eprintln!("Failed {}: {e}", vod.video_id);
                if !continue_on_error { return Err(e); }
            }
        }
    }
    Ok(())
}
```

5. Add transcribe_all() top-level fn:
```rust
async fn transcribe_all(
    output_root: &std::path::Path,
    continue_on_error: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let items = artifact::scan_artifact_statuses(output_root)?;
    let pending: Vec<_> = items.into_iter().filter_map(|(video_id, status)| {
        let s = status?;
        if s.downloaded && !s.transcribed { Some((video_id, s)) } else { None }
    }).collect();

    if pending.is_empty() {
        println!("No artifacts pending transcription.");
        return Ok(());
    }
    println!("Transcribing {} artifact(s)...", pending.len());
    for (video_id, mut status) in pending {
        let artifact_dir = output_root.join(&video_id);
        let media_path = artifact::find_media_file(&artifact_dir)
            .ok_or_else(|| format!("media file missing for {} despite downloaded=true", video_id))?;
        match transcribe_artifact(&video_id, &artifact_dir, &media_path, &mut status) {
            Ok(()) => println!("Transcribed {}", video_id),
            Err(e) => {
                eprintln!("Failed {}: {e}", video_id);
                if !continue_on_error { return Err(e); }
            }
        }
    }
    Ok(())
}
```

6. Add CliCommand::DownloadAll and CliCommand::TranscribeAll variants to src/cli.rs:
   - DownloadAll { channel: String, output_root: PathBuf, quality: QualityPreference, continue_on_error: bool }
   - TranscribeAll { output_root: PathBuf, continue_on_error: bool }
   Add subcommands 'download-all' and 'transcribe-all' to the clap tree.
   Add match arms in parse_args().

7. Wire both commands in main():
   - CliCommand::DownloadAll => download_all()
   - CliCommand::TranscribeAll => transcribe_all()

8. Run `cargo build` — must compile cleanly. Run `cargo test artifact::tests` — all 3 tests must pass. Confirm that `cargo run -- process` still compiles and the help text now shows all 5 subcommands.
  - Estimate: 60m
  - Files: src/main.rs, src/cli.rs
  - Verify: cargo build && cargo test artifact::tests && cargo run -- --help 2>&1 | grep -E 'download-all|transcribe-all|status'
