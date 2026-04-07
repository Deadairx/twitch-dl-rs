---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T01: Widen artifact discovery to accept non-numeric IDs

The `existing_artifact_ids()` function in `src/artifact.rs` currently rejects any directory whose name is not all-ASCII digits. YouTube video IDs are 11-character alphanumeric strings (e.g., `jNQXAC9IVRw`) — this filter makes YouTube artifacts invisible to `scan_artifact_statuses`, `show_status`, and `transcribe-all`. Replace the all-digit check with a `status.json` presence check: any subdirectory that contains `status.json` is a valid artifact. This is the de-facto artifact contract since S01's normalization and is backward-compatible with all Twitch artifacts. Also update the affected unit test (`test_scan_queue_dedup_with_artifact`) which creates artifact dir `300/` without a `status.json`; under the new rule, `300/` won't appear in results. Fix: add `status.json` to the `300/` setup so the test's dedup-verification intent still holds.

## Inputs

- ``src/artifact.rs``

## Expected Output

- ``src/artifact.rs``

## Verification

cargo test 2>&1 | tail -5
