---
id: S02
milestone: M002-z48awz
status: ready
---

# S02: Status Legibility — Context

## Goal

Replace the current ID-only status table with a human-readable display that shows title, date, and channel for every row, including queued-but-not-yet-downloaded items.

## Why this Slice

The current `status` command outputs VIDEO_ID, DOWNLOADED, OUTCOME, READY, and REASON — all machine-oriented fields that require the operator to cross-reference queue files to understand what each row represents. S01 lands `read_metadata` and the display fields in `metadata.json`; S02 is the first slice that actually makes those fields visible to the operator. S05 (Queue-Aware Filtering) depends on S02 being done first.

## Scope

### In Scope

- Read `metadata.json` per artifact dir via `read_metadata` (delivered by S01) and surface `title`, `channel`, `uploaded_at` in the status table
- Replace the DOWNLOADED boolean column with a STAGE column expressing the item's current state as a single human-readable token: `queued`, `downloaded`, `suspect`, `failed`, `ready`
- Show queued-but-not-downloaded items in the **default** status view — no flag required. These come from queue files in `queues/`, not artifact dirs. S02 must read both sources and merge them
- Truncate title to ~40 chars; format `uploaded_at` as a short date (take the `YYYY-MM-DD` prefix of the ISO 8601 string — no date library needed)
- Show channel name in a CHANNEL column
- Items without `metadata.json` (artifact dir exists, metadata missing) show `—` for title/channel/date rather than failing
- Items with a media file present but no `status.json` (pre-S01 bare-download artifacts) show STAGE as `downloaded` — inferred from media file presence, not from status.json
- **Column layout: single-row, 6 visible columns** — `STAGE | TITLE | CHANNEL | DATE | OUTCOME | REASON`. REASON is shown truncated inline for all rows (empty items show `—`). This fits comfortably at 130-char terminal width with aggressive truncation. If total width still overflows, executor may drop REASON and add a `--verbose` flag for it — document the choice in a decision
- `cargo test` still passes; `cargo build` succeeds

### Out of Scope

- `--filter` flag on status — that is S05
- Sorting or ordering controls — explicitly deferred. Future enhancement; note the absence in code comments or a decision entry so it's easy to add later
- Any changes to queue-video, download-all, or transcribe-all
- Color/ANSI formatting — plain text table only
- Pagination
- `--verbose` flag (unless the executor determines REASON causes overflow — see constraints)

## Constraints

- S01 must be complete — `read_metadata` and the three new `ArtifactMetadata` fields are required
- Old artifact dirs without `metadata.json` must not panic or hard-error — degrade gracefully to `—` values
- Old artifact dirs without `status.json` but with a media file present must show STAGE `downloaded` — infer from filesystem, not status
- Queued items that also have an artifact dir must not appear twice — the artifact-dir row wins (it has richer state)
- `scan_artifact_statuses` in `artifact.rs` must not change its signature — S05 depends on it. Add a new helper or extend inline in `show_status` instead
- All queue files under `queues/*.json` must be walked; the output root may have multiple channels' queue files

## Integration Points

### Consumes

- `src/artifact.rs` — `read_metadata(artifact_dir)` (new, from S01); `scan_artifact_statuses` (existing); `QueueFile` struct and `read_queue_file` (existing) to load queued VodEntry items
- `src/main.rs` — `show_status` function (existing, gets rewritten)

### Produces

- `src/artifact.rs` — new `scan_queue_files(output_root) -> Result<Vec<VodEntry>>` helper that reads all `queues/*.json` files and returns the union of their `queued` vecs (or this logic lives inline in `show_status` if the executor prefers)
- `src/main.rs` — updated `show_status` with merged queue+artifact view and new 6-column layout: `STAGE | TITLE | CHANNEL | DATE | OUTCOME | REASON`

## Open Questions

- **REASON column overflow**: If 6 columns don't fit at 120 chars with reasonable truncation, the executor should drop REASON from the default view and add a `--verbose` flag that includes it. Document this as a decision in DECISIONS.md if taken.
- **Sorting**: No ordering is imposed in S02 — rows appear in filesystem/queue-walk order. A sort-by-date-desc or sort-by-stage option is an explicit future enhancement. Executor should leave a `// TODO(sort): ...` comment at the sort site so it's easy to locate later.
