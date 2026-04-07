# S06 Research: Retry And Operational Hardening

**Slice:** S06 — Retry And Operational Hardening  
**Milestone:** M002-z48awz — Workflow Polish  
**Depth:** Targeted (known patterns, known codebase, one library decision to make)

---

## Summary

S06 has two independent changes: (1) add `--force-suspect` to `transcribe-all`, and (2) add a blocking exclusive lock to `write_status` in `artifact.rs`. Both are well-scoped. No risky integration, no architectural ambiguity. The code is small and the patterns are already established.

**Recommended approach:** `fs4` for file locking (see Crate Decision below). Two tasks: T01 for `--force-suspect`, T02 for file locking.

---

## Recommendation

### Task split

**T01 — Force-suspect retry flag**
- Add `force_suspect: bool` field to `CliCommand::TranscribeAll` in `cli.rs`
- Wire the new `--force-suspect` clap arg (same pattern as `--continue-on-error`, `--video-id`)
- Thread `force_suspect` through dispatch in `main.rs` → `transcribe_all()`
- In `transcribe_all()`: expand the filter predicate to include suspect items when `force_suspect=true`
- In `transcribe_artifact()`: remove the early-exit guard (`srt_path.exists() && vtt_path.exists() && status.transcribed`) for suspect items — suspect items have `transcribed=false` already so the guard won't fire, but must verify the reuse-check condition handles the force path cleanly (see Seam 3 below)
- Update `status.json` via existing `write_status` call (already in place)
- Add unit test in `artifact.rs` tests that verifies suspect items appear in pending vec when `force_suspect=true` and are excluded when `false`

**T02 — Blocking file lock on write_status**
- Add `fs4 = { version = "0.13", features = ["sync"] }` to `Cargo.toml`
- In `artifact.rs`, `write_status()`: open (or create) a separate lock file at `<artifact_dir>/status.lock`, acquire an exclusive blocking lock with `lock_exclusive()`, perform the JSON write to `status.json`, then drop the lock file handle (releases automatically on Drop)
- The lock file is a separate file (`status.lock`) from the data file (`status.json`) — this is the standard pattern when writing a whole-file replacement (fs4/flock semantics lock the file handle itself, not the path; locking `status.json` directly while also truncating/overwriting it is safe but less obvious)
- No API change to `write_status` — same signature, same return type
- Add a unit test that spawns two threads, both calling `write_status` concurrently on the same artifact dir, and verifies both succeed with no corruption

---

## Implementation Landscape

### src/cli.rs — TranscribeAll variant

Current `TranscribeAll` struct:
```rust
TranscribeAll {
    output_root: PathBuf,
    continue_on_error: bool,
    video_id: Option<String>,
}
```

Add `force_suspect: bool` field. Mirror the `--continue-on-error` arg pattern exactly:
```rust
.arg(
    Arg::new("force-suspect")
        .long("force-suspect")
        .help("Re-transcribe suspect items (does not re-download)")
        .action(ArgAction::SetTrue),
)
```

Parse in the match arm:
```rust
force_suspect: transcribe_all_matches.get_flag("force-suspect"),
```

### src/main.rs — dispatch and transcribe_all()

Dispatch passes `force_suspect` through to `transcribe_all()`:
```rust
cli::CliCommand::TranscribeAll { output_root, continue_on_error, video_id, force_suspect } => {
    if let Err(error) = transcribe_all(&output_root, continue_on_error, video_id.as_deref(), force_suspect).await {
```

`transcribe_all()` signature change:
```rust
async fn transcribe_all(
    output_root: &std::path::Path,
    continue_on_error: bool,
    video_id: Option<&str>,
    force_suspect: bool,
) -> Result<(), Box<dyn std::error::Error>>
```

Current filter predicate (lines ~495-502):
```rust
let pending: Vec<_> = items
    .into_iter()
    .filter_map(|(vid, status)| {
        let s = status?;
        if s.downloaded
            && !s.transcribed
            && s.transcription_outcome.as_deref() != Some("suspect")
        {
            Some((vid, s))
        } else {
            None
        }
    })
    .collect();
```

New predicate with `force_suspect`:
```rust
let pending: Vec<_> = items
    .into_iter()
    .filter_map(|(vid, status)| {
        let s = status?;
        let is_suspect = s.transcription_outcome.as_deref() == Some("suspect");
        let include = s.downloaded && (
            (!s.transcribed && !is_suspect) ||
            (force_suspect && is_suspect)
        );
        if include { Some((vid, s)) } else { None }
    })
    .collect();
```

### src/main.rs — transcribe_artifact() reuse guard

Current guard in `transcribe_artifact()`:
```rust
let srt_path = artifact_dir.join("transcript.srt");
let vtt_path = artifact_dir.join("transcript.vtt");
if srt_path.exists() && vtt_path.exists() && status.transcribed {
    println!("Reusing existing transcript for {}", video_id);
    return Ok(());
}
```

