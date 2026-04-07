# S03: Intake Flexibility

**Goal:** Add `queue-video <url>` for single-VOD intake and make `download-all` channel-argument optional so it drains all queues without requiring a channel name.
**Demo:** After this: After this: run queue-video on a Twitch URL, then run download-all with no arguments and watch the queued item download.

## Tasks
- [x] **T01: Add `queue-video` command** — Add `queue-video <url>` as a new CLI subcommand. It resolves the video ID from the URL, fetches channel and display metadata via `fetch_vod_metadata_by_id`, reads the existing queue file for that channel (if any), deduplicates by video_id, appends the new entry, and writes back using `write_queue_file`. Idempotent: if the video_id is already present, print "Already queued: <id>" and exit 0.

Key implementation notes:
- `VodEntry.duration` is not returned by the metadata GQL call — use "PT0S" as a placeholder. Duration is only used by the `min_seconds` filter in `build_queue`, which `queue-video` bypasses entirely.
- `QueueFile.past_broadcasts_only`, `min_seconds`, and `skipped_existing_ids` are private fields. Accept their defaults (false, 0, vec![]) when calling `write_queue_file` — these are channel-queue-generation filters irrelevant to single-video ad-hoc intake.
- GQL failure must abort with a clear `eprintln!` message. Never silently write a malformed queue entry.
  - Estimate: 45m
  - Files: src/cli.rs, src/main.rs, src/artifact.rs
  - Verify: cargo build && cargo test && ./target/debug/vod-pipeline queue-video --help
- [x] **T02: Make `download-all` channel optional and add no-channel walk path** — Change the `channel` argument on `download-all` from required to optional. When omitted, walk all `queues/*.json` files via `artifact::scan_queue_files`, deduplicate against artifact statuses (skip any video_id where `status.downloaded == true`), and process the remaining pending entries with the existing `download_vod_to_artifact` helper. When `channel` is provided, keep the existing single-queue path unchanged.

Prior task T01 must be complete before end-to-end testing (so there is a queue file from `queue-video` to drain). The code changes themselves are independent.

Key implementation notes:
- `artifact::scan_queue_files(output_root)` returns Vec<VodEntry> across all queues/*.json — already public and tested.
- `artifact::scan_artifact_statuses(output_root)` returns Vec<(String, Option<ProcessStatus>)> — already public. Collect video_ids where status.downloaded == true into a HashSet<String> for O(1) lookup.
- The existing single-channel path (Some(channel)) must be byte-for-byte identical to current behavior — no regressions.
  - Estimate: 45m
  - Files: src/cli.rs, src/main.rs, src/artifact.rs
  - Verify: cargo build && cargo test && ./target/debug/vod-pipeline download-all --help
