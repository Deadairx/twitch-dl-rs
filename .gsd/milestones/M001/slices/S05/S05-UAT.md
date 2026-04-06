# S05: End-to-end operator flow proof — UAT

**Milestone:** M001
**Written:** 2026-04-06T03:43:16.045Z

# S05: End-to-end operator flow proof — UAT

**Milestone:** M001
**Written:** 2026-04-06

## UAT Type

- UAT mode: **artifact-driven** (CLI commands against durable local artifacts; no live Twitch API; no live transcription)
- Why this mode is sufficient: S01–S04 each proved their contracts in isolation. S05's goal is to prove they wire together correctly. Operator walkthrough against pre-existing artifacts + one manufactured failure scenario is sufficient to demonstrate integration.

## Preconditions

1. Binary built: `cargo build` succeeds.
2. Real artifacts present: 25+ items in `artifacts/` with status.json.
3. Proof fixtures created: `proofs/scratch-artifacts/9900000001/` (completed) and `proofs/scratch-artifacts/9900000002/` (failure).
4. Proof script ready: `proofs/run-proof.sh` executable with three phases.

## Smoke Test

```bash
bash proofs/run-proof.sh 2>&1
test -f proofs/proof.log && wc -l proofs/proof.log
```

**Expected:** Script completes without hanging, log has 50+ lines.

## Test Cases

### 1. Phase 1: Real Artifacts Indexed and Status Visible

1. Run: `./target/debug/twitch-dl-rs status --output-root artifacts`
2. **Expected:**
   - Table with `VIDEO_ID`, `DOWNLOADED`, `OUTCOME`, `READY` columns
   - At least 20 artifact IDs listed
   - At least one `OUTCOME=completed` or `OUTCOME=failed`
   - One artifact with a reason field
   - Footer shows "25 artifact(s) total"

### 2. Phase 2: Completed Item Listed as Cleanup Candidate

1. Run: `./target/debug/twitch-dl-rs cleanup --output-root proofs/scratch-artifacts`
2. **Expected:**
   - Artifact ID `9900000001` appears in list
   - Files listed: `audio.m4a` (5 B), `transcript.srt` (203 B)
   - Footer shows "Total space to be freed: 208 B"
   - Only 9900000001 listed (not 9900000002)

### 3. Phase 3: Failure Artifact Updated with Clear Error Reason

1. Run: `./target/debug/twitch-dl-rs transcribe-all --output-root proofs/scratch-artifacts --continue-on-error`
2. **Expected:**
   - Output shows "Transcribing 9900000002 with hear..."
   - Output shows "Failed 9900000002: hear exited with status exit status: 1: File format not supported."
   - Script exits with code 0
   - Subsequent `status --output-root proofs/scratch-artifacts` shows:
     - `9900000002`: `OUTCOME=failed`, `REASON=hear exited...`
     - `status.json` has `transcription_outcome="failed"` and `transcription_reason` field

### 4. All Tests Pass Without Mutation

1. Run: `cargo test`
2. **Expected:** All 14 tests pass, no failures.

## Edge Cases

### Real Artifact with Prior Failure
- Identify artifact 2693295712 in Phase 1 output
- Verify `status.json` has `transcription_outcome="failed"`
- **Expected:** Prior failure legible, system not corrupted

### Manufactured Failure Recoverable
- After Phase 3, inspect `proofs/scratch-artifacts/9900000002/status.json`
- **Expected:** `downloaded=true`, `transcribed=false`, `transcription_outcome="failed"`, `ready_for_notes=false`, directory structure intact

## Failure Signals
- Status command crashes or returns no artifacts
- Cleanup lists 0 candidates when 9900000001 complete
- Transcribe-all succeeds when should fail
- `cargo test` shows < 14 passing
- Any `updated_at_epoch_s` changed in real `artifacts/`

## Not Proven By This UAT
- Live Twitch API integration (proven in S01–S03)
- Performance at scale (only 27 artifacts tested)
- Network resilience
- Complex failure modes (only corrupt audio)
- Real transcription quality (proven in S03)

## Notes for Tester
- Hear failure in Phase 3 is expected and intentional
- Script uses `|| true` to allow failure; should exit 0
- All Phase 2–3 commands use `--output-root proofs/scratch-artifacts`
- Proof log is the evidence source
- Unit tests must still pass after walkthrough

## Result
✅ All tests pass. Full M001 pipeline integration confirmed."
