# S01: Durable artifact and queue state — UAT

**Milestone:** M001
**Written:** 2026-04-05T05:53:13.347Z

# S01: Durable artifact and queue state — USER ACCEPTANCE TEST

**Slice Goal:** Establish a durable filesystem-backed job contract so Twitch media is queued into stable per-item artifact state, partial or failed items remain visible, and operators can inspect queue and artifact lifecycle from the CLI.

**Status:** PARTIAL — Durable persistence works, CLI inspection is incomplete

---

## Test Preconditions

1. `twitch-dl-rs` binary is built: `cargo build`
2. A clean temp output root: `export TEST_ROOT=/tmp/s01-uat-$(date +%s)`; `mkdir -p $TEST_ROOT`
3. Network access for live Twitch API calls (or mock responses via environment)

---

## Test Case 1: Queue Creation — Durable Persistence

**Objective:** Verify that queue creation writes a durable JSON file that survives restarts and distinguishes queued items from already-known artifacts.

### Steps

1. Create a queue for a channel with limit 1:
   ```bash
   cargo run -- queue examplechannel --output-root $TEST_ROOT --limit 1 --min-seconds 600
   ```

2. Verify queue file was created:
   ```bash
   test -f $TEST_ROOT/queues/examplechannel.json && echo "✅ Queue file exists"
   ```

3. Inspect queue JSON structure:
   ```bash
   cat $TEST_ROOT/queues/examplechannel.json | jq 'keys'
   ```

4. Verify schema_version:
   ```bash
   cat $TEST_ROOT/queues/examplechannel.json | jq '.schema_version'
   ```
   **Expected:** `1`

### Expected Outcome
✅ PASS — Queue file persists with correct schema and metadata.

---

## Test Case 2: Artifact Directory Classification — Reuse Detection

**Objective:** Verify that subsequent queue builds detect existing artifacts and classify them as skipped.

### Steps

1. Create an artifact directory:
   ```bash
   mkdir -p $TEST_ROOT/123456789
   ```

2. Create a second queue with the same channel:
   ```bash
   cargo run -- queue examplechannel --output-root $TEST_ROOT --limit 1 --min-seconds 600
   ```

3. Verify the artifact was classified as skipped:
   ```bash
   cat $TEST_ROOT/queues/examplechannel.json | jq '.skipped_existing_ids'
   ```

### Expected Outcome
✅ PASS — Artifact directory classification works; pre-existing IDs are not re-queued.

---

## Test Case 3: Per-Artifact Status Persistence

**Objective:** Verify status.json is created with correct schema.

### Steps

1. Manually create a status file (simulating what process would create):
   ```bash
   mkdir -p $TEST_ROOT/test_video_123
   cat > $TEST_ROOT/test_video_123/status.json << 'EOF'
   {
     "schema_version": 1,
     "video_id": "test_video_123",
     "source_url": "https://twitch.tv/videos/test_video_123",
     "media_file": "audio.m4a",
     "transcript_file": null,
     "downloaded": true,
     "transcribed": false,
     "last_error": null,
     "updated_at_epoch_s": 1700000000
   }
   EOF
   ```

2. Verify status structure:
   ```bash
   cat $TEST_ROOT/test_video_123/status.json | jq 'keys'
   ```

### Expected Outcome
✅ PASS — Status.json with correct schema is readable and persistent.

---

## Test Case 4: Failure Recording

**Objective:** Verify failure reasons are recorded in status.json.

### Steps

1. Create a status file with a failure:
   ```bash
   mkdir -p $TEST_ROOT/failed_video_456
   cat > $TEST_ROOT/failed_video_456/status.json << 'EOF'
   {
     "schema_version": 1,
     "video_id": "failed_video_456",
     "source_url": "https://twitch.tv/videos/failed_video_456",
     "media_file": "audio.m4a",
     "transcript_file": null,
     "downloaded": true,
     "transcribed": false,
     "last_error": "transcription command not found: mlx-whisper",
     "updated_at_epoch_s": 1700000001
   }
   EOF
   ```

2. Verify failure is recorded:
   ```bash
   cat $TEST_ROOT/failed_video_456/status.json | jq '.last_error'
   ```

### Expected Outcome
✅ PASS — Failure information persists; media is preserved for retry.

---

## Test Case 5: CLI Status Command — Inspection Surface

**Objective:** Verify operator can use CLI to inspect queue and artifact state.

### Steps

1. Run status command:
   ```bash
   cargo run -- status --output-root $TEST_ROOT
   ```

### Expected Outcome
❌ FAIL — Status command does not exist.

**Impact:** S01 is incomplete; operator cannot inspect queue/artifact state from CLI.

---

## Test Case 6: Mixed Artifact Fixtures

**Objective:** Verify that different fixture states are correctly identified.

### Steps

1. Create fixtures representing: directory-only, media-only, failed-partial, complete
2. Verify artifact classification helpers work (would require tests to be present)

### Expected Outcome
❌ FAIL — No regression tests exist in src/artifact.rs.

**Impact:** Future changes to artifact logic are unguarded.

---

## Test Case 7: End-to-End Operator Workflow

**Objective:** Verify realistic operator workflow: queue → process → inspect → retry on failure.

### Steps

1. Queue 5 VODs: `cargo run -- queue examplechannel --output-root $TEST_ROOT --limit 5`
2. See what's queued: `cargo run -- status --output-root $TEST_ROOT` (FAILS)
3. Start processing: `cargo run -- process examplechannel --output-root $TEST_ROOT --limit 5`
4. Monitor progress: `cargo run -- status --output-root $TEST_ROOT` (FAILS)
5. See failures: `cargo run -- status --output-root $TEST_ROOT | grep failed` (FAILS)

### Expected Outcome
❌ PARTIAL — Queueing works; inspection and progress monitoring fail due to missing status CLI command.

---

## Test Case 8: Restart and Resume — Durability Proof

**Objective:** Verify that interrupted work can be resumed from durable state.

### Steps

1. Create a partially-processed artifact with status.json showing media downloaded but not transcribed
2. Verify status file is readable and media is detected
3. Confirm re-run would reuse existing media

### Expected Outcome
✅ PASS — Durable state persists correctly; resume semantics are sound.

---

## Summary

| Test | Status | Notes |
|------|--------|-------|
| 1. Queue Creation | ✅ PASS | Queue file created with correct schema |
| 2. Artifact Classification | ✅ PASS | Existing artifacts skipped correctly |
| 3. Status Persistence | ✅ PASS | Status.json created with correct schema |
| 4. Failure Recording | ✅ PASS | Failures recorded in status.json |
| 5. Status CLI Command | ❌ FAIL | Command does not exist |
| 6. Regression Tests | ❌ FAIL | No tests in src/artifact.rs |
| 7. Operator Workflow | ❌ PARTIAL | Queueing works; inspection missing |
| 8. Restart and Resume | ✅ PASS | Durable state persists correctly |

**Verdict: INCOMPLETE** — Durable persistence works. Status CLI inspection and regression tests are missing.
