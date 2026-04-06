---
id: S03
milestone: M002-z48awz
status: ready
---

# S03: Intake Flexibility — Context

## Goal

Add `queue-video <url>` for single-VOD intake and make `download-all` channel-argument optional so it drains all queues without requiring a channel name.

## Why this Slice

The current intake model requires the operator to know and spell out a channel name for both `queue` and `download-all`. There's no way to queue a single video by URL. S03 removes both friction points: `queue-video` handles ad-hoc single intake, and `download-all` with no arg handles the "just drain everything" workflow. S04 (Selective Processing) depends on S03 being done because `download-all` without a channel is the base command S04 adds `--video-id` filtering to.

## Scope

### In Scope

- `queue-video <url>` command — resolves video ID from URL, calls GQL to get channel name, merges entry into `queues/<channel>.json` (creates the file if it doesn't exist)
- If the video ID is already present in the queue file: print `"Already queued: <id>"` and exit 0 — idempotent, safe to re-run
- Make `channel` argument on `download-all` optional — when omitted, walk all `queues/*.json` files and process each in turn
- Deduplication in no-channel `download-all` is artifact-state-based: if `status.json` shows `downloaded`, skip regardless of which queue file the entry came from — same logic as the existing single-channel path
- `cargo test` passes; `cargo build` succeeds

### Out of Scope

- `--video-id` filtering on `download-all` — that is S04
- `transcribe-all` changes — untouched in this slice
- Non-Twitch URL support for `queue-video` — that is S07
- Any status display changes — that is S02
- Concurrent access safety for queue files — queue files are written by one operator command at a time; no locking needed here

## Constraints

- `queue-video` channel inference is via the same GQL `fetch_vod_metadata_by_id` function added in S01 — no new API client needed
- If the GQL channel inference call fails, `queue-video` aborts with a clear error message. Every Twitch VOD has a channel so this is a defensive edge case, but it must not silently produce a malformed queue file
- The `QueueFile` struct and `read_queue_file` / `write_queue_file` patterns from `artifact.rs` must be reused — do not introduce a parallel queue write path
- `download-all` with a channel arg must behave identically to the current implementation — no regressions on the existing path

## Integration Points

### Consumes

- `src/twitch.rs` — `fetch_vod_metadata_by_id` (from S01) for channel inference in `queue-video`; `extract_video_id` for URL parsing
- `src/artifact.rs` — `QueueFile`, `read_queue_file`, existing queue write helpers; `read_status` for downloaded-check deduplication in no-channel `download-all`
- `src/cli.rs` — `download-all` channel arg made optional; new `queue-video` subcommand added

### Produces

- `src/cli.rs` — `queue-video <url>` subcommand; `channel` on `download-all` changed from required to optional
- `src/main.rs` — `queue_video(url, output_root)` async handler; `download_all` updated to accept `Option<&str>` for channel; no-channel path walks `queues/*.json` and calls existing per-vod download logic

## Open Questions

- None. All decisions resolved in discussion.
