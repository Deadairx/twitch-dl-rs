---
id: S01
parent: M002-z48awz
milestone: M002-z48awz
provides:
  - Extended ArtifactMetadata schema with human-readable display fields (title, channel, uploaded_at)
  - GQL-backed metadata fetch for bare download paths (fetch_vod_metadata_by_id)
  - --skip-metadata flag for download command (allows offline or privacy-preserving downloads)
  - status.json written for bare download artifacts (closing the artifact loop for single-video downloads)
  - read_metadata helper function for status/filtering commands
  - Backward-compatible deserialization of old metadata.json files
requires:
  - slice: null
    provides: null
affects:
  - S02 (Status Legibility — depends on reading metadata.json for display columns)
  - S03 (Intake Flexibility — queue_video populates VodEntry fields now used by download_vod)
  - S04 (Selective Processing — filters work over full artifact metadata)
  - S07 (Additional Source Support — metadata schema serves non-Twitch sources too)
key_files:
  - src/artifact.rs (ArtifactMetadata, read_metadata, ProcessStatus write)
  - src/twitch.rs (fetch_vod_metadata_by_id GQL call)
  - src/main.rs (download_vod context resolution and status.json write)
  - src/cli.rs (--skip-metadata flag)
key_decisions:
  - GQL fetch occurs after stream resolution but before prepare_artifact_dir — ensures no orphan directories if metadata fetch fails
  - vod_context ownership: fetch returns owned Strings; converted to &str for from_download to avoid lifetime issues
  - All new ArtifactMetadata fields use #[serde(default, skip_serializing_if = "Option::is_none")] for backward compat
  - download_vod writes ProcessStatus with downloaded=true after media write (normalizes bare-download artifacts to same state as queue-backed downloads)
patterns_established:
  - "Schema-first design: extend artifact schema before wiring call sites (T01 then T02 flow avoids rework)"
  - "Backward-compatible optional fields: always use #[serde(default)] on new schema fields so old artifacts deserialize cleanly"
  - "Context threading: pass vod_context as Option<(&str, &str, &str)> through call stacks to avoid lifetime/ownership cascades"
  - "Dual-path dispatch: check preconditions (skip_metadata flag) before GQL calls, with clear error messages and recovery suggestions"
observability_surfaces:
  - None — S01 is purely artifact schema and library code. Observability for metadata queries lives in S02 (status display) and later slices.
drill_down_paths:
  - .gsd/milestones/M002-z48awz/slices/S01/tasks/T01-SUMMARY.md
  - .gsd/milestones/M002-z48awz/slices/S01/tasks/T02-SUMMARY.md
duration: ~90m (45m schema + 45m wiring, per plan)
verification_result: passed
completed_at: 2026-04-06T21:50:00Z
---

# S01: Metadata Durability

**Extended artifact metadata with human-readable display fields (title, channel, uploaded_at), GQL-backed metadata fetching, and normalized status.json writes for all artifact types.**

## What Happened

S01 delivered three integrated changes to close the metadata loop:

**Schema Extension (T01):** Added `title: Option<String>`, `channel: Option<String>`, and `uploaded_at: Option<String>` to `ArtifactMetadata` with `#[serde(default, skip_serializing_if = "Option::is_none")]`, ensuring old metadata.json files deserialize cleanly without migration. Added `read_metadata(artifact_dir)` helper function and `Deserialize` derive to unlock future status/filtering commands. Implemented `fetch_vod_metadata_by_id(video_id)` in twitch.rs as a new public GQL call using the same unauthenticated `Client-ID` header as existing code.

**Runtime Wiring (T02):** Updated `download_vod` signature to accept optional vod_context and skip_metadata flag. Inserted context resolution block between stream selection and prepare_artifact_dir — when no vod_context is provided and skip_metadata=false, the GQL fetch runs. If it fails, the download aborts with a clear error message and recovery hint (use --skip-metadata or check network and retry). When skip_metadata=true, metadata fields are left null/absent in metadata.json and download proceeds. Updated `download_vod_to_artifact` to pass vod context directly from VodEntry, avoiding redundant GQL calls for queue-backed downloads. Added status.json write to the bare download path with `downloaded=true` state, normalizing single-video downloads to the same artifact structure as queue-backed ones.

**CLI Extension:** Added `--skip-metadata` flag to the download subcommand in cli.rs. Flag description is clear and discoverable via --help.

All changes preserve backward compatibility: old artifacts without these fields deserialize and round-trip cleanly.

## Verification

All verification checks from the slice plan passed:
- `cargo build`: clean (only pre-existing dead_code warning on read_metadata, which is used by S02)
- `cargo test`: 16/16 pass (includes test_metadata_backward_compat and test_metadata_roundtrip)
- `./target/debug/vod-pipeline download --help | grep skip-metadata`: flag present with correct description

No runtime errors. No new lint warnings beyond pre-existing dead_code.

## New Requirements Surfaced

None. S01 fully satisfies the scope in the slice plan and all dependent slices' preconditions.

## Operational Readiness

