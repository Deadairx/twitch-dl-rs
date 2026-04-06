# S02: Status Legibility — Research

**Date:** 2026-04-06

## Summary

S02 is straightforward application of already-landed infrastructure. S01 delivered `read_metadata`, the three new `ArtifactMetadata` display fields, and normalized artifact structure for bare downloads. Everything S02 needs exists and is functional — the work is entirely in `show_status` (main.rs) and a new `scan_queue_files` helper (artifact.rs).

The current `show_status` prints a 5-column table: `VIDEO_ID | DOWNLOADED | OUTCOME | READY | REASON`. It reads only artifact dirs via `scan_artifact_statuses`. S02 rewrites this to: (1) add a `scan_queue_files` helper to artifact.rs that reads all `queues/*.json` files, (2) rewrite `show_status` to merge queued VodEntries with artifact-dir statuses, (3) replace the column layout with `STAGE | TITLE | CHANNEL | DATE | OUTCOME | REASON`.

No new dependencies. No risky integration. No architectural decisions to make — the slice context is specific about every case.

## Recommendation

Implement in two focused units: first add `scan_queue_files` to `artifact.rs` (one function, well-bounded), then rewrite `show_status` in `main.rs`. Test both with unit tests in artifact.rs; verify display output manually or via a fixture-based test.

The REASON column **fits** at reasonable truncation (30-35 chars). The 6-column layout works at 130 chars:
- STAGE: 12 chars
- TITLE: 42 chars (40 + padding)
- CHANNEL: 18 chars
- DATE: 12 chars
- OUTCOME: 12 chars
- REASON: ~35 chars
Total: ~131 chars — tight but workable with disciplined truncation. No need to drop REASON or add `--verbose` flag.

## Implementation Landscape

### Key Files

- `src/artifact.rs` — Add `scan_queue_files(output_root) -> Result<Vec<VodEntry>, std::io::Error>` helper. Pattern: walk `queues/` dir with `fs::read_dir`, call `read_queue_file`-style logic per file (or inline), return union of `.queued` vecs. Existing `read_queue_file` is channel-keyed — the new helper must be channel-agnostic (walk all `*.json` files).
- `src/main.rs` — Rewrite `show_status`. Needs to: (a) call `scan_queue_files` for queued-only items, (b) call `scan_artifact_statuses` for artifact-dir items, (c) build a deduplication map keyed on `video_id` (artifact-dir row wins over queued row), (d) call `read_metadata` per artifact dir to get title/channel/uploaded_at, (e) print 6-column table. The `show_status` function signature must not change (`async fn show_status(output_root: &Path) -> Result<(), Box<dyn std::error::Error>>`).

### Concrete Implementation Notes

