---
estimated_steps: 37
estimated_files: 2
skills_used: []
---

# T02: Extend download_all and transcribe_all handlers with video_id filter and not-found error; add 4 unit tests

Update `src/main.rs` to: (1) add `video_id: Option<&str>` parameter to `download_all` and `transcribe_all`, (2) apply a post-filter on `pending` in both functions, (3) return a clear error when the ID is not found, (4) update the dispatch block to pass `.as_deref()` for both commands. Add 4 unit tests in `src/artifact.rs` proving filter and not-found behavior.

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

## Inputs

- `src/cli.rs`
- `src/main.rs`
- `src/artifact.rs`

## Expected Output

- `src/main.rs`
- `src/artifact.rs`

## Verification

cargo test 2>&1 | grep -E 'test result|passed' | tail -3
