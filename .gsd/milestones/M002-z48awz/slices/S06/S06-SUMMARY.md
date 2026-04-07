---
id: S06
parent: M002-z48awz
milestone: M002-z48awz
provides:
  - Retry path for suspect transcriptions without re-download via --force-suspect flag
  - Blocking file locking on status.json writes to prevent concurrent corruption
  - Composable flag design (--force-suspect + --video-id for targeted re-retry)
requires:
  - slice: S01
    provides: ProcessStatus schema with transcription_outcome field and artifact structure
  - slice: S04
    provides: --video-id filtering infrastructure composing with new --force-suspect flag
affects:
  - S07
key_files:
  - src/cli.rs
  - src/main.rs
  - src/artifact.rs
  - src/lib.rs
  - Cargo.toml
key_decisions:
  - D027
patterns_established:
  - Filter predicates at handler level (not CLI level) enable composable flag stacking
  - Blocking locks via RAII (fs4) avoid manual unlock() ceremony and release-on-drop safety
  - ProcessStatus.transcribed=false guard skips override check for suspect items without code duplication
observability_surfaces:
  - CLI help text shows --force-suspect flag with clear description
  - Suspect items appear in status output with transcription_outcome="suspect" before retry
  - After --force-suspect run, status.json outcome field updates (completed, suspect, or failed)
drill_down_paths:
  - .gsd/milestones/M002-z48awz/slices/S06/tasks/T01-SUMMARY.md
  - .gsd/milestones/M002-z48awz/slices/S06/tasks/T02-SUMMARY.md
duration: ""
verification_result: passed
completed_at: 2026-04-07T04:36:07.866Z
blocker_discovered: false
---

# S06: Retry And Operational Hardening

**Added --force-suspect flag for retrying suspect transcriptions and implemented blocking file locking on status.json writes to prevent concurrent corruption.**

## What Happened

S06 closes two critical operational gaps: retry path for suspect transcriptions and concurrent-write safety for status.json. With --force-suspect, operators can re-run transcription on items that produced low-confidence outputs without re-downloading. With fs4-based blocking locks on writes, parallel download-all and transcribe-all invocations no longer corrupt artifact state. Both features reuse existing infrastructure (filter predicates, ProcessStatus schema) and follow established patterns (handler-level filtering, RAII locking). T01 added the --force-suspect flag to transcribe-all following the established pattern for boolean CLI flags, with a filter predicate that includes suspect items when force_suspect=true. Suspect items have transcribed=false, so the existing reuse guard evaluates false and doesn't block re-transcription—no parallel code path needed. T02 implemented fs4-based blocking exclusive locks on all write_status() calls via RAII, preventing concurrent writes from different processes (download-all and transcribe-all running in parallel) from corrupting artifact state. All 65 tests pass (33 lib + 32 bin) including new unit tests for filter predicate and concurrent write safety.

## Verification

cargo build succeeds with no errors. cargo test passes all 65 tests (33 lib + 32 bin). test_force_suspect_filter_predicate verifies filter logic with normal pending, suspect, and completed items in both force_suspect=true and force_suspect=false modes. test_concurrent_write_status_no_corruption spawns two threads writing different ProcessStatus values to same artifact dir; both complete without panic and final file is valid JSON. CLI help text shows --force-suspect flag with description. fs4 v0.13.1 present in Cargo.lock. No API changes to public functions; backward compatible.

## Requirements Advanced

None.

## Requirements Validated

- R012 — S06 adds file locking to write_status() preventing concurrent corruption. Parallel download-all and transcribe-all now safely serialize writes via blocking fs4 lock, validating the continuity requirement that interrupted work resumes without state corruption.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Deviations

None.

## Known Limitations

fs4's lock_exclusive() is blocking with no timeout. If a process crashes while holding the lock, the next invocation may hang indefinitely. Mitigation: macOS flock is released automatically by OS on process death, so hang is only possible in rare uninterruptible sleep scenarios. No explicit monitoring of lock acquisition times is implemented, but microsecond-scale lock contention is expected and acceptable.

## Follow-ups

None.

## Files Created/Modified

- `src/cli.rs` — Added --force-suspect flag to TranscribeAll variant with clap ArgAction::SetTrue
- `src/main.rs` — Updated transcribe_all() to accept force_suspect parameter and apply new filter predicate logic
- `src/artifact.rs` — Wrapped write_status() with fs4 blocking exclusive lock on status.lock file
- `src/lib.rs` — Added unit test for filter predicate logic and concurrent write safety test
- `Cargo.toml` — Added fs4 v0.13 dependency with sync feature
- `Cargo.lock` — Updated with fs4 v0.13.1 and transitive dependencies
