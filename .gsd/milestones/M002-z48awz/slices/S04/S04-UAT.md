# S04: Selective Processing — UAT

**Milestone:** M002-z48awz
**Written:** 2026-04-07T03:38:08.832Z

# S04: Selective Processing — UAT

**Milestone:** M002-z48awz
**Written:** 2026-04-07

## UAT Type

- **UAT mode:** artifact-driven (static test data) + live-runtime (CLI invocations against test artifacts)
- **Why this mode is sufficient:** The feature is stateless filtering logic. No async side effects, no external APIs, no state mutations beyond normal download/transcribe flow.

## Preconditions

1. Binary built: `cargo build` succeeded
2. Create temp output root with queue files and artifact directories:

```bash
TESTDIR=$(mktemp -d)
mkdir -p $TESTDIR/queues $TESTDIR/artifacts

# chan1.json with video ID 111111
cat > $TESTDIR/queues/chan1.json <<'EOF'
{"schema_version":1,"channel":"chan1","generated_at_epoch_s":0,"past_broadcasts_only":false,"min_seconds":600,"queued_count":1,"queued":[{"channel":"chan1","title":"VOD A","url":"https://www.twitch.tv/videos/111111","video_id":"111111","uploaded_at":"2026-01-01T00:00:00Z","duration":"PT3600S"}],"skipped_existing_ids":[]}
EOF

# chan2.json with video ID 222222
cat > $TESTDIR/queues/chan2.json <<'EOF'
{"schema_version":1,"channel":"chan2","generated_at_epoch_s":0,"past_broadcasts_only":false,"min_seconds":600,"queued_count":1,"queued":[{"channel":"chan2","title":"VOD B","url":"https://www.twitch.tv/videos/222222","video_id":"222222","uploaded_at":"2026-01-02T00:00:00Z","duration":"PT3600S"}],"skipped_existing_ids":[]}
EOF

# Artifact 333333: downloaded, not transcribed
mkdir -p $TESTDIR/artifacts/333333
echo '{"video_id":"333333","url":"https://www.twitch.tv/videos/333333","downloaded":true,"transcribed":false,"transcription_outcome":null,"ready_for_notes":false}' > $TESTDIR/artifacts/333333/status.json

# Artifact 444444: downloaded, not transcribed
mkdir -p $TESTDIR/artifacts/444444
echo '{"video_id":"444444","url":"https://www.twitch.tv/videos/444444","downloaded":true,"transcribed":false,"transcription_outcome":null,"ready_for_notes":false}' > $TESTDIR/artifacts/444444/status.json
```

## Smoke Test

```bash
./target/debug/vod-pipeline download-all --help | grep -q 'video-id' && echo "✅ download-all has --video-id"
./target/debug/vod-pipeline transcribe-all --help | grep -q 'video-id' && echo "✅ transcribe-all has --video-id"
```

## Test Cases

### 1. Download-all --video-id filters to one queued item

1. Run: `./target/debug/vod-pipeline download-all --video-id 111111 $TESTDIR 2>&1`
2. **Expected:** Command processes only 111111; output contains "Downloading 1 pending VOD(s)"; 222222 is not attempted.

### 2. Download-all --video-id non-existent ID returns error

1. Run: `./target/debug/vod-pipeline download-all --video-id 999999 $TESTDIR 2>&1`
2. **Expected:** Exit non-zero. Stderr contains: "Download-all failed: video ID 999999 not found in any queue". No items processed.

### 3. Download-all without --video-id processes all pending (regression check)

1. Run: `./target/debug/vod-pipeline download-all $TESTDIR 2>&1`
2. **Expected:** Output contains "Downloading 2 pending VOD(s) across all channels"; both 111111 and 222222 are attempted.

### 4. Transcribe-all --video-id filters to one artifact

1. Run: `./target/debug/vod-pipeline transcribe-all --video-id 333333 $TESTDIR 2>&1`
2. **Expected:** Command processes only 333333; output contains "Transcribing 1 pending artifacts"; 444444 is not attempted.

### 5. Transcribe-all --video-id non-existent ID returns error

1. Run: `./target/debug/vod-pipeline transcribe-all --video-id 555555 $TESTDIR 2>&1`
2. **Expected:** Exit non-zero. Stderr contains: "Transcribe-all failed: video ID 555555 not found in any artifact". No items processed.

### 6. Transcribe-all without --video-id processes all pending (regression check)

1. Run: `./target/debug/vod-pipeline transcribe-all $TESTDIR 2>&1`
2. **Expected:** Output contains "Transcribing 2 pending artifacts"; both 333333 and 444444 are attempted.

### 7. Flag name consistency with cleanup

1. Run `./target/debug/vod-pipeline cleanup --help | grep video-id`
2. Run `./target/debug/vod-pipeline download-all --help | grep video-id`
3. Run `./target/debug/vod-pipeline transcribe-all --help | grep video-id`
4. **Expected:** All three show `--video-id` (exact same flag name).

## Edge Cases

### A. continue-on-error is orthogonal to video-id filter

1. Run: `./target/debug/vod-pipeline transcribe-all --continue-on-error $TESTDIR`
2. **Expected:** All pending items attempted regardless of individual failures; --video-id filter and --continue-on-error are independent flags.

### B. Empty queue/artifact directory

1. Create empty output root, run: `./target/debug/vod-pipeline download-all --video-id 123456 $TESTDIR`
2. **Expected:** Clean "not found in any queue" error, no panic.
