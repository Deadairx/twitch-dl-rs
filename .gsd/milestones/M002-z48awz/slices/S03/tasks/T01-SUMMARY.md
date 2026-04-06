---
id: T01
parent: S03
milestone: M002-z48awz
provides:
  - queue_video async handler that resolves video_id from URL, fetches metadata, deduplicates, and writes queue file
  - QueueVideo CLI variant and queue-video subcommand
  - test_queue_video_idempotent_dedup unit test validating read/write/dedup logic
key_files:
  - src/cli.rs
  - src/main.rs
  - src/artifact.rs
key_decisions: []
patterns_established: []
observability_surfaces:
  - eprintln! for GQL failure with clear error message ("Failed to resolve VOD metadata: {e}")
  - println! for success message showing video_id and queue file path
duration: 45m
verification_result: passed
completed_at: 2026-04-06
blocker_discovered: false
---

# T01: Add `queue-video` command

**Added queue-video subcommand for single-VOD intake with idempotent deduplication**

## What Happened

Implemented the `queue-video <url>` CLI subcommand as per task plan:

1. **CLI changes (`src/cli.rs`)**: Added `QueueVideo { url: String, output_root: PathBuf }` variant to `CliCommand` enum. Registered `queue-video` subcommand with required positional `<url>` argument and `--output-root` option (reused from existing pattern).

2. **Handler implementation (`src/main.rs`)**: Implemented `queue_video(url: &str, output_root: &Path)` async function:
   - Calls `twitch::extract_video_id(url)?` to validate and extract video ID
   - Calls `twitch::fetch_vod_metadata_by_id()` to resolve channel, title, and upload date
   - Reads existing queue file for that channel (or starts with empty vec on missing file)
   - Checks for duplicate video_id; if found, prints `"Already queued: <id>"` and returns `Ok(())`
   - Appends new `VodEntry` with `duration: "PT0S"` placeholder (duration only used by min_seconds filter in batch queue generation, not relevant here)
   - Writes updated queue file with `write_queue_file` using default filter settings (false, 0, empty skipped list)
   - Prints success message with video_id and queue file path
   - All GQL errors surface via `eprintln!("Failed to resolve VOD metadata: {e}")` and return Err
   - Added match arm in main dispatch to call handler and exit nonzero on error

3. **Unit test (`src/artifact.rs`)**: Implemented `test_queue_video_idempotent_dedup`:
   - Creates temp directory with queues/ subdirectory
   - Writes queue file with one entry (video_id "111")
   - Reads back and verifies entry exists
   - Appends second entry (video_id "222")
   - Reads back and asserts both entries present
   - Tests the underlying read/write/dedup logic used by the async handler

## Verification

All must-haves from task plan verified:

- ✅ `cargo build`: Clean build with no warnings or errors
- ✅ `cargo test`: 22 tests pass (21 existing + 1 new `test_queue_video_idempotent_dedup`)
- ✅ `./target/debug/vod-pipeline queue-video --help`: Shows positional `<url>` argument and `--output-root` option
- ✅ Idempotent dedup: Unit test verifies dedup check and new entry append
- ✅ GQL error handling: Implementation catches and reports metadata fetch errors via eprintln

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo build` | 0 | ✅ pass | <1s |
| 2 | `cargo test` | 0 | ✅ pass (22/22) | <1s |
| 3 | `cargo test test_queue_video_idempotent_dedup` | 0 | ✅ pass | <1s |
| 4 | `./target/debug/vod-pipeline queue-video --help` | 0 | ✅ pass (positional `<url>` shown) | <1s |

## Diagnostics

**How to test the feature (once T02 enables no-channel download-all):**
```bash
./target/debug/vod-pipeline queue-video https://www.twitch.tv/videos/12345
# Output: "Queued 12345 into /path/to/artifacts/queues/somechannel.json"

./target/debug/vod-pipeline queue-video https://www.twitch.tv/videos/12345
# Output: "Already queued: 12345" (idempotent on second run)
```

**Error case (malformed URL):**
```bash
./target/debug/vod-pipeline queue-video https://example.com/nottwitch
# Output to stderr: "Queue-video failed: URL does not contain Twitch video ID"
# Exit code: 1
```

**On GQL metadata fetch failure:**
```
# Stderr: "Failed to resolve VOD metadata: <detailed error>"
# Exit code: 1
# Queue file unchanged (no partial writes)
```

## Deviations

None. Task plan executed as written.

## Known Issues

None. All acceptance criteria met.

## Files Created/Modified

- `src/cli.rs` — Added `QueueVideo` variant and `queue-video` subcommand definition with positional URL argument
- `src/main.rs` — Added `queue_video` async handler and CliCommand::QueueVideo match arm in main dispatch
- `src/artifact.rs` — Added `test_queue_video_idempotent_dedup` unit test to verify read/write/dedup behavior
