---
id: T01
parent: S07
milestone: M002-z48awz
key_files:
  - src/artifact.rs
key_decisions:
  - (none)
duration: 
verification_result: passed
completed_at: 2026-04-07T04:46:00.323Z
blocker_discovered: false
---

# T01: Replaced all-digit directory check with status.json presence check in artifact discovery, enabling YouTube IDs

**Replaced all-digit directory check with status.json presence check in artifact discovery, enabling YouTube IDs**

## What Happened

Changed existing_artifact_ids() in src/artifact.rs to recognize any subdirectory containing status.json as a valid artifact, instead of requiring all-digit names. This enables YouTube artifacts with alphanumeric IDs (e.g., jNQXAC9IVRw) to be discovered alongside Twitch artifacts by scan_artifact_statuses, show_status, and transcribe-all. Updated test_scan_queue_dedup_with_artifact to add status.json to the 300/ test artifact so the test's dedup-verification intent still holds under the new rule. All 65 tests pass.

## Verification

Ran `cargo test` which executed 33 lib tests + 32 bin tests, all passing. Specifically verified test_scan_queue_dedup_with_artifact passes with the updated 300/ artifact setup containing status.json. The implementation is backward-compatible: all existing Twitch artifacts have had status.json since S01 normalization.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test` | 0 | ✅ pass | 10ms |
| 2 | `cargo test test_scan_queue_dedup_with_artifact` | 0 | ✅ pass | 80ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/artifact.rs`
