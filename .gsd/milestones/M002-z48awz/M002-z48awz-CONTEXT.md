# M002-z48awz: Workflow Polish — Context

**Gathered:** 2026-04-06
**Status:** Ready for planning

## Project Description

vod-pipeline is a queue-first CLI for ingesting Twitch VODs into durable local artifacts and transcribing them with `hear`. M001 delivered the reliable intake-to-transcript backbone. M002 makes the operator workflow legible, flexible, and robust.

## Why This Milestone

M001's `status` command shows only video IDs with no human-readable context. The queue model forces channel-name arguments even when the operator just wants to drain everything. Single-video intake doesn't exist. Suspect transcriptions can't be retried. The architecture is correct but the operator experience is rough.

This milestone fixes the friction points that make daily use awkward without touching the core pipeline semantics.

## User-Visible Outcome

### When this milestone is complete, the user can:

- Run `vod-pipeline status` and see truncated title, human-readable date, and channel name alongside each item
- See queued-but-not-yet-downloaded items mixed into the status view by default
- Run `vod-pipeline queue-video <url>` to queue a single VOD by URL
- Run `vod-pipeline download-all` with no channel argument to drain all queues
- Filter `status` output by stage (queued-only, pending, failed, ready)
- Force-retry a suspect transcription without re-downloading
- Ingest media from at least one non-Twitch source through the same artifact model

### Entry point / environment

- Entry point: `vod-pipeline` CLI
- Environment: local dev / operator workstation
- Live dependencies: Twitch GQL API, `hear`, `ffmpeg`

## Completion Class

- Contract complete means: all commands compile, unit tests pass, status display verified against fixture artifacts
- Integration complete means: queue-video, download-all (no channel), status with filters, force-retry exercised against real VODs
- Operational complete means: none

## Final Integrated Acceptance

- `queue-video <url>` queues a real Twitch VOD and `download-all` (no channel) picks it up
- `status` shows title, date, channel for a mix of queued-only, downloaded, and transcribed artifacts
- A suspect artifact force-retries to completed or failed without re-downloading
- At least one non-Twitch URL ingests into a valid artifact directory with status.json

## Risks and Unknowns

- ArtifactMetadata currently lacks title/uploaded_at/channel; download path must carry VodEntry context through
- `download` command (direct URL) does not write status.json; normalization needed
- Non-Twitch source: yt-dlp subprocess vs Rust library unresolved
- Concurrent status.json writes currently unsafe; file locking needed

## Existing Codebase / Prior Art

- `src/artifact.rs` — ArtifactMetadata needs title/channel/uploaded_at fields; ProcessStatus unchanged
- `src/cli.rs` — needs queue-video command and optional channel on download-all
- `src/main.rs` — download_vod_to_artifact and transcribe_artifact helpers; show_status, download_all need updates
- `src/twitch.rs` — VodEntry already has title, uploaded_at, channel; these flow into metadata.json
- `src/transcribe.rs` — TranscriptionOutcome variants; force-retry reuses transcribe_artifact with force flag

## Relevant Requirements

- R005 — status legibility slices advance failure visibility
- R009 — retry hardening supports safe cleanup candidate detection
- R010 — S07 delivers first non-Twitch source
- R012 — selective processing and retry hardening reinforce resumability

## Scope

### In Scope

- status display with title, date, channel columns (reads metadata.json per artifact)
- Queued-but-not-downloaded items visible in status by default
- --filter flag on status (queued-only, pending, failed, ready)
- queue-video <url> command — infers channel, merges into queues/<channel>.json
- download-all channel arg optional — no arg walks all queue files
- Selective download and transcribe targeting by video ID
- Normalize download to always produce status.json
- Force-retry for suspect transcriptions (no re-download)
- Concurrent access safety for status.json
- At least one non-Twitch source (YouTube via yt-dlp subprocess)

### Out of Scope / Non-Goals

- Note generation or Ember memory (M003)
- Chat capture
- GUI or web interface
- Auto-cleanup

## Technical Constraints

- serde(default) on all new fields — old artifacts must deserialize cleanly
- hear invocation must not change
- Source-specific intake stays isolated so downstream stages are source-agnostic

## Integration Points

- Twitch GQL API — queue, queue-video, stream resolution
- hear CLI subprocess — transcription, unchanged
- ffmpeg subprocess — download assembly, unchanged
- yt-dlp — non-Twitch download subprocess (planned)

## Open Questions

- Force-retry UX: --force-suspect flag on transcribe-all, or separate retry command?
