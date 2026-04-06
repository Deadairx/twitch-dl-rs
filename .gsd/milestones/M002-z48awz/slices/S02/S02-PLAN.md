# S02: Status Legibility

**Goal:** Replace the ID-only status table with a human-readable 6-column display (STAGE | TITLE | CHANNEL | DATE | OUTCOME | REASON) and mix queued-but-not-downloaded items into the default view.
**Demo:** After this: run status against an output root with queued, downloaded, and transcribed items and see a readable table with title, date, and channel for every row.

## Must-Haves

- `scan_queue_files(output_root)` helper in `artifact.rs` that walks all `queues/*.json` files and returns the union of their `queued` VodEntries
- Status table displays: STAGE | TITLE (≤40 chars) | CHANNEL | DATE (YYYY-MM-DD) | OUTCOME | REASON
- Queued-but-not-downloaded items appear in the default status view — no flag required
- Deduplication: artifact-dir row wins when a video_id appears in both queue files and artifact dirs
- Graceful degradation: items without `metadata.json` show `—` for TITLE/CHANNEL/DATE; no panic or hard error
- Pre-S01 bare-download artifacts (media file present, no status.json) show STAGE as `downloaded`
- `scan_artifact_statuses` signature is unchanged
- `cargo test` passes with new tests for `scan_queue_files` and STAGE deduplication

## Requirement Impact

- **Requirements touched**: R005 (failure-visibility — OUTCOME/REASON columns now human-readable), R001 (primary-user-loop — STAGE replaces boolean DOWNLOADED), R012 (continuity — status view now reflects full queue state)
- **Re-verify**: status command output format changes; downstream S05 (Queue-Aware Filtering) depends on `scan_artifact_statuses` signature remaining unchanged
- **Decisions revisited**: D013 (display metadata schema ownership — confirmed; status reads both metadata.json and status.json per artifact), D014 (queued item visibility — implemented in default view, no flag needed)

## Proof Level

- This slice proves: contract — verified by `cargo test` with unit tests for `scan_queue_files`, STAGE derivation edge cases, and a fixture-dir deduplication scenario; display column layout verified by manual run against fixture dir
- Real runtime required: no (unit tests are sufficient for correctness; manual run is optional confirmation)
- Human/UAT required: no

## Verification

- `cargo test --quiet 2>&1 | grep -E 'test result|FAILED'` — must show all tests passed (20+ ok), 0 failed
- `cargo build --quiet 2>&1 | grep '^error'` — must produce no output (zero errors)
- Fixture test in `src/artifact.rs`: scan_queue_files + scan_artifact_statuses return correct dedup inputs for a mixed temp dir (video IDs in both queue and artifact dirs appear only once, queued-only items appear correctly)

## Integration Closure

- Upstream surfaces consumed: `artifact::read_metadata` (S01), `artifact::scan_artifact_statuses` (existing), `artifact::QueueFile` + `VodEntry` (existing), `artifact::find_media_file` (existing)
- New wiring introduced in this slice: `scan_queue_files` (artifact.rs) → `show_status` (main.rs); `read_metadata` first consumer in runtime path
- What remains before the milestone is truly usable end-to-end: S03 (Intake Flexibility — queue-video command), S04 (Selective Processing), S05 (Queue-Aware Filtering adds --filter flag to status)

## Tasks

- [ ] **T01: Add scan_queue_files helper to artifact.rs with unit tests** `est:45m`
  - Why: `show_status` (T02) needs a channel-agnostic queue walker. `read_queue_file` is channel-keyed and unsuitable. This task adds the helper and tests it before T02 depends on it.
  - Files: `src/artifact.rs`
  - Do: Add `pub fn scan_queue_files(output_root: &Path) -> Result<Vec<VodEntry>, std::io::Error>` that walks `queues/*.json`, silently skips malformed files, returns `Ok(vec![])` if `queues/` dir doesn't exist. Add 4 unit tests covering: no queues dir, single file, multiple files, malformed file mixed with valid.
  - Verify: `cargo test --quiet 2>&1 | grep -E 'test result|FAILED'` — must show 20 passed, 0 failed
  - Done when: `scan_queue_files` is public, all 4 new tests pass alongside all 16 existing tests

- [ ] **T02: Rewrite show_status in main.rs with 6-column layout and STAGE derivation** `est:60m`
  - Why: The current show_status is machine-oriented (VIDEO_ID, DOWNLOADED, OUTCOME, READY, REASON) and omits queued items. This task replaces it with the human-readable merged view.
  - Files: `src/main.rs`, `src/artifact.rs`
  - Do: Add `truncate()` helper. Rewrite `show_status` to merge `scan_queue_files` results with `scan_artifact_statuses` results (dedup by video_id, artifact-dir row wins). Derive STAGE per item. Call `read_metadata` per artifact dir. Print 6-column table with truncation. Add deduplication fixture test to `artifact.rs`. Add `// TODO(sort):` comment.
  - Verify: `cargo build --quiet && cargo test --quiet 2>&1 | grep -E 'test result|FAILED'` — must show all tests passed, 0 failed
  - Done when: `show_status` outputs STAGE | TITLE | CHANNEL | DATE | OUTCOME | REASON; queued-only items appear; no duplicates; build clean

## Files Likely Touched

- `src/artifact.rs`
- `src/main.rs`
