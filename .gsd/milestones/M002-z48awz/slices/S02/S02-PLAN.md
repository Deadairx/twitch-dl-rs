# S02: Status Legibility

**Goal:** Replace the ID-only status table with a human-readable 6-column display (STAGE | TITLE | CHANNEL | DATE | OUTCOME | REASON) and mix queued-but-not-downloaded items into the default view.
**Demo:** After this: After this: run status against an output root with queued, downloaded, and transcribed items and see a readable table with title, date, and channel for every row.

## Tasks
- [x] **T01: Add scan_queue_files helper to artifact.rs with unit tests** — Add pub fn scan_queue_files(output_root: &Path) -> Result<Vec<VodEntry>, std::io::Error> to src/artifact.rs. Channel-agnostic walker over all queues/*.json files. Silently skips malformed files. Returns Ok(vec![]) if queues/ dir does not exist. Add 4 unit tests: no_queues_dir, single_file, multiple_files, malformed_file.
  - Estimate: 45m
  - Files: src/artifact.rs
  - Verify: cargo test --quiet 2>&1 | grep -E 'test result|FAILED' — must show 20 passed, 0 failed
- [x] **T02: Rewrite show_status in main.rs with 6-column layout and STAGE derivation** — Rewrite show_status to merge scan_queue_files results with scan_artifact_statuses results (dedup by video_id, artifact-dir row wins). Add truncate() and derive_stage() helpers at module level. Print 6-column table: STAGE | TITLE | CHANNEL | DATE | OUTCOME | REASON. Add fixture dedup test to artifact.rs.
  - Estimate: 60m
  - Files: src/main.rs, src/artifact.rs
  - Verify: cargo build --quiet && cargo test --quiet 2>&1 | grep -E 'test result|FAILED' — must show all tests passed, 0 failed
