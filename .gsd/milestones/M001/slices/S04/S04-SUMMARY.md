---
id: S04
parent: M001
milestone: M001
provides:
  - (none)
requires:
  []
affects:
  []
key_files:
  - (none)
key_decisions:
  - (none)
patterns_established:
  - (none)
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-04-06T03:21:56.366Z
blocker_discovered: false
---

# S04: Ready-for-notes and manual cleanup workflow

**Automatic ready-for-notes lifecycle state plus safe, two-step cleanup command that lists deletion candidates for operator review before any files are removed.**

## What Happened

S04 delivered two core components: (1) Automatic `ready_for_notes` field on ProcessStatus that gets set to true when transcription completes successfully in the `transcribe_artifact()` function. The field includes `#[serde(default)]` for backward compatibility with old status.json files. (2) A new `cleanup` CLI subcommand that implements a two-step safe deletion workflow: list-only mode shows all `ready_for_notes == true` artifacts with per-item file sizes, and `--delete` mode (requiring explicit `--all` or `--video-id` argument) removes only `audio.m4a` and `transcript.srt` while protecting transcript.vtt, metadata.json, status.json, and source_url.txt. Filtering correctly excludes items with `suspect` or `failed` transcription outcomes. Updated `show_status()` to display a READY column showing \"yes\" for ready artifacts and \"-\" for others. All artifact tests pass, and comprehensive integration testing confirms the lifecycle works end-to-end: ready candidates are correctly identified, selective deletion works per-item and bulk, protected files survive cleanup, and non-ready artifacts are excluded from candidates.

## Verification

Verification across all tasks: (1) `cargo build` — zero errors, zero warnings. (2) `cargo test artifact::tests` — all 7 tests pass including backward-compat and roundtrip tests for ready_for_notes field. (3) `cleanup --help` shows delete flag and subcommand appears in main help. (4) Integration tests: list mode correctly shows ready candidates with sizes, `--delete --all` removes audio.m4a and transcript.srt, `--delete --video-id <id>` removes files for specific artifacts only, protected files remain after deletion, `--delete` without args returns error code 1, non-ready and failed transcriptions excluded from candidates, `show_status` displays READY column with correct values. UAT preconditions met: project builds, ProcessStatus has ready_for_notes field with serde(default), cleanup command implemented with all flags, show_status updated for READY column.

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

None.

## Follow-ups

None.

## Files Created/Modified

None.
