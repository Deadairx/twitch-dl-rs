---
estimated_steps: 5
estimated_files: 2
skills_used: []
---

# T02: Rewrite show_status in main.rs with 6-column layout and STAGE derivation

**Slice:** S02 — Status Legibility
**Milestone:** M002-z48awz

## Description

Rewrite the `show_status` function in `src/main.rs` to produce a human-readable 6-column status table:
`STAGE | TITLE | CHANNEL | DATE | OUTCOME | REASON`

The rewrite must:
1. Merge queued-only items from `artifact::scan_queue_files` with artifact-dir items from `artifact::scan_artifact_statuses`
2. Deduplicate by video_id (artifact-dir row wins)
3. Derive a STAGE token per item
4. Look up display metadata (title, channel, uploaded_at) via `artifact::read_metadata` for artifact-dir items
5. Print a properly truncated, fixed-width table that fits at 130-char terminal width

T01 must be complete before starting T02 — `scan_queue_files` must be available in `artifact.rs`.

## Steps

1. **Add `truncate` helper in `src/main.rs`** (module-level, not inside `show_status`):

```rust
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}
```

Use `saturating_sub(1)` instead of `max-1` to avoid any potential underflow (defensive, even though max will always be > 1 in practice).

2. **Rewrite `show_status`** in `src/main.rs`. The signature must NOT change:
```rust
async fn show_status(output_root: &std::path::Path) -> Result<(), Box<dyn std::error::Error>>
```

Full implementation outline:

```rust
async fn show_status(output_root: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Artifact-dir items
    let artifact_items = artifact::scan_artifact_statuses(output_root)?;
    let artifact_ids: std::collections::HashSet<String> = artifact_items
        .iter()
        .map(|(id, _)| id.clone())
        .collect();

    // 2. Queued-only items (not already in artifact dirs)
    let queued_vods = artifact::scan_queue_files(output_root)?;
    let queued_only: Vec<_> = queued_vods
        .into_iter()
        .filter(|v| !artifact_ids.contains(&v.video_id))
        .collect();

    // Early exit if nothing to show
    if artifact_items.is_empty() && queued_only.is_empty() {
        println!("No artifacts found in {}", output_root.display());
        return Ok(());
    }

    // TODO(sort): rows appear in filesystem/queue-walk order; sort-by-date-desc is a future enhancement
    let total = artifact_items.len() + queued_only.len();

    println!(
        "{:<10} {:<42} {:<16} {:<12} {:<12} {}",
        "STAGE", "TITLE", "CHANNEL", "DATE", "OUTCOME", "REASON"
    );
    println!("{}", "-".repeat(105));

    // Queued-only rows first
    for vod in &queued_only {
        let date = if vod.uploaded_at.len() >= 10 {
            vod.uploaded_at[..10].to_string()
        } else {
            "—".to_string()
        };
        println!(
            "{:<10} {:<42} {:<16} {:<12} {:<12} {}",
            "queued",
            truncate(&vod.title, 40),
            truncate(&vod.channel, 14),
            date,
            "—",
            "—",
        );
    }

    // Artifact-dir rows
    for (video_id, status) in &artifact_items {
        let artifact_dir = output_root.join(video_id);
        let metadata = artifact::read_metadata(&artifact_dir).unwrap_or(None);

        let title = metadata
            .as_ref()
            .and_then(|m| m.title.as_deref())
            .unwrap_or("—")
            .to_string();
        let channel = metadata
            .as_ref()
            .and_then(|m| m.channel.as_deref())
            .unwrap_or("—")
            .to_string();
        let uploaded_at = metadata
            .as_ref()
            .and_then(|m| m.uploaded_at.as_deref())
            .unwrap_or("");
        let date = if uploaded_at.len() >= 10 {
            uploaded_at[..10].to_string()
        } else {
            "—".to_string()
        };

        let stage = derive_stage(status, &artifact_dir);

        let outcome = status
            .as_ref()
            .and_then(|s| s.transcription_outcome.as_deref())
            .unwrap_or("—");
        let reason = status
            .as_ref()
            .and_then(|s| {
                s.transcription_reason
                    .as_deref()
                    .or(s.last_error.as_deref())
            })
            .unwrap_or("—");

        println!(
            "{:<10} {:<42} {:<16} {:<12} {:<12} {}",
            stage,
            truncate(&title, 40),
            truncate(&channel, 14),
            date,
            outcome,
            truncate(reason, 35),
        );
    }

    println!("\n{} item(s) total", total);
    Ok(())
}
```

3. **Add `derive_stage` helper function** in `src/main.rs` (module-level). This makes the STAGE logic testable in isolation and keeps `show_status` readable:

