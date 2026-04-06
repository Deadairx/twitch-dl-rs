# S03: Intake Flexibility

**Goal:** Add `queue-video <url>` for single-VOD intake and make `download-all` channel-argument optional so it drains all queues without requiring a channel name.
**Demo:** Run `queue-video` on a Twitch URL, then run `download-all` with no arguments and watch the queued item download.

## Must-Haves

- `queue-video <url>` subcommand: resolves channel via `fetch_vod_metadata_by_id`, merges entry into `queues/<channel>.json`
- `queue-video` is idempotent: running it twice with the same URL prints `"Already queued: <id>"` and exits 0
- `queue-video` aborts with a clear error if the GQL metadata fetch fails
- `download-all` channel argument is optional: omitting it walks all `queues/*.json` and downloads all pending entries
- `download-all` with a channel arg behaves identically to the current implementation (no regressions)
- Deduplication in no-channel `download-all` is artifact-state-based: skip video_id if `status.downloaded == true`
- `cargo build` succeeds; `cargo test` passes (all 21 existing tests + new unit tests for both features)

## Requirement Impact

- **Requirements touched**: R012 (resumable/skippable work — the no-channel path must correctly skip already-downloaded items)
- **Re-verify**: `download-all` single-channel path unchanged; skip logic still works after signature change
- **Decisions revisited**: D012 (download-all channel arg optionality — this is the implementation of that decision)

## Proof Level

- This slice proves: contract
- Real runtime required: no (build + test + help-text inspection)
- Human/UAT required: no

## Verification

```bash
cargo build
cargo test                                            # 21 existing + new tests must all pass
./target/debug/vod-pipeline queue-video --help        # subcommand visible, positional <URL> arg shown
./target/debug/vod-pipeline download-all --help       # channel shown as [CHANNEL], not <CHANNEL>
```

New tests in `src/artifact.rs` `#[cfg(test)]` module:
- `test_queue_video_idempotent_dedup` — write a queue file with one entry, verify idempotent check triggers on same video_id, then verify new video_id appends correctly
- `test_download_all_no_channel_filter` — set up two queue files (3 total entries, 1 with a downloaded artifact), verify pending list has 2 entries
- `test_download_all_channel_regression` — verify single-channel path still reads only its queue file and filters by downloaded status

## Integration Closure

- Upstream surfaces consumed: `src/artifact.rs` (read_queue_file, write_queue_file, scan_queue_files, scan_artifact_statuses), `src/twitch.rs` (extract_video_id, fetch_vod_metadata_by_id), `src/cli.rs` (CliCommand), `src/main.rs` (dispatch)
- New wiring introduced in this slice: QueueVideo CLI variant + queue_video async handler; DownloadAll.channel changed to Option<String>; no-channel walk path added
- What remains before the milestone is truly usable end-to-end: S04 adds --video-id filtering on top of the optional-channel download-all

## Tasks

- [ ] **T01: Add `queue-video` command** `est:45m`
  - Why: Adds single-VOD intake by URL; removes the requirement to pre-know a channel name to queue a video
  - Files: `src/cli.rs`, `src/main.rs`, `src/artifact.rs`
  - Do: Add `QueueVideo { url, output_root }` variant to `CliCommand`; add `queue-video` subcommand def with positional `url` arg; add `queue_video` async handler in `main.rs` (extract_video_id → fetch_vod_metadata_by_id → read existing queue or start fresh → dedup check → append VodEntry with duration "PT0S" → write_queue_file); add idempotent dedup unit test in `src/artifact.rs`
  - Verify: `cargo build && cargo test && ./target/debug/vod-pipeline queue-video --help`
  - Done when: `cargo test test_queue_video` passes and help shows `<URL>` positional arg
- [ ] **T02: Make `download-all` channel optional and add no-channel walk path** `est:45m`
  - Why: Closes the "drain everything" workflow; operators no longer need to supply a channel name to download all pending VODs
  - Files: `src/cli.rs`, `src/main.rs`, `src/artifact.rs`
  - Do: Change `DownloadAll.channel` from `String` to `Option<String>`; change clap arg from `.required(true)` to `.required(false)`; update `download_all` signature to `Option<&str>`; wrap existing body in `match channel { Some(ch) => existing_path, None => walk_all_queues }` — the None arm calls `scan_queue_files` + `scan_artifact_statuses` + HashSet dedup + iterate with `download_vod_to_artifact`; add regression and no-channel-filter unit tests in `src/artifact.rs`
  - Verify: `cargo build && cargo test && ./target/debug/vod-pipeline download-all --help`
  - Done when: `cargo test test_download_all` passes and help shows `[CHANNEL]` (square brackets = optional)

## Files Likely Touched

- `src/cli.rs`
- `src/main.rs`
- `src/artifact.rs`
