---
estimated_steps: 6
estimated_files: 2
skills_used: []
---

# T01: Extend ArtifactMetadata schema and add GQL metadata fetch

**Slice:** S01 — Metadata Durability
**Milestone:** M002-z48awz

## Description

Add three new display fields to `ArtifactMetadata` and a new GQL function in `twitch.rs`. This task is purely library code — no call-site changes in `main.rs` or `cli.rs`. T02 wires the results into the runtime paths.

The key contract: `from_download` gains an `Option<(&str, &str, &str)>` parameter (title, channel, uploaded_at). When `Some`, the three fields are populated. When `None`, they remain `None`. All three fields use `#[serde(default)]` so old `metadata.json` files on disk deserialize cleanly with the new schema.

`ArtifactMetadata` currently only derives `Serialize`. Adding `Deserialize` is required before `read_metadata` can exist and before backward-compat unit tests can be written.

`fetch_vod_metadata_by_id` in `twitch.rs` uses the existing unauthenticated GQL pattern (same `Client-ID: kimne78kx3ncx6brgo4mv6wki5h1ko`, no auth header). The GQL query uses the top-level `video` field:

```graphql
{ video(id: "<video_id>") { title publishedAt owner { login } } }
```

Returns `(title, channel, uploaded_at)` as a `(String, String, String)` tuple, or `TwitchError::Parse` if the video node is absent.

## Failure Modes

| Dependency | On error | On timeout | On malformed response |
|------------|----------|-----------|----------------------|
| Twitch GQL (`fetch_vod_metadata_by_id`) | Return `TwitchError::Parse` or `TwitchError::Http`; caller in T02 decides abort vs skip | `reqwest` surfaces as `TwitchError::Http`; propagate | Return `TwitchError::Parse` with details |

## Negative Tests

- **Malformed inputs**: old `metadata.json` without `title`, `channel`, `uploaded_at` must deserialize without error — assert all new fields are `None`
- **Error paths**: GQL response missing `data.video` node must return `TwitchError::Parse`, not panic
- **Boundary conditions**: `ArtifactMetadata` with all display fields `None` must serialize cleanly (fields absent due to `skip_serializing_if`)

## Steps

1. In `src/artifact.rs`: add `#[derive(Deserialize)]` to `ArtifactMetadata` (it currently only derives `Serialize`). The file already has `use serde::{Deserialize, Serialize}` imported for other types — verify and use it.
2. Add three fields to `ArtifactMetadata` after `selected_codecs` and before `is_audio_only`:
   ```rust
   #[serde(default, skip_serializing_if = "Option::is_none")]
   pub title: Option<String>,
   #[serde(default, skip_serializing_if = "Option::is_none")]
   pub channel: Option<String>,
   #[serde(default, skip_serializing_if = "Option::is_none")]
   pub uploaded_at: Option<String>,
   ```
3. Change `from_download` to accept a final `vod_context: Option<(&str, &str, &str)>` parameter. Inside the constructor, destructure it:
   ```rust
   let (title, channel, uploaded_at) = match vod_context {
       Some((t, c, u)) => (Some(t.to_string()), Some(c.to_string()), Some(u.to_string())),
       None => (None, None, None),
   };
   ```
   Populate the new struct fields from these locals.
4. Add `pub fn read_metadata(artifact_dir: &Path) -> Result<Option<ArtifactMetadata>, Box<dyn std::error::Error>>` after `write_metadata`. Read `metadata.json`; return `Ok(None)` if absent; deserialize and return `Ok(Some(...))` if present.
5. In `src/twitch.rs`: add private GQL response structs for the single-video query (follow the existing pattern of private structs already in the file). Add `pub async fn fetch_vod_metadata_by_id(video_id: &str) -> Result<(String, String, String), TwitchError>`. Use `serde_json::json!` for the request body, same `Client::new() + .header("Client-ID", "kimne78kx3ncx6brgo4mv6wki5h1ko")` pattern. Parse `data.video.title`, `data.video.publishedAt`, and `data.video.owner.login`. Return `TwitchError::Parse(...)` if any field is absent.
6. Add two unit tests in the `#[cfg(test)]` block in `src/artifact.rs`:
   - `test_metadata_backward_compat`: deserialize a hard-coded old-format JSON string (no `title`/`channel`/`uploaded_at` keys) into `ArtifactMetadata`; assert all three new fields are `None` and an existing field (`video_id`) is correct. Old format example: `{"schema_version":1,"video_id":"123","source_url":"https://twitch.tv/videos/123","downloaded_at_epoch_s":0,"used_auth_token":false,"output_file":"audio.m4a","output_size_bytes":100,"stream_name":null,"selected_bandwidth":null,"selected_resolution":null,"selected_codecs":null,"is_audio_only":true}`.
   - `test_metadata_roundtrip`: create a temp file (using `tempfile::tempdir()` already used in other tests), call `from_download` with `Some(("Test VOD", "testchan", "2026-01-01T00:00:00Z"))` as `vod_context`, serialize to JSON string, deserialize back into `ArtifactMetadata`, assert `title == Some("Test VOD")`, `channel == Some("testchan")`, `uploaded_at == Some("2026-01-01T00:00:00Z")`.

## Must-Haves

- [ ] `ArtifactMetadata` derives both `Serialize` and `Deserialize`
- [ ] Three new fields (`title`, `channel`, `uploaded_at`) have `#[serde(default)]` and `#[serde(skip_serializing_if = "Option::is_none")]`
- [ ] `from_download` accepts `vod_context: Option<(&str, &str, &str)>` as last parameter
- [ ] `read_metadata` function exists and returns `Result<Option<ArtifactMetadata>, Box<dyn std::error::Error>>`
- [ ] `fetch_vod_metadata_by_id` exists in `twitch.rs`, uses unauthenticated GQL, returns `(String, String, String)` or `TwitchError`
- [ ] `test_metadata_backward_compat` passes (old JSON without new fields deserializes cleanly)
- [ ] `test_metadata_roundtrip` passes (full-field serialize → deserialize round-trip)
- [ ] `cargo test` exits 0

## Verification

- `cargo test 2>&1 | grep -E 'test result|FAILED|^error'` — must show `test result: ok` with no FAILED lines
- `cargo build 2>&1 | grep '^error'` — must produce no output

## Observability Impact

- Signals added/changed: `read_metadata` exposes previously write-only `metadata.json` as a typed struct for programmatic inspection
- How a future agent inspects this: call `artifact::read_metadata(&artifact_dir)` or `cat <artifact_dir>/metadata.json`
- Failure state exposed: `read_metadata` returns `Err(...)` on malformed JSON so callers get a typed error rather than a silent `None`

## Inputs

- `src/artifact.rs` — existing `ArtifactMetadata` struct and `from_download` signature to extend
- `src/twitch.rs` — existing unauthenticated GQL client pattern (see `fetch_vod_access_token` and `fetch_channel_archive_vods`) and private struct patterns to follow

## Expected Output

- `src/artifact.rs` — `ArtifactMetadata` with `Deserialize`, three new fields, updated `from_download` signature, new `read_metadata` function, two new unit tests
- `src/twitch.rs` — new `fetch_vod_metadata_by_id` pub async function with supporting private GQL response structs
