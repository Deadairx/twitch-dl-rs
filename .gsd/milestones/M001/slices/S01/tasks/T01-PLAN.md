---
estimated_steps: 5
estimated_files: 3
---

# T01: Define the durable artifact and queue contract with regression tests

**Slice:** S01 — Durable artifact and queue state
**Milestone:** M001

## Description

Create the load-bearing durable state contract in `src/artifact.rs` so S01 stops relying on directory existence and boolean drift. This task should add the richer queue/job and per-stage lifecycle schema, keep it additive and JSON-on-disk, and lock the behavior down with Rust tests that classify mixed fixture states correctly. Load the installed `test` skill before refining or expanding the test coverage.

## Steps

1. Review the existing artifact data structures in `src/artifact.rs` and define an additive durable schema for queue items, overall job state, per-stage state, timestamps, and failure metadata that S02/S03 can extend without a breaking rewrite.
2. Implement read/write/list helpers in `src/artifact.rs` for the richer queue and status files, including artifact classification helpers that inspect durable files rather than assuming any numeric directory means completed work.
3. Add focused Rust unit tests in `src/artifact.rs` or the project’s preferred Rust test location that cover serialization/deserialization and mixed fixture directories such as directory-only, metadata-only, media-only, failed status, and complete status.
4. Keep `src/twitch.rs` coupling minimal: if queue persistence currently embeds raw `VodEntry`, wrap or adapt it so the durable queue record can survive future source-model changes without redefining the whole file format.
5. Run the targeted artifact tests and tighten any naming, field semantics, or helper behavior until the contract is explicit enough for later CLI wiring to use directly.

## Must-Haves

- [ ] Durable queue and status structures express explicit lifecycle state, timestamps, and failure details in additive JSON-friendly form.
- [ ] Tests prove mixed artifact fixtures classify correctly and guard against regressing back to directory-exists heuristics.

## Verification

- `cargo test artifact::tests -- --nocapture`
- Test output shows the new schema serializes cleanly and mixed fixture state cases pass.

## Observability Impact

- Signals added/changed: `queue.json` and `status.json` gain explicit job/stage state, timestamps, and failure metadata.
- How a future agent inspects this: read durable files through `src/artifact.rs` helpers or via later CLI status commands.
- Failure state exposed: partial and failed artifacts remain visible in persisted state instead of being hidden by coarse directory checks.

## Inputs

- `src/artifact.rs` — existing filesystem-backed metadata, queue, and status helpers that need schema expansion.
- `src/twitch.rs` — current `VodEntry` shape used by queue persistence today.
- `.gsd/milestones/M001/slices/S01/S01-RESEARCH.md` — recommends additive schema evolution, fixture-based testing, and keeping `src/artifact.rs` as the authority seam.

## Expected Output

- `src/artifact.rs` — richer durable queue/status contract plus helpers for reading, writing, and classifying artifact state.
- `src/twitch.rs` — any minimal adapter or wrapper changes required to persist queue items durably without over-coupling to raw source structs.
- Rust test coverage — regression tests that lock mixed fixture classification and JSON contract behavior in place.
