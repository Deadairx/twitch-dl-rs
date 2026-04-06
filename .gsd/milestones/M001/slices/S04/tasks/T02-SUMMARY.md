---
id: T02
parent: S04
milestone: M001
key_files:
  - (none)
key_decisions:
  - (none)
duration: 
verification_result: passed
completed_at: 2026-04-06T03:18:44.762Z
blocker_discovered: false
---

# T02: Add cleanup CLI command with candidate listing and --delete flag for safe artifact cleanup

**Add cleanup CLI command with candidate listing and --delete flag for safe artifact cleanup**

## What Happened

Implemented the cleanup subcommand as specified in the task contract. The command scans artifacts, filters to only ready_for_notes candidates with completed transcription outcome, and displays them with file sizes. In list mode, it shows candidates without deletion. With --delete --all, it removes audio.m4a and transcript.srt for all candidates. With --delete --video-id <id>, it removes files for a specific artifact. Protected files (status.json, transcript.vtt, metadata.json, source_url.txt) are never touched. Artifact filtering correctly excludes items with suspect or failed outcomes. Error handling validates that --delete requires either --all or --video-id, preventing accidental misuse. All 14 project tests pass including new cleanup candidate filtering test.

## Verification

Ran cargo build (0 errors), cargo test (14 passed), and synthetic integration tests covering list mode, specific video deletion, all-deletion mode, error cases, and file preservation. Verified that cleanup --help contains 'delete', --help contains cleanup subcommand, and that protected files remain after deletion while only audio.m4a and transcript.srt are removed from candidates.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo build` | 0 | ✅ pass | 540ms |
| 2 | `cargo test artifact::tests` | 0 | ✅ pass | 100ms |
| 3 | `./target/debug/twitch-dl-rs cleanup --help | grep delete` | 0 | ✅ pass | 150ms |
| 4 | `./target/debug/twitch-dl-rs --help | grep cleanup` | 0 | ✅ pass | 150ms |
| 5 | `cleanup list mode with synthetic artifact` | 0 | ✅ pass | 200ms |
| 6 | `cleanup --delete --video-id <id> mode` | 0 | ✅ pass | 200ms |
| 7 | `cleanup --delete --all mode` | 0 | ✅ pass | 200ms |
| 8 | `error: --delete without args returns error` | 1 | ✅ pass | 150ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

None.
