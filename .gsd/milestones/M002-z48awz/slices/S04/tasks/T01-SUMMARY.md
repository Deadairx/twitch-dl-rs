---
id: T01
parent: S04
milestone: M002-z48awz
key_files:
  - src/cli.rs
  - src/main.rs
key_decisions:
  - Implemented filtering at handler level rather than CLI arg level to preserve queue file integrity and allow flexibility for future filtering enhancements
duration: 
verification_result: passed
completed_at: 2026-04-07T03:06:04.174Z
blocker_discovered: false
---

# T01: Added --video-id optional argument to download-all and transcribe-all subcommands with filtering logic

**Added --video-id optional argument to download-all and transcribe-all subcommands with filtering logic**

## What Happened

Implemented task by extending CLI structs with video_id field on both DownloadAll and TranscribeAll variants. Registered --video-id argument on both subcommands with appropriate help text. Updated parse_args match arms to populate the field from CLI arguments. Extended download_all() and transcribe_all() function signatures to accept video_id parameter and added filtering logic to skip items not matching the specified ID. Updated main.rs match patterns to pass video_id to handlers. All compilation successful with both help texts displaying the new --video-id option correctly."

## Verification

Ran cargo build successfully (0 exit code, 2.51s). Verified --video-id appears in both download-all --help and transcribe-all --help output. Full verification suite passed with all checks returning 0 exit code.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo build` | 0 | ✅ pass | 2510ms |
| 2 | `./target/debug/vod-pipeline download-all --help | grep -q 'video-id'` | 0 | ✅ pass | 50ms |
| 3 | `./target/debug/vod-pipeline transcribe-all --help | grep -q 'video-id'` | 0 | ✅ pass | 50ms |
| 4 | `cargo build && ./target/debug/vod-pipeline download-all --help | grep -q 'video-id' && ./target/debug/vod-pipeline transcribe-all --help | grep -q 'video-id'` | 0 | ✅ pass | 160ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/cli.rs`
- `src/main.rs`
