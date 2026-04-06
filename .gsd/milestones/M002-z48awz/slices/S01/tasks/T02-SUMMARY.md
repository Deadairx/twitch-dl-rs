---
id: T02
parent: S01
milestone: M002-z48awz
key_files:
  - src/cli.rs
  - src/main.rs
key_decisions:
  - GQL fetch occurs after stream resolution but before prepare_artifact_dir — ensures no artifact dir is created if metadata fetch fails
  - vod_context ownership: fetch returns owned Strings, then borrow as &str for from_download — avoids lifetime issues cleanly
duration: 
verification_result: passed
completed_at: 2026-04-06T19:17:20.887Z
blocker_discovered: false
---

# T02: Wired GQL metadata fetch into download_vod with pre-dir-creation ordering and added --skip-metadata CLI flag; bare download now writes status.json

**Wired GQL metadata fetch into download_vod with pre-dir-creation ordering and added --skip-metadata CLI flag; bare download now writes status.json**

## What Happened

T01 delivered the schema and fetch_vod_metadata_by_id. This task connected those pieces to the two actual download call paths.

cli.rs: Added skip_metadata: bool to CliCommand::Download and the --skip-metadata flag to the download subcommand. Threaded get_flag("skip-metadata") into the struct in parse_args.

main.rs download_vod: Extended signature with vod_context and skip_metadata. Inserted context resolution block between stream selection and prepare_artifact_dir — GQL failure returns before dir creation. Added status.json write after write_metadata in the bare download path.

download_vod_to_artifact: Updated to pass Some((vod.title, vod.channel, vod.uploaded_at)) as vod_context and false for skip_metadata — no extra GQL call made.

## Verification

cargo build: clean (1 dead_code warning on read_metadata, pre-existing). cargo test: 16/16 pass. ./target/debug/vod-pipeline download --help | grep skip-metadata: flag present with correct description.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo build 2>&1 | grep '^error'` | 1 | ✅ pass (no errors) | 3100ms |
| 2 | `cargo test 2>&1 | grep -E 'test result|FAILED'` | 0 | ✅ pass (16 ok, 0 failed) | 2000ms |
| 3 | `./target/debug/vod-pipeline download --help 2>&1 | grep skip-metadata` | 0 | ✅ pass | 50ms |

## Deviations

None.

## Known Issues

read_metadata in artifact.rs generates a dead_code warning; it will be used by future tasks. Not a bug.

## Files Created/Modified

- `src/cli.rs`
- `src/main.rs`