**Key observation:** suspect items have `status.transcribed = false`. So `srt_path.exists() && vtt_path.exists() && status.transcribed` will be `false` for suspect items — the guard won't trigger even without modification. However, suspect items _do_ have existing `.srt` and `.vtt` files from the prior run. `transcribe_to_srt_and_vtt()` already cleans these up with `fs::remove_file` at the top of the function. So the force-retry path is: filter includes suspect → `transcribe_artifact()` runs → old files removed → `hear` re-invoked → new outcome written. **No change to `transcribe_artifact()` is needed.**

### src/artifact.rs — write_status() with file lock

Current `write_status()`:
```rust
pub fn write_status(
    artifact_dir: &Path,
    status: &ProcessStatus,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let status_path = artifact_dir.join("status.json");
    let json = serde_json::to_string_pretty(status)?;
    fs::write(&status_path, format!("{json}\n"))?;
    Ok(status_path)
}
```

New `write_status()` with fs4 lock:
```rust
use fs4::fs_std::FileExt;
use std::fs::OpenOptions;

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

The lock is released when `lock_file` goes out of scope (RAII). No need to call `unlock()` explicitly.

---

## Crate Decision: fs4 over fs2

| Criterion | fs2 (0.4.3) | fs4 (0.13.1) |
|---|---|---|
| Last updated | ~8 years ago | ~11 months ago |
| Downloads (all-time) | ~50M | ~31M |
| Downloads (recent) | ~5.9M | ~7.1M (higher!) |
| libc dependency | yes | no (uses rustix) |
| Async support | no | yes (optional feature) |
| API surface needed | `lock_exclusive()` | `lock_exclusive()` via `sync` feature |
| Maintenance | stale | actively maintained |

**Choice: `fs4`** — more actively maintained, no libc dependency, higher recent downloads. The `sync` feature provides the same `FileExt` trait with `lock_exclusive()`. This needs to be documented in DECISIONS.md.

Cargo.toml addition:
```toml
fs4 = { version = "0.13", features = ["sync"] }
```

Import in `artifact.rs`:
```rust
use fs4::fs_std::FileExt;
```

---

## Seams and Natural Task Boundaries

**Seam 1: CLI + dispatch (cli.rs + dispatch block in main.rs)**
- Self-contained change to add flag and thread it through
- No risk; follows exact pattern of `--continue-on-error` and `--video-id`

**Seam 2: transcribe_all() filter predicate (main.rs)**
- Single predicate change; composable with existing `--video-id` filter via independent post-filter
- The two filters are applied in sequence (pending vec built → video-id filter applied), so force_suspect and video-id compose cleanly: `transcribe-all --force-suspect --video-id <id>` works without additional logic

**Seam 3: transcribe_artifact() reuse guard**
- No change needed (verified above: suspect items have `transcribed=false`, guard won't fire)
- `transcribe_to_srt_and_vtt()` already cleans stale SRT/VTT at entry — force-retry will correctly overwrite
- Risk: none

**Seam 4: write_status() lock (artifact.rs)**
- Isolated to one function, no signature change
- Lock file is separate (`status.lock`) — does not interfere with the JSON write
- Risk: fs4 is a new dependency, needs to resolve cleanly; `cargo add` dry-run confirmed v0.13.1 available

---

## Verification Plan

**T01 verification:**
- `cargo test` passes (all 31 existing + new force-suspect filter test)
- Manual: create a suspect artifact fixture, run `transcribe-all --force-suspect`, confirm status.json updated
- `transcribe-all` without `--force-suspect` still skips suspect items (regression)
- `transcribe-all --force-suspect --video-id <id>` targets specific suspect item only

**T02 verification:**
- `cargo test` passes (all existing + new concurrent-write test)
- `cargo build` succeeds clean
- `status.lock` file created alongside `status.json` in artifact dir after any write operation
- Concurrent write test: two threads write different status structs to same dir, both complete without panic or file corruption

---

## Files Changed

| File | Change |
|---|---|
| `Cargo.toml` | Add `fs4 = { version = "0.13", features = ["sync"] }` |
| `src/cli.rs` | Add `force_suspect: bool` to `TranscribeAll`; add `--force-suspect` clap arg; parse in match arm |
| `src/main.rs` | Thread `force_suspect` through dispatch; update `transcribe_all()` signature and filter predicate |
| `src/artifact.rs` | Add fs4 import; wrap `write_status()` with blocking exclusive lock on `status.lock` |

**No changes to:** `src/transcribe.rs`, `src/lib.rs`, `src/twitch.rs`, `src/downloader.rs`, `src/ffmpeg.rs`

---

## Known Constraints

- `flock(2)` on macOS is OS-level, not advisory-only — satisfies the constraint that locking must work on macOS
- `fs4::fs_std::FileExt::lock_exclusive()` is blocking (not try-lock) — matches the requirement that the waiting process simply waits without error
- `transcribe_artifact()` requires no changes — the existing `transcribed=false` on suspect items and `transcribe_to_srt_and_vtt()`'s stale-file cleanup handle force-retry correctly without special-casing
- The `--force-suspect` flag must only include suspect items, not `failed` items — the predicate handles this: `is_suspect` checks for `transcription_outcome == "suspect"` specifically
