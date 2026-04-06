# S05: End-to-end operator flow proof — Research

**Date:** 2026-04-06
**Calibration:** Light/targeted — S05 is pure verification work. No new features. The task is running CLI commands, capturing output, and manufacturing one failure scenario.

## Summary

S05 requires running the real CLI against existing artifacts and a manufactured failure, then capturing all output to a proof log. The CLI is fully built and working (8 commands, `cargo build` clean, 14/14 tests pass). The key complication is a **state gap**: all 25 real artifacts (`artifacts/`) were transcribed by the old `mlx-whisper` path and have `transcript.txt` files + the old schema (`transcribed: true`, no `transcription_outcome`, no `ready_for_notes`). The S03/S04 features (`.srt`/`.vtt`, `ready_for_notes=true`, `transcription_outcome=completed`) only activate on items processed by the new `transcribe-all` path.

This means the happy-path proof (status shows `outcome=completed`, `ready=yes`; cleanup lists candidates) requires either: (a) running `transcribe-all` on one real artifact that `hear` can process successfully, or (b) constructing a scratch artifact with correctly-shaped `status.json` fields to simulate a completed item without re-transcribing. Option (b) is the safer, faster approach for the proof — it avoids a long transcription run and doesn't mutate real artifact state.

The manufactured failure scenario is straightforward: create a scratch artifact dir with `downloaded=true, transcribed=false` and a corrupted/absent audio file, then run `transcribe-all`. The failure will surface in `status.json` and the `status` output.

## Recommendation

The proof walkthrough should proceed in three phases:

**Phase 1 — Happy path status inspection (pre-existing artifacts)**
Run `status`, `download-all` (no-op, all downloaded), and `cleanup` (will show "No cleanup candidates" since old artifacts lack new schema) against the real `artifacts/` directory. This proves the status command surfaces all 25 real artifacts with their state (downloaded=true, outcome=-, last_error visible for 2693295712).

**Phase 2 — Manufactured completed artifact (scratch dir)**
Create a scratch artifact directory at `proofs/scratch-artifacts/` with:
- One artifact with a properly shaped `status.json` (`transcribed=true, ready_for_notes=true, transcription_outcome=completed`) plus `transcript.srt` and `transcript.vtt` stub files
- Run `status --output-root proofs/scratch-artifacts` → shows `outcome=completed, ready=yes`
- Run `cleanup --output-root proofs/scratch-artifacts` → shows that artifact as a candidate
This proves the S04 cleanup path works without re-transcribing a real VOD.

**Phase 3 — Manufactured failure scenario (scratch dir)**
Create a second scratch artifact in `proofs/scratch-artifacts/` with:
- `status.json` with `downloaded=true, transcribed=false` and a missing or zero-byte audio file
- Run `transcribe-all --output-root proofs/scratch-artifacts` → `hear` fails, status.json updated with `transcription_outcome=failed` and `transcription_reason`
- Run `status --output-root proofs/scratch-artifacts` → shows `outcome=failed` with reason
This proves failure visibility without corrupting real artifacts.

All output is captured via shell redirection to `proofs/proof.log`.

## Implementation Landscape

### Key Files

- `src/main.rs` — All CLI handlers: `show_status`, `transcribe_all`, `download_all`, `cleanup`
- `src/artifact.rs` — `ProcessStatus` schema (has `ready_for_notes`, `transcription_outcome`, `transcription_reason` with `#[serde(default)]`), `scan_artifact_statuses`, `write_status`, `read_status`
- `src/transcribe.rs` — `transcribe_to_srt_and_vtt` invokes `hear -d -i <file> -S`; `TranscriptionOutcome::Failed` returned when `hear` exits nonzero
- `artifacts/` — Real artifact store: 25 items, all `downloaded=true`, all old-schema (`transcript.txt`, no VTT/SRT, no `ready_for_notes` field); one item (2693295712) has `last_error` from prior failure
- `artifacts/queues/theprimeagen.json` — Real persisted queue file (25 entries, old schema compatible)

