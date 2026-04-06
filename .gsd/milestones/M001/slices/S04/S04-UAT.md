# S04: Ready-for-notes and manual cleanup workflow — UAT

**Milestone:** M001
**Written:** 2026-04-06T03:21:56.367Z

---
id: S04
parent: M001
milestone: M001
uat_type: artifact-driven
---

# S04: Ready-for-notes and manual cleanup workflow — UAT

**Milestone:** M001  
**Written:** 2026-04-06

## UAT Type

- **UAT mode:** artifact-driven
- **Why this mode is sufficient:** The slice produces artifacts (ready_for_notes field, cleanup command output) that are verifiable through filesystem inspection and command-line testing. No runtime services, background jobs, or user interaction needed — all logic is synchronous and deterministic. Artifact state inspection via `show_status`, cleanup listing, and file presence/absence checks are sufficient to prove the lifecycle works end-to-end.

## Preconditions

1. Project built successfully with `cargo build`
2. Test directory created with no pre-existing artifacts
3. ProcessStatus struct includes `ready_for_notes: bool` field with `#[serde(default)]`
4. `cleanup` subcommand implemented with `--delete`, `--all`, `--video-id` flags
5. `show_status` command updated to display READY column

## Smoke Test

1. Create temp dir with one artifact: numeric video ID directory (e.g., `123456789`)
2. Write `status.json` with `"ready_for_notes": true` and `"transcription_outcome": "completed"`
3. Create dummy `audio.m4a` and `transcript.srt` files in the artifact directory
4. Run `./target/debug/twitch-dl-rs cleanup --output-root <tmpdir>`
5. **Expected:** Command lists the artifact as a cleanup candidate with file sizes shown

## Test Cases

### 1. Ready-for-notes automatic state transition

1. Create an artifact directory with numeric ID (e.g., `999888777`)
2. Write `status.json` with `ready_for_notes: false` (initial state)
3. Simulate transcription completion by:
   - Setting `transcribed: true`
   - Setting `transcription_outcome: "completed"`
   - Setting `ready_for_notes: true`
   - Writing back to `status.json`
4. Run `./target/debug/twitch-dl-rs status --output-root <tmpdir>`
5. **Expected:** READY column shows "yes" for the artifact

### 2. Cleanup list mode shows only ready-for-notes candidates

1. Create three artifact directories: `111111111`, `222222222`, `333333333`
2. In `111111111`: write status with `ready_for_notes: true, transcription_outcome: "completed"`
3. In `222222222`: write status with `ready_for_notes: false, transcription_outcome: null`
4. In `333333333`: write status with `ready_for_notes: false, transcription_outcome: "failed"`
5. Create dummy `audio.m4a` and `transcript.srt` in all three
6. Run `./target/debug/twitch-dl-rs cleanup --output-root <tmpdir>`
7. **Expected:** Output shows only `111111111` as a cleanup candidate; others are not listed

### 3. Cleanup --delete --all removes audio.m4a and transcript.srt

1. Create artifact `444444444` with `ready_for_notes: true, transcription_outcome: "completed"`
2. Create dummy files: `audio.m4a` (5MB), `transcript.srt` (50B), `transcript.vtt`, `status.json`, `metadata.json`
3. Run `./target/debug/twitch-dl-rs cleanup --output-root <tmpdir> --delete --all`
4. Check artifact directory:
   - `audio.m4a` should NOT exist ✓
   - `transcript.srt` should NOT exist ✓
   - `transcript.vtt` should still exist ✓
   - `status.json` should still exist ✓
   - `metadata.json` should still exist ✓
5. **Expected:** Only audio.m4a and transcript.srt are deleted; protected files remain

### 4. Cleanup --delete --video-id removes files only for specified artifact

1. Create two artifacts: `555555555` and `666666666`, both with `ready_for_notes: true`
2. Create dummy files in both
3. Run `./target/debug/twitch-dl-rs cleanup --output-root <tmpdir> --delete --video-id 555555555`
4. Check:
   - `555555555/audio.m4a` should NOT exist ✓
   - `666666666/audio.m4a` should still exist ✓
5. **Expected:** Only the specified video_id is affected; other candidates untouched

### 5. Cleanup --delete without --all or --video-id returns error

1. Run `./target/debug/twitch-dl-rs cleanup --output-root <tmpdir> --delete`
2. **Expected:** Command fails with error message: "Error: --delete requires either --all or --video-id <video_id>"
3. **Expected:** Exit code is 1

### 6. Cleanup shows per-item file sizes correctly

