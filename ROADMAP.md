# VOD Pipeline Roadmap

## Goal

Turn `twitch-dl-rs` into a durable, operator-friendly media pipeline that ingests media into stable artifacts, transcribes it reliably, and supports downstream notes and memory workflows without losing operator clarity.

## Current State

M001 is complete.

Working today:

- `queue`, `download`, `process`, `status`, `download-all`, `transcribe-all`, `cleanup`
- durable per-artifact state in `status.json`
- `hear`-backed transcription with `completed` / `suspect` / `failed` outcomes
- `ready_for_notes` as the handoff point to downstream notes work

Future work clusters:

- M002: notes and Ember memory workflow
- M003: source expansion and workflow polish

## Milestones

### M001: Reliable Media-to-Transcript Pipeline

Status: complete

Delivered durable queue/artifact state, staged processing, trustworthy transcription, ready-for-notes state, cleanup workflow, and proof of the operator flow.

### M002: Notes And Ember Memory Workflow

Status: future

Focus:

- manual-first note generation with prompt/style choice
- Ember memory persistence for selected note outputs
- support/contradict analysis against existing memory context

### M003: Source Expansion And Workflow Polish

Status: future

Focus:

- additional media sources beyond Twitch
- status legibility and metadata durability
- intake flexibility and selective processing
- queue-aware views and filtering
- retry and operational hardening

## Immediate Next Planning Focus

Metadata + status foundations:

- preserve title, uploaded-at, and channel context durably
- add title/date/channel columns to `status`
- clarify ownership between `metadata.json` and `status.json`

## Planning Docs

- `docs/planning/PRINCIPLES.md`: durable product constraints and implementation guardrails
- `docs/planning/MILESTONES.md`: milestone and slice breakdown for future work
- `docs/planning/OPEN-QUESTIONS.md`: unresolved design questions and tradeoffs
- `docs/planning/NOTES/2026-04-06-operator-feedback.md`: captured operator feedback that informed current M003 planning

## Code Map

- `src/main.rs`: command dispatch and pipeline orchestration
- `src/cli.rs`: CLI surface and argument parsing
- `src/artifact.rs`: artifact metadata, status model, queue/state file helpers
- `src/transcribe.rs`: `hear` transcription path and quality heuristics
- `src/twitch.rs`, `src/downloader.rs`, `src/ffmpeg.rs`: Twitch intake and media assembly
