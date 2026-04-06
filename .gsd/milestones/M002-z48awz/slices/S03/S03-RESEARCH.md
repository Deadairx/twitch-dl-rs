# S03: Intake Flexibility — Research

**Date:** 2026-04-06
**Slice:** Add `queue-video <url>` and make `download-all` channel-arg optional

---

## Summary

S03 is light research. All required building blocks exist in the codebase and the patterns are established. Two changes are needed: (1) a new `queue-video <url>` CLI subcommand that resolves a VOD's channel via `fetch_vod_metadata_by_id`, then writes the entry into `queues/<channel>.json` using the existing `read_queue_file`/`write_queue_file` helpers; and (2) making the `channel` argument on `download-all` optional, with the no-arg path walking all `queues/*.json` files via the existing `scan_queue_files` helper.

Both changes wire known functions together — no new abstractions, no new API calls, no new file formats. The deduplication logic for no-channel `download-all` mirrors exactly what `show_status` already does: collect artifact IDs into a HashSet, filter pending items by absence. Every function this slice needs (`extract_video_id`, `fetch_vod_metadata_by_id`, `read_queue_file`, `write_queue_file`, `scan_queue_files`, `read_status`, `download_vod_to_artifact`) is already public and in use.

---

## Recommendation

Two tasks, built in order:

**T01 — `queue-video` command:** Add `CliCommand::QueueVideo { url: String, output_root: PathBuf }` to `cli.rs`; add the clap subcommand definition; implement `queue_video(url, output_root)` async handler in `main.rs`. The handler calls `extract_video_id`, then `fetch_vod_metadata_by_id` to get `(title, channel, uploaded_at)`, constructs a `VodEntry`, reads the existing queue file for that channel (if any), deduplicates by video_id, appends the new entry, and writes back using `write_queue_file`. On duplicate: print `"Already queued: <id>"` and exit 0.

**T02 — optional channel on `download-all`:** Change `CliCommand::DownloadAll.channel` from `String` to `Option<String>`. In clap, change the `channel` positional arg from `required(true)` to `required(false)`. In `main.rs`, update `download_all` signature to `Option<&str>`. When `None`, call `scan_queue_files(output_root)` to collect all entries from all queue files, deduplicate against artifact statuses using HashSet (same pattern as `show_status`), and iterate with the existing `download_vod_to_artifact` helper. When `Some(channel)`, keep existing behavior (read single queue file, filter by `downloaded` status). The existing-channel path must be unchanged — no regressions.

---

## Implementation Landscape

### Key Files

- `src/cli.rs` — Add `QueueVideo` variant to `CliCommand`; add `queue-video` subcommand definition; change `DownloadAll.channel` from `String` to `Option<String>` and update both the clap definition and the match arm.
- `src/main.rs` — Add `queue_video(url, output_root)` async fn; update `download_all` to accept `Option<&str>` and add the no-channel walk path; update the `CliCommand::DownloadAll` match arm to pass `channel.as_deref()`.
- `src/artifact.rs` — No changes needed. All required helpers (`read_queue_file`, `write_queue_file`, `scan_queue_files`, `read_status`) are already public.
- `src/twitch.rs` — No changes needed. `extract_video_id` and `fetch_vod_metadata_by_id` are already public.

### Build Order

T01 first (`queue-video`), T02 second (`download-all` optional channel). T01 has zero dependency on T02. T02 depends on the no-channel path needing something to drain — completing T01 first makes the integration test story coherent.

### queue_video Handler Pattern

```rust
// In main.rs
async fn queue_video(
    url: &str,
    output_root: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let video_id = twitch::extract_video_id(url)?;

    // Resolve channel and metadata via GQL
    let (title, channel, uploaded_at) = match twitch::fetch_vod_metadata_by_id(&video_id).await {
        Ok(meta) => meta,
        Err(e) => {
            eprintln!("Failed to resolve VOD metadata: {e}");
            return Err(format!("GQL metadata fetch failed: {e}").into());
        }
    };

    // Read existing queue file for this channel (may not exist yet)
    let mut queued: Vec<twitch::VodEntry> = match artifact::read_queue_file(output_root, &channel) {
        Ok(qf) => qf.queued,
        Err(_) => vec![],  // no file yet — start fresh
    };

    // Idempotent dedup
    if queued.iter().any(|v| v.video_id == video_id) {
        println!("Already queued: {video_id}");
        return Ok(());
    }

    // Append and write back
    queued.push(twitch::VodEntry {
        channel: channel.clone(),
        title,
        url: url.to_string(),
        video_id: video_id.clone(),
        uploaded_at,
        duration: "PT0S".to_string(), // unknown; not needed for download
    });

    let path = artifact::write_queue_file(
        output_root,
        &channel,
        false,       // past_broadcasts_only — unknown for single-video intake
        0,           // min_seconds — not applicable
        queued,
        vec![],
    )?;

    println!("Queued {video_id} into {}", path.display());
    Ok(())
}
```

