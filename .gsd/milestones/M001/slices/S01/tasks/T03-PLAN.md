---
estimated_steps: 5
estimated_files: 3
---

# T03: Expose queue and artifact lifecycle through the CLI and document operator usage

**Slice:** S01 — Durable artifact and queue state
**Milestone:** M001

## Description

Turn the durable state model into an operator-facing surface. Add a read-only CLI command that reports queue contents and artifact lifecycle clearly enough to distinguish queued, partial, failed, and complete items without opening JSON files manually, then update the README so later slices and human operators have one shared contract to follow.

## Steps

1. Extend `src/cli.rs` with a read-only inspection command such as `status` or `list`, including any filters or arguments needed to point at the output root or inspect a single item if that fits the existing CLI style.
2. Implement the command wiring in `src/main.rs` using the durable state helpers from T01/T02, and format output so it surfaces queue membership, overall lifecycle state, stage details, and failure reason in a compact operator-readable form.
3. Add tests for the new command path where practical, using fixture roots or golden-style assertions consistent with the project’s Rust conventions.
4. Update `README.md` to document the durable artifact layout, queue semantics, and new inspection workflow with concrete examples that match the implemented CLI behavior.
5. Run the CLI-focused tests and a manual temp-root command check to confirm the slice demo is reproducible from the docs.

## Must-Haves

- [ ] Operators can inspect queue contents and artifact lifecycle from the CLI without opening raw JSON manually.
- [ ] Documentation matches the implemented durable state contract and inspection command.

## Verification

- `cargo test cli -- --nocapture`
- `cargo run -- status --output-root <tmp-dir>` prints queued plus artifact lifecycle state that matches the temp-root durable files.

## Observability Impact

- Signals added/changed: CLI output now surfaces queue state, lifecycle stage, and failure details already persisted on disk.
- How a future agent inspects this: run the new CLI status/list command against any output root.
- Failure state exposed: failed and partial items become visible through a stable user-facing command instead of requiring raw file inspection.

## Inputs

- `src/cli.rs` — existing clap subcommands currently stop at download, queue, and process.
- `src/main.rs` — command dispatch and formatting entrypoints that need the new read-only inspection surface.
- `README.md` — current docs still describe `status.json` as a rerun helper rather than the main inspection surface.
- T01/T02 outputs — durable contract and real command wiring that the new command must reflect accurately.

## Expected Output

- `src/cli.rs` — new read-only status/list command definition.
- `src/main.rs` — command implementation and formatting for queue/artifact lifecycle inspection.
- `README.md` — updated operator docs for durable queue state, artifact status, and inspection workflow.
