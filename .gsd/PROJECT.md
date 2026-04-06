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

M002-z48awz (workflow polish) is active.

## Architecture / Key Patterns

- **Artifact model:** `<output-root>/<video_id>/` containing `metadata.json`, `source_url.txt`, `audio.m4a`/`video.mp4`, `transcript.srt`, `transcript.vtt`, `status.json`
- **Queue model:** `<output-root>/queues/<channel>.json` — one file per channel, holds `VodEntry` array
- **Stage state:** `ProcessStatus` in `status.json` — the durable per-item job record; display metadata (title, date, channel) lives in `metadata.json` only
- **Composable helpers:** `download_vod_to_artifact` and `transcribe_artifact` are atomic helpers composed into batch commands
- **Source isolation:** Twitch-specific intake lives in `twitch.rs`; downstream stages are source-agnostic

## Capability Contract

See `.gsd/REQUIREMENTS.md` for the explicit capability contract, requirement status, and coverage mapping.

## Milestone Sequence

- [x] M001: Reliable Media-to-Transcript Pipeline — durable queue/artifact state, staged processing, trustworthy transcription, proof of operator flow
- [ ] M002-z48awz: Workflow Polish — status legibility, metadata durability, intake flexibility, selective processing, queue-aware views, retry hardening, additional source support
- [ ] M003 (future): Notes And Ember Memory Workflow — manual-first note generation, Ember memory persistence, support/contradict analysis
