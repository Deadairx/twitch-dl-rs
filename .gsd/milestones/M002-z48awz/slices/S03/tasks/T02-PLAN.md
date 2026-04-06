---
estimated_steps: 4
estimated_files: 3
skills_used: []
---

# T02: Make `download-all` channel optional and add no-channel walk path

**Slice:** S03 — Intake Flexibility
**Milestone:** M002-z48awz

## Description

Change the `channel` argument on `download-all` from required to optional. When omitted, walk all `queues/*.json` files via `artifact::scan_queue_files`, deduplicate against artifact statuses (skip any video_id where `status.downloaded == true`), and process the remaining pending entries with the existing `download_vod_to_artifact` helper. When `channel` is provided, keep the existing single-queue path unchanged.

Prior task T01 must be complete before this task is tested end-to-end (so there's a queue file created by `queue-video` to drain). The code changes themselves are independent.

Key implementation notes:
- `artifact::scan_queue_files(output_root)` returns `Vec<VodEntry>` across all `queues/*.json` files — already public and tested.
- `artifact::scan_artifact_statuses(output_root)` returns `Vec<(String, Option<ProcessStatus>)>` — already public. Collect video_ids where `status.downloaded == true` into a `HashSet<String>` for O(1) lookup.
- The existing single-channel path (`Some(channel)`) must be identical to the current behavior — no regressions.
- The `DownloadAll` dispatch in `main` currently destructures `channel: String`. After this change it destructures `channel: Option<String>` and passes `channel.as_deref()` to `download_all`.

## Steps

1. **`src/cli.rs`** — Change `DownloadAll.channel` from `String` to `Option<String>`. In the `download-all` subcommand definition, change the `channel` arg from `.required(true)` to `.required(false)` (keep `index(1)`). Update the match arm: `get_one::<String>("channel").cloned()` (returns `Option<String>`, remove the `.expect(...)` call). Update `.about()` text to reflect optionality.

2. **`src/main.rs`** — Change `download_all` signature from `channel: &str` to `channel: Option<&str>`. Wrap the existing body in `match channel { Some(ch) => { /* existing body unchanged, replace bare `channel` refs with `ch` */ }, None => { /* new path */ } }`. In the `None` arm:
   - Call `artifact::scan_queue_files(output_root)?` to get all `VodEntry` items
   - Call `artifact::scan_artifact_statuses(output_root)?`, collect `video_id`s with `downloaded == true` into a `HashSet<String>`
   - Filter `all_vods` to those whose `video_id` is not in the HashSet
   - If empty, `println!("All queued VODs already downloaded.")` and return `Ok(())`
   - `println!("Downloading {} pending VOD(s) across all channels...", pending.len())`
   - Iterate with `download_vod_to_artifact` (same pattern as the `Some(ch)` arm); respect `continue_on_error`

3. **`src/main.rs`** — Update the `CliCommand::DownloadAll` match arm in `main`: change `channel: String` destructure to `channel: Option<String>`, pass `channel.as_deref()` to `download_all`.

4. **`src/artifact.rs` `#[cfg(test)]` module** — Add two unit tests:
   - `test_download_all_no_channel_filter`: set up two queue files (3 total `VodEntry` items) using `write_queue_file`, create one artifact dir with `status.json` (`downloaded=true`) using `write_status`. Run the filter logic directly: call `scan_queue_files`, call `scan_artifact_statuses`, build HashSet of downloaded IDs, filter. Assert pending has 2 entries.
   - `test_download_all_channel_regression`: set up one queue file with 2 entries, 1 with a downloaded artifact dir. Assert `read_queue_file` + filter-by-status produces 1 pending entry. (This mirrors the existing single-channel path to guard against regressions.)
   
   Note: this is a binary-only crate with no `src/lib.rs`. The async `download_all` handler in `main.rs` cannot be directly unit-tested; tests verify the underlying data-layer logic used by the handler.

## Must-Haves

- [ ] `download-all --help` shows `[CHANNEL]` (optional, square brackets), not `<CHANNEL>` (required)
- [ ] `download-all` with no channel arg walks all `queues/*.json` and downloads pending items
- [ ] `download-all` with a channel arg behaves identically to the previous implementation
- [ ] Items with `status.downloaded == true` are skipped in the no-channel path
- [ ] `cargo build` clean; `cargo test` passes (all prior tests + new filter tests)

## Verification

```bash
cargo build
cargo test
./target/debug/vod-pipeline download-all --help
# Confirm channel arg shows as [CHANNEL] (square brackets = optional)
```

New test targets: `cargo test test_download_all`

## Inputs

- `src/cli.rs` — `CliCommand::DownloadAll` variant and `download-all` subcommand definition (modified by T01 or already present)
- `src/main.rs` — `download_all` async fn and `CliCommand::DownloadAll` match arm
- `src/artifact.rs` — `scan_queue_files`, `scan_artifact_statuses` (public API, no changes needed to these functions)
- `src/artifact.rs` — existing `#[cfg(test)]` module (T01 may have added tests here)

## Expected Output

- `src/cli.rs` — `DownloadAll.channel` changed to `Option<String>`; clap arg changed to `.required(false)`; match arm updated to pass `channel.as_deref()`
- `src/main.rs` — `download_all` signature updated to `Option<&str>`; no-channel walk path added; dispatch updated
- `src/artifact.rs` — `test_download_all_no_channel_filter` and `test_download_all_channel_regression` added to existing `#[cfg(test)]` module
