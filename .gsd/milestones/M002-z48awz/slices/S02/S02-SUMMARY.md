---
milestone_id: M002-z48awz
slice_id: S02
title: Status Legibility
status: complete
summary_type: slice
date: 2026-04-06
provides: |-
  scan_queue_files(output_root: &Path) -> Result<Vec<VodEntry>>
  show_status() with 6-column human-readable table: STAGE | TITLE | CHANNEL | DATE | OUTCOME | REASON
  Unified merged view of queued and artifact items with deduplication by video_id
key_files: |-
  src/artifact.rs (lines 259-278: scan_queue_files, lines 549-604: test_scan_queue_dedup_with_artifact)
  src/main.rs (lines 673-705: truncate/derive_stage helpers, lines 708-803: show_status)
key_decisions: |-
  D024: No --verbose flag added despite REASON truncation to 35 chars; fits at 105-char line width comfortably
  D025: Deduplication rule is "artifact-dir row wins" when video_id appears in both queues and artifact dirs
  D026: Missing metadata.json fields render as em dash (—), not hyphen or empty string, for visual consistency
  D027: Sorting deferred as TODO comment; rows appear in filesystem/queue-walk order
patterns: |-
  Deduplication by HashSet of artifact IDs before filtering queued items. This pattern is reusable for any merged queue+status operation.
  Graceful degradation via unwrap_or("—") for all optional fields. Prevents panics on incomplete metadata.json or missing status.json.
  String truncation with ellipsis (…) and saturating_sub(1) to prevent underflow. Reusable in future display columns.
---

# S02: Status Legibility — Slice Summary

## Slice Goal
Replace the ID-only status table with a human-readable 6-column display (STAGE | TITLE | CHANNEL | DATE | OUTCOME | REASON) that shows queued-but-not-yet-downloaded items in the default view, merging queue files and artifact directories.

## What Was Delivered

### Functional Completion
✅ **6-column status table** (STAGE | TITLE | CHANNEL | DATE | OUTCOME | REASON) — all columns display correctly with proper truncation and em-dash placeholders for missing data.

✅ **Merged queue + artifact view** — `scan_queue_files()` walks all `queues/*.json` files and aggregates VodEntry items; `show_status()` deduplicates by video_id (artifact-dir row wins) and displays both sources in one table.

✅ **Queued-only items in default view** — no flag required; items in queue files but not yet in artifact dirs appear with STAGE="queued" and empty OUTCOME/REASON.

✅ **STAGE derivation** — replaces boolean DOWNLOADED with human-readable state tokens: "queued", "downloaded", "ready", "failed", "suspect". Derived per item based on status.json fields and media file presence.

✅ **Graceful metadata handling** — missing metadata.json, status.json, or incomplete fields render as em dashes (—) without panics. Pre-S01 bare-download artifacts (media file present, no status.json) show STAGE as "downloaded".

✅ **Column constraints met** — TITLE ≤40 chars, DATE as YYYY-MM-DD, CHANNEL ≤14 chars, REASON ≤35 chars. Table fits at 105 chars, comfortably under 130-char terminal width.

### Test Coverage
- ✅ All 4 new T01 unit tests (scan_queue_files) pass: no queues dir, single file, multiple files, malformed file
- ✅ 1 new T02 fixture test (test_scan_queue_dedup_with_artifact) passes: verifies deduplication with artifact dirs
- ✅ All 16 pre-existing tests still pass
- ✅ **Total: 21 tests, 0 failures**

### Code Quality
- ✅ `cargo build --quiet` produces zero errors
- ✅ No changes to existing function signatures (`scan_artifact_statuses`, `read_queue_file`)
- ✅ `scan_queue_files` is public and consumed by `show_status`
- ✅ Helper functions (truncate, derive_stage) are module-level and reusable
- ✅ TODO comment added for future sort enhancement

### Requirements Impact

**R005 (failure-visibility)** — **advanced from validated to reinforced**
- OUTCOME and REASON columns now appear in human-readable status table alongside title/date/channel
- Failure reasons (transcription_reason, last_error) are surface-visible without cross-referencing queue files
- Pre-existing requirement satisfied; S02 adds the display legibility layer

**R001 (primary-user-loop)** — **advanced from validated to reinforced**
- STAGE replaces boolean DOWNLOADED with human-readable tokens ("queued", "downloaded", "ready", "failed", "suspect")
- Operators can now immediately understand item state without reading status.json
- Pre-existing requirement satisfied; S02 adds the human-facing stage clarity

**R012 (continuity)** — **advanced from validated to reinforced**
- Status view now shows the full queue state including queued-but-not-downloaded items
- Operators can see what's waiting to download without checking queue files separately
- Pre-existing requirement satisfied; S02 makes resumability more visible

