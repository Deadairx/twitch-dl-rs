---
estimated_steps: 29
estimated_files: 9
skills_used: []
---

# T01: Write proof script and manufacture scratch artifacts

Create the `proofs/` directory, write `proofs/run-proof.sh` with all three walkthrough phases documented and executable, and manufacture the scratch artifact fixtures that Phase 2 (completed) and Phase 3 (failure) depend on.

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

## Inputs

- ``src/artifact.rs` (ProcessStatus schema — know which fields are required vs optional)`
- ``src/main.rs` (cleanup candidate filter logic — know ready_for_notes + outcome=completed requirement)`

## Expected Output

- ``proofs/run-proof.sh``
- ``proofs/scratch-artifacts/9900000001/status.json``
- ``proofs/scratch-artifacts/9900000001/transcript.srt``
- ``proofs/scratch-artifacts/9900000001/transcript.vtt``
- ``proofs/scratch-artifacts/9900000001/audio.m4a``
- ``proofs/scratch-artifacts/9900000001/metadata.json``
- ``proofs/scratch-artifacts/9900000002/status.json``
- ``proofs/scratch-artifacts/9900000002/audio.m4a``
- ``proofs/scratch-artifacts/9900000002/metadata.json``

## Verification

test -x proofs/run-proof.sh && test -f proofs/scratch-artifacts/9900000001/status.json && test -f proofs/scratch-artifacts/9900000002/status.json && python3 -c "import json; s=json.load(open('proofs/scratch-artifacts/9900000001/status.json')); assert s['ready_for_notes']==True and s['transcription_outcome']=='completed', 'completed fixture wrong'" && python3 -c "import json; s=json.load(open('proofs/scratch-artifacts/9900000002/status.json')); assert s['transcribed']==False and s.get('transcription_outcome') is None, 'failure fixture wrong'" && echo 'T01 fixtures OK'
