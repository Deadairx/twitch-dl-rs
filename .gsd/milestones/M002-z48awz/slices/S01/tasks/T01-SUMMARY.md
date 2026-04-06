---
task_id: T01
slice_id: S01
milestone_id: M002-z48awz
status: complete
blocker_discovered: false
---

# T01 Summary: Extend ArtifactMetadata schema and add GQL metadata fetch

## What Was Done

Implemented library-level schema extensions to support human-readable VOD context in artifact metadata:

### artifact.rs Changes
1. **Added Deserialize derive**: `ArtifactMetadata` now derives both `Serialize` and `Deserialize` to enable metadata deserialization from disk.
2. **Extended schema with three new fields** (placed after `selected_codecs`, before `is_audio_only`):
   - `title: Option<String>` — VOD title
   - `channel: Option<String>` — streamer login/channel name  
   - `uploaded_at: Option<String>` — ISO 8601 publication timestamp
   - All three use `#[serde(default, skip_serializing_if = "Option::is_none")]` for backward compatibility and clean serialization
3. **Updated from_download signature**: Added `vod_context: Option<(&str, &str, &str)>` parameter (title, channel, uploaded_at). When `Some`, populates the three new fields; when `None`, all remain `None`.
4. **Implemented read_metadata function**: Reads `metadata.json` from artifact directory. Returns `Ok(None)` if file absent, `Ok(Some(metadata))` if present and valid, or `Err(...)` if JSON malformed.
5. **Added two unit tests**:
   - `test_metadata_backward_compat`: Deserializes old-format JSON (without new fields) and verifies all three new fields default to `None` while existing fields load correctly.
   - `test_metadata_roundtrip`: Creates metadata with all three new fields populated, serializes to JSON, deserializes back, and asserts values match.

### twitch.rs Changes
1. **Added private GQL response structs** following existing patterns:
   - `VideoMetadataResponse` — top-level response envelope
   - `VideoMetadataData` — wraps optional `video` node
   - `VideoMetadataNode` — video details: `title`, `publishedAt`, `owner`
   - `VideoOwner` — owner login
2. **Implemented fetch_vod_metadata_by_id** async function:
   - Takes video_id as parameter
   - Uses unauthenticated GQL with same `Client-ID: kimne78kx3ncx6brgo4mv6wki5h1ko` header pattern as existing code
   - Constructs single-video GQL query: `{ video(id: "<id>") { title publishedAt owner { login } } }`
   - Returns `(String, String, String)` tuple: (title, channel, uploaded_at)
   - Returns `TwitchError::Parse` with details if video node absent

### main.rs Update
- Updated the single call site in `download_to_artifact` to pass `None` for `vod_context`. (T02 will wire actual metadata fetch into call sites.)

## Must-Haves Verification

✅ `ArtifactMetadata` derives both `Serialize` and `Deserialize`
✅ Three new fields (`title`, `channel`, `uploaded_at`) have `#[serde(default)]` and `#[serde(skip_serializing_if = "Option::is_none")]`
✅ `from_download` accepts `vod_context: Option<(&str, &str, &str)>` as last parameter
✅ `read_metadata` function exists and returns `Result<Option<ArtifactMetadata>, Box<dyn std::error::Error>>`
✅ `fetch_vod_metadata_by_id` exists in `twitch.rs`, uses unauthenticated GQL, returns `(String, String, String)` or `TwitchError`
✅ `test_metadata_backward_compat` passes (old JSON without new fields deserializes cleanly)
✅ `test_metadata_roundtrip` passes (full-field serialize → deserialize round-trip)
✅ `cargo test` exits 0

## Verification Evidence

| Check | Command | Exit Code | Verdict | Duration |
|-------|---------|-----------|---------|----------|
| Full test suite | `cargo test 2>&1` | 0 | ✅ pass | 0.07s |
| Backward compat test | `cargo test test_metadata_backward_compat` | 0 | ✅ pass | 0.01s |
| Roundtrip test | `cargo test test_metadata_roundtrip` | 0 | ✅ pass | 0.01s |
| Build success | `cargo build 2>&1` | 0 | ✅ pass | 0.07s |
| No errors | `cargo build 2>&1 \| grep '^error'` | 1 (no match) | ✅ pass | — |

## Observability Impact

- **Signal added**: `read_metadata(&artifact_dir)` now exposes `metadata.json` as typed struct for programmatic inspection. Callers can check field presence, validate data, etc.
- **Failure state clarified**: Malformed JSON returns `Err(...)` rather than silent `None`, allowing callers to log, retry, or abort cleanly.
- **Future inspection**: Agents can call `artifact::read_metadata()` to load and inspect metadata programmatically, or `cat <artifact_dir>/metadata.json` for manual inspection.

## Key Design Decisions

- **Option<T> with serde defaults**: All three new fields use `#[serde(default)]` to deserialize missing keys as `None`. This guarantees backward compatibility—old metadata files on disk will load cleanly with the new schema.
- **skip_serializing_if**: Omits `null` values from JSON output when all three fields are `None`. Keeps serialized metadata compact and clean.
- **GQL pattern consistency**: `fetch_vod_metadata_by_id` follows the same unauthenticated pattern as existing `fetch_vod_access_token` and `fetch_channel_archive_vods` functions (same Client-ID header, error handling via `TwitchError`).
- **vod_context as Option**: The `from_download` parameter accepts `Option<(&str, &str, &str)>` instead of three optional parameters. This makes the intent clearer at call sites (callers either have all three or none) and simplifies test construction.

## What Remains for T02

- Wire the GQL fetch into the actual download paths (update call sites to pass `Some(title, channel, uploaded_at)` instead of `None`)
- Add integration tests that verify end-to-end metadata flow from GQL → artifact → disk

## One-Liner

Extended ArtifactMetadata with title, channel, uploaded_at fields; implemented read_metadata and fetch_vod_metadata_by_id library functions with full backward compatibility.
