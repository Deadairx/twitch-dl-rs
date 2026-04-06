# S02: Decoupled staged processing — UAT

**Milestone:** M001
**Written:** 2026-04-06T02:51:28.586Z

# S02: Decoupled staged processing — UAT

**Milestone:** M001
**Written:** 2026-04-05T06:15:00Z

## UAT Type

- UAT mode: artifact-driven + live-runtime
- Why this mode is sufficient: The slice adds no external dependencies beyond the existing download/transcribe infrastructure. All core logic is deterministic and testable through the CLI. We verify by checking artifact structure, status.json format, and command output rather than live integration with Twitch APIs.

## Preconditions

1. `twitch-dl-rs` binary is built and available at `./target/debug/twitch-dl-rs`
2. Test artifacts directory is clean: `rm -rf /tmp/test_s02_artifacts`
3. Queue file with test VOD entries has been created at `test_s02_artifacts/queues/testchan.json` (populated by S01 queue command)
4. All three artifact tests pass: `cargo test artifact::tests`

## Smoke Test

```bash
cd /Users/codyarnold/repos/Projects/twitch-dl-rs
./target/debug/twitch-dl-rs status --output-root /tmp/test_s02_artifacts
```

**Expected outcome:** Command exits with code 0. Output shows either "No artifacts found" (if empty) or a table with VIDEO_ID, DOWNLOADED, TRANSCRIBED, LAST_ERROR columns.

---

## Test Cases

### 1. Status command on empty artifact directory

1. Create empty test directory: `mkdir -p /tmp/test_s02_empty`
2. Run: `./target/debug/twitch-dl-rs status --output-root /tmp/test_s02_empty`
3. **Expected:**
   - Exit code 0
   - Output contains: "No artifacts found in /tmp/test_s02_empty"

### 2. Status command lists all artifacts with correct columns

1. Create artifact structure with three artifacts
2. Run: `./target/debug/twitch-dl-rs status --output-root /tmp/test_s02_artifacts`
3. **Expected:**
   - Exit code 0
   - Output shows table header and artifact rows with proper columns
   - Footer: "3 artifact(s) total"

### 3. Status command truncates long error messages

1. Create artifact with long error message in status.json
2. Run: `./target/debug/twitch-dl-rs status --output-root /tmp/test_s02_error`
3. **Expected:**
   - Exit code 0
   - Error column shows truncated message (first 40 chars)
   - Full error remains in status.json file

### 4. Queue file deserialization round-trip

1. Use S01 queue command to create a valid queue file
2. Run internal test: `cargo test artifact::tests::test_read_queue_file_roundtrip`
3. **Expected:**
   - Test passes
   - Confirms QueueFile deserializes correctly from disk
   - Confirms VodEntry deserializes from queued array

### 5. Download-all command parsing and help

1. Run: `./target/debug/twitch-dl-rs download-all --help`
2. **Expected:**
   - Exit code 0
   - Help text contains: Arguments: &lt;channel&gt;, Options: --output-root, --quality, --continue-on-error

### 6. Transcribe-all command parsing and help

1. Run: `./target/debug/twitch-dl-rs transcribe-all --help`
2. **Expected:**
   - Exit code 0
   - Help text contains: Options: --output-root, --continue-on-error
   - No channel argument required

### 7. Scan artifact statuses with missing status.json files

1. Create artifacts with and without status.json
2. Run: `./target/debug/twitch-dl-rs status --output-root /tmp/test_s02_scan`
3. **Expected:**
   - Exit code 0
   - Both artifacts appear in output
   - Missing status shows "(no status)" in columns
   - Confirms scan gracefully handles missing status files

### 8. Backward compatibility: process command still works

1. Run: `./target/debug/twitch-dl-rs process --help`
2. **Expected:**
   - Exit code 0
   - Help text unchanged from previous behavior
   - Confirms process command still delegates to extracted helpers correctly

### 9. Status output formatting consistency

1. Create multiple artifacts with varying ID lengths and error message lengths
2. Run: `./target/debug/twitch-dl-rs status --output-root /tmp/test_s02_format`
3. **Expected:**
   - All rows align to same column widths
   - No word wrapping within columns
   - TABLE format is consistent

---

## Edge Cases

### Missing queue file for download-all

1. Run: `./target/debug/twitch-dl-rs download-all nonexistent_channel --output-root /tmp/test_s02_artifacts`
2. **Expected:**
   - Exit code 1 (failure)
   - Error message: "No queue file found for channel 'nonexistent_channel'"
   - No partial state left behind

### Empty queue file for download-all

1. Create queue file with queued array = []
2. Run: `./target/debug/twitch-dl-rs download-all mychannel --output-root /tmp/test_s02_artifacts`
3. **Expected:**
   - Exit code 0 (success, nothing to do)
   - Output: message about all VODs already downloaded

### Transcribe-all with no artifacts

1. Create empty output directory
2. Run: `./target/debug/twitch-dl-rs transcribe-all --output-root /tmp/test_s02_empty`
3. **Expected:**
   - Exit code 0
   - Output: message about no artifacts pending transcription
   - No changes to filesystem

### Status command with non-existent output directory

1. Run: `./target/debug/twitch-dl-rs status --output-root /tmp/nonexistent_path_xyz`
2. **Expected:**
   - Exit code 0
   - Output: "No artifacts found"
   - Directory is NOT created (read-only behavior)

### Invalid status.json JSON

1. Create artifact with malformed status.json
2. Run: `./target/debug/twitch-dl-rs status --output-root /tmp/test_s02_bad_json`
3. **Expected:**
   - Exit code 0 (scan is best-effort)
   - Artifact still appears in output
   - Status column shows "(no status)" since JSON parsing failed
   - No panic or crash

---

## Failure Signals

- **Build failure**: `cargo build` returns non-zero — indicates code structure issue
- **Test failure**: `cargo test artifact::tests` shows failed test — indicates deserialization or roundtrip issue
- **Help text missing**: `--help` output doesn't show expected options — indicates clap parsing not wired
- **Exit code non-zero**: Any status/download-all/transcribe-all command returns non-zero on valid input — indicates logic error
- **Output misalignment**: Table columns don't align or have wrong widths — indicates formatting bug
- **Panic on missing file**: Any command panics instead of graceful error — indicates missing error handling
- **Process command broken**: Original `process` command shows different behavior than before — indicates regression

---

## Not Proven By This UAT

- **Actual download functionality**: This UAT does not attempt to download real VODs from Twitch (no network call). S05 will prove this end-to-end.
- **Actual transcription**: This UAT does not verify transcription works (no mlx-whisper call). S03 and S05 will prove this.
- **Concurrency safety**: This UAT tests single-threaded command invocation. Concurrent access to status.json is not tested.
- **Large-scale performance**: This UAT uses small test cases. Behavior with thousands of artifacts is untested.
- **Error recovery correctness**: This UAT verifies commands complete and return proper exit codes. Whether the recovered state is actually usable is verified in S05.

---

## Notes for Tester

- All test commands assume the binary is at `./target/debug/twitch-dl-rs`. If you build in release mode, use `./target/release/twitch-dl-rs` instead.
- The test artifacts directories can be cleaned up afterward: `rm -rf /tmp/test_s02_*`
- Status.json files are human-readable JSON. For debugging, you can `cat` them directly to see the full artifact state.
- The unit tests (`cargo test artifact::tests`) are the definitive proof that deserialization works. Manual testing above adds confidence for CLI integration.
- If any test fails, check the exit code first—it tells you whether the command succeeded (0) or failed (non-zero) before looking at output format.
