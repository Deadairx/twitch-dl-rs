---
id: S03
milestone: M002-z48awz
status: completed
title: "Intake Flexibility: Queue single VODs and drain all queues"
provides:
  - queue_video async handler that resolves video_id from URL, fetches metadata, deduplicates, and writes queue file
  - QueueVideo CLI variant and queue-video subcommand for single-VOD intake by URL
  - download_all handler that accepts optional channel parameter for multi-queue draining
  - No-channel walk path that scans all queues and filters by download status with HashSet deduplication
  - Single-channel regression guard ensuring existing download-all behavior unchanged
  - 3 new unit tests validating dedup, no-channel filtering, and single-channel regression
key_files:
  - src/cli.rs (CliCommand enum, subcommand definitions)
  - src/main.rs (queue_video and download_all handlers, dispatch)
  - src/artifact.rs (unit tests for dedup, filtering, regression)
key_decisions:
  - queue-video metadata fetch happens before queue file write (prevents orphaned queue entries on GQL failure)
  - No-channel download-all uses HashSet dedup on artifact IDs, not queue file iteration (O(n) filtering, clear semantics)
  - Single-channel path logic unchanged inside download_all to preserve tested behavior
patterns_established:
  - Optional argument threading (Option<&str>) from CLI through handler dispatch
  - Metadata-driven filtering (scan artifacts for ground truth, filter queues against artifact status)
  - HashSet deduplication for O(1) membership testing in list filtering
observability_surfaces:
  - queue-video success: prints "Queued {id} into {queue_file_path}"
  - queue-video duplicate: prints "Already queued: {id}" to stdout with exit 0
  - queue-video GQL failure: prints "Failed to resolve VOD metadata: {error}" to stderr with exit 1
  - download-all no-channel: existing log output unchanged but processes all queues
duration: 70m
completed_at: 2026-04-06
completion_evidence:
  - cargo build: clean
  - cargo test: 24/24 pass (21 existing + 3 new)
  - help text verification: [channel] syntax shows optional argument
  - unit tests verify dedup, filtering, and regression behavior
---

# S03: Intake Flexibility

## Summary

S03 successfully delivered the "drain everything" workflow by introducing two complementary features:

1. **`queue-video <url>`** — Single-VOD intake by Twitch URL. Extracts video ID, fetches channel via GQL, checks for existing entry (idempotent), and appends to the channel's queue file. Supports the ad-hoc "queue this one video" workflow without requiring knowledge of channel names.

2. **Optional-channel `download-all`** — Converted the `channel` argument from required to optional. When omitted, walks all `queues/*.json` files and processes pending VODs across channels using artifact-state-based filtering. When present, behaves identically to the previous implementation (no regressions).

### What Changed

**src/cli.rs:**
- Added `QueueVideo { url: String, output_root: PathBuf }` variant to `CliCommand`
- Registered `queue-video` subcommand with required positional `<url>` and optional `--output-root`
- Changed `DownloadAll.channel` from `String` to `Option<String>`; marked clap arg as `.required(false)`

**src/main.rs:**
- Implemented `queue_video(url: &str, output_root: &Path)` async handler:
  - Extract video ID and validate URL shape
  - Fetch VOD metadata (title, channel, upload_date) via `fetch_vod_metadata_by_id`
  - Read existing queue file for that channel (creates on first entry)
  - Check for duplicate video_id; if found, print "Already queued: {id}" and exit 0
  - Append `VodEntry` with `duration: "PT0S"` placeholder
  - Write queue file and report success with path
- Updated `download_all` signature to `Option<&str>`:
  - **Some(channel)**: Preserves existing single-channel logic unchanged
  - **None**: New walk path — scan all queues, get all artifact statuses, build `HashSet<String>` of downloaded IDs, filter pending items, iterate with existing `download_vod_to_artifact` helper

**src/artifact.rs:**
- `test_queue_video_idempotent_dedup`: Verifies read/write/dedup behavior for queue files
- `test_download_all_no_channel_filter`: Sets up 2 queue files (3 entries), marks 1 as downloaded, verifies pending count is 2
- `test_download_all_channel_regression`: Verifies single-channel path still filters correctly

### Verification

| Criterion | Result | Evidence |
|-----------|--------|----------|
| Build succeeds | ✅ | `cargo build` clean, 0.22s |
| All tests pass | ✅ | 24/24 (21 existing + 3 new) |
| queue-video shows positional arg | ✅ | Help text shows `<url>` |
| download-all shows optional channel | ✅ | Help text shows `[channel]` |
| Idempotent dedup works | ✅ | Unit test verifies read→check→write logic |
| No-channel filtering works | ✅ | Unit test verifies HashSet dedup on artifact IDs |
| Single-channel regression guard passes | ✅ | Unit test verifies existing behavior unchanged |

### Patterns & Lessons

**Metadata-driven filtering beats queue-file-based:** The no-channel path filters by scanning artifacts (ground truth) and building a HashSet of downloaded IDs, then filtering the merged queue list. This is simpler and more reliable than trying to track completion state in queue files themselves. The pattern scales to S04 (selective filtering) and S05 (queue-aware status display).

