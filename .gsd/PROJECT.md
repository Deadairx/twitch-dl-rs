# Project

## What This Is

A queue-first CLI for ingesting media and turning it into usable transcript artifacts without babysitting the process. It started as a Twitch downloader and is evolving into a personal media processing pipeline: acquire media, preserve durable artifacts, transcribe reliably, surface clear stage state, and prepare finished transcripts for downstream notes and memory work.

## Core Value

The one thing that must work even if everything else gets cut is this: queue media intake work and come back later to trustworthy transcript artifacts with clear state, failures, and resume behavior.

## Current State

The project is a Rust CLI with Twitch VOD download, backlog queue generation, and a combined `process` flow that downloads and transcribes into artifact directories under `artifacts/`. It already writes files like `metadata.json`, `source_url.txt`, `status.json`, `audio.m4a`, and `transcript.txt`. Current transcription uses `mlx-whisper`, but reliability concerns and stage coupling make the pipeline feel too fragile for the intended unattended workflow.

## Architecture / Key Patterns

Rust CLI using `clap` for commands, `reqwest` for Twitch requests, `ffmpeg` for media capture, and filesystem-backed artifact directories as the working substrate. Current structure centers on modules such as `src/cli.rs`, `src/twitch.rs`, `src/downloader.rs`, `src/transcribe.rs`, and `src/artifact.rs`. The planned direction is artifact-first and queue-first: each media item becomes a durable job record with explicit stage state rather than a one-off command side effect.

## Capability Contract

See `.gsd/REQUIREMENTS.md` for the explicit capability contract, requirement status, and coverage mapping.

## Milestone Sequence

- [ ] M001: Reliable media-to-transcript pipeline — Make Twitch-first media intake, staged processing, trustworthy transcripts, and safe operator workflow reliable enough for unattended use.
- [ ] M002: Notes and Ember memory workflow — Add manual-first notes generation, promptable downstream analysis, and Ember persistence for selected outputs.
- [ ] M003: Source expansion and workflow polish — Add YouTube and other sources, plus later operational refinements around the broader workflow.
