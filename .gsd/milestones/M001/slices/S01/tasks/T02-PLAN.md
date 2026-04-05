---
estimated_steps: 5
estimated_files: 3
---

# T02: Rewire queue creation and artifact writers to persist state-aware job status

**Slice:** S01 — Durable artifact and queue state
**Milestone:** M001

## Description

Wire the new durable contract into the real execution path in `src/main.rs`. Queue creation must stop hiding partial or failed items behind directory existence checks, and download/process flows must always leave a coherent durable status file after success, reuse, or failure. This task closes the gap between the schema from T01 and the actual command behavior the operator will rely on.

## Steps

1. Replace any existing artifact-directory heuristics in queue building with calls into the new artifact classification helpers, so known items are classified from durable metadata/status rather than folder presence alone.
2. Update queue creation to write a durable queue record that distinguishes newly queued work from already-known artifacts, preserving enough metadata for later inspection without depending on a fresh Twitch fetch.
3. Refactor download/process writer paths to initialize and update `status.json` coherently across success, reused-media, partial, and error flows, keeping semantics limited to S01 persistence and not adding S02 orchestration.
4. Add or expand Rust tests around the affected orchestration helpers so queue generation and status persistence are exercised through the real command-layer logic where practical.
5. Run the targeted tests and verify the resulting durable files line up with the schema and slice demo expectations.

## Must-Haves

- [ ] Queue generation no longer treats any artifact directory as equivalent to a completed job.
- [ ] Real command flows persist coherent durable status for queued, partial, failed, reused, and complete work.

## Verification

- `cargo test artifact::tests -- --nocapture && cargo test main -- --nocapture`
- A temp-root run leaves `queue.json` and `artifacts/<id>/status.json` reflecting explicit lifecycle state instead of raw directory presence.

## Observability Impact

- Signals added/changed: queue records and per-item status files are now written from the real command path, not just helper code.
- How a future agent inspects this: inspect temp-root artifacts or run the later CLI status command against the same root.
- Failure state exposed: success, reuse, and failure paths each leave durable state that can be inspected after interruption.

## Inputs

- `src/main.rs` — current queue/process orchestration still uses Twitch refetch plus directory-exists filtering.
- `src/artifact.rs` — richer durable contract and helpers from T01.
- T01 output — classification helpers and tests defining the authoritative queue/status semantics.

## Expected Output

- `src/main.rs` — queue and processing paths rewritten around the durable state contract.
- `src/artifact.rs` — any supporting helper refinements needed by the main command wiring.
- Rust tests — command-layer coverage proving state-aware queueing and status persistence behave correctly.
