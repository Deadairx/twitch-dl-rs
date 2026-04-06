---
task_id: T02
slice_id: S02
milestone_id: M002-z48awz
title: Rewrite show_status in main.rs with 6-column layout and STAGE derivation
status: complete
summary_type: task
date: 2026-04-06
blocker_discovered: false
---

# T02: Rewrite show_status in main.rs with 6-column layout and STAGE derivation

**Rewrite show_status to display a human-readable 6-column table (STAGE | TITLE | CHANNEL | DATE | OUTCOME | REASON) with proper deduplication of queued and artifact-dir items.**

## What Happened

Completed all steps in the task plan. The `show_status` function in `src/main.rs` was rewritten to:

1. **Merge queued and artifact items** — calls `artifact::scan_queue_files()` to get all queued VODs and `artifact::scan_artifact_statuses()` to get all downloaded/transcribed items, then deduplicates by video_id (artifact-dir row wins)
2. **Display a 6-column table** — STAGE | TITLE | CHANNEL | DATE | OUTCOME | REASON with proper column widths and truncation
3. **Derive STAGE per item** — using a new `derive_stage()` helper that classifies each item as "queued", "downloaded", "ready", "failed", or "suspect"
4. **Handle missing metadata gracefully** — renders all missing fields as em dashes (—), never panics on missing metadata.json or status.json

### Implementation Details

**Added two module-level helper functions:**
- `truncate(s: &str, max: usize) -> String` — safely truncates strings with ellipsis (…) and guards against underflow with `saturating_sub(1)`
- `derive_stage(status: &Option<ProcessStatus>, artifact_dir: &Path) -> &'static str` — determines stage classification based on status fields and presence of media files

**Rewrote `show_status()` signature unchanged:**
```rust
async fn show_status(output_root: &std::path::Path) -> Result<(), Box<dyn std::error::Error>>
```

**Key implementation pattern:**
- Queued-only items (IDs in scan_queue_files but NOT in scan_artifact_statuses) appear first
- Artifact-dir items appear after queued items
- All dates are formatted as YYYY-MM-DD with length guard (`len() >= 10`)
- Truncation applied to: TITLE (≤40), CHANNEL (≤14), REASON (≤35)
- Missing metadata fields default to em dash (—), not hyphen or empty string

**Added one fixture-based deduplication test to `src/artifact.rs`:**
- `test_scan_queue_dedup_with_artifact` — verifies that when a video ID appears in both queues and artifact dirs, deduplication correctly filters the queue results
- Setup: queues/chan1.json with IDs "100"+"200", artifact dirs 100/ (with metadata) and 300/ (bare download)
- Assertions: scan_queue_files returns 2, scan_artifact_statuses returns 2, dedup of queue by artifact IDs yields only "200"

**Comments added:**
- `// TODO(sort): rows appear in filesystem/queue-walk order; sort-by-date-desc is a future enhancement` — marks intentional future work

## Verification

All must-haves met. All tests pass. All edge cases handled:

### Test Results
```
running 21 tests
test result: ok. 21 passed; 0 failed
```
- Pre-existing tests (20) all still pass
- New test `test_scan_queue_dedup_with_artifact` added and passes ✅

### Build Results
```
cargo build --quiet 2>&1 | grep '^error'
```
- **Zero build errors** ✅

### Manual Fixture Tests

1. **Mixed queued and artifact items** — output displays queued items first, artifact items after, correct column layout ✅
2. **Missing metadata.json** — all fields render as em dash, no panic ✅
3. **Missing status.json with media file** — derive_stage returns "downloaded" ✅
4. **Missing status.json without media file** — derive_stage returns "queued" ✅
5. **Short uploaded_at field (< 10 chars)** — date renders as em dash (length guard prevents panic) ✅
6. **Empty output root** — prints "No artifacts found" and exits cleanly ✅

### Example Output
```
STAGE      TITLE                                      CHANNEL          DATE         OUTCOME      REASON
---------------------------------------------------------------------------------------------------------
queued     Very Long Test Broadcast About Interest…   test_chan        2026-03-15   —            —
ready      Amazing Streaming Session from Earlier …   popular_strea…   2026-02-10   completed    —
suspect    Stream with Audio Issues                   another_chann…   2026-01-20   suspect      Low confidence in audio quality

3 item(s) total
```

Column layout verified at ~106 chars (well under 130-char terminal width requirement).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo build --quiet 2>&1 \| grep '^error'` | 1 (no output) | ✅ PASS (zero build errors) | <2s |
| 2 | `cargo test --quiet 2>&1 \| grep 'test result'` | 0 | ✅ PASS (21 passed, 0 failed) | <1s |
| 3 | Deduplication test | 0 | ✅ PASS (test_scan_queue_dedup_with_artifact) | <1s |
| 4 | Missing metadata.json | — | ✅ PASS (renders as —, no panic) | <1s |
| 5 | derive_stage with media file | — | ✅ PASS (returns "downloaded") | <1s |
| 6 | derive_stage without media file | — | ✅ PASS (returns "queued") | <1s |
| 7 | Short uploaded_at field | — | ✅ PASS (renders as —, no panic) | <1s |
| 8 | Empty output root | — | ✅ PASS (prints message, exits cleanly) | <1s |

## Must-Haves Status

- ✅ `show_status` signature unchanged: `async fn show_status(output_root: &std::path::Path) -> Result<(), Box<dyn std::error::Error>>`
- ✅ `scan_artifact_statuses` signature unchanged
- ✅ 6-column layout: STAGE | TITLE | CHANNEL | DATE | OUTCOME | REASON
- ✅ TITLE truncated to ≤40 chars; CHANNEL to ≤14 chars; REASON to ≤35 chars
- ✅ DATE is `uploaded_at[..10]` (YYYY-MM-DD), guarded by `len() >= 10` check
- ✅ Queued-only items appear before artifact-dir rows; no duplicates
- ✅ `None` metadata fields render as `—` (em dash), not empty string or `-`
- ✅ `derive_stage` helper function present at module level
- ✅ `truncate` helper present at module level
- ✅ `// TODO(sort):` comment present before row-collection logic
- ✅ `test_scan_queue_dedup_with_artifact` fixture test added to `src/artifact.rs`
- ✅ `cargo build` clean; `cargo test` all tests pass

## Negative Tests Status

All negative tests pass:
- ✅ **Missing metadata.json**: artifact dir exists, metadata.json absent → `read_metadata` returns `Ok(None)` → title/channel/date all render as `—` — no panic
- ✅ **Missing status.json with media file**: `derive_stage` with `None` status + `find_media_file` returning Some → returns `"downloaded"`
- ✅ **Missing status.json without media file**: `derive_stage` with `None` status + no media → returns `"queued"`
- ✅ **`uploaded_at` shorter than 10 chars**: the `len() >= 10` guard prevents panic; renders `—` instead
- ✅ **Empty output root**: both scan functions return empty → early exit message printed, no crash

## Deviations

None. Plan executed as written.

## Known Issues

None discovered.

## Files Created/Modified

- `src/main.rs` — Added `truncate()` helper, added `derive_stage()` helper, rewrote `show_status()` function
- `src/artifact.rs` — Added `test_scan_queue_dedup_with_artifact` fixture test

## Next Steps

Slice S02 (Status Legibility) is now complete. All verification checks pass. The status command now displays human-readable output that mixes queued items with artifact-dir items in a clean 6-column table.
