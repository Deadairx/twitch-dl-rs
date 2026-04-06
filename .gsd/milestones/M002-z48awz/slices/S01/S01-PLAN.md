# S01: Metadata Durability

**Goal:** Extend ArtifactMetadata with title, channel, and uploaded_at fields and thread them through every code path that writes metadata.json, so every new artifact carries human-readable context and old artifacts load cleanly.
**Demo:** After this: After this: run download on a Twitch URL and inspect metadata.json to see title, uploaded_at, channel alongside existing fields. Old artifact directories still load cleanly.

## Tasks
- [x] **T01: Extend ArtifactMetadata schema and add GQL metadata fetch** — Add three new display fields to ArtifactMetadata and a new GQL function in twitch.rs. Purely library code — no call-site changes in main.rs or cli.rs. T02 wires the results into the runtime paths.
  - Estimate: 45m
  - Files: src/artifact.rs, src/twitch.rs
  - Verify: cargo test 2>&1 | grep -E 'test result|FAILED|^error'
- [x] **T02: Wired GQL metadata fetch into download_vod with pre-dir-creation ordering and added --skip-metadata CLI flag; bare download now writes status.json** — T01 produced the schema changes and fetch_vod_metadata_by_id. This task wires them into the two download call paths and adds the --skip-metadata CLI flag.
  - Estimate: 45m
  - Files: src/main.rs, src/cli.rs
  - Verify: cargo build 2>&1 | grep '^error'; cargo test 2>&1 | grep -E 'test result|FAILED'; ./target/debug/vod-pipeline download --help 2>&1 | grep skip-metadata
