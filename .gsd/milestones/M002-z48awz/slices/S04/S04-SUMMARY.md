---
id: S04
parent: M002-z48awz
milestone: M002-z48awz
provides:
  - --video-id filtering on download-all and transcribe-all for single-item targeting
  - Clear not-found error messages distinguishing queue vs artifact miss
  - 4 unit tests covering filter and not-found paths for both commands
  - Handler-level filter pattern usable by S05 for --filter flag implementation
requires:
  - slice: S03
    provides: no-channel download-all path with scan_queue_files and pending vec construction
affects:
  - S05
key_files:
  - src/cli.rs
  - src/main.rs
  - src/artifact.rs
key_decisions:
  - Handler-level post-filter approach rather than CLI arg validation — preserves queue/artifact file immutability and enables future filter stacking without CLI parsing changes
  - Distinct error messages per command: 'not found in any queue' (download-all) vs 'not found in any artifact' (transcribe-all) — helps operators distinguish intake vs artifact state problems
  - Filter applies to both single-channel and no-channel paths in download_all — the video_id block appears after each branch's pending vec is fully assembled
patterns_established:
  - Video ID post-filter pattern: apply after pending vec is fully built, check for empty result, return formatted error — reusable for any future single-item targeting
  - Unit test pattern for filter logic: build temp queue/artifact structure, simulate handler data transformation, assert filtered-vec length — avoids async I/O in tests
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-04-07T03:38:08.832Z
blocker_discovered: false
---

# S04: Selective Processing

**Added `--video-id` filtering to `download-all` and `transcribe-all` for single-item targeting, with not-found error handling and 4 unit tests.**

## What Happened

S04 implemented precise item targeting for batch operations in two tasks. T01 extended the CLI structs with `video_id: Option<String>` on both `DownloadAll` and `TranscribeAll` variants, registered the `--video-id` argument on both subcommands with appropriate help text, and updated the parse_args match arms to populate the field. T02 extended both `download_all()` and `transcribe_all()` function signatures to accept `video_id: Option<&str>`, applied a post-filter after the pending vec is built in each function (covering both the single-channel and no-channel paths in download_all), and returned a clear error when the filtered result is empty. Four unit tests were added to src/artifact.rs covering the filter-with-match and filter-without-match cases for both commands. The filtering is applied at handler level (not CLI arg level) to preserve queue/artifact file integrity and enable future filter stacking.

## Verification

cargo build succeeds (exit 0). Both `./target/debug/vod-pipeline download-all --help` and `./target/debug/vod-pipeline transcribe-all --help` show the `--video-id` flag with correct descriptions. `cargo test` reports 28/28 pass (24 existing + 4 new). New tests: test_download_all_video_id_filter, test_download_all_video_id_not_found, test_transcribe_all_video_id_filter, test_transcribe_all_video_id_not_found — all pass.

## Requirements Advanced

None.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Deviations

None.

## Known Limitations

Single video ID per invocation only. Multi-ID targeting (e.g., --video-ids 123,456) is explicitly deferred as a future enhancement.

## Follow-ups

None.

## Files Created/Modified

- `src/cli.rs` — Added video_id: Option<String> field to DownloadAll and TranscribeAll CLI variants; registered --video-id arg on both subcommands; updated parse_args match arms
- `src/main.rs` — Extended download_all() and transcribe_all() signatures with video_id: Option<&str>; added post-filter logic with not-found error handling in both functions; updated dispatch block
- `src/artifact.rs` — Added 4 unit tests: test_download_all_video_id_filter, test_download_all_video_id_not_found, test_transcribe_all_video_id_filter, test_transcribe_all_video_id_not_found