```rust
fn derive_stage(status: &Option<artifact::ProcessStatus>, artifact_dir: &std::path::Path) -> &'static str {
    match status {
        None => {
            if artifact::find_media_file(artifact_dir).is_some() {
                "downloaded"
            } else {
                "queued"
            }
        }
        Some(s) => {
            if !s.downloaded {
                "queued"
            } else if s.transcription_outcome.as_deref() == Some("failed") {
                "failed"
            } else if s.transcription_outcome.as_deref() == Some("suspect") {
                "suspect"
            } else if s.ready_for_notes {
                "ready"
            } else if s.transcribed {
                "ready"
            } else {
                "downloaded"
            }
        }
    }
}
```

**STAGE derivation canonical table:**
| Condition | STAGE |
|-----------|-------|
| status=None, media file present | `downloaded` |
| status=None, no media file | `queued` |
| status.downloaded=false | `queued` |
| status.transcription_outcome="failed" | `failed` |
| status.transcription_outcome="suspect" | `suspect` |
| status.ready_for_notes=true | `ready` |
| status.transcribed=true | `ready` |
| status.downloaded=true, !transcribed | `downloaded` |

4. **Add a fixture-based deduplication test in `src/artifact.rs`** (not `main.rs` — `show_status` isn't directly unit-testable from lib.rs). Test name: `test_scan_queue_dedup_with_artifact`. Setup:
   - `queues/chan1.json` with 2 VodEntries: IDs "100" and "200"
   - Artifact dir `100/` with `metadata.json` (title="My VOD", downloaded=true) — "100" appears in both sources
   - Artifact dir `300/` with `audio.m4a` file but no status.json (pre-S01 bare download)
   - Assert: `scan_queue_files` returns 2 entries (IDs "100" and "200")
   - Assert: `scan_artifact_statuses` returns 2 entries (IDs "100" and "300")
   - Assert: filtering `scan_queue_files` results by IDs not in `scan_artifact_statuses` yields only "200"
   - This test verifies the dedup logic inputs are correct without testing `show_status` directly

5. Run `cargo build` and `cargo test`. Both must pass cleanly.

## Must-Haves

- [ ] `show_status` signature unchanged: `async fn show_status(output_root: &std::path::Path) -> Result<(), Box<dyn std::error::Error>>`
- [ ] `scan_artifact_statuses` signature unchanged
- [ ] 6-column layout: STAGE | TITLE | CHANNEL | DATE | OUTCOME | REASON
- [ ] TITLE truncated to ≤40 chars; CHANNEL to ≤14 chars; REASON to ≤35 chars
- [ ] DATE is `uploaded_at[..10]` (YYYY-MM-DD), guarded by `len() >= 10` check
- [ ] Queued-only items appear before artifact-dir rows; no duplicates
- [ ] `None` metadata fields render as `—` (em dash), not empty string or `-`
- [ ] `derive_stage` helper function present at module level
- [ ] `truncate` helper present at module level
- [ ] `// TODO(sort):` comment present before row-collection logic
- [ ] `test_scan_queue_dedup_with_artifact` fixture test added to `src/artifact.rs`
- [ ] `cargo build` clean; `cargo test` all tests pass

## Negative Tests

- **Missing metadata.json**: artifact dir exists, metadata.json absent → `read_metadata` returns `Ok(None)` → title/channel/date all render as `—` — no panic
- **Missing status.json with media file**: `derive_stage` with `None` status + `find_media_file` returning Some → returns `"downloaded"`
- **Missing status.json without media file**: `derive_stage` with `None` status + no media → returns `"queued"`
- **`uploaded_at` shorter than 10 chars**: the `len() >= 10` guard must prevent a panic; render `—` instead
- **Empty output root**: both scan functions return empty → early exit message printed, no crash

## Verification

- `cargo build --quiet 2>&1 | grep '^error'` — must produce no output (zero build errors)
- `cargo test --quiet 2>&1 | grep -E 'test result|FAILED'` — must show all tests passed (21+ ok), 0 failed
- Manual spot-check: run `./target/debug/vod-pipeline status --output-root /tmp/fixture_test` with a hand-created fixture dir and confirm the column layout renders legibly at ~130 chars

## Inputs

- `src/main.rs` — existing `show_status` function to be rewritten; existing imports and module structure
- `src/artifact.rs` — `scan_queue_files` (from T01), `scan_artifact_statuses`, `read_metadata`, `find_media_file`, `ProcessStatus`, `ArtifactMetadata`

## Expected Output

- `src/main.rs` — rewritten `show_status`, new `derive_stage` helper, new `truncate` helper
- `src/artifact.rs` — `test_scan_queue_dedup_with_artifact` fixture test added
