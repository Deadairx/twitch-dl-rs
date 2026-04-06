---
estimated_steps: 24
estimated_files: 2
skills_used: []
---

# T02: Execute proof walkthrough and capture proof log

Run `proofs/run-proof.sh` to execute the full three-phase operator walkthrough, capture all CLI output to `proofs/proof.log`, and verify the log contains durable evidence for every M001 pipeline contract.

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

## Inputs

- ``proofs/run-proof.sh``
- ``proofs/scratch-artifacts/9900000001/status.json``
- ``proofs/scratch-artifacts/9900000001/transcript.srt``
- ``proofs/scratch-artifacts/9900000001/transcript.vtt``
- ``proofs/scratch-artifacts/9900000001/audio.m4a``
- ``proofs/scratch-artifacts/9900000002/status.json``
- ``proofs/scratch-artifacts/9900000002/audio.m4a``
- ``./target/debug/twitch-dl-rs` (binary — run cargo build first)`

## Expected Output

- ``proofs/proof.log``
- ``proofs/scratch-artifacts/9900000002/status.json` (mutated by transcribe-all with failure outcome)`

## Verification

test -f proofs/proof.log && wc -l proofs/proof.log | awk '{if($1<10) exit 1}' && grep -q '267[0-9]\{7\}' proofs/proof.log && grep -qi 'completed' proofs/proof.log && grep -qi 'yes' proofs/proof.log && grep -qi 'failed\|hear\|error' proofs/proof.log && cargo test 2>&1 | grep -q '14 passed' && echo 'T02 proof log OK'
