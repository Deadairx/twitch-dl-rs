---
estimated_steps: 4
estimated_files: 3
skills_used: []
---

# T01: Add `queue-video` command

**Slice:** S03 — Intake Flexibility
**Milestone:** M002-z48awz

## Description

Add `queue-video <url>` as a new CLI subcommand. It resolves the video ID from the URL, fetches channel and display metadata via `fetch_vod_metadata_by_id`, reads the existing queue file for that channel (if any), deduplicates by video_id, appends the new entry, and writes back using `write_queue_file`. Idempotent: if the video_id is already present, print `"Already queued: <id>"` and exit 0.

Key implementation notes:
- `VodEntry.duration` is not returned by the metadata GQL call — use `"PT0S"` as a placeholder. Duration is only used by the `min_seconds` filter in `build_queue`, which `queue-video` bypasses entirely.
- `QueueFile.past_broadcasts_only`, `min_seconds`, and `skipped_existing_ids` are private fields with no public accessor. Accept their defaults (`false`, `0`, `vec![]`) when calling `write_queue_file` — these are channel-queue-generation filters irrelevant to single-video ad-hoc intake. Downstream `download-all` only reads `QueueFile.queued`, which is `pub`.
- GQL failure must abort with a clear `eprintln!` message. Never silently write a malformed queue entry.

## Failure Modes

| Dependency | On error | On timeout | On malformed response |
|------------|----------|-----------|----------------------|
| `fetch_vod_metadata_by_id` GQL call | `eprintln!("Failed to resolve VOD metadata: {e}")`, return `Err`, exit nonzero | Same — reqwest timeout surfaces as an error variant | GQL returns null video node → `TwitchError::Parse` → same error path |
| `read_queue_file` (existing file) | On `Err` (file missing or malformed), treat as empty queue (`vec![]`) — idempotent safe to start fresh | N/A — local filesystem | On malformed JSON, start fresh |
| `write_queue_file` | Surface error, abort, exit nonzero | N/A — local filesystem | N/A |

## Negative Tests

- **Malformed inputs**: `extract_video_id` returns `TwitchError` for non-Twitch URLs — `queue-video` must surface this error, not panic
- **Error paths**: GQL fetch failure → abort with error message, queue file unchanged
- **Boundary conditions**: video_id already in queue → print `"Already queued: <id>"`, exit 0, queue file entry count unchanged

## Steps

1. **`src/cli.rs`** — Add `QueueVideo { url: String, output_root: PathBuf }` variant to `CliCommand`. Add the `queue-video` subcommand definition in `parse_args`: positional `url` arg (required, index 1), `--output-root` (reuse `output_root_arg`). Add the `Some(("queue-video", ...))` match arm in `parse_args` to produce the variant.

2. **`src/main.rs`** — Add `async fn queue_video(url: &str, output_root: &std::path::Path) -> Result<(), Box<dyn std::error::Error>>`. Implementation:
   - Call `twitch::extract_video_id(url)?` to get `video_id`
   - Call `twitch::fetch_vod_metadata_by_id(&video_id).await` — on error, `eprintln!` and return `Err`
   - Call `artifact::read_queue_file(output_root, &channel)` — on error (file missing), start with `vec![]`
   - Check `queued.iter().any(|v| v.video_id == video_id)` — if true, `println!("Already queued: {video_id}")` and return `Ok(())`
   - Push new `VodEntry` with `duration: "PT0S".to_string()`
   - Call `artifact::write_queue_file(output_root, &channel, false, 0, queued, vec![])` — return error if it fails
   - `println!("Queued {video_id} into {}", path.display())`

3. **`src/main.rs`** — Add `CliCommand::QueueVideo { url, output_root }` match arm in `main` that calls `queue_video(&url, &output_root).await`, `eprintln!` on error, `std::process::exit(1)`.

4. **`src/artifact.rs` `#[cfg(test)]` module** — Add unit test `test_queue_video_idempotent_dedup` using `tempfile::TempDir`. Write a `queues/testchan.json` with one `VodEntry` (video_id `"111"`) via `write_queue_file`. Read it back with `read_queue_file`, check `.iter().any(|v| v.video_id == "111")` returns true (dedup would trigger). Then write a second VodEntry (video_id `"222"`) via write_queue_file with two entries, read back, assert two entries. This tests the underlying read/write/dedup logic used by the handler. Note: this is a binary-only crate with no `src/lib.rs` — the async `queue_video` handler in `main.rs` cannot be directly unit-tested; test the data-layer logic instead.

## Must-Haves

- [ ] `queue-video --help` shows positional `<URL>` argument and `--output-root` option
- [ ] `queue-video <url>` with a duplicate video_id prints `"Already queued: <id>"` and exits 0
- [ ] `queue-video <url>` with a new video_id writes the entry into `queues/<channel>.json`
- [ ] GQL failure produces a clear error message and exits nonzero
- [ ] `cargo build` clean; `cargo test` passes (21 existing + new dedup test)

## Verification

```bash
cargo build
cargo test
./target/debug/vod-pipeline queue-video --help
# Confirm <URL> positional arg visible in help output
```

New test target: `cargo test test_queue_video`

## Observability Impact

- Signals added/changed: GQL metadata fetch failure is surfaced via `eprintln!` and nonzero exit
- How a future agent inspects this: `./target/debug/vod-pipeline queue-video <url>` — error message on stderr on GQL failure; success message shows video_id and queue file path
- Failure state exposed: queue file is unchanged on any failure path (GQL error, write_queue_file error)

## Inputs

- `src/cli.rs` — current `CliCommand` enum and `parse_args` to extend
- `src/main.rs` — `main` match dispatch and existing async handler pattern to follow
- `src/artifact.rs` — `read_queue_file`, `write_queue_file`, `QueueFile`, `VodEntry` (public API)
- `src/twitch.rs` — `extract_video_id`, `fetch_vod_metadata_by_id`, `VodEntry` struct

## Expected Output

- `src/cli.rs` — `QueueVideo` variant added; `queue-video` subcommand definition added; match arm producing the variant added
- `src/main.rs` — `queue_video` async fn added; `CliCommand::QueueVideo` match arm added
- `src/artifact.rs` — `test_queue_video_idempotent_dedup` unit test added to existing `#[cfg(test)]` module
