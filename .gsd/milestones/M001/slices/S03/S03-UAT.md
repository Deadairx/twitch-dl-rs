# S03: Trustworthy transcription and failure surfacing — UAT

**Milestone:** M001
**Written:** 2026-04-06T03:10:51.661Z

# S03: Trustworthy transcription and failure surfacing — UAT

**Milestone:** M001
**Written:** 2026-04-06

## UAT Type

- **Mode**: Artifact-driven (inspect status.json and transcript artifacts directly; no runtime server)
- **Why this mode is sufficient**: Transcription is stateless per-item and produces filesystem artifacts (transcript.srt, transcript.vtt, status.json fields). All outcomes are observable as persistent artifact state.

## Preconditions

1. **Build must be current**: `cargo build && cargo test --lib` passes with zero warnings and 11/11 tests passing.
2. **hear binary must be available**: `which hear` succeeds.
3. **ffprobe must be available**: `which ffprobe` succeeds.

## Test Cases

### 1. Completed Outcome: Valid audio produces transcript artifacts with completed status
- Run transcription on valid 30-second audio
- Expected: status.json contains `"transcription_outcome": "completed"`, both transcript.srt and transcript.vtt exist, status.transcribed=true

### 2. Suspect Outcome: Short audio triggers word-count heuristic
- Run transcription on very short audio (10 seconds, minimal speech)
- Expected: status.json contains `"transcription_outcome": "suspect"`, reason contains "word count", transcript.srt and transcript.vtt exist, status.transcribed=false

### 3. Suspect Outcome: Repetition heuristic
- Run transcription on audio with high repetition
- Expected: status.json contains `"transcription_outcome": "suspect"`, reason contains "repeated trigram"

### 4. Failed Outcome: Invalid audio causes transcription failure
- Run transcription on corrupted audio file
- Expected: status.json contains `"transcription_outcome": "failed"`, reason contains error message, transcript artifacts do NOT exist

### 5. Suspect Items Are Skipped on Re-run
- After suspect outcome, re-run transcribe-all
- Expected: Item is skipped (not re-transcribed), status.json outcome remains "suspect", transcript file timestamps unchanged

### 6. Completed Items Are Skipped on Re-run
- After completed outcome, re-run transcribe-all
- Expected: Item is skipped, status.json unchanged

### 7. Failed Items Are Retried on Re-run
- After failed outcome, replace invalid audio with valid file
- Re-run transcribe-all
- Expected: transcription succeeds, outcome changes to "completed" or "suspect"

### 8. Status Display Shows OUTCOME and REASON Columns
- Run `cargo run -- status`
- Expected: Table shows VIDEO_ID, DOWNLOADED, OUTCOME, REASON columns with correct values

### 9. SRT↔VTT Conversion Correctness
- After successful transcription, compare transcript.srt and transcript.vtt
- Expected: VTT has WEBVTT header, timestamps use dots (not commas), no sequence numbers, text content identical"
