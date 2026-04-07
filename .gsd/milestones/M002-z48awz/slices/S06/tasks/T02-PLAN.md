---
estimated_steps: 42
estimated_files: 2
skills_used: []
---

# T02: Add blocking exclusive file lock to write_status via fs4

Add `fs4` as a dependency in `Cargo.toml` and wrap `write_status()` in `artifact.rs` with a blocking exclusive lock on a `status.lock` file, then add a concurrent-write unit test.

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

## Inputs

- ``Cargo.toml` — dependencies section`
- ``src/artifact.rs` — write_status() function and test module`

## Expected Output

- ``Cargo.toml` — fs4 dependency added under [dependencies]`
- ``src/artifact.rs` — write_status() wrapped with blocking exclusive lock on status.lock; concurrent-write unit test added`
- ``Cargo.lock` — updated with fs4 and its transitive dependencies`

## Verification

cargo test 2>&1 | grep -E 'result|FAILED'; cargo build 2>&1 | grep error; test -f Cargo.lock && grep -A1 'name = "fs4"' Cargo.lock | head -3
