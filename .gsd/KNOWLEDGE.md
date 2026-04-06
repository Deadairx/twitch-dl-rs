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

## M002 — Workflow Polish

### Metadata fetch ordering prevents orphan directories
**Context:** S01's bare download path needed to validate VOD metadata before creating any filesystem artifacts. The choice was whether to fetch metadata first (pre-directory) or inline (post-directory).
**Lesson:** In critical paths, always fetch and validate external data *before* creating local filesystem artifacts. This prevents orphaned partial directories if validation fails. The error message is cleaner and recovery is simpler (no cleanup needed). Cost is slightly longer failure latency, which is acceptable for user-facing downloads.

### Optional schema fields need backward-compat tests at addition time
**Context:** S01 added three new Option<String> fields to ArtifactMetadata. Without explicit backward-compat tests, future schema evolutions risk silent deserialization errors on old artifacts.
**Lesson:** Whenever adding optional fields to a durable JSON schema, write a backward-compat test that deserializes a file *without* those fields and confirms the new fields default to None. Make this test pass as part of the initial addition, not as a follow-up. The test is cheap and prevents rework.

### Context threading with borrowed references scales better than owned types
**Context:** S01 threaded vod_context (title, channel, uploaded_at) through download_vod -> from_download. Using Option<(&str, &str, &str)> instead of owned Strings or context structs kept the call signatures readable and avoided lifetime cascades.
**Lesson:** When passing small immutable data through a short call stack (2-3 levels), prefer borrowed references over owned types. The owned types are created at the boundary (GQL fetch), converted to borrows locally, and never propagated further. This pattern is simpler than lifetime parameters and avoids allocation overhead.

### Escape hatches (--skip-metadata) are essential for external API dependencies
**Context:** S01's GQL metadata fetch is critical path, but GQL API availability is not under operator control. Without --skip-metadata, a Twitch API issue blocks all downloads.
**Lesson:** For any CLI command with external API dependencies in the critical path, provide a non-required escape hatch (flag or env var) that disables the dependency and proceeds with reduced functionality. Document it as an exception path, not the happy path. This keeps the system operational during transient API issues.

### Deduplication by HashSet scales better than nested loops
**Context:** S02 merged queued items (from scan_queue_files) with artifact items (from scan_artifact_statuses). Deduplication needed to ensure a video_id appearing in both sources displayed only once.
**Pattern:** Collect one set's primary key into a HashSet, then filter the other set by membership: `let ids: HashSet<_> = artifacts.iter().map(|(id, _)| id.clone()).collect(); let deduped: Vec<_> = queued.into_iter().filter(|v| !ids.contains(&v.id)).collect();`
**Lesson:** O(n + m) time complexity with clear intent. Superior to nested loops (O(n*m)) for any non-trivial dataset sizes. The pattern is reusable wherever two sources need merging with single-instance constraint (e.g., S05's queue-aware filtering). Always collect the smaller set into HashSet to minimize memory overhead.

### Graceful degradation via unwrap_or reduces panic surface
**Context:** S02 display reads metadata.json, status.json, and queue files, any of which might be missing or incomplete. Rather than panicking on Option::None, every missing field defaulted to em dash (—).
**Pattern:** `metadata.as_ref().and_then(|m| m.title.as_deref()).unwrap_or("—").to_string()` — chains safely through nullable intermediate values, defaults to a meaningful fallback.
**Lesson:** For any display-layer code reading durable artifacts, defaulting missing fields to a visual placeholder (em dash, "N/A", "unknown") is always preferable to panicking. Operator can immediately see incomplete data and decide whether to investigate or proceed. Prevents a single missing field from blocking visibility of the entire artifact.

---
