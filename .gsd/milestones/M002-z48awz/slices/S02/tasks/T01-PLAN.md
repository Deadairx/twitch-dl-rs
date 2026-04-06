---
estimated_steps: 4
estimated_files: 1
skills_used: []
---

# T01: Add scan_queue_files helper to artifact.rs with unit tests

**Slice:** S02 — Status Legibility
**Milestone:** M002-z48awz

## Description

Add `pub fn scan_queue_files(output_root: &Path) -> Result<Vec<VodEntry>, std::io::Error>` to `src/artifact.rs`. This is a channel-agnostic walker over all `queues/*.json` files — unlike `read_queue_file` which requires a channel name, `scan_queue_files` discovers all queue files by filesystem walk. The function is consumed by `show_status` (T02) to find queued-but-not-downloaded items.

Also add 4 unit tests covering the full behavioral envelope: no queues dir, single file, multiple files, and malformed file mixed with valid.

## Steps

1. In `src/artifact.rs`, add the function after `read_queue_file`:

```rust
pub fn scan_queue_files(output_root: &Path) -> Result<Vec<VodEntry>, std::io::Error> {
    let queue_dir = output_root.join("queues");
    if !queue_dir.exists() {
        return Ok(vec![]);
    }
    let mut entries = Vec::new();
    for entry in fs::read_dir(&queue_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            let content = fs::read_to_string(&path).unwrap_or_default();
            if let Ok(qf) = serde_json::from_str::<QueueFile>(&content) {
                entries.extend(qf.queued);
            }
        }
    }
    Ok(entries)
}
```

Key constraints:
- Return `Ok(vec![])` if `queues/` dir doesn't exist (not an error)
- Use `unwrap_or_default()` on file read so a filesystem error on a single file doesn't abort the whole walk
- Silently skip malformed JSON (the `if let Ok(qf)` pattern — don't propagate parse errors)
- Use the existing `QueueFile` struct for deserialization — do NOT create a new type

2. In the `#[cfg(test)]` block at the bottom of `src/artifact.rs`, add 4 tests:

**test_scan_queue_files_no_queues_dir**: Create a temp dir with no `queues/` subdirectory. Call `scan_queue_files(dir.path())`. Assert it returns `Ok` and the vec is empty.

**test_scan_queue_files_single_file**: Create `queues/chan1.json` with 2 VodEntries (video_ids "aaa", "bbb"). Assert `scan_queue_files` returns a vec of length 2 with those IDs (use a sorted comparison or check both IDs are present).

**test_scan_queue_files_multiple_files**: Create `queues/chan1.json` (2 entries, IDs "aaa", "bbb") and `queues/chan2.json` (1 entry, ID "ccc"). Assert the returned vec has length 3 and contains all three IDs.

**test_scan_queue_files_malformed_file**: Create `queues/good.json` (1 valid VodEntry, ID "zzz") and `queues/bad.json` (content: `"not valid json at all"`). Assert `scan_queue_files` returns `Ok`, vec has length 1, and contains ID "zzz".

For the VodEntry JSON in fixtures, use this minimal structure (matches the existing test pattern in the file):
```json
{"channel":"testchan","title":"Test VOD","url":"https://www.twitch.tv/videos/123","video_id":"aaa","uploaded_at":"2026-01-01T00:00:00Z","duration":"PT3600S"}
```

The QueueFile wrapper structure is:
```json
{"schema_version":1,"channel":"testchan","generated_at_epoch_s":0,"past_broadcasts_only":false,"min_seconds":600,"queued_count":1,"queued":[{...VodEntry...}],"skipped_existing_ids":[]}
```

3. Run `cargo test` and confirm all tests pass.

4. Run `cargo build` and confirm no new errors or warnings (the pre-existing `dead_code` warning on `read_metadata` is acceptable — don't try to suppress it; T02 will use it).

## Must-Haves

- [x] `scan_queue_files` is `pub` and accepts `&Path`
- [x] Returns `Ok(vec![])` if `queues/` dir does not exist
- [x] Silently skips malformed queue files (no error propagation for bad JSON)
- [x] Does NOT change `read_queue_file` or `scan_artifact_statuses` signatures
- [x] All 4 new unit tests are present and pass
- [x] All 16 pre-existing tests still pass (20 total)

## Negative Tests

- **Malformed inputs**: `test_scan_queue_files_malformed_file` — verifies that `"not valid json at all"` in a queue file is silently skipped, not returned as an error
- **Boundary conditions**: `test_scan_queue_files_no_queues_dir` — verifies that a missing `queues/` dir returns empty vec, not an IO error
- **Multiple files**: `test_scan_queue_files_multiple_files` — verifies union across multiple files, not just first file wins

## Verification

- `cargo test --quiet 2>&1 | grep -E 'test result|FAILED'` — must show 20 passed, 0 failed
- `cargo build --quiet 2>&1 | grep '^error'` — must produce no output

## Inputs

- `src/artifact.rs` — existing file with `QueueFile`, `VodEntry` (imported from twitch.rs), `read_queue_file`, `scan_artifact_statuses` — the context for where to add the new function

## Expected Output

- `src/artifact.rs` — updated with `scan_queue_files` function and 4 new unit tests