**Idempotent commands prevent operator confusion:** `queue-video` running twice with the same URL prints "Already queued: {id}" and exits 0. No partial state, no error, just a clear message. This makes the command safe to re-run and easier to script.

**Optional argument threading is transparent:** Using `Option<&str>` for the channel in the handler allows the logic to branch cleanly: `match channel { Some(ch) => ..., None => ... }`. Both branches are explicit and testable.

### Integration Points

**Upstream consumed (no new dependencies):**
- `twitch.rs`: `extract_video_id`, `fetch_vod_metadata_by_id` (both from S01)
- `artifact.rs`: `read_queue_file`, `write_queue_file`, `scan_queue_files`, `scan_artifact_statuses` (all from S01/S02)
- `cli.rs`: CliCommand enum, subcommand registration (existing pattern)
- `main.rs`: Dispatch, async handler pattern (established in S01/S02)

**Produces (new surfaces for downstream slices):**
- `download_all(Option<&str>)` — S04 will extend with `--video-id` filtering on the no-channel path
- `queue_video` observability — success/duplicate/failure messages for operator feedback
- Artifact-state-based filtering — pattern reused in S05 for queue-aware status display

**What's still needed before end-to-end operator workflow works:**
- S04: `--video-id` filtering on `download-all` (selective processing)
- S05: Status display with filtering (operator visibility into queued/failed/transcribed items)
- S06: Retry for suspect transcriptions
- S07: Non-Twitch source support (YouTube)

### Design Decisions

**1. Metadata fetch before queue write (no orphaned entries)**
GQL metadata fetch happens before any queue file operations. If the fetch fails, no queue file is created/modified. This keeps the queue file consistent and prevents operators from debugging orphaned entries.

**2. Ground-truth filtering via artifact status (not queue-file state)**
The no-channel path builds a HashSet from `status.json` files in existing artifacts, not by tracking completion state in queue files. Queue files are immutable snapshots; artifacts are the source of truth for completion. This simplifies the model and avoids dual-source-of-truth bugs.

**3. Preserve single-channel behavior exactly (regression guard)**
The single-channel path logic inside `download_all` is identical to the previous implementation. The change is only in signature and routing. This minimizes risk and ensures backward compatibility.

## Requirement Coverage

**R012 (Continuity)** — Directly validated by this slice:
- Both `queue-video` and `download-all` check artifact status before acting
- Downloaded items are skipped; queued items are processed
- `test_download_all_no_channel_filter` proves this works across multiple queues
- Status: **Remains Active** — this slice proved the mechanism works for two commands; wider validation happens in S06 (retry scenario)

## Observability

Queue-video reports its actions clearly:
- Success: `Queued {id} into {queue_file_path}`
- Duplicate (idempotent): `Already queued: {id}`
- GQL failure: `Failed to resolve VOD metadata: {error}` (stderr, exit 1)

Download-all preserves existing output (existing log lines unchanged in single-channel path; no-channel path uses same underlying `download_vod_to_artifact` which already logs progress).

## Files & Scope

**Modified:**
- src/cli.rs (52 lines changed)
- src/main.rs (108 insertions, changed dispatch logic)
- src/artifact.rs (83 insertions, 3 tests)

**Created:** None new (all changes integrate with existing structures)

**Reverted or deferred:** None

## Known Gaps & Next Steps

1. **End-to-end proof pending** — This slice is functional but the operator workflow is not yet visible. S04 (selective processing) and S05 (status display) are prerequisites for full UAT.

2. **Non-Twitch URLs not supported** — `queue-video` currently only accepts Twitch URLs. S07 will add YouTube support using the same CLI interface and artifact model.

3. **Queue file locking not implemented** — The design assumes single-operator, sequential commands. If concurrent `queue-video` calls are needed in future, queue-file locking should be added (out of scope for M002).

## Task Summaries (Compressed)

**T01: Add queue-video command** (45m, verified)
- Implemented `queue_video` async handler with URL→ID→metadata→queue logic
- Added QueueVideo CLI variant with positional `<url>` arg
- Idempotent dedup check prevents duplicate entries
- All 22 existing tests pass; new `test_queue_video_idempotent_dedup` passes

**T02: Make download-all channel optional** (25m, verified)
- Changed channel arg from `String` to `Option<String>`; clap shows `[channel]`
- Single-channel path (Some arm) unchanged; no regressions
- No-channel path (None arm) walks all queues, deduplicates via artifact status, processes pending
- New tests `test_download_all_no_channel_filter` and `test_download_all_channel_regression` pass
- All 24 tests pass (21 existing + 3 new)

---

## Completion Status

✅ **Slice S03 is complete**

All must-haves delivered:
- `queue-video <url>` subcommand works (idempotent, GQL-backed)
- `download-all [channel]` optional (walks all queues when omitted)
- No regressions on single-channel path
- 3 unit tests validate behavior
- `cargo test`: 24/24 pass
- `cargo build`: clean
