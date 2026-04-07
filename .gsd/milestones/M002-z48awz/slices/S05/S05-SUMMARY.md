---
id: S05
parent: M002-z48awz
milestone: M002-z48awz
provides:
  - --filter <stage> flag on status command enabling operator-driven view narrowing to specific pipeline stages
  - Handler-level filtering pattern (post-assembly, shadow-rebind) for clean composition with future multi-stage filters
requires:
  - slice: S02
    provides: Merged queue+artifact item list with STAGE column already computed; show_status foundation
affects:
  - S06 (Retry and Operational Hardening — depends on filter-capable status for selective recovery)
key_files:
  - src/cli.rs
  - src/main.rs
  - src/lib.rs
  - src/artifact.rs
key_decisions:
  - is_valid_filter_stage defined in lib.rs for test access; private inline VALID_STAGES copy in main.rs binary since binary uses mod declarations and cannot reference lib crate externally
  - filter applied post-assembly via two shadow-rebound bindings (queued_only and artifact_items) to keep logic symmetric and testable without modifying scan_artifact_statuses signature
  - case-sensitive stage matching enforced (no aliases, no folding) to avoid operator confusion and simplify the validation model
patterns_established:
  - Handler-level filtering (post-assembly) decouples filtering from CLI parsing and scan logic. New filters in S06+ can stack by chaining post-filter operations without touching core scan functions or CLI arg complexity.
  - Symmetrical shadow-rebinding of both item collections with the same filter predicate maintains consistency and reduces cognitive load for future agents maintaining this code.
observability_surfaces:
  - status command with --filter flag: operator diagnostic tool for narrowing view to pipeline stage (no new metrics or logging surfaces added)
drill_down_paths:
  - .gsd/milestones/M002-z48awz/slices/S05/tasks/T01-SUMMARY.md
duration: ""
verification_result: passed
completed_at: 2026-04-07T04:16:54.549Z
blocker_discovered: false
---

# S05: Queue-Aware Filtering

**Added --filter <stage> flag to status command enabling operators to narrow artifact+queue table to specific pipeline stages (queued, downloaded, suspect, failed, ready)**

## What Happened

Implemented queue-aware filtering by adding a --filter <stage> argument to the status subcommand. The filter threads through the CLI struct, dispatch block, and show_status function. Inside show_status, after both item vecs (queued_only and artifact_items) are assembled, the filter is validated against VALID_STAGES constant, then applied via shadow-rebound bindings to both collections. This post-assembly approach preserves scan_artifact_statuses contract and enables future multi-stage filters to compose cleanly. Three unit tests verify valid stages pass, invalid/typo inputs fail, and case-sensitivity is enforced. All 31 tests pass, build succeeds clean, and manual verification confirms filtering works correctly for all five stage values, not-found cases print appropriate messages, and invalid filters exit with helpful error text listing valid values.

## Verification

cargo test: 31/31 pass including 3 new filter tests. cargo build: succeeds clean. Manual verification: status --filter failed shows only failed items (exit 0), status --filter queued shows 'No items matching filter' message (exit 0), status --filter typo exits 1 with 'unknown filter' error and valid values list. Help text documents --filter flag. Backward compatibility preserved: status with no filter shows all items.

## Requirements Advanced

None.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Deviations

None.

## Known Limitations

Single-stage filters only; composite filters deferred. No sorting within filtered results (already deferred in S02). No --filter on download-all, transcribe-all, or queue (out of scope).

## Follow-ups

None.

## Files Created/Modified

None.
