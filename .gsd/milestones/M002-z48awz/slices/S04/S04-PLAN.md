# S04: Selective Processing

**Goal:** Add `--video-id <id>` filtering to `download-all` and `transcribe-all` so the operator can target a single item without processing the entire queue.
**Demo:** After this: After this: run download-all --video-id 123456789 and watch only that one item download while others are skipped.

## Tasks
- [x] **T01: Added --video-id optional argument to download-all and transcribe-all subcommands with filtering logic** — Add `video_id: Option<String>` to `CliCommand::DownloadAll` and `CliCommand::TranscribeAll` in `src/cli.rs`, register the `--video-id` arg on both subcommands, and populate the field in the two `parse_args()` match arms. This is purely additive — no handler logic changes yet.

## Steps

1. Open `src/cli.rs`. Locate the `DownloadAll` variant (line ~42). Add `video_id: Option<String>` field after `continue_on_error`.
2. Locate the `TranscribeAll` variant (line ~48). Add `video_id: Option<String>` field after `continue_on_error`.
3. Locate the `Command::new("download-all")` block (line ~224). Copy the `Arg::new("video-id")` registration from the `cleanup` subcommand (line ~281) and add it to the download-all args. Update the help text to: `"Process only the VOD with this video ID"`.
4. Locate the `Command::new("transcribe-all")` block (~line 250). Add the same `Arg::new("video-id")` arg with help text: `"Transcribe only the artifact with this video ID"`.
5. Locate the `Some(("download-all", download_all_matches))` match arm (line ~378). Add `video_id: download_all_matches.get_one::<String>("video-id").cloned()` to the `CliCommand::DownloadAll { ... }` struct literal.
6. Locate the `Some(("transcribe-all", transcribe_all_matches))` match arm (line ~396). Add `video_id: transcribe_all_matches.get_one::<String>("video-id").cloned()` to the struct literal.
7. Run `cargo build` to confirm no errors.
  - Estimate: 20m
  - Files: src/cli.rs
  - Verify: cargo build && ./target/debug/vod-pipeline download-all --help | grep -q 'video-id' && ./target/debug/vod-pipeline transcribe-all --help | grep -q 'video-id'
- [x] **T02: Extend download_all and transcribe_all with video_id post-filter and not-found error; add 4 unit tests** — Update `src/main.rs` to: (1) add `video_id: Option<&str>` parameter to `download_all` and `transcribe_all`, (2) apply a post-filter on `pending` in both functions, (3) return a clear error when the ID is not found, (4) update the dispatch block to pass `.as_deref()` for both commands. Add 4 unit tests in `src/artifact.rs` proving filter and not-found behavior.

## Steps

1. Open `src/main.rs`. Change `download_all` signature from `(channel: Option<&str>, output_root: ..., quality: ..., continue_on_error: bool)` to add `video_id: Option<&str>` as the last parameter.

2. In `download_all`, after both the `Some(ch)` branch and the `None` branch build their `pending: Vec<VodEntry>`, apply this filter (applies to BOTH branches identically, so it goes after the pending vec is assembled in each branch — insert once per branch after the existing filter logic):
```rust
let pending = if let Some(id) = video_id {
    let filtered: Vec<_> = pending.into_iter().filter(|v| v.video_id == id).collect();
    if filtered.is_empty() {
        return Err(format!("video ID {id} not found in any queue").into());
    }
    filtered
} else {
    pending
};
```

3. Change `transcribe_all` signature to add `video_id: Option<&str>` as the last parameter.

4. In `transcribe_all`, after the existing `let pending: Vec<_> = items.into_iter().filter_map(...).collect();` block, apply:
```rust
let pending = if let Some(id) = video_id {
    let filtered: Vec<_> = pending.into_iter().filter(|(vid, _)| vid == id).collect();
    if filtered.is_empty() {
        return Err(format!("video ID {id} not found in any artifact").into());
    }
    filtered
} else {
    pending
};
```

5. In the dispatch block, update the `CliCommand::DownloadAll` arm to destructure `video_id` and pass it: change the `download_all(channel.as_deref(), ...)` call to `download_all(channel.as_deref(), &output_root, quality, continue_on_error, video_id.as_deref()).await`.

6. Update the `CliCommand::TranscribeAll` arm to destructure `video_id` and pass it: change the `transcribe_all(&output_root, continue_on_error)` call to `transcribe_all(&output_root, continue_on_error, video_id.as_deref()).await`.

7. Open `src/artifact.rs`. Add 4 unit tests following the patterns from `test_download_all_no_channel_filter` and `test_download_all_channel_regression`:

- `test_download_all_video_id_filter`: Create a temp dir with two queue files containing different video IDs. Build the pending vec (simulate what download_all does: scan_queue_files + filter by artifact status). Apply the video_id filter to one ID. Assert filtered vec has exactly 1 entry with the correct video_id.

- `test_download_all_video_id_not_found`: Same setup. Apply video_id filter with an ID that doesn't exist. Assert the filtered vec is empty (this simulates what causes the not-found error path to trigger).

- `test_transcribe_all_video_id_filter`: Create two artifact dirs with status.json (downloaded=true, transcribed=false). Build the pending vec (simulate scan_artifact_statuses + filter_map). Apply video_id filter to one ID. Assert filtered vec has exactly 1 entry.

- `test_transcribe_all_video_id_not_found`: Same setup. Apply video_id filter with a non-existent ID. Assert filtered vec is empty.

Note: The unit tests test the filter *logic* (building the vec and filtering it), not the full async handler. This is the same pattern used in S03 tests — test the data transformation, not the I/O dispatch. The not-found check is `filtered.is_empty()` — tests that return empty vectors prove the not-found path would trigger.

8. Run `cargo test` and confirm 28/28 pass.
  - Estimate: 35m
  - Files: src/main.rs, src/artifact.rs
  - Verify: cargo test 2>&1 | grep -E 'test result|passed' | tail -3
