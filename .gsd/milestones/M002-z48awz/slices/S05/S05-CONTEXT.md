---
id: S05
milestone: M002-z48awz
status: ready
---

# S05: Queue-Aware Filtering — Context

## Goal

Add `--filter <stage>` to the `status` command so the operator can narrow the view to items in a specific pipeline stage.

## Why this Slice

S02 delivers the full merged status table (queued + artifact items). S05 adds the ability to focus that table — essential once the pipeline has many items and the operator only cares about failures or items waiting to download. S06 (Retry and Operational Hardening) depends on S05 being complete first.

## Scope

### In Scope

- `--filter <stage>` flag on `status` — filters the merged table to rows whose STAGE column matches the given value
- Valid filter values align exactly with S02's STAGE tokens: `queued`, `downloaded`, `suspect`, `failed`, `ready`
- If the filter value is unrecognized: exit non-zero with a clear message listing valid values, e.g. `"unknown filter 'typo'; valid values: queued, downloaded, suspect, failed, ready"`
- If the filter matches nothing: print `"No items matching filter '<value>'."` and exit 0 — not an error
- `cargo test` passes; `cargo build` succeeds

### Out of Scope

- Multiple filter values in one invocation (e.g. `--filter failed,suspect`) — single value only; composing filters is a future enhancement
- `--filter` on any command other than `status`
- Sorting or ordering — explicitly deferred (see S02 context)
- Any changes to download-all, transcribe-all, or queue commands

## Constraints

- Filter values must exactly match the STAGE tokens defined in S02 — no aliases, no case folding beyond what the STAGE column already emits
- `scan_artifact_statuses` signature must not change — the filter is applied in `show_status` after the full set is assembled, not inside the scan helper
- The `--filter` flag name is canonical; do not use `--stage`

## Integration Points

### Consumes

- `src/main.rs` — `show_status` function (from S02); merged queue+artifact item list with STAGE values already computed
- `src/cli.rs` — `status` subcommand definition (gets `--filter` flag added)

### Produces

- `src/cli.rs` — `--filter <stage>` optional flag on `status` subcommand
- `src/main.rs` — `show_status` updated to accept and apply `Option<&str>` filter before rendering the table; not-found and invalid-value error paths

## Open Questions

- None. All decisions resolved in discussion.
