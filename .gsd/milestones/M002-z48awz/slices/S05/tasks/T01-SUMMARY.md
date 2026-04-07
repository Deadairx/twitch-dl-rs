---
id: T01
parent: S05
milestone: M002-z48awz
key_files:
  - src/cli.rs
  - src/main.rs
  - src/lib.rs
  - src/artifact.rs
key_decisions:
  - is_valid_filter_stage defined in lib.rs for test access; private inline copy in main.rs binary since binary uses mod declarations not the lib crate
  - filter applied post-assembly via two shadow-rebound bindings to keep queued_only and artifact_items logic symmetric
duration: 
verification_result: passed
completed_at: 2026-04-07T03:46:52.261Z
blocker_discovered: false
---

# T01: Wired --filter &lt;stage&gt; end-to-end in status subcommand with validation, filtering, not-found messaging, and 3 unit tests

**Wired --filter &lt;stage&gt; end-to-end in status subcommand with validation, filtering, not-found messaging, and 3 unit tests**

## What Happened

Added filter: Option&lt;String&gt; to the Status CLI variant, registered --filter STAGE arg on the status subcommand, and threaded it through the dispatch block into show_status. Inside show_status, after both item vecs are assembled: validates against the five valid stage strings (stderr + exit 1 on unknown), shadow-rebiinds queued_only and artifact_items via filter application (queued_only passes only when f == queued; artifact_items filtered by derive_stage), and updates the empty-check to distinguish no-filter vs filter-but-no-match. Defined pub fn is_valid_filter_stage in lib.rs for test access; private inline copy in main.rs because the binary uses mod declarations and cannot reference the lib crate externally. Added 3 unit tests in artifact.rs covering all valid stages, invalid/typo inputs, and case-sensitivity.

## Verification

cargo test: 31/31 pass. cargo build: clean. ./target/debug/vod-pipeline status --filter typo 2>&1 | grep -q 'unknown filter' &amp;&amp; echo 'VERIFY OK' printed VERIFY OK.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test 2>&1 | grep -E 'test result|FAILED'` | 0 | ✅ pass | 3000ms |
| 2 | `cargo build --quiet` | 0 | ✅ pass | 100ms |
| 3 | `./target/debug/vod-pipeline status --filter typo 2>&1 | grep -q 'unknown filter' && echo 'VERIFY OK'` | 0 | ✅ pass | 50ms |

## Deviations

Defined both pub const VALID_FILTER_STAGES and pub fn is_valid_filter_stage in lib.rs rather than only a const in main.rs, because main.rs binary cannot reference twitch_dl_rs:: externally — using it caused E0433.

## Known Issues

None.

## Files Created/Modified

- `src/cli.rs`
- `src/main.rs`
- `src/lib.rs`
- `src/artifact.rs`
