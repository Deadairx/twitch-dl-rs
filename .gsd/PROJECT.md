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

M002-z48awz (workflow polish) is active. S01 (Metadata Durability) complete.

**S01 delivered:**
- `ArtifactMetadata` extended with `title`, `channel`, `uploaded_at` (all `Option<String>`, backward-compatible)
- `fetch_vod_metadata_by_id` GQL call in `twitch.rs` for bare download paths
- `--skip-metadata` flag on the `download` command (escape hatch for offline/API-unavailable scenarios)
- `status.json` written for bare download artifacts (normalizes artifact structure across intake paths)
- `read_metadata(artifact_dir)` helper for S02 status display

Next slice: S02 (Status Legibility) — reads `metadata.json` per artifact to populate title, date, and channel columns in the status table.

## Architecture / Key Patterns

- **Artifact model:** `<output-root>/<video_id>/` containing `metadata.json`, `source_url.txt`, `audio.m4a`/`video.mp4`, `transcript.srt`, `transcript.vtt`, `status.json`
- **Queue model:** `<output-root>/queues/<channel>.json` — one file per channel, holds `VodEntry` array
- **Stage state:** `ProcessStatus` in `status.json` — the durable per-item job record; display metadata (title, date, channel) lives in `metadata.json` only (D013)
- **Composable helpers:** `download_vod_to_artifact` and `transcribe_artifact` are atomic helpers composed into batch commands
- **Source isolation:** Twitch-specific intake lives in `twitch.rs`; downstream stages are source-agnostic
- **Metadata threading:** vod_context passed as `Option<(&str, &str, &str)>` through download call stacks; GQL fetch occurs pre-directory-creation to prevent orphan artifacts

## Capability Contract

See `.gsd/REQUIREMENTS.md` for the explicit capability contract, requirement status, and coverage mapping.

## Milestone Sequence

- [x] M001: Reliable Media-to-Transcript Pipeline — durable queue/artifact state, staged processing, trustworthy transcription, proof of operator flow
- [ ] M002-z48awz: Workflow Polish — status legibility, metadata durability, intake flexibility, selective processing, queue-aware views, retry hardening, additional source support
  - [x] S01: Metadata Durability — ArtifactMetadata schema extended, GQL fetch wired, --skip-metadata flag, status.json normalized for bare downloads
  - [ ] S02: Status Legibility — reads metadata.json for human-readable status table
  - [ ] S03: Intake Flexibility — queue-video command + download-all without channel arg
  - [ ] S04: Selective Processing — download-all --video-id filter
  - [ ] S05: Queue-Aware Filtering — status --filter flag
  - [ ] S06: Retry And Operational Hardening — transcribe-all --force-suspect
  - [ ] S07: Additional Source Support — YouTube source through artifact model
- [ ] M003 (future): Notes And Ember Memory Workflow — manual-first note generation, Ember memory persistence, support/contradict analysis
