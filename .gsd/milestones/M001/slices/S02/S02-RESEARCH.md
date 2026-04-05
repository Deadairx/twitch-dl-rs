# S02: Decoupled Staged Processing — Research

**Date:** 2026-04-05
**Status:** Ready for planning

## Summary

S02 exists to decouple download progress from transcription progress (R003) and to make interrupted work resumable without redoing completed stages (R012). The current `process` command is monolithic: it downloads and transcribes each VOD serially in a single loop, with no way to run all downloads first and transcriptions later, or to resume a partial run and only transcribe items that are downloaded-but-not-yet-transcribed.

The fix is straightforward given the existing `ProcessStatus` schema — we need two new CLI subcommands, `download-all` and `transcribe-all`, that each operate over the artifact store by reading `status.json` to decide what work remains. The `status` command (identified as missing in the S01 follow-up) must also land in this slice since operators need CLI-level visibility before they can meaningfully drive staged runs.

The work is entirely within three files (`src/cli.rs`, `src/main.rs`, `src/artifact.rs`) using patterns already established in S01. No new dependencies, no new backend technology, no architectural uncertainty. This is targeted work.

## Recommendation

Implement three additions:

1. **`download-all` command** — reads the queue for a channel, finds VODs not yet downloaded (`downloaded: false` in status.json or no status.json), downloads each one, writes status.json after each. Skips already-downloaded items. Does not transcribe.

2. **`transcribe-all` command** — scans all artifact dirs under `output_root`, finds VODs where `downloaded: true` but `transcribed: false`, transcribes each, writes status.json after each. Optionally scoped to a channel. Runs independently of download.

3. **`status` command** — scans artifact dirs, reads status.json for each, prints a human-readable summary table of video IDs with their download/transcribe state and last error. Missing from S01 and called out in S01 follow-ups as a prerequisite for S02 operation.

The existing `process` command should remain unchanged — it provides the combined one-shot path. The two new staged commands give operators the choice to decouple stages when needed.

**Why this shape:** The current `ProcessStatus` struct already carries `downloaded`, `transcribed`, `last_error`, and `media_file`. No schema change is required. The artifact classification logic in `process_channel` and `process_vod` can be refactored into reusable helpers that both `process` and the new staged commands share. Resume behavior already works via the file-presence checks — the new commands just need to scope themselves to the right subset of work.

## Implementation Landscape

### Key Files

- `src/artifact.rs` — Contains `ProcessStatus`, `read_status()`, `write_status()`, `find_media_file()`, `existing_artifact_ids()`. No schema changes needed for S02. May need a new helper: `read_queue_file()` to load the persisted queue JSON from disk (currently the queue is only written, never read back). Also needs `artifact_ids_for_channel()` or equivalent to find artifact dirs that belong to a specific channel (via `source_url` or a metadata field).
- `src/main.rs` — Contains `process_vod()` (download + transcribe in one function) and `process_channel()`. The download logic inside `process_vod()` needs to be extractable as a standalone `download_vod_to_artifact()` async fn. The transcription logic needs to be extractable as `transcribe_artifact()`. The new `download_all()` and `transcribe_all()` top-level functions call these helpers.
- `src/cli.rs` — Needs three new `CliCommand` variants: `DownloadAll`, `TranscribeAll`, `Status`. Each maps to a subcommand in the clap tree. `status` and `transcribe-all` take only `--output-root` (they operate on the artifact store, not a specific channel). `download-all` takes `--channel` + `--output-root` + `--quality` to read from the persisted queue.

### Build Order

1. **Add `read_queue_file()` to `artifact.rs`** — the `download-all` command needs to consume the persisted queue rather than re-fetching Twitch. The `QueueFile` struct already exists but is `Serialize`-only; add `Deserialize` and a `read_queue_file()` fn. This also adds `VodEntry` deserialization (it's `Serialize`-only in `twitch.rs` — needs `Deserialize` added too).

2. **Add `status` command** — scan artifact dirs, read status.json, print table. This is self-contained and proves the artifact scanning pattern works. Unblocks operator inspection.

3. **Extract download and transcribe helpers in `main.rs`** — refactor `process_vod()` into two composable fns: one for the download stage (returns early if already downloaded), one for the transcription stage (returns early if already transcribed). The existing `process_vod()` can delegate to them.

4. **Add `download-all` command** — reads `queues/<channel>.json`, iterates items not yet downloaded, calls the download helper per item.

5. **Add `transcribe-all` command** — scans `output_root` for artifact dirs with `downloaded: true, transcribed: false`, calls the transcription helper per item. `--continue-on-error` flag mirrors the existing pattern.

### Verification Approach

- `cargo build` — compilation is the first gate; verifies all CLI variants and handlers exist
- `cargo run -- status --output-root artifacts` — should print a table of existing artifact states without error
- `cargo run -- download-all <channel> --output-root artifacts` — reading from the persisted queue, should skip already-downloaded items and report
- `cargo run -- transcribe-all --output-root artifacts` — should find the one artifact with `downloaded: true, transcribed: false` (video 2693295712) and attempt transcription
- Manual check: after a simulated partial run (download some, abort before transcription), `transcribe-all` should only transcribe the incomplete items
- Resume test: kill `download-all` mid-run, re-run it — already-downloaded items should be skipped

## Common Pitfalls

- **`VodEntry` and `QueueFile` are Serialize-only** — adding `Deserialize` to both is required before `read_queue_file()` can work. Easy to miss; the code compiles fine without it until you try to deserialize.
- **`process_vod()` mixes download and transcribe in one fn** — refactoring it into two composable stages must not break the existing `process` command. The correct approach is to extract helpers and have `process_vod()` call them, not rewrite `process_vod()` entirely.
- **Channel-scoping for `transcribe-all`** — the current artifact dirs only have `source_url.txt` and `status.json`. To filter by channel, `transcribe-all` should read `status.json` and check `source_url` against the known channel URL pattern, or accept a `--channel` filter but default to processing all. Simpler default: operate on all artifact dirs under `output_root`, no channel filter unless explicitly requested.
- **Status command output format** — keep it simple: one line per artifact with video_id, downloaded, transcribed, and last_error truncated. Don't over-engineer; this is operator inspection, not a UI.
- **Queue file path convention** — `download-all` reads from `<output_root>/queues/<channel>.json`. If the file doesn't exist, error clearly rather than silently processing nothing.

## Open Risks

- The `status` command is a S01 follow-up that was scoped into S02 by necessity. If it proves more complex than expected (e.g., needs richer formatting), keep it minimal — a simple line-per-artifact print is sufficient for this slice.
- The `QueueFile.queued` list may be stale (written at queue time, items added since then won't appear). This is expected behavior for this milestone; `download-all` reads what was queued, not a live Twitch fetch.
