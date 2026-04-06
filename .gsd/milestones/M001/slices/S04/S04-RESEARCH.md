# S04: Ready-for-notes and manual cleanup workflow — Research

**Date:** 2026-04-05
**Calibration:** Light research — additive work on well-understood patterns already in the codebase.

## Summary

S04 adds a `ready_for_notes` lifecycle field to `ProcessStatus` and wires a new `cleanup` CLI command. The existing codebase already establishes the exact signal S04 needs: `transcribe_artifact()` in `main.rs` sets `transcribed = true` and `transcription_outcome = "completed"` atomically when `TranscriptionOutcome::Completed` is returned. S04 only needs to also set `ready_for_notes = true` at that same boundary, then build the `cleanup` command that queries it.

The work is two self-contained tasks: (1) extend the data model + wire the transition in `main.rs`, and (2) add the `cleanup` CLI subcommand. Both tasks are isolated — the CLI command depends on the model field existing but not on the full command being done first.

## Recommendation

Add `ready_for_notes: bool` to `ProcessStatus` with a serde default of `false` (matching the pattern used for `transcription_outcome` et al.). Set it in `transcribe_artifact()` alongside the existing `transcribed = true` assignment when outcome is `Completed`. Then add a `cleanup` command that:
- lists all artifact dirs where `ready_for_notes == true` (and `transcribed == true`, double safety)
- prints per-item file sizes for `audio.m4a` and `transcript.srt`
- with `--delete <video_id>` (or `--delete --all`), removes those two files for the specified item(s)

Also update `show_status` to mark `ready_for_notes` items visually (e.g., `READY` in the outcome column or a dedicated marker column).

## Implementation Landscape

### Key Files

- `src/artifact.rs` — Add `ready_for_notes: bool` field to `ProcessStatus` (line ~155). Use `#[serde(default)]` for backward compat, exactly like `transcription_outcome`. Add a unit test verifying the field round-trips and old JSON without it deserializes as `false`.

- `src/main.rs` — In `transcribe_artifact()` (line ~305), add `status.ready_for_notes = true;` alongside the existing `status.transcribed = true;` inside the `Completed` match arm. Implement a new `cleanup_candidates()` async function and `run_cleanup()` dispatcher. Update `show_status()` to include a `READY` column or marker.

- `src/cli.rs` — Add `Cleanup { output_root: PathBuf, delete: bool, video_id: Option<String> }` variant to `CliCommand` enum. Register the `cleanup` subcommand with clap. Wire it in the `main()` match block.

### Build Order

1. **`ProcessStatus` field + transition** (T01) — Add the field to `artifact.rs`, set it in `transcribe_artifact()` in `main.rs`, update `show_status()`. This is the foundation. Add backward-compat unit test.

2. **`cleanup` CLI command** (T02) — Add CLI variant, implement candidate discovery + file-size display, implement `--delete` deletion path. Depends on `ready_for_notes` existing on the struct.

Building T01 first unblocks T02 and keeps each task to a single reviewable change.

### Verification Approach

```
# Build must pass clean
cargo build

# All existing + new unit tests pass
cargo test

# Simulate a completed artifact and check ready_for_notes persists
# (unit test in artifact.rs)

# CLI smoke tests
./target/debug/twitch-dl-rs --help           # cleanup in subcommand list
./target/debug/twitch-dl-rs cleanup --help   # shows --delete flag
./target/debug/twitch-dl-rs cleanup --output-root /tmp/test  # lists candidates or "no ready items"
```

For a real integration check: create a synthetic artifact directory with a fabricated `status.json` containing `ready_for_notes: true`, run `cleanup --output-root`, verify it lists the item with file sizes.

## Constraints

- `ready_for_notes` must be gated on explicit lifecycle state (`ready_for_notes == true`), not file existence. The cleanup command must NOT scan for `transcript.vtt` presence as a proxy — it reads the status field.
- `suspect` items must never appear as cleanup candidates regardless of files present. The filter is `ready_for_notes == true && transcribed == true`.
- `--delete` without a video_id argument should require `--all` to proceed, rather than silently deleting everything. This matches D004 (cleanup is explicit operator action).
- Backward compat: existing `status.json` files without `ready_for_notes` must deserialize cleanly as `false` (use `#[serde(default)]`).
- `transcript.vtt`, `metadata.json`, `status.json`, and `source_url.txt` are never touched by cleanup — only `audio.m4a` and `transcript.srt`.
