---
id: S03
milestone: M001
status: ready
---

# S03: Trusted hear transcript contract and failure surfacing — Context

## Goal

Replace the current `mlx-whisper` transcription path with an explicit `hear -d -i <audio-file> -S` invocation, capture stdout as `transcript.srt`, convert to `transcript.vtt`, detect bad output (too short, high repetition), and surface three terminal transcription outcomes — `completed`, `suspect`, and `failed` — with structured reasons visible in artifact state.

## Why this Slice

S02 establishes staged processing and hands off to whatever transcription backend is wired. S03 replaces that backend with `hear` and defines the canonical transcript artifact contract that S04 (`ready-for-notes`) and downstream notes work depend on. This is also where the milestone's highest risk is retired: transcription reliability. Must follow S01 and S02 because it builds on the artifact state model and the `transcribe-all` command dispatch established there.

## Scope

### In Scope

- Replace `mlx-whisper` invocation in `src/transcribe.rs` with `hear -d -i <audio-file> -S`
- Capture `hear` stdout and write `transcript.srt` to the artifact directory
- Convert `transcript.srt` → `transcript.vtt` and persist both as canonical artifact outputs
- Three terminal transcription outcomes written to artifact state:
  - `completed` — hear exited 0, output passes quality checks, both `.srt` and `.vtt` present
  - `suspect` — hear exited 0 but output fails quality heuristics (see below); pipeline does NOT block, item is labeled and visible in status
  - `failed` — hear exited non-zero or threw an error; item stays blocked and will be retried by `transcribe-all`
- Bad output detection heuristics (applied before marking `completed`):
  - Word count threshold: output below a minimum word count relative to audio duration is flagged suspect (exact threshold is agent's discretion at planning time, e.g. <50 words per hour of audio)
  - Repetition detection: high density of repeated phrases in a short span is flagged suspect
- Structured failure reason captured in `status.json` for both `failed` and `suspect` outcomes: reason string, word count, and relevant heuristic that triggered
- On retry (re-run of `transcribe-all`): delete partial/suspect transcript files before restarting — same artifact integrity rule as partial download cleanup
- `suspect` items do NOT block pipeline advancement — `transcribe-all` skips them on subsequent runs unless explicitly forced; they remain visible in status output

### Out of Scope

- AI-assisted error correction pass over transcripts — deferred to M002 (noted as a planned next step for improving technical term accuracy)
- Chunked audio splitting for long VODs — if `hear` handles long files acceptably, this is not needed in S03; if it proves necessary, it becomes a blocker to surface
- Locale / language selection flags — `hear` default locale is sufficient for M001
- Punctuation flag (`-p`) — not part of the canonical invocation for M001; can be revisited if output quality warrants it
- Transcript format beyond `.srt` and `.vtt` — no plain `.txt` derivative required in this slice

## Constraints

- The canonical `hear` invocation is exactly: `hear -d -i <audio-file> -S` — on-device only (`-d`), subtitle mode (`-S`), file input (`-i`). Do not add flags without a specific reason.
- Both `transcript.srt` and `transcript.vtt` must be present for an item to be marked `completed` — partial outputs (e.g. `.srt` written but conversion failed) leave the item in `failed` state
- `suspect` items must remain visible in status output with their reason — they must not silently blend in with `completed` items
- `transcribe-all` must skip `completed` and `suspect` items on re-run unless a force/retry flag is added (that flag is out of scope for S03 — simple re-run semantics only)
- Do not introduce a database or external service — all state remains in `status.json` on the filesystem
- Build on the stage state model from S01 and the `transcribe-all` dispatch from S02

## Integration Points

### Consumes

- `src/transcribe.rs` — existing transcription module; `hear` invocation replaces the current `mlx-whisper` call here
- `artifacts/<video_id>/audio.m4a` — source audio file passed to `hear -i`
- `artifacts/<video_id>/status.json` — artifact state from S01; transcription stage fields updated with `completed`, `suspect`, or `failed` outcome and structured reason

### Produces

- `artifacts/<video_id>/transcript.srt` — raw subtitle output from `hear -S`, written from stdout capture
- `artifacts/<video_id>/transcript.vtt` — WebVTT conversion of `transcript.srt`; canonical downstream format
- Updated `src/transcribe.rs` — `hear` invocation with stdout capture, quality heuristics, and three-outcome result type
- Transcription stage state in `status.json`: outcome (`completed` / `suspect` / `failed`), reason string, word count, heuristic triggered (if any)
- Transcript completion signal for S04: `completed` state + both artifact files present = safe to advance to `ready-for-notes`; `suspect` state = advanceable but flagged

## Open Questions

- What are the right word-count and repetition thresholds for the suspect heuristics? — current thinking is to set conservative initial values (e.g. <50 words/hour flags suspect, >20% repeated trigrams in any 200-word window flags suspect) and tune after seeing real `hear` output on long VODs; these are agent's discretion at planning time
- Does `hear` handle multi-hour `.m4a` files reliably in a single invocation, or does it stall or OOM on large inputs? — unknown until tested; if it proves unreliable for long files, audio chunking becomes a required blocker task within S03
- Should `suspect` items be retried by a future `--retry-suspect` flag on `transcribe-all`? — current thinking is yes, but that flag is out of scope for S03; for now, re-running only retries `failed` items
- AI correction pass for technical terms and proper nouns — explicitly deferred to M002; the expectation is that `hear` gets close enough for a notes-quality pass, and a subsequent LLM correction step handles the rest
