# Knowledge Base

Reusable insights, patterns, and gotchas discovered during milestone execution.
Append-only. Never edit or remove existing entries. Add a new entry to supersede.

---

## M001 — Reliable media-to-transcript pipeline

### Task summary inflation is a systemic risk
**Context:** S01's task summaries claimed complex lifecycle types (JobLifecycleState, StageLifecycleState), regression tests, and a status CLI command. None of these were in the actual code.
**Lesson:** Treat task summary claims as aspirational until verified against actual code. Slice summaries should be written by an agent that has *read the code*, not by one that wrote the plan. A concrete check: run `cargo test` and `git diff --stat` before writing any summary.
**Mitigation used:** Honest gap documentation in slice summary; downstream agents worked from the documented reality, not the claimed promise.

### Simple schemas beat premature complexity
**Context:** S01 used `downloaded: bool` and `transcribed: bool` instead of the complex multi-variant lifecycle types planned. These simple flags remained sufficient through S04 with no refactoring needed.
**Lesson:** Start with the minimum schema that satisfies current slice needs. Add expressiveness when a concrete requirement appears, not when planning.

### Backward-compatible schema evolution is cheap and essential
**Context:** S03 and S04 added 4 new fields to `ProcessStatus`. Every field used `#[serde(default)]`. Old status.json files from S01 deserialized cleanly without migration.
**Lesson:** In any durable JSON schema, default all optional fields from day one. The cost is a one-line attribute. The benefit is zero migration work across schema versions. Add a backward-compat unit test whenever a new field lands.

### Binary-only Rust crates cannot be unit-tested without `lib.rs`
**Context:** S03 discovered that `cargo test` on a binary-only crate only runs integration tests — modules in `main.rs` are not reachable by unit tests.
**Lesson:** For any Rust CLI project, add `src/lib.rs` as a crate root at the start and re-export modules there. Do not treat this as optional or deferrable — adding it mid-project requires touching every module's visibility.

### Composable helpers before batch commands
**Context:** S02 extracted `download_vod_to_artifact` and `transcribe_artifact` as standalone async helpers before writing `download-all` and `transcribe-all`. S03 and S04 extended these helpers cleanly without rewriting dispatch logic.
**Pattern:** Extract the atomic unit of work as a composable helper first. Then compose it into batch commands. This separates "what to do for one item" from "how to iterate over many items" — making extensions and overrides clean.

### Proof logs as durable milestone evidence
**Context:** S05 wrote a `proofs/proof.log` capturing three-phase CLI output as a durable file. This became the primary evidence source for milestone validation.
**Lesson:** On any milestone with complex multi-command integration, plan a dedicated proof slice that writes a structured, timestamped log of CLI outputs. The proof log is cheaper to write than it is to reconstruct from scattered summaries, and it makes milestone validation unambiguous.

### Slice frontmatter must be populated at slice completion
**Context:** S04's summary frontmatter (provides, key_files, key_decisions, patterns) was left empty despite full functional delivery. The implementation was verified via tests and proof log, making this a documentation gap rather than a delivery gap — but the gap required extra validation work.
**Lesson:** Slice completion tooling should validate that frontmatter is populated before marking a slice complete. The frontmatter is the machine-readable contract used by downstream agents; empty sections break the provides/requires chain.

### `--continue-on-error` is essential for any batch processing command
**Context:** S02 added `--continue-on-error` to both `download-all` and `transcribe-all`. Without it, a single failed item would halt batch processing and leave the rest of the queue unprocessed.
**Lesson:** Any CLI command that processes multiple items should default to stopping on first error (safe default) but expose `--continue-on-error` for operator-controlled partial recovery. This flag should be designed into the command at inception, not added as a patch.

---