1. Create artifact `777777777` with `ready_for_notes: true`
2. Create `audio.m4a` with known size (e.g., 5 MB)
3. Create `transcript.srt` with known size (e.g., 50 B)
4. Run `./target/debug/twitch-dl-rs cleanup --output-root <tmpdir>`
5. **Expected:** Output shows:
   ```
   Cleanup candidates (1 found):
   ------
   VIDEO_ID        audio.m4a            transcript.srt      
   ------
   777777777       5.0 MB               50 B                
   ------
   Total space to be freed: 5.0 MB
   ```

### 7. show_status displays READY column with correct values

1. Create two artifacts: one with `ready_for_notes: true`, one with `ready_for_notes: false`
2. Run `./target/debug/twitch-dl-rs status --output-root <tmpdir>`
3. **Expected:** Output includes READY column with "yes" for ready artifact and "-" for non-ready

### 8. Backward compatibility: old status.json without ready_for_notes field

1. Create artifact with `status.json` that has NO `ready_for_notes` field (simulating old format)
2. Run `./target/debug/twitch-dl-rs status --output-root <tmpdir>`
3. **Expected:** Command succeeds; artifact is readable and shows READY as "-" (defaults to false)
4. Run cleanup: `./target/debug/twitch-dl-rs cleanup --output-root <tmpdir>`
5. **Expected:** Old artifact is not listed as a cleanup candidate (because `ready_for_notes` defaults to false)

## Edge Cases

### Missing transcript.vtt but ready_for_notes is true

1. Create artifact with `ready_for_notes: true, transcription_outcome: "completed"`
2. Create `audio.m4a` and `transcript.srt`, but NOT `transcript.vtt`
3. Run cleanup list: `./target/debug/twitch-dl-rs cleanup --output-root <tmpdir>`
4. **Expected:** Artifact still appears as cleanup candidate (eligibility is based on status field, not file presence)

### Partially transcribed artifact (ready_for_notes true but outcome not "completed")

1. Create artifact with `ready_for_notes: false, transcription_outcome: "suspect"`
2. Run cleanup: `./target/debug/twitch-dl-rs cleanup --output-root <tmpdir>`
3. **Expected:** Artifact is NOT a cleanup candidate (filtering checks both ready_for_notes AND outcome == "completed")

### Non-numeric artifact directory names

1. Create directory with non-numeric name (e.g., `test_video_123`)
2. Write valid `status.json` with `ready_for_notes: true`
3. Run cleanup: `./target/debug/twitch-dl-rs cleanup --output-root <tmpdir>`
4. **Expected:** Directory is ignored (scanner only recognizes numeric video IDs)

### Cleanup on empty artifact root

1. Run `./target/debug/twitch-dl-rs cleanup --output-root <empty-tmpdir>`
2. **Expected:** Output shows "No cleanup candidates found."

### Double-deletion attempt

1. Create artifact with `ready_for_notes: true`
2. Run cleanup with `--delete --all`
3. Run cleanup again with `--delete --all`
4. **Expected:** Second run shows "No cleanup candidates found." (already deleted, so no candidates)

## Failure Signals

- **Cleanup command doesn't list expected artifacts** → Check if status.json exists and has correct schema (schema_version, ready_for_notes, transcription_outcome fields)
- **Protected files were deleted** → Bug in cleanup deletion logic (should never touch transcript.vtt, status.json, metadata.json)
- **READY column doesn't appear in status output** → ready_for_notes field not wired into show_status() function
- **Non-ready-for-notes artifacts appear as cleanup candidates** → Filtering logic broken; check if status.ready_for_notes is being read correctly
- **--delete without args succeeds** → Error validation missing; should require --all or --video-id

## Not Proven By This UAT

- **Integration with S03's transcription completion signal** — This UAT manually sets `ready_for_notes: true`; actual pipeline wiring (transcribe_artifact() setting the field) is tested via unit tests
- **Large-scale cleanup performance** — Tests use small artifacts; performance with thousands of items not measured
- **Concurrent cleanup operations** — No concurrency testing; assumes sequential single-operator use
- **Interaction with actual Twitch VOD downloads** — Uses synthetic status.json files; not connected to real download pipeline

## Notes for Tester

- All test artifacts must use numeric video IDs (all digits) to be discovered by the scanner
- `status.json` must match the ProcessStatus struct schema exactly — extra fields are ignored, missing fields default appropriately
- The cleanup command is read-only by default (list mode); no files are deleted without `--delete` flag
- Artifacts that reach `ready_for_notes` state are eligible for cleanup permanently until explicitly deleted — there is no time-based expiration or automatic aging-out
