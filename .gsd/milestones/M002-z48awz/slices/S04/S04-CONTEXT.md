---
id: S04
milestone: M002-z48awz
status: ready
---

# S04: Selective Processing — Context

## Goal

Add `--video-id <id>` filtering to both `download-all` and `transcribe-all` so the operator can target a single item without processing the entire queue.

## Why this Slice

After S03, `download-all` can drain all queues but has no way to target a specific item. S04 adds that precision. It unblocks S05 (Queue-Aware Filtering), which depends on both S02 and S04 being complete before adding `--filter` to the status command.

## Scope

### In Scope

- `--video-id <id>` flag on `download-all` — when provided, only the matching item is processed; all others are skipped
- `--video-id <id>` flag on `transcribe-all` — same behaviour: only the matching artifact is transcribed
- If `--video-id` is provided and the ID is not found in any queue (for `download-all`) or any artifact dir (for `transcribe-all`): exit non-zero with a clear message, e.g. `"video ID 123456789 not found in any queue"`
- `cargo test` passes; `cargo build` succeeds

### Out of Scope

- `--force-suspect` on `transcribe-all` — that is S06
- `--filter` on `status` — that is S05
- Multiple `--video-id` values in one invocation — single ID only; multi-select is a future enhancement
- Any queue file or artifact dir mutations beyond what the normal download/transcribe path already does

## Constraints

- `--video-id` on `download-all` searches across all queue files (the no-channel path from S03) — it is not scoped to a single channel
- `--video-id` on `transcribe-all` searches across all artifact dirs via `scan_artifact_statuses` — same scan path as the current no-filter transcribe-all
- The existing `--continue-on-error` flag on both commands is unchanged and still applies (though with a single target it rarely matters)
- The `--video-id` flag name must match the flag already used on `cleanup` for consistency

## Integration Points

### Consumes

- `src/cli.rs` — existing `download-all` and `transcribe-all` subcommand definitions; `--video-id` flag pattern from `cleanup`
- `src/main.rs` — `download_all` and `transcribe_all` async functions (both get an `Option<&str>` video_id parameter)
- `src/artifact.rs` — `scan_artifact_statuses` (transcribe-all filter); queue-file walking logic from S03 (download-all filter)

### Produces

- `src/cli.rs` — `--video-id` flag added to `download-all` and `transcribe-all` subcommands
- `src/main.rs` — `download_all` and `transcribe_all` updated to accept and apply `Option<String>` video_id filter; not-found error handling for both

## Open Questions

- None. All decisions resolved in discussion.
