# S01: Durable artifact and queue state

**Goal:** Establish a durable filesystem-backed job contract so Twitch media is queued into stable per-item artifact state, partial or failed items remain visible, and operators can inspect queue and artifact lifecycle from the CLI instead of inferring from raw files.
**Demo:** After this: You can queue Twitch media into durable per-item artifact folders with explicit status, and inspect what exists without guessing from raw files.

## Tasks
- [x] **T01: Define the durable artifact and queue contract with regression tests** — 
  - Files: src/artifact.rs, src/twitch.rs, Cargo.toml
  - Verify: `cargo test artifact::tests -- --nocapture`
- [x] **T02: Rewire queue creation and artifact writers to persist state-aware job status** — 
  - Files: src/main.rs, src/artifact.rs, src/twitch.rs
  - Verify: `cargo test artifact::tests -- --nocapture && cargo test main -- --nocapture`
- [x] **T03: Expose queue and artifact lifecycle through the CLI and document operator usage** — 
  - Files: src/cli.rs, src/main.rs, README.md
  - Verify: `cargo test cli -- --nocapture` and `cargo run -- status --output-root <tmp-dir>`
