---
estimated_steps: 76
estimated_files: 4
skills_used: []
---

# T01: Add --filter flag to status command and apply in show_status

Wire `--filter <stage>` end-to-end: add the field to the CLI struct, register the arg, thread it through dispatch, apply it in show_status, handle the two edge cases, and add unit tests.

## Steps

1. **`src/cli.rs` — CLI struct**: Add `filter: Option<String>` to the `Status { output_root }` variant (line ~39). After the change it reads: `Status { output_root: PathBuf, filter: Option<String> }`.

2. **`src/cli.rs` — subcommand definition**: Inside the `Command::new("status")` block (line ~219), add a new `Arg`:
   ```rust
   .arg(
       Arg::new("filter")
           .long("filter")
           .value_name("STAGE")
           .help("Show only items in the given stage: queued, downloaded, suspect, failed, ready")
           .required(false),
   )
   ```

3. **`src/cli.rs` — parse arm**: In the `Some(("status", status_matches))` arm (line ~383), populate the new field:
   ```rust
   filter: status_matches.get_one::<String>("filter").cloned(),
   ```

4. **`src/main.rs` — dispatch block**: Destructure `filter` from the variant and pass it to `show_status` (line ~72):
   ```rust
   cli::CliCommand::Status { output_root, filter } => {
       if let Err(error) = show_status(&output_root, filter.as_deref()).await {
   ```

5. **`src/main.rs` — `show_status` signature**: Change the function signature at line ~850:
   ```rust
   async fn show_status(output_root: &std::path::Path, filter: Option<&str>) -> Result<(), Box<dyn std::error::Error>>
   ```

6. **`src/main.rs` — filter validation**: Insert immediately after the two item vecs are assembled (after `queued_only` is built, before the early-exit empty check):
   ```rust
   const VALID_STAGES: &[&str] = &["queued", "downloaded", "suspect", "failed", "ready"];
   if let Some(f) = filter {
       if !VALID_STAGES.contains(&f) {
           eprintln!("unknown filter '{}'; valid values: queued, downloaded, suspect, failed, ready", f);
           std::process::exit(1);
       }
   }
   ```

7. **`src/main.rs` — apply filter**: After the validation block, apply the filter to both item collections:
   ```rust
   let queued_only: Vec<_> = if let Some(f) = filter {
       if f == "queued" { queued_only } else { vec![] }
   } else {
       queued_only
   };
   // For artifact_items: build a filtered vec by computing stage inline
   let artifact_items: Vec<_> = if let Some(f) = filter {
       artifact_items
           .into_iter()
           .filter(|(id, status)| {
               let artifact_dir = output_root.join(id);
               derive_stage(status, &artifact_dir) == f
           })
           .collect()
   } else {
       artifact_items
   };
   ```
   Note: `queued_only` is already declared above — shadow/rebind it rather than mutating in place.

8. **`src/main.rs` — not-found case**: Update the early-exit empty check (currently checks pre-filter emptiness) to run after filtering, and add the not-found message:
   ```rust
   if artifact_items.is_empty() && queued_only.is_empty() {
       if let Some(f) = filter {
           println!("No items matching filter '{}'.", f);
       } else {
           println!("No artifacts found in {}", output_root.display());
       }
       return Ok(());
   }
   ```

9. **`src/artifact.rs` — unit tests**: Add 3 tests using the same pattern established in S04. These test the filter logic itself (no I/O needed for the validation path):
   - `test_status_filter_valid_stage_queued`: verify that filter value "queued" is in VALID_STAGES and "downloaded" is also valid
   - `test_status_filter_invalid_stage`: verify that "typo", "QUEUED", and "ready_for_notes" are NOT in VALID_STAGES
   - `test_status_filter_case_sensitive`: verify that "Queued", "FAILED", "Ready" are not valid (case folding is explicitly not supported)
   
   Since VALID_STAGES is defined inside show_status, expose it as a `pub const` at module level in main.rs (or define a helper the tests can call). The simplest approach: define `pub const VALID_FILTER_STAGES: &[&str]` in `src/lib.rs` or at the top of `src/main.rs`, then reference it in show_status and in tests.
   
   Alternatively, write the tests as compile checks against a function: add a `pub fn is_valid_filter_stage(s: &str) -> bool` in `src/main.rs` (accessible via lib.rs) and test that function directly. This is cleanest.

## Inputs

- ``src/cli.rs` — Status variant definition and status subcommand arg registration (lines ~39, ~219, ~383)`
- ``src/main.rs` — dispatch block (line ~72), show_status signature and body (lines ~850–955)`
- ``src/artifact.rs` — existing unit tests for pattern reference (lines ~500+)`
- ``src/lib.rs` — module re-exports for test visibility`

## Expected Output

- ``src/cli.rs` — Status variant with filter field; status subcommand with --filter arg; parse arm populating filter`
- ``src/main.rs` — show_status with filter parameter; VALID_FILTER_STAGES const or is_valid_filter_stage helper; filter validation, application, and not-found message`
- ``src/artifact.rs` — 3 new unit tests for filter validation logic`

## Verification

cargo test 2>&1 | grep -E 'test result|FAILED' && cargo build --quiet 2>&1 && ./target/debug/vod-pipeline status --filter typo 2>&1 | grep -q 'unknown filter' && echo 'VERIFY OK'
