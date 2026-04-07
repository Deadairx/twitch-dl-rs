# vod-pipeline

## What This Is

A queue-first, artifact-first CLI for ingesting Twitch VODs (and eventually other sources) into durable local artifacts, transcribing them with `hear`, and preparing trustworthy transcript outputs for downstream notes and memory workflows.

## Core Value

Each media item is a durable tracked job with explicit stage state. Operators can queue work, walk away, come back, and immediately understand what succeeded, what failed, and what's next — without babysitting runs.

## Current State

M001 complete. The full intake-to-transcript pipeline works:
- `queue`, `download`, `process`, `status`, `download-all`, `transcribe-all`, `cleanup`
- Durable per-artifact state in `status.json`
- `hear`-backed transcription with `completed` / `suspect` / `failed` outcomes
- `ready_for_notes` as the handoff point to downstream notes work

M002-z48awz (workflow polish) is active. S01–S04 are complete.

**S01 delivered:**
- `ArtifactMetadata` extended with `title`, `channel`, `uploaded_at` (all `Option<String>`, backward-compatible)
- `fetch_vod_metadata_by_id` GQL call in `twitch.rs` for bare download paths
- `--skip-metadata` flag on the `download` command (escape hatch for offline/API-unavailable scenarios)
- `status.json` written for bare download artifacts (normalizes artifact structure across intake paths)
- `read_metadata(artifact_dir)` helper for S02 status display

**S02 delivered:**
- `scan_queue_files(output_root)` — channel-agnostic helper that walks all `queues/*.json` and aggregates queued VodEntries
- `show_status()` rewritten with 6-column human-readable table: STAGE | TITLE | CHANNEL | DATE | OUTCOME | REASON
- Queued-but-not-downloaded items appear in default status view (no flag required)
- Deduplication by video_id: artifact-dir row wins when item appears in both queue files and artifact dirs
- STAGE derivation: "queued", "downloaded", "ready", "failed", "suspect" from status.json + media file presence
- Graceful degradation: missing metadata.json fields render as em dash (—), no panics

**S03 delivered:**
- `queue-video <url>` command for single-VOD intake by Twitch URL
- URL validation + channel inference via `fetch_vod_metadata_by_id` GQL
- Idempotent deduplication: running twice with same URL prints "Already queued: {id}" and exits 0
- Optional `[channel]` argument on `download-all` — omitting it walks all `queues/*.json` and processes all pending
- No-channel path uses artifact-state-based filtering (HashSet of downloaded IDs) for O(1) dedup
- Single-channel path preserved unchanged (no regressions)

**S04 delivered:**
- `--video-id <id>` flag on `download-all` — filters pending queue items to exact match; exits non-zero with "not found in any queue" if ID absent
- `--video-id <id>` flag on `transcribe-all` — filters pending artifacts to exact match; exits non-zero with "not found in any artifact" if ID absent
- Filtering implemented at handler level (post pending-vec construction) to preserve file immutability and enable future filter stacking
- 4 new unit tests covering filter-with-match and filter-without-match for both commands
- 28/28 tests pass; build clean

Next slice: S05 (Queue-Aware Filtering) — `status --filter` flag.

## Architecture / Key Patterns

- **Artifact model:** `<output-root>/<video_id>/` containing `metadata.json`, `source_url.txt`, `audio.m4a`/`video.mp4`, `transcript.srt`, `transcript.vtt`, `status.json`
- **Queue model:** `<output-root>/queues/<channel>.json` — one file per channel, holds `VodEntry` array
- **Stage state:** `ProcessStatus` in `status.json` — the durable per-item job record; display metadata (title, date, channel) lives in `metadata.json` only
- **Status display:** STAGE column derives human-readable tokens from status.json + media presence; merged queue+artifact view via scan_queue_files + scan_artifact_statuses; deduplication by HashSet of artifact IDs
- **Composable helpers:** `download_vod_to_artifact` and `transcribe_artifact` are atomic helpers composed into batch commands
- **Handler-level filtering:** `--video-id` applied after pending vec is built; pattern extensible to S05's `--filter` flag without CLI parsing changes
- **Source isolation:** Twitch-specific intake lives in `twitch.rs`; downstream stages are source-agnostic
- **Metadata threading:** vod_context passed as `Option<(&str, &str, &str)>` through download call stacks; GQL fetch occurs pre-directory-creation to prevent orphan artifacts
- **Graceful degradation:** All optional display fields default to em dash (—) via `unwrap_or("—")` pattern; no panics on incomplete artifacts

## Capability Contract

See `.gsd/REQUIREMENTS.md` for the explicit capability contract, requirement status, and coverage mapping.

## Milestone Sequence

- [x] M001: Reliable Media-to-Transcript Pipeline — durable queue/artifact state, staged processing, trustworthy transcription, proof of operator flow
- [ ] M002-z48awz: Workflow Polish — status legibility, metadata durability, intake flexibility, selective processing, queue-aware views, retry hardening, additional source support
  - [x] S01: Metadata Durability — ArtifactMetadata schema extended, GQL fetch wired, --skip-metadata flag, status.json normalized for bare downloads
  - [x] S02: Status Legibility — 6-column human-readable status table, merged queue+artifact view, STAGE derivation, deduplication, graceful degradation
  - [x] S03: Intake Flexibility — queue-video command + download-all without channel arg (idempotent dedup, artifact-state filtering); 24/24 tests pass
  - [x] S04: Selective Processing — --video-id filter on download-all and transcribe-all; handler-level post-filter with not-found errors; 28/28 tests pass
  - [ ] S05: Queue-Aware Filtering — status --filter flag
  - [ ] S06: Retry And Operational Hardening — transcribe-all --force-suspect
  - [ ] S07: Additional Source Support — YouTube source through artifact model
- [ ] M003 (future): Notes And Ember Memory Workflow — manual-first note generation, Ember memory persistence, support/contradict analysis
