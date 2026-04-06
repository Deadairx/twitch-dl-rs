---
id: S02
parent: M001
milestone: M001
provides:
  - Three new CLI commands (status, download-all, transcribe-all) for decoupled staged processing
  - Queue file deserialization and artifact status scanning capabilities
  - Composable download and transcription helper functions for reusable work stages
  - Full backward compatibility with existing process command
requires:
  - slice: S01
    provides: Durable artifact directories with status.json tracking and queue file persistence
affects:
  - S03
  - S04
  - S05
key_files:
  - src/main.rs
  - src/cli.rs
  - src/artifact.rs
  - src/twitch.rs
  - Cargo.toml
key_decisions:
  - Made QueueFile and VodEntry fully serializable/deserializable to enable roundtrip persistence
  - Extracted download and transcribe logic into composable async helpers for reuse across commands
  - Implemented continue-on-error flag for both batch commands to support partial recovery
  - Used filter_map pattern in transcribe_all to gracefully handle missing status.json files
  - Fixed 70-character table formatting in status command for consistent output
patterns_established:
  - Three-tier processing model: queue → download-all → transcribe-all allows flexible operator pacing
  - Helper functions (download_vod_to_artifact, transcribe_artifact) are composable and testable
  - Status scanning across artifact directories enables batch operation visibility
  - Each command checks status.json before acting, enabling safe resumption of interrupted work
observability_surfaces:
  - status command provides human-readable table showing download/transcribe state for all artifacts
  - Each failed operation updates artifact status.json with error message for later inspection
  - All batch commands report progress per-item with success/failure count
drill_down_paths:
  - .gsd/milestones/M001/slices/S02/tasks/T01-SUMMARY.md
  - .gsd/milestones/M001/slices/S02/tasks/T02-SUMMARY.md
duration: ""
verification_result: passed
completed_at: 2026-04-06T02:51:28.586Z
blocker_discovered: false
---

# S02: Decoupled staged processing

**Three decoupled batch processing commands (status, download-all, transcribe-all) that allow downloads and transcription to progress independently with safe resumption on interruption**

## What Happened

Slice S02 successfully implemented the infrastructure for decoupled staged processing. The work was divided into two tasks: (1) T01 added queue file deserialization, the status command, and artifact scanning helpers, and (2) T02 extracted composable download/transcribe helpers and implemented the download-all and transcribe-all batch commands.

The slice makes two foundational changes to the codebase. First, it makes QueueFile and VodEntry fully serializable by adding Deserialize derives, allowing persisted queue JSON files to be read back from disk. Second, it extracts the core download and transcription logic from process_vod() into standalone helpers (download_vod_to_artifact and transcribe_artifact) that can be composed independently.

The three new CLI commands are:
- status: Scans all artifact directories and displays a human-readable table showing download/transcribe state
- download-all: Reads the persisted queue for a channel and downloads all pending items, respecting status.json to avoid redundant downloads
- transcribe-all: Scans for downloaded-but-untranscribed artifacts and batch-processes them

Both batch commands support --continue-on-error to allow partial recovery and resumption. The original process command was refactored to delegate to the extracted helpers, maintaining full backward compatibility. All 3 unit tests pass, cargo build completes cleanly with no warnings, and manual CLI testing confirms correct command registration and help text. The implementation directly satisfies R003 (operability—downloads can progress independently) and R012 (continuity—interrupted work is resumable via status checks before acting).

## Verification

All verification checks passed:
- Unit tests: cargo test artifact::tests — 3/3 pass (test_read_queue_file_roundtrip, test_scan_artifact_statuses_empty, test_status_roundtrip)
- Build: cargo build — completes cleanly, no errors or warnings
- CLI structure: ./target/debug/twitch-dl-rs --help lists all 5 subcommands including status, download-all, transcribe-all
- Command help: Each new command shows correct argument parsing and defaults
- Backward compatibility: process --help still works identically with no behavior changes
- Status command behavior: Empty directory returns "No artifacts found"; properly formatted table with VIDEO_ID, DOWNLOADED, TRANSCRIBED, LAST_ERROR columns when artifacts exist

All tests passed with zero failures.

## Requirements Advanced

- R001 — Made each item trackable as a durable job with explicit stage state visible via the status command
- R005 — Ensured each failed operation updates status.json with error messages, and status command surfaces them

## Requirements Validated

- R003 — download-all and transcribe-all are now independent commands, allowing downloads to progress while transcription is pending/failed
- R012 — Each command checks status.json before acting, enabling safe resumption of interrupted work

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Deviations

None. Implementation follows the slice plan exactly as specified.

## Known Limitations

- Status command truncates errors to 40 chars for table display; full errors remain in status.json files
- Batch commands process items sequentially rather than in parallel; future optimization possible
- No automatic retry mechanism; all work is manual trigger via CLI commands
- No concurrent access locking for status.json files; simultaneous writes from multiple processes could corrupt files
- Status command silently ignores non-numeric artifact directories (works as intended, but could be stricter)

## Follow-ups

S03 will add trustworthy transcription with explicit failure categorization to improve diagnostics. S04 will add cleanup command. S05 will demonstrate full end-to-end workflow combining all commands.

## Files Created/Modified

- `src/main.rs` — Added download_vod_to_artifact, transcribe_artifact, download_all, transcribe_all, show_status async handlers; refactored process_vod to delegate to helpers
- `src/cli.rs` — Added DownloadAll and TranscribeAll CLI enum variants with full clap argument parsing and defaults
- `src/artifact.rs` — Added Deserialize derive to QueueFile; implemented read_queue_file() and scan_artifact_statuses() functions
- `src/twitch.rs` — Added Deserialize derive to VodEntry struct
- `Cargo.toml` — No changes (tempfile already present for tests)
