---
id: S01
parent: M002-z48awz
milestone: M002-z48awz
uat_type: artifact-driven
---

# S01: Metadata Durability — UAT

**Milestone:** M002-z48awz  
**Date:** 2026-04-06

## UAT Type

- **Mode:** artifact-driven (inspect metadata.json and status.json on disk)
- **Why this mode is sufficient:** S01 is purely a schema and artifact structure change. The proof is reading the files written by download commands and verifying they contain the new fields. No live-runtime behavior or multi-step workflows are required to validate the schema extension.

## Preconditions

- `cargo build` has been run and binary exists at `./target/debug/vod-pipeline`
- Network connectivity for Twitch GQL API (for test cases 1 and 2; test case 3 avoids this)
- Writable temporary directory for artifact output
- A valid Twitch video URL (e.g. https://www.twitch.tv/videos/1234567890 — replace with a real VOD)

## Smoke Test

```bash
# Quick check: binary exists and download command works
./target/debug/vod-pipeline download --help | grep -q skip-metadata && echo "✅ CLI flag present"

# Verify schema types are correct
cargo test test_metadata && echo "✅ Schema tests pass"
```

## Test Cases

### 1. Download a Twitch VOD with GQL metadata fetch

1. Run: `./target/debug/vod-pipeline download <TWITCH_VIDEO_URL>`
   - Replace `<TWITCH_VIDEO_URL>` with a real Twitch VOD URL (e.g., https://www.twitch.tv/videos/1234567890)
2. Wait for download to complete
3. **Expected:** 
   - Download succeeds (exits 0)
   - Artifact directory created at `<video_id>/`
   - `<video_id>/metadata.json` exists and contains:
     - `"title": "..."` (string, non-null)
     - `"channel": "..."` (string, non-null)
     - `"uploaded_at": "..."` (string, non-null)
   - `<video_id>/status.json` exists and contains:
     - `"downloaded": true`
     - `"media_file": "audio.m4a"` or `"video.mp4"` (depending on stream type)
4. **Verify:** `jq '.title, .channel, .uploaded_at' <video_id>/metadata.json` shows three non-null strings

### 2. Download with --skip-metadata flag (no GQL fetch)

1. Run: `./target/debug/vod-pipeline download <TWITCH_VIDEO_URL> --skip-metadata`
   - Use a different video ID to create a new artifact
2. Wait for download to complete
3. **Expected:**
   - Download succeeds (exits 0)
   - Artifact directory created at `<video_id>/`
   - `<video_id>/metadata.json` exists but lacks title/channel/uploaded_at (or they are null)
   - `<video_id>/status.json` exists with `"downloaded": true`
   - No GQL errors in output (download proceeded without attempting GQL fetch)
4. **Verify:** `jq '.title' <video_id>/metadata.json` returns `null` (or the field is absent)

### 3. Backward compatibility: old metadata.json deserializes cleanly

1. Create a temporary artifact directory: `mkdir -p /tmp/test_compat_old_artifact`
2. Write an old-style metadata.json (without new fields):
   ```json
   {
     "schema_version": 1,
     "video_id": "123456789",
     "source_url": "https://www.twitch.tv/videos/123456789",
     "downloaded_at_epoch_s": 1700000000,
     "used_auth_token": false,
     "output_file": "video.mp4",
     "output_size_bytes": 1000000,
     "stream_name": null,
     "selected_bandwidth": null,
     "selected_resolution": "1080p",
     "selected_codecs": "h264",
     "is_audio_only": false
   }
   ```
3. Create an old-style status.json:
   ```json
   {
     "schema_version": 1,
     "video_id": "123456789",
     "source_url": "https://www.twitch.tv/videos/123456789",
     "media_file": "video.mp4",
     "transcript_file": null,
     "downloaded": true,
     "transcribed": false,
     "last_error": null,
     "updated_at_epoch_s": 1700000000,
     "ready_for_notes": false
   }
   ```
4. Run: `cargo test test_metadata_backward_compat` (unit test)
5. **Expected:**
   - Test passes (exits 0)
   - Old metadata.json fields present and correct
   - New fields (title, channel, uploaded_at) are defaulted to None
   - Old status.json fields present and correct
6. **Verify:** No deserialization errors in test output

### 4. Roundtrip: new metadata fields serialize and deserialize cleanly

1. Run: `cargo test test_metadata_roundtrip`
2. **Expected:**
   - Test passes (exits 0)
   - Metadata struct with all fields (old and new) serializes to JSON
   - JSON deserializes back to struct with identical field values
   - No data loss
3. **Verify:** No serialization/deserialization errors in test output

### 5. read_metadata function loads existing artifact metadata

1. After test case 1, inspect the artifact directory created
2. Run: `cargo test test_read_queue` (or any test that uses artifact functions to verify read_metadata is callable)
3. **Expected:**
   - Compilation succeeds (read_metadata is in scope and type-correct)
   - Tests that use artifact functions pass
4. **Verify:** No compilation errors or unresolved references to read_metadata

## Edge Cases

### A. GQL fetch fails when network is unavailable (no --skip-metadata)

1. Run: `./target/debug/vod-pipeline download <TWITCH_VIDEO_URL>` (without --skip-metadata) while:
   - Network is disconnected, OR
   - Use a malformed/invalid video ID to trigger GQL failure
2. **Expected:**
   - Download command exits non-zero
   - Error message printed to stderr: `Failed to fetch VOD metadata:` followed by the reason
   - Hint printed: `use --skip-metadata to download without metadata`
   - No artifact directory left behind (or incomplete artifact is not written)
3. **Verify:** `echo $?` shows non-zero exit code after failure

### B. Large metadata values don't break serialization

1. Use a VOD with a very long title (e.g., 200+ characters)
2. Run: `./target/debug/vod-pipeline download <LONG_TITLE_VOD_URL>`
3. **Expected:**
   - Download completes successfully
   - metadata.json is valid JSON and contains the full title string
   - `jq '.title | length' <video_id>/metadata.json` shows the correct character count
4. **Verify:** No truncation or serialization errors

## Failure Signals

- ❌ `cargo test test_metadata_backward_compat` fails → old metadata.json does not deserialize (schema regression)
- ❌ `jq '.title'` on new metadata.json returns null when --skip-metadata was NOT used → GQL fetch silently failed without error
- ❌ `./target/debug/vod-pipeline download --help` does not mention --skip-metadata → CLI flag registration failed
- ❌ Download succeeds but status.json is missing `"downloaded": true` → bare download path did not write status
- ❌ Compilation of artifact.rs fails with "field `title` not found" → schema extension did not apply

## Not Proven By This UAT

- **Live GQL API behavior over extended time (rate limits, API changes).** This UAT tests against a single GQL call at a point in time. Long-running deployments or rate-limited scenarios are not covered.
- **Integration with S02 status display.** The status command reads metadata.json and uses these fields. That display layer is tested in S02 UAT, not here.
- **Performance of GQL fetch under load.** This UAT is a single synchronous download. Batch scenarios (S03) are not covered.
- **Non-Twitch source metadata.** The GQL fetch is Twitch-specific. S07 (Additional Source Support) will extend this pattern to other sources; this UAT does not verify non-Twitch behavior.

## Notes for Tester

- **Use real video URLs.** Test case 1 requires a working Twitch VOD. If you don't have one, use a popular streamer's channel and pick a recent broadcast.
- **Ephemeral artifacts are fine.** The test artifact directories created during UAT can be left in place or deleted; they don't affect future runs. If you want to clean up, `rm -rf <video_id>` after each test case.
- **GQL fetch is expected to work.** If GQL calls fail, check network connectivity and Client-ID header. The Client-ID in the code is public and should work. If it doesn't, Twitch API may have changed.
- **--skip-metadata is meant for privacy, not as a default escape hatch.** Normal operation assumes metadata is available. Use --skip-metadata only for offline or privacy-sensitive scenarios.
- **Test case 3 (backward compat) is the most critical.** If old artifacts fail to deserialize, S01 breaks the use case. All other test cases are incremental validation; this one is a deal-breaker.

