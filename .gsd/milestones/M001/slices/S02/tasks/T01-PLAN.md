---
estimated_steps: 104
estimated_files: 5
skills_used: []
---

# T01: Add QueueFile deserialization, read_queue_file helper, and status command

Two changes with zero risk of breaking existing behavior:

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

## Inputs

- ``src/twitch.rs``
- ``src/artifact.rs``
- ``src/cli.rs``
- ``src/main.rs``
- ``Cargo.toml``

## Expected Output

- ``src/twitch.rs``
- ``src/artifact.rs``
- ``src/cli.rs``
- ``src/main.rs``
- ``Cargo.toml``

## Verification

cargo test artifact::tests && cargo build 2>&1 | grep -v 'warning' | head -20
