---
estimated_steps: 158
estimated_files: 2
skills_used: []
---

# T02: Extract download/transcribe helpers and add download-all and transcribe-all commands

Refactor process_vod() into two composable stage helpers, then add download-all and transcribe-all commands that call them. The existing process command must continue to work.

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

## Inputs

- ``src/main.rs``
- ``src/cli.rs``
- ``src/artifact.rs``

## Expected Output

- ``src/main.rs``
- ``src/cli.rs``

## Verification

cargo build && cargo test artifact::tests && cargo run -- --help 2>&1 | grep -E 'download-all|transcribe-all|status'
