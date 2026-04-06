# S05: End-to-end operator flow proof

**Goal:** Prove the full M001 pipeline works as an integrated whole by running a real operator walkthrough against existing artifacts and a manufactured scenario, capturing all CLI output to a durable proof log.
**Demo:** After this: In one real CLI workflow, you can queue media, let staged processing run without babysitting, inspect failures, see ready-for-notes items, and review cleanup candidates.

## Tasks
- [x] **T01: Created proof script and manufactured completed/failure fixtures for end-to-end pipeline verification** — Create the `proofs/` directory, write `proofs/run-proof.sh` with all three walkthrough phases documented and executable, and manufacture the scratch artifact fixtures that Phase 2 (completed) and Phase 3 (failure) depend on.

This task produces everything the next task needs to execute — the script is the recipe, and the scratch fixtures simulate the S03/S04 new-schema state that real artifacts lack due to the state gap.

## Steps

1. Create `proofs/` and `proofs/scratch-artifacts/` directories.
2. Create scratch artifact `proofs/scratch-artifacts/9900000001/` — the manufactured COMPLETED item:
   - `status.json`: schema_version=1, video_id=9900000001, downloaded=true, transcribed=true, transcription_outcome="completed", ready_for_notes=true, transcript_word_count=1500, last_error=null, updated_at_epoch_s=(current epoch)
   - `transcript.srt`: stub SRT file with a few lines of valid SRT content
   - `transcript.vtt`: stub VTT file with WEBVTT header and a few cue lines
   - `audio.m4a`: small stub binary file (just `echo 'stub' > audio.m4a` — it won't be transcribed, just needs to exist for cleanup size display)
   - `metadata.json`: minimal JSON `{"id": "9900000001", "title": "Proof Stub: Completed Item"}`
3. Create scratch artifact `proofs/scratch-artifacts/9900000002/` — the manufactured FAILURE candidate:
   - `status.json`: schema_version=1, video_id=9900000002, downloaded=true, transcribed=false, transcription_outcome=null, last_error=null, updated_at_epoch_s=(current epoch)
   - `audio.m4a`: corrupt audio file — write `echo 'not valid audio data' > audio.m4a` so `hear` will exit nonzero and trigger `TranscriptionOutcome::Failed`
   - `metadata.json`: minimal JSON `{"id": "9900000002", "title": "Proof Stub: Failure Candidate"}`
4. Write `proofs/run-proof.sh` as a bash script with `set -euo pipefail` REMOVED (we want the script to continue past `transcribe-all` errors) and explicit `|| true` on the transcribe-all phase. The script structure:
   ```
   #!/usr/bin/env bash
   # Phase 1: Real artifacts — status inspection
   # Phase 2: Scratch completed artifact — cleanup candidate visibility  
   # Phase 3: Scratch failure artifact — failure reason visibility
   ```
   Each phase uses `echo` banners and redirects output with `tee -a` into `proofs/proof.log`. The log is truncated at start (`> proofs/proof.log` before first append).
5. Make the script executable: `chmod +x proofs/run-proof.sh`

## Key constraints
- The script MUST use `--output-root proofs/scratch-artifacts` for all Phase 2/3 commands — never touch real `artifacts/`
- Phase 1 commands operate against the real `artifacts/` directory (default output-root)
- The script calls the binary as `./target/debug/twitch-dl-rs` — the caller must have run `cargo build` first
- `transcribe-all --output-root proofs/scratch-artifacts --continue-on-error || true` ensures Phase 3 doesn't abort the script even if hear fails (which is expected)
- Do NOT add `source_url.txt` to scratch artifacts — the schema doesn't require it and the cleanup command doesn't check for it
  - Estimate: 30m
  - Files: proofs/run-proof.sh, proofs/scratch-artifacts/9900000001/status.json, proofs/scratch-artifacts/9900000001/transcript.srt, proofs/scratch-artifacts/9900000001/transcript.vtt, proofs/scratch-artifacts/9900000001/audio.m4a, proofs/scratch-artifacts/9900000001/metadata.json, proofs/scratch-artifacts/9900000002/status.json, proofs/scratch-artifacts/9900000002/audio.m4a, proofs/scratch-artifacts/9900000002/metadata.json
  - Verify: test -x proofs/run-proof.sh && test -f proofs/scratch-artifacts/9900000001/status.json && test -f proofs/scratch-artifacts/9900000002/status.json && python3 -c "import json; s=json.load(open('proofs/scratch-artifacts/9900000001/status.json')); assert s['ready_for_notes']==True and s['transcription_outcome']=='completed', 'completed fixture wrong'" && python3 -c "import json; s=json.load(open('proofs/scratch-artifacts/9900000002/status.json')); assert s['transcribed']==False and s.get('transcription_outcome') is None, 'failure fixture wrong'" && echo 'T01 fixtures OK'
- [ ] **T02: Execute proof walkthrough and capture proof log** — Run `proofs/run-proof.sh` to execute the full three-phase operator walkthrough, capture all CLI output to `proofs/proof.log`, and verify the log contains durable evidence for every M001 pipeline contract.

This task is the milestone completion evidence. The proof log must be produced by actually running the CLI — not hand-constructed.

## Steps

1. Run `cargo build` and confirm it succeeds with zero errors.
2. Run `bash proofs/run-proof.sh 2>&1` to execute the full walkthrough. The script writes `proofs/proof.log`.
3. If the script errors unexpectedly (not in the expected `hear` failure), inspect the error, fix the fixture or script, and re-run. The `|| true` guard on `transcribe-all` means a `hear` failure is expected and should not abort the script.
4. Inspect `proofs/proof.log` for the required evidence signals:
   - Phase 1: real artifact IDs appear (2676094572, 2693295712 or others from the 25 real items)
   - Phase 1: status table with DOWNLOADED/TRANSCRIBED columns visible
   - Phase 2: artifact 9900000001 shows `completed` in OUTCOME and `yes` in READY
   - Phase 2: cleanup lists 9900000001 as a candidate
   - Phase 3: `transcribe-all` attempts 9900000002, `hear` fails, outcome becomes `failed` with a reason
   - Phase 3: subsequent `status` on scratch dir shows `failed` for 9900000002
5. Run `cargo test` to confirm all 14 tests still pass (nothing mutated real state).
6. If any phase produces wrong output (e.g. cleanup shows no candidates, failure reason missing), diagnose and fix:
   - Cleanup empty: verify `ready_for_notes=true` AND `transcription_outcome="completed"` both present in 9900000001/status.json
   - Failure reason missing: verify 9900000002/status.json was updated after transcribe-all (check `transcription_outcome` field)
   - hear produces unexpected outcome: check what `hear -d -i proofs/scratch-artifacts/9900000002/audio.m4a -S` returns directly

## Key constraints
- The binary path is `./target/debug/twitch-dl-rs` — must have cargo build done first
- Do NOT manually edit `proofs/proof.log` — it must be produced by CLI execution
- Phase 3 deliberately produces a `hear` failure; this is expected behavior, not a bug
- After Phase 3, `proofs/scratch-artifacts/9900000002/status.json` will be mutated by `transcribe-all` — this is expected and intentional
- Real `artifacts/` directory must remain completely unchanged throughout
  - Estimate: 30m
  - Files: proofs/proof.log, proofs/scratch-artifacts/9900000002/status.json
  - Verify: test -f proofs/proof.log && wc -l proofs/proof.log | awk '{if($1<10) exit 1}' && grep -q '267[0-9]\{7\}' proofs/proof.log && grep -qi 'completed' proofs/proof.log && grep -qi 'yes' proofs/proof.log && grep -qi 'failed\|hear\|error' proofs/proof.log && cargo test 2>&1 | grep -q '14 passed' && echo 'T02 proof log OK'