**`scan_queue_files` in artifact.rs:**
```rust
pub fn scan_queue_files(output_root: &Path) -> Result<Vec<VodEntry>, std::io::Error> {
    let queue_dir = output_root.join("queues");
    if !queue_dir.exists() { return Ok(vec![]); }
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
Silently skips malformed queue files (don't abort status on a corrupt queue). This matches the degrade-gracefully constraint for missing metadata.json.

**`show_status` rewrite outline in main.rs:**
```rust
async fn show_status(output_root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Collect artifact-dir items (existing scan)
    let artifact_items = artifact::scan_artifact_statuses(output_root)?;
    
    // 2. Build dedup set from artifact video IDs
    let artifact_ids: HashSet<_> = artifact_items.iter().map(|(id, _)| id.clone()).collect();
    
    // 3. Collect queued-only items (not yet in artifact dirs)
    let queued_vods = artifact::scan_queue_files(output_root)?;
    let queued_only: Vec<_> = queued_vods.into_iter()
        .filter(|v| !artifact_ids.contains(&v.video_id))
        .collect();
    
    // 4. Build display rows — queued rows first (no artifact dir)
    // For each: derive STAGE, TITLE, CHANNEL, DATE, OUTCOME, REASON
    // STAGE logic:
    //   - queued-only row → "queued"
    //   - artifact row, status=None but media exists → "downloaded"
    //   - artifact row, downloaded=false → "queued" (shouldn't happen post-S01 but be safe)
    //   - downloaded=true, transcription_outcome="failed" → "failed"
    //   - downloaded=true, transcription_outcome="suspect" → "suspect"
    //   - ready_for_notes=true → "ready"
    //   - downloaded=true, transcribed=false → "downloaded"
    
    // 5. Print header + rows with truncation
    // TODO(sort): rows appear in filesystem/queue-walk order; sort-by-date-desc is a future enhancement
}
```

**STAGE derivation logic** (complete, canonical):
| Condition | STAGE |
|-----------|-------|
| queued-only (no artifact dir) | `queued` |
| artifact dir, no status.json, media file present | `downloaded` |
| artifact dir, no status.json, no media file | `queued` |
| status.downloaded=false | `queued` |
| status.downloaded=true, transcription_outcome="failed" | `failed` |
| status.downloaded=true, transcription_outcome="suspect" | `suspect` |
| status.ready_for_notes=true | `ready` |
| status.downloaded=true, !transcribed | `downloaded` |
| status.transcribed=true | `ready` |

**Display field sources:**
- For artifact-dir rows: call `artifact::read_metadata(&artifact_dir)` — returns `Option<ArtifactMetadata>`. Use `metadata.title`, `metadata.channel`, `metadata.uploaded_at`. All are `Option<String>` — render `"—"` for None.
- For queued-only rows: `VodEntry.title`, `VodEntry.channel`, `VodEntry.uploaded_at` — all non-optional strings, use directly.
- `uploaded_at` date formatting: `&uploaded_at[..10]` — takes "YYYY-MM-DD" prefix safely. Check `len() >= 10` first; otherwise show `"—"`.

**Column widths (tested at 130 chars):**
```
STAGE(10) TITLE(42) CHANNEL(16) DATE(12) OUTCOME(12) REASON(35)
```
Header line: `{:<10} {:<42} {:<16} {:<12} {:<12} {}` — right-pad all but REASON, which can overflow to end.

**Truncation helpers:**
```rust
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max { s.to_string() }
    else { format!("{}…", &s[..max-1]) }
}
```

### Constraints (all from slice plan)

- `scan_artifact_statuses` signature must NOT change — S05 depends on it
- Old artifact dirs without `metadata.json` must not panic — `read_metadata` returns `Ok(None)`, display `"—"`
- Old artifact dirs without `status.json` but with media must show STAGE `downloaded`
- Queued items that also have an artifact dir must NOT appear twice — artifact-dir row wins
- All `queues/*.json` files must be walked (not just one channel)

### Build Order

1. **Add `scan_queue_files` to `artifact.rs`** + unit test — this is self-contained and unblocks the status rewrite.
2. **Rewrite `show_status` in `main.rs`** — uses `scan_queue_files`, `scan_artifact_statuses`, `read_metadata`.
3. **Run `cargo test`** — existing 16 tests must still pass; add new tests for `scan_queue_files` and the STAGE derivation logic if extracted to a helper.

### Verification Approach

- `cargo build` — must succeed with no new warnings
- `cargo test` — all 16 existing tests pass; new tests for `scan_queue_files` pass
- Fixture test: create a temp dir with:
  - `queues/chan1.json` with 2 VodEntries (IDs "aaa", "bbb")
  - artifact dir `aaa/` with metadata.json (downloaded=true) — "aaa" is in both; artifact row wins
  - artifact dir `ccc/` with media file but no status.json (pre-S01 bare download artifact)
  - artifact dir `ddd/` with status.json (transcription_outcome="suspect")
  - Expected: "bbb" shows as `queued`; "aaa" shows its artifact state; "ccc" shows `downloaded`; "ddd" shows `suspect`
- Manual verification: `./target/debug/vod-pipeline status --output-root <fixture_dir>` and inspect column widths visually

## Common Pitfalls

- **Deduplication key case sensitivity** — `video_id` values in queue files and artifact dir names are numeric strings ("123456789"), so case isn't an issue. But ensure the dedup uses the same string representation.
- **`uploaded_at` slice panic** — Use `len() >= 10` guard before slicing `&s[..10]`. ISO 8601 strings from Twitch are always longer, but queued items from old queue files might have unusual formats.
- **`queues/` dir doesn't exist** — `scan_queue_files` must return `Ok(vec![])` if `output_root/queues/` doesn't exist (not an error — fresh output root has no queues).
- **`read_metadata` on every artifact dir is a per-dir file read** — for typical queue sizes (25-100 items) this is fine. Don't cache or batch; keep it simple.

## Open Risks

None. This is well-scoped, purely additive, and the prior art (existing helpers in artifact.rs) patterns everything needed.