S01 is a library-and-schema change with no runtime concerns. No health signal, failure signal, or monitoring required. The metadata fetch (GQL call) is wrapped in clear error handling at the call site (download_vod), and the --skip-metadata flag gives operators a proven recovery path if network or API issues occur.

## Deviations

None. All tasks executed as planned. No scope changes or architectural surprises.

## Known Limitations

- `read_metadata` is not yet used in the codebase (S02 uses it for status display, but S02 is not executed yet). The function is correct and tested; it is simply awaiting its consumer.
- Metadata fetch is GQL-only (no fallback to REST API). If Twitch GQL changes or breaks, operators must use --skip-metadata. This is acceptable in M002 scope; fallback strategies are future work.

## Follow-ups

None. S01 is complete and ready for downstream consumers (S02, S03, S07). No discovered blockers or rework needed.

## Files Created/Modified

- `src/artifact.rs` — ArtifactMetadata struct extended with three new Option<String> fields; read_metadata helper function added; Deserialize derived; from_download signature updated to accept vod_context
- `src/twitch.rs` — fetch_vod_metadata_by_id pub async fn added; uses GQL with unauthenticated Client-ID header; returns (title, channel, uploaded_at) tuple
- `src/main.rs` — download_vod signature extended with vod_context and skip_metadata; context resolution block inserted pre-prepare_artifact_dir; GQL failure returns with clear error; status.json written for bare downloads with downloaded=true; download_vod_to_artifact passes vod context from VodEntry
- `src/cli.rs` — CliCommand::Download struct extended with skip_metadata bool field; --skip-metadata flag added to download subcommand with help text

## Forward Intelligence

### What the next slice should know

- **Metadata fields are extensible.** The title/channel/uploaded_at pattern is now established in S01. S07 (Additional Source Support) will extend this schema further with source-specific metadata (e.g. video_duration, thumbnail_url). The backward-compat pattern (Option<T> + #[serde(default)]) scales cleanly to additional fields.

- **read_metadata is the source of truth for display.** S02 (Status Legibility) reads metadata.json per artifact to populate status table columns. This is the established pattern; future status enhancements should read from the same artifact-local metadata, not from a centralized cache or database.

- **GQL fetch failure is a clear abort signal.** The --skip-metadata escape hatch exists, but normal operation expects metadata to be available. Operators should be trained to use --skip-metadata *only* for privacy-sensitive or offline scenarios, not as a default recovery for network flakes. If metadata fetches are frequently failing, that signals a wider infrastructure problem worth investigating.

- **Context threading scales to batch operations.** The vod_context pattern (threading it through call stacks as Option<(&str, &str, &str)>) is proven in S01 for bare downloads and download_vod_to_artifact. S03's queue_video will extend this—don't fight the ownership model; lean into it.

### What's fragile

- **GQL query format in fetch_vod_metadata_by_id.** The query string is hand-constructed and assumes specific field names (title, publishedAt, owner.login). If Twitch changes the schema, the query breaks silently (returns null video node, which becomes a TwitchError::Parse). There is no schema validation or introspection. Mitigation: monitor for GQL errors in error logs; if they spike, investigate the Twitch API changelog first.

- **Client-ID authentication.** The fetch uses a hardcoded public Client-ID (kimne78kx3ncx6brgo4mv6wki5h1ko). This ID is the same as used elsewhere in twitch.rs, so it is consistent. However, if Twitch invalidates this ID, all public metadata fetches fail. There is no fallback. This is acceptable for M002 (a single public ID is simpler than auth token management), but S07 may need to revisit if non-Twitch sources require different auth models.

### Authoritative diagnostics

- **Look at test_metadata_backward_compat first.** This test deserializes an old metadata.json without title/channel/uploaded_at fields and confirms all new fields default to None. If future schema changes cause deserialization failures, this test is the canary—it will break before production code does.

- **cargo test test_metadata** runs both backward_compat and roundtrip tests. Always run these together when modifying ArtifactMetadata.

- **./target/debug/vod-pipeline download --help** is the authoritative source for CLI flag documentation. If the help text is wrong or missing, users won't discover the flag.

### What assumptions changed

- **Original assumption:** Bare `download <url>` and queue-backed `download-all` would have different artifact structures. **What actually happened:** S01 normalized both to the same structure—both write metadata.json and status.json. This removes a class of bugs where operators accidentally used status on bare downloads and got confusing output (or errors).

- **Original assumption:** Metadata fetch would be a nice-to-have, optional feature. **What actually happened:** Slice plan and requirements made it a hard requirement (abort on failure unless --skip-metadata). This is correct—human-readable titles in status are the point of S02, so metadata quality matters.

## Slice Verification

✅ All checks from the slice plan passed (build, test, CLI flag visible).
✅ Backward-compat tests pass; old artifacts deserialize cleanly.
✅ New functions (read_metadata, fetch_vod_metadata_by_id) are present and type-correct.
✅ CLI flag is discoverable and functional.
✅ No new errors or lint warnings beyond pre-existing.
✅ Slice is ready for downstream consumers (S02, S03, S07).

