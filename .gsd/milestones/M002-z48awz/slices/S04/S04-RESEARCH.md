# S04: Selective Processing — Research

**Date:** 2026-04-06
**Scope:** Add `--video-id <id>` filtering to `download-all` and `transcribe-all`

## Summary

This is a light, pattern-extension slice. The `--video-id` flag already exists on `cleanup` and the exact clap registration pattern (`.long("video-id").value_name("VIDEO_ID")` with `.get_one::<String>("video-id").cloned()`) can be copied verbatim. Both `download_all` and `transcribe_all` are already internally structured as filter-then-iterate loops, so adding a video_id filter is a two-line addition per function. The "not found" exit path follows the same `return Err("...".into())` pattern used throughout the codebase. No new libraries or external APIs are involved.

The only non-trivial design decision is **where to apply the filter** in `download_all`, which has two branches (single-channel and no-channel). The `--video-id` filter should apply to both branches: it is independent of whether a channel is specified.

## Recommendation

Implement in two tasks: (1) extend the CLI structs and arg registration, (2) extend the two handler functions. Both are in separate files with clear interfaces, but they're small enough that an executor could do both in one pass. Splitting by file (cli.rs vs main.rs) is the natural seam.

## Implementation Landscape

### Key Files

- `src/cli.rs` — `CliCommand::DownloadAll` and `CliCommand::TranscribeAll` struct variants need `video_id: Option<String>` field added. The `download-all` and `transcribe-all` subcommand `.subcommand(Command::new(...))` blocks each need a `--video-id` arg added (copy from `cleanup`'s registration at line ~281). The `Some(("download-all", ...))` and `Some(("transcribe-all", ...))` match arms in `parse_args()` need to read and populate the new field.

- `src/main.rs` — `download_all` signature extends to `Option<&str>` video_id parameter; both `Some(ch)` and `None` branches get an early filter applied to `pending` before the iteration loop. `transcribe_all` signature extends similarly; the `filter_map` that builds `pending` gets an additional `video_id` guard. Both functions need a not-found check: after filtering, if `video_id.is_some()` and `pending.is_empty()`, return `Err("video ID {id} not found in any queue/artifact")`. The dispatch block in `main()` for both commands needs to destructure `video_id` and pass `.as_deref()` to the handler.

### Exact Clap Pattern to Copy (from cleanup, cli.rs ~line 281)

```rust
Arg::new("video-id")
    .long("video-id")
    .help("Process only the artifact with this video ID")
    .value_name("VIDEO_ID"),
```

Parse side (also from cleanup, cli.rs ~line 414):
```rust
let video_id = matches.get_one::<String>("video-id").cloned();
```

### Filter Logic in `download_all` (both branches)

After building `pending: Vec<VodEntry>`, add:
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

This pattern works identically for both the single-channel and no-channel branches since it operates on the already-filtered `pending` vec, after the downloaded-ID deduplication.

### Filter Logic in `transcribe_all`

The existing `filter_map` produces `pending: Vec<(String, ProcessStatus)>`. After building it, apply the same pattern:
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

### Build Order

1. **cli.rs first** — extend the two `CliCommand` variants and register the args. This is purely additive and can be verified with `cargo build` before touching main.rs.
2. **main.rs second** — extend handler signatures, update dispatch, add filter + not-found logic. `cargo test` confirms no regressions.

Both steps are small. The entire slice is ~40 lines of real change across two files.

### Error Message Convention

Error messages should name the specific resource being searched:
- `download-all`: `"video ID {id} not found in any queue"`
- `transcribe-all`: `"video ID {id} not found in any artifact"`

These are returned as `Err(...)` which bubble to the top-level `eprintln!("Download-all failed: {error}")` / `eprintln!("Transcribe-all failed: {error}")` handler in `main()` with `exit(1)`. This is the established pattern — no special exit-code handling needed.

### Test Coverage

Add unit tests in `src/artifact.rs` (per the established pattern from S03):
- `test_download_all_video_id_filter` — queue two VODs, call download_all with one's ID, verify only that ID is in pending
- `test_download_all_video_id_not_found` — provide a video_id not in any queue, verify error is returned
- `test_transcribe_all_video_id_filter` — set up two downloaded artifacts, call transcribe_all with one's ID, verify only that one is processed
- `test_transcribe_all_video_id_not_found` — provide a video_id not in any artifact, verify error is returned

These follow the exact same fixture-and-assert pattern as S03's `test_download_all_no_channel_filter` and `test_download_all_channel_regression`.

### Current Test Count

`cargo test` passes 24/24 at S03 completion. S04 should bring this to 28/28.