### State Gap (critical awareness)

The existing 25 real artifacts were processed by the old pipeline. Their `status.json` files have:
- `transcribed: true` (or `false` for the one failure)
- `transcript_file: "transcript.txt"` — **not** `.srt`
- No `transcription_outcome`, `transcription_reason`, `transcript_word_count`, `ready_for_notes` fields

Since `transcribe_all` filters on `!s.transcribed`, these will be skipped — they won't be re-processed through the S03 path. The cleanup command correctly returns "No cleanup candidates" because `ready_for_notes` defaults to `false` on deserialization.

This is **not a bug** — it is correct backward-compatible behavior. The proof must account for it by manufacturing the new-schema state explicitly rather than expecting it to appear in real artifacts.

### Build Order

1. **Create `proofs/` directory** and define the proof script (`proofs/run-proof.sh`) with all steps documented, including failure scenario setup
2. **Run Phase 1** against real `artifacts/` — status, download-all (no-op), cleanup (empty) — capturing output
3. **Manufacture scratch artifacts** in `proofs/scratch-artifacts/` — two items: one completed (with proper status.json + stub SRT/VTT), one pending-download
4. **Run Phase 2** against scratch dir — status shows completed, cleanup shows candidate
5. **Manufacture failure** — add third scratch item with `downloaded=true` but missing audio, run transcribe-all
6. **Run Phase 3** against scratch dir — status shows failure with reason
7. Concatenate all output into `proofs/proof.log`

### Verification Approach

```bash
# Build is clean
cargo build 2>&1 | grep -E "error|warning" | wc -l  # should be 0

# All tests pass
cargo test 2>&1 | tail -3  # "14 passed; 0 failed"

# Proof log exists and is non-empty
test -f proofs/proof.log && wc -l proofs/proof.log

# Status command surfaces real artifacts
grep "2676094572\|2693295712" proofs/proof.log

# Failure reason visible in log
grep "failed\|Transcription\|hear" proofs/proof.log

# Ready-for-notes state visible in log
grep "completed.*yes\|yes.*completed" proofs/proof.log

# Cleanup lists candidate in log
grep "transcript.srt\|transcript.vtt\|audio.m4a" proofs/proof.log
```

## Constraints

- Scratch artifacts MUST go in a dedicated directory (`proofs/scratch-artifacts/`) — never mutate `artifacts/` real items
- The proof log must be produced by actually running CLI commands, not hand-constructed
- `hear` is installed at `/usr/local/bin/hear` — it is available but we deliberately want a failure scenario where the audio is missing or zero-byte to trigger `TranscriptionOutcome::Failed`
- The `--output-root` flag defaults to `artifacts` — all scratch runs must explicitly pass `--output-root proofs/scratch-artifacts`

## Common Pitfalls

- **Scratch audio file for failure** — `hear` needs a non-existent or unreadable file to fail cleanly. A zero-byte file may produce a different error than an absent file. Use an absent file (don't create `audio.m4a`) so `find_media_file` returns `None` and the item is skipped gracefully. Instead, explicitly manufacture a `hear` failure by creating a bad audio file (e.g. `echo "not audio" > audio.m4a`) which causes `hear` to exit nonzero and trigger `TranscriptionOutcome::Failed`.
- **Old-schema artifacts and transcribe-all** — Items with `transcribed: true` are filtered out by `transcribe_all`. Only items with `transcribed: false` AND `transcription_outcome != "suspect"` get picked up. The scratch failure artifact must have `transcribed: false` to be picked up.
- **cleanup requires both ready_for_notes=true AND transcription_outcome="completed"** (based on the filtering logic in `scan_artifact_statuses` for cleanup candidates) — the manufactured completed artifact must have both fields set correctly.
- **Proof log append vs overwrite** — use `tee -a` or `>>` consistently so all phases land in one log without wiping earlier output.
