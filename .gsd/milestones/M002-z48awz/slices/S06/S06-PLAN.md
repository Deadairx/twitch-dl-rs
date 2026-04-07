# S06: Retry And Operational Hardening

**Goal:** Add `--force-suspect` to `transcribe-all` so suspect transcriptions can be retried without re-downloading, and add blocking file locking to `status.json` writes so parallel `download-all` and `transcribe-all` invocations do not corrupt artifact state.
**Demo:** After this: After this: run transcribe-all --force-suspect and watch a suspect item re-transcribe and update its outcome in status.

## Tasks
- [x] **T01: Added --force-suspect flag to transcribe-all command with updated filter predicate logic** — Add `force_suspect: bool` to the `TranscribeAll` CLI variant, wire the `--force-suspect` clap arg, thread the flag through dispatch into `transcribe_all()`, update the filter predicate to include suspect items when `force_suspect=true`, and add a unit test verifying the filter behavior.

The executor should follow the exact pattern established by `--continue-on-error` and `--video-id` — all three are bool/Option fields on `TranscribeAll`, clap args using `ArgAction::SetTrue` (for the bool) or value args (for Option<String>), and the flag is destructured in the dispatch match arm.

**No changes to `transcribe_artifact()` are needed.** Suspect items have `status.transcribed = false`, so the existing reuse guard (`srt_path.exists() && vtt_path.exists() && status.transcribed`) evaluates to `false` for suspect items and does not block re-transcription. `transcribe_to_srt_and_vtt()` already cleans up stale SRT/VTT at entry.

The filter predicate in `transcribe_all()` currently is:
```rust
if s.downloaded && !s.transcribed && s.transcription_outcome.as_deref() != Some("suspect")
```
Replace with:
```rust
let is_suspect = s.transcription_outcome.as_deref() == Some("suspect");
let include = s.downloaded && ((!s.transcribed && !is_suspect) || (force_suspect && is_suspect));
if include
```
This correctly composes with the `--video-id` post-filter (which applies to `pending` after it is built) without any additional logic.

Add a unit test in `src/lib.rs` (the project's test home, following the lib.rs re-export pattern established in M001/S03) that:
1. Creates three mock `(vid, ProcessStatus)` entries: one normal pending (downloaded=true, transcribed=false, no outcome), one suspect (downloaded=true, transcribed=false, outcome="suspect"), one completed (downloaded=true, transcribed=true)
2. Applies the filter predicate with `force_suspect=false` and asserts only the normal pending item passes
3. Applies the filter with `force_suspect=true` and asserts both the normal pending item and the suspect item pass (completed still excluded)

The test should exercise the predicate logic directly (not call `transcribe_all` which is async and touches the filesystem).
  - Estimate: 45m
  - Files: src/cli.rs, src/main.rs, src/lib.rs
  - Verify: cargo test 2>&1 | grep -E 'result|FAILED'; cargo build 2>&1 | grep -E 'error|warning.*unused'
- [x] **T02: Added fs4-based blocking exclusive file lock to write_status preventing concurrent corruption of status.json** — Add `fs4` as a dependency in `Cargo.toml` and wrap `write_status()` in `artifact.rs` with a blocking exclusive lock on a `status.lock` file, then add a concurrent-write unit test.

**Cargo.toml change:**
```toml
fs4 = { version = "0.13", features = ["sync"] }
```
Add under `[dependencies]`.

**Import in artifact.rs:**
```rust
use fs4::fs_std::FileExt;
use std::fs::OpenOptions;
```

**New write_status():**
```rust
pub fn write_status(
    artifact_dir: &Path,
    status: &ProcessStatus,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let lock_path = artifact_dir.join("status.lock");
    let lock_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&lock_path)?;
    lock_file.lock_exclusive()?;  // blocking — waits until acquired

    let status_path = artifact_dir.join("status.json");
    let json = serde_json::to_string_pretty(status)?;
    let result = fs::write(&status_path, format!("{json}\n"));
    // lock_file drops here, releasing the lock automatically
    result?;
    Ok(status_path)
}
```
The lock is released via RAII when `lock_file` drops at function return. No explicit `unlock()` call needed.

**No API change**: signature and return type remain identical. All existing callers (e.g. `write_status(&artifact_dir, &status).unwrap()` in tests) work without modification.

**Unit test** (add to the test module in `src/artifact.rs`):
- Spawn two `std::thread::spawn` threads, both calling `write_status` on the same temp artifact dir
- Thread 1 writes a status with `downloaded=true, transcribed=false`
- Thread 2 writes a status with `downloaded=true, transcribed=true`
- Join both threads; assert both `unwrap()` without panic
- Read back `status.json` and assert it deserializes successfully (no corruption — the file is valid JSON)
- The test proves both writes complete and the final file is coherent JSON; it does not assert which write won (last writer wins is correct behavior)

Document the fs4 crate choice in DECISIONS.md via the gsd_decision_save tool after the implementation is complete.
  - Estimate: 30m
  - Files: Cargo.toml, src/artifact.rs
  - Verify: cargo test 2>&1 | grep -E 'result|FAILED'; cargo build 2>&1 | grep error; test -f Cargo.lock && grep -A1 'name = "fs4"' Cargo.lock | head -3