Note: `write_queue_file` overwrites the entire file. Preserving existing `past_broadcasts_only` / `min_seconds` / `skipped_existing_ids` from the current file requires reading the `QueueFile` struct fields directly — the struct fields are private. **This is the only constraint worth checking:** currently `QueueFile.past_broadcasts_only`, `QueueFile.min_seconds`, and `QueueFile.skipped_existing_ids` are not `pub`. The planner should decide: either (a) make the fields pub so `queue_video` can read and preserve them, or (b) accept that `queue_video` resets those fields to defaults (false / 0 / vec![]). Option (b) is acceptable because `past_broadcasts_only` and `min_seconds` are queue-generation filters that only matter for `queue` (not `queue-video`), and the downstream consumer (`download-all`) only reads `QueueFile.queued`, which is already pub.

### download-all No-Channel Path Pattern

```rust
async fn download_all(
    channel: Option<&str>,
    output_root: &std::path::Path,
    quality: cli::QualityPreference,
    continue_on_error: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    match channel {
        Some(ch) => {
            // existing single-channel path — unchanged
            let queue_file = artifact::read_queue_file(output_root, ch)?;
            let pending = filter_pending(queue_file.queued, output_root);
            download_pending(pending, ch, output_root, quality, continue_on_error).await
        }
        None => {
            // walk all queue files
            let all_vods = artifact::scan_queue_files(output_root)?;
            let artifact_ids: std::collections::HashSet<_> = {
                artifact::scan_artifact_statuses(output_root)?
                    .into_iter()
                    .filter_map(|(id, status)| {
                        if status.map(|s| s.downloaded).unwrap_or(false) {
                            Some(id)
                        } else {
                            None
                        }
                    })
                    .collect()
            };
            let pending: Vec<_> = all_vods
                .into_iter()
                .filter(|v| !artifact_ids.contains(&v.video_id))
                .collect();
            // ... iterate pending with download_vod_to_artifact
        }
    }
}
```

Deduplication rule: filter out any `video_id` whose artifact dir has `status.downloaded == true`. This is the same logic as the existing single-channel path — just scoped to all queue files rather than one.

### Verification

```bash
cargo build        # must pass clean
cargo test         # 21 existing tests must pass; new unit tests for queue_video dedup and no-channel path should be added
./target/debug/vod-pipeline queue-video --help   # subcommand and positional url arg visible
./target/debug/vod-pipeline download-all --help  # channel arg shown as [CHANNEL], not <CHANNEL>
```

Unit test targets:
- `queue_video` idempotent dedup: write a queue file with an existing entry, call handler logic, verify count unchanged and "Already queued" message.
- `download-all` no-channel filter: set up two queue files (3 total entries, 1 with downloaded artifact), verify pending has 2 entries.
- `download-all` with channel arg: verify single-queue path unchanged (regression guard).

### Existing Constraints / Gotchas

- **`QueueFile` private fields:** `past_broadcasts_only`, `min_seconds`, `skipped_existing_ids`, `channel`, `generated_at_epoch_s`, `queued_count`, `schema_version` are all private (no `pub`). Only `queued: Vec<VodEntry>` is `pub`. The `queue_video` handler needs to call `write_queue_file` with those fields — and must either accept that they reset to defaults or require the planner to expose those fields via pub or a constructor. **Recommending**: accept defaults for queue-video writes. The `write_queue_file` signature already takes all values; pass sensible defaults. This is consistent with the S03 scope (queue-video is ad-hoc single-video intake, not a channel queue refresh).

- **`VodEntry.duration` for queue-video:** Twitch GQL `fetch_vod_metadata_by_id` returns `(title, channel, uploaded_at)` — no duration. The `VodEntry` struct requires `duration: String`. Use `"PT0S"` as a placeholder. Duration is used only in `build_queue`'s `min_seconds` filter — which `queue-video` bypasses entirely. No downstream code reads VodEntry duration after queueing.

- **No new API client needed:** All GQL calls use the same unauthenticated public Client-ID already in `twitch.rs`. `queue-video` uses `fetch_vod_metadata_by_id` exactly as specified in the S03 scope.

- **Existing channel path in `download-all` must be regression-safe:** The match arm for `Some(channel)` wraps the existing read_queue_file → filter → iterate logic unchanged. The planner should keep that path isolated to avoid introducing regressions.
