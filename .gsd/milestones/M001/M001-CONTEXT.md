# M001: Reliable media-to-transcript pipeline

**Gathered:** 2026-03-17
**Status:** Ready for planning

## Project Description

Build the first reliable version of a queue-first, artifact-first media processing CLI for DeadAir's real workflow. The system should ingest Twitch media into durable local artifacts, process media through explicit stages instead of a monolithic one-shot flow, produce usable transcripts without babysitting the process, and surface a clear `ready for notes` state for later downstream work. The current project started as a simple Twitch downloader, but this milestone reframes it as the trustworthy intake and transcription backbone of a broader media-to-notes-to-memory pipeline.

## Why This Milestone

This milestone solves the biggest trust problem first: the existing combined download/transcribe flow is too fragile for unattended use, especially because transcription reliability issues and stage coupling make it hard to know what happened or what can safely resume. M001 is about making the core intake-to-transcript path dependable before layering on notes, Ember persistence, or additional sources.

## User-Visible Outcome

### When this milestone is complete, the user can:

- Queue Twitch media intake work and return later to clear per-item state showing what downloaded, what transcribed, what failed, and why.
- See completed transcripts enter a real `ready for notes` state and run an explicit cleanup-candidate workflow without risking automatic deletion of original audio.

### Entry point / environment

- Entry point: Rust CLI commands for queueing, processing stages, inspecting status, and reviewing cleanup candidates
- Environment: local dev / local operator workflow
- Live dependencies involved: Twitch, ffmpeg, local transcription backend, filesystem-backed artifact store

## Completion Class

- Contract complete means: artifact directories, queue state, status files, and transcript outputs prove the staged pipeline and state model exist with substantive implementation.
- Integration complete means: a real local CLI run exercises queueing, staged download/transcription progress, failure visibility, ready-for-notes state, and cleanup candidate discovery.
- Operational complete means: interrupted or partial work can be resumed later without redoing completed stages or losing stage visibility.

## Final Integrated Acceptance

To call this milestone complete, we must prove:

- A real Twitch queue can be built and processed through separate download and transcription stages without transcription blocking additional downloads.
- A failed or partial item remains visible with a clear reason and can be resumed without corrupting completed artifact state.
- Completed transcript artifacts are surfaced as `ready for notes`, and cleanup only appears as an explicit candidate review workflow rather than automatic deletion.

## Risks and Unknowns

- Transcription reliability may still drop or mangle content on the fast path — this matters because downstream notes and trust depend on transcript quality.
- Staged orchestration may become muddled if artifact state is not explicit enough — this matters because the product promise is "without babysitting the process."
- Cleanup candidate logic needs strong safety boundaries — this matters because deleting originals is irreversible even when deletion stays manual.

## Existing Codebase / Prior Art

- `src/main.rs` — current top-level flow with `download`, `queue`, and combined `process` behavior
- `src/cli.rs` — existing clap command parsing and current command surface
- `src/artifact.rs` — current artifact directory, queue file, and status file helpers
- `src/transcribe.rs` — current `mlx-whisper` invocation path and the present reliability boundary
- `README.md` — documents current Twitch-first pipeline shape and existing artifact layout

> See `.gsd/DECISIONS.md` for all architectural and pattern decisions — it is an append-only register; read it during planning, append to it during execution.

## Relevant Requirements

- R001 — establish queue-first, artifact-first job handling as the core product experience
- R002 — preserve Twitch as the first supported source and prove the pipeline on it
- R003 — separate download progress from transcription progress
- R004 — improve transcript trustworthiness enough to support downstream use
- R005 — make failure state and per-item lifecycle legible without digging
- R006 — surface a real `ready for notes` state after transcript completion
- R009 — provide explicit manual cleanup candidate review rather than automatic deletion
- R012 — make long-running work resumable without babysitting

## Scope

### In Scope

- Twitch-first queue building and artifact tracking
- explicit stage state for download and transcription
- resume behavior across interrupted or partial work
- transcript reliability improvements and clearer failure surfacing
- `ready for notes` state in artifact lifecycle
- explicit cleanup candidate discovery with safety checks

### Out of Scope / Non-Goals

- LLM notes generation itself
- Ember memory persistence
- support/contradict analysis against current view or memory system
- YouTube or other new media sources
- automatic destructive cleanup

## Technical Constraints

- Preserve the filesystem-backed artifact model instead of introducing a heavier service or database for M001.
- Build on the existing Rust CLI and module layout rather than replacing the whole tool.
- Keep the workflow queue-first and artifact-first: commands should operate on durable state, not hidden transient assumptions.
- Do not make cleanup automatic; manual operator action is a product constraint, not a temporary implementation shortcut.

## Integration Points

- Twitch — source discovery and media intake for the first milestone
- ffmpeg — media extraction/download into local artifact files
- local transcription backend — transcript generation from acquired media
- local filesystem — durable artifact store, queue state, transcript outputs, and cleanup candidate discovery

## Open Questions

- What transcript backend strategy best balances trustworthiness with acceptable unattended throughput? — current thinking is to bias toward trustworthy output over raw speed.
- Should a safe recap/summary eventually auto-run after transcription, or remain manual-first? — current thinking is that summary may become auto-friendly later, but memory-shaping work should stay explicit.
