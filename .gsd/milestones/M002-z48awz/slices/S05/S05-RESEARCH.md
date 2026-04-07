# S05: Queue-Aware Filtering — Research

## Summary

Straightforward wiring task. The `derive_stage` function and merged queue+artifact display already exist from S02. S05 adds a `--filter <stage>` flag to the `status` subcommand and applies it in `show_status` before rendering. No new patterns needed — the implementation uses existing conventions throughout.

## Recommendation

Single task. Add `--filter` to CLI, thread it through the dispatch chain, apply a post-assembly filter in `show_status`, and handle the two edge cases (unrecognized value → exit 1 with error; valid value matches nothing → print message, exit 0). Estimated 30–45 minutes.

## Implementation Landscape

### `src/cli.rs`

**`CliCommand::Status` variant** (line 39–41):
```rust
Status {
    output_root: PathBuf,
},
```
Add `filter: Option<String>` field here.

**`status` subcommand definition** (line 219–228):
```rust
Command::new("status")
    .about("Show status of all downloaded/transcribed artifacts")
    .arg(output_root_arg(...))
```
Add a `--filter` `Arg` with `required(false)` and appropriate help text listing valid values.

**Parse arm** (line 383–390):
```rust
Some(("status", status_matches)) => Cli {
    command: CliCommand::Status {
        output_root: ...,
    },
},
```
Add `.get_one::<String>("filter").cloned()` to populate the new field.

### `src/main.rs`

**Dispatch block** (line 72–75):
```rust
cli::CliCommand::Status { output_root } => {
    if let Err(error) = show_status(&output_root).await {
```
Destructure `filter` from the variant and pass it to `show_status`.

**`show_status` signature** (line 850):
```rust
async fn show_status(output_root: &std::path::Path) -> Result<(), Box<dyn std::error::Error>>
```
Change to accept `filter: Option<&str>`.

**Filter logic** — insert after the merged list is assembled (after the queued_only/artifact_items collection, before the early-exit check). Two behaviors:
1. Unrecognized value: `eprintln!("unknown filter '{}'; valid values: queued, downloaded, suspect, failed, ready", v); std::process::exit(1);`
2. Valid value: filter `queued_only` and `artifact_items` to only rows whose stage matches. For `queued_only`, all rows have stage `"queued"`. For `artifact_items`, compute stage inline (or pass through `derive_stage`).

**Not-found case**: after filtering, if both lists are empty (and filter was provided), print `"No items matching filter '{value}'."` and return `Ok(())`.

### Valid stage tokens

From `derive_stage` (line 823–848), the five emitted values are:
- `"queued"` — status.json absent + no media file, OR `downloaded == false`
- `"downloaded"` — `downloaded == true`, not transcribed, no outcome
- `"failed"` — `transcription_outcome == "failed"`
- `"suspect"` — `transcription_outcome == "suspect"`
- `"ready"` — `ready_for_notes == true` OR `transcribed == true`

These are the exact values the `--filter` flag must accept. No aliases, no case folding.

### Filter application pattern

For `queued_only` rows: their stage is always `"queued"`. Filter is trivial: `if filter != "queued" { queued_only.clear() }` (or equivalently, `queued_only.retain(|_| filter == "queued")`).

For `artifact_items` rows: stage must be re-derived to filter. Two options:
1. Compute stage inline during filter pass (call `derive_stage` once per item before rendering loop).
2. Pre-compute stage into a parallel vec, zip for filter, then render.

Option 1 is simpler. `derive_stage` is pure and cheap — calling it twice (once for filter, once for render) is fine. Or build a filtered intermediate that carries `(video_id, status, stage)`.

The cleanest approach: build a `Vec<(String, Option<ProcessStatus>, &'static str)>` for artifact rows by computing stage once, then filter by stage, then render. This avoids calling `derive_stage` twice in the render loop.

### No new tests needed beyond the existing pattern

S02's fixture-based test (test_scan_queue_dedup_with_artifact) already covers the merged view. S05 should add unit tests for:
- Filter with a known matching value returns only matching rows
- Filter with an unrecognized value returns early (this is harder to unit-test for exit code; integration-style test or just verify in cargo build + manual check)
- Filter with valid value that matches nothing prints the "No items matching" message

The existing 28 tests must continue passing.

## Constraints

- `scan_artifact_statuses` signature must not change (S05 context explicit)
- `--filter` flag name is canonical — do not use `--stage`
- Filter is single-value only (composable multi-filter is future scope)
- Filter applies only to `status` — no changes to download-all, transcribe-all, or queue commands
- Valid values must exactly match `derive_stage` output tokens — no aliases, no case folding

## Files Touched

| File | Change |
|------|--------|
| `src/cli.rs` | Add `filter: Option<String>` to `Status` variant; add `--filter` arg to status subcommand; populate in parse arm |
| `src/main.rs` | Thread `filter` through dispatch; update `show_status` signature; add validation + filter application before render loop |
| `src/artifact.rs` | Add 1–2 unit tests for filter behavior (optional but consistent with S04 pattern) |