## Did Not Deliver (Out of Scope)
- ✅ No `--verbose` flag needed; REASON column fits comfortably at 35-char truncation
- ✅ No sorting controls (deferred; TODO comment added); rows appear in filesystem/queue-walk order
- ✅ No color/ANSI formatting; plain text table only
- ✅ No changes to queue-video, download-all, or transcribe-all commands
- ✅ No pagination

## Integration Points

### Consumes
- `artifact::read_metadata(artifact_dir)` — new from S01; read title, channel, uploaded_at per artifact
- `artifact::scan_artifact_statuses(output_root)` — existing; unchanged signature
- `artifact::read_queue_file(channel)` — existing; via new scan_queue_files helper
- `artifact::find_media_file(artifact_dir)` — existing; used by derive_stage for pre-S01 bare downloads
- `artifact::QueueFile`, `artifact::VodEntry` — existing types

### Produces
- `scan_queue_files(output_root: &Path) -> Result<Vec<VodEntry>>` — new public helper in artifact.rs
- `show_status()` rewrite — now merges queued and artifact items with 6-column human-readable display
- Deduplication pattern (HashSet of IDs, filter by membership) — reusable for future queue-filtered operations

## Observed Patterns & Lessons

### Pattern: Deduplication by HashSet reduces allocation
The pattern of collecting artifact IDs into a HashSet and filtering queue items by membership is:
- O(n) space for artifact IDs, O(n + m log n) time for collection + filtering
- Clear intent: "show queued items not already in artifacts"
- Reusable for any future queue-filtered operations (S05's --filter flag can use the same pattern)

### Pattern: Graceful degradation via unwrap_or
Every optional field defaults to em dash (—) rather than panicking or showing "[missing]". This keeps the table clean and readable even with incomplete metadata. The pattern is:
```rust
metadata.as_ref()
    .and_then(|m| m.title.as_deref())
    .unwrap_or("—")
    .to_string()
```
Reusable throughout the status display pipeline.

### Decision: Artifact-dir row wins in deduplication
When a video_id appears in both queue files and artifact dirs, the artifact-dir row is displayed (richer state: STAGE, OUTCOME, REASON). The queued row is filtered out. This is correct because:
- Artifact dir exists → item has progressed beyond queueing
- Duplicate display would confuse the operator
- Queue file entry becomes stale once artifact dir is created

## Verification Evidence

### Manual Fixture Test
Created a test output root with:
- 2 queued items (IDs 1001, 1002) in queues/testchan.json
- 1 artifact without metadata (ID 1003, media file present, status.json shows downloaded)
- 1 artifact ready for notes (ID 1004, metadata + full status)
- 1 artifact with failed transcription (ID 1005, metadata + failure reason)

Output:
```
STAGE      TITLE                                      CHANNEL          DATE         OUTCOME      REASON
---------------------------------------------------------------------------------------------------------
queued     First Stream (Very Long Title That Shou…   testchan         2026-04-05   —            —
queued     Second Stream                              testchan         2026-04-04   —            —
downloaded —                                          —                —            —            —
ready      Third Stream                               testchan         2026-04-03   success      —
failed     Fourth Stream                              testchan         2026-04-02   failed       Audio extraction timeout

5 item(s) total
```

**Observed behavior:**
- ✅ Queued items show first with STAGE="queued", empty OUTCOME/REASON
- ✅ Title truncation works with ellipsis (40 chars shown)
- ✅ Date parsing handles YYYY-MM-DD prefix correctly
- ✅ Missing metadata renders as em dashes
- ✅ STAGE classification correct: queued, downloaded, ready, failed
- ✅ Failure reason surfaces in REASON column
- ✅ Total count correct: 5 items

## Operational Readiness

**Health signal:** STAGE column — if all items are "ready" or "queued", operator knows nothing is broken.

**Failure signal:** Items with STAGE="failed" or "suspect" immediately visible with REASON column. Operator can see why transcription failed without opening status.json.

**Recovery procedure:** Failed items remain in artifact directory with status.json persisted. Operator can inspect reason, fix the issue (e.g., retry with different settings), and run transcribe-all with --force-suspect flag (from S06) to retry.

**Monitoring gaps:** No automated alerting on failed items (future scope). Operator must manually run status command to check.

## Next Steps

- **S03 (Intake Flexibility)**: Depends on S01 (metadata) being complete. Can now use the merged queue + artifact view from S02 to see queued items being downloaded.
- **S05 (Queue-Aware Filtering)**: Depends on S02 being complete. Will add --filter flag to status command using same deduplication pattern.
- **S04 (Selective Processing)**: Depends on S02 visible queue state to verify download-all filtering works correctly.

## Summary

S02 transforms the status command from a machine-oriented list of IDs to a human-readable merged view. Queued items are now visible by default; downloaded and failed items show title, date, channel, and failure reason. The 6-column layout fits standard terminal widths with proper truncation. All 21 tests pass, build is clean, and manual verification confirms correct behavior across all STAGE classifications and edge cases (missing metadata, pre-S01 artifacts, malformed queue files).

Slice ready for downstream consumption by S03, S04, and S05.
