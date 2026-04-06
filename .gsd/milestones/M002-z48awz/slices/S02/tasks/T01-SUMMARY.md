---
task_id: T01
slice_id: S02
milestone_id: M002-z48awz
title: Add scan_queue_files helper to artifact.rs with unit tests
status: complete
summary_type: task
date: 2026-04-06
---

# T01 Summary: Add scan_queue_files helper to artifact.rs with unit tests

## What Was Done

Added the public `scan_queue_files` function to `src/artifact.rs` that performs a filesystem walk over all `queues/*.json` files and aggregates all queued VodEntry items. This function is the companion to the existing `read_queue_file` (which requires a channel name) and will be consumed by T02's `show_status` command to discover queued-but-not-downloaded items.

### Implementation Details

**Function signature:**
```rust
pub fn scan_queue_files(output_root: &Path) -> Result<Vec<VodEntry>, std::io::Error>
```

**Behavior:**
- Returns `Ok(vec![])` if `queues/` directory does not exist (not an error condition)
- Walks all `.json` files in `queues/`
- Uses `unwrap_or_default()` on individual file reads so a single malformed file doesn't abort the walk
- Silently skips invalid JSON (pattern: `if let Ok(qf)` — no error propagation)
- Aggregates all `VodEntry` items from valid QueueFile structs into a single Vec

**Tests added (4 new unit tests):**
1. **test_scan_queue_files_no_queues_dir**: Verifies that a missing `queues/` dir returns empty vec (boundary condition)
2. **test_scan_queue_files_single_file**: Verifies scan extracts 2 entries from a single queue file
3. **test_scan_queue_files_multiple_files**: Verifies aggregation across multiple files (3 entries from 2 files)
4. **test_scan_queue_files_malformed_file**: Verifies that a malformed JSON file is silently skipped while valid files are still processed (negative test for malformed inputs)

All tests use temp directories and minimal, valid JSON fixtures matching the existing test pattern in the file.

## Verification

### Test Results
```
running 20 tests
....................
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```
- Pre-existing tests: 16 ✅
- New tests: 4 ✅
- **Total: 20 passed, 0 failed**

### Build Results
```
cargo build --quiet 2>&1
```
- **Zero compilation errors** ✅
- Warning on `read_metadata` (dead code) is pre-existing and expected
- Warning on `scan_queue_files` (dead code) is expected — will be consumed in T02

### Verification Evidence

| Check | Command | Exit Code | Verdict | Duration |
|-------|---------|-----------|---------|----------|
| Unit tests | `cargo test --quiet 2>&1 \| grep 'test result'` | 0 | ✅ PASS (20 passed, 0 failed) | <1s |
| Build | `cargo build --quiet 2>&1` | 0 | ✅ PASS (no errors) | <2s |
| Fixture test: malformed skip | test_scan_queue_files_malformed_file | 0 | ✅ PASS | <1s |
| Fixture test: single file | test_scan_queue_files_single_file | 0 | ✅ PASS | <1s |
| Fixture test: multiple files | test_scan_queue_files_multiple_files | 0 | ✅ PASS | <1s |
| Fixture test: no queues dir | test_scan_queue_files_no_queues_dir | 0 | ✅ PASS | <1s |

## Implementation Notes

- The function follows the existing error-handling pattern in the codebase: channel-agnostic filesystem walks return `io::Error` (vs. the channel-specific `read_queue_file` which wraps errors as `Box<dyn Error>`)
- No changes to `read_queue_file`, `scan_artifact_statuses`, or any existing signatures — pure additive
- The silent skip of malformed files is intentional design (per task plan Q7 negative tests): it allows a queue dir with a corrupted file to still yield results from valid files nearby
- All fixtures use the existing `QueueFile` struct with no new types introduced

## Must-Haves Status

- ✅ `scan_queue_files` is `pub` and accepts `&Path`
- ✅ Returns `Ok(vec![])` if `queues/` dir does not exist
- ✅ Silently skips malformed queue files (no error propagation for bad JSON)
- ✅ Does NOT change `read_queue_file` or `scan_artifact_statuses` signatures
- ✅ All 4 new unit tests are present and pass
- ✅ All 16 pre-existing tests still pass (20 total)

## Next Steps

T02 will consume `scan_queue_files` in the `show_status` command to merge queued items with downloaded/transcribed items and render the 6-column status table (STAGE | TITLE | CHANNEL | DATE | OUTCOME | REASON).
