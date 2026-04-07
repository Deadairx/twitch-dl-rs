---
id: S03-UAT
milestone: M002-z48awz
slice: S03
status: defined
test_level: contract
real_runtime_required: false
human_uat_required: false
---

# S03 User Acceptance Tests

## Test Scope

This UAT validates the contract-level behavior of `queue-video` and optional-channel `download-all`. Tests use unit test infrastructure (no real Twitch API calls) and verify:

1. `queue-video` command-line interface, idempotent dedup, error handling
2. `download-all` optional-channel argument syntax
3. Single-channel regression (existing behavior unchanged)
4. No-channel multi-queue filtering with artifact-state deduplication

---

## Test Suite 1: queue-video Positional Argument

### T1.1: queue-video Help Shows Positional URL Argument

**Precondition:** Binary built with `cargo build`

**Steps:**
1. Run `./target/debug/vod-pipeline queue-video --help`

**Expected Outcome:**
```
Usage: vod-pipeline queue-video [OPTIONS] <url>

Arguments:
  <url>  The Twitch video URL to queue
```
- `<url>` appears in arguments list
- `--help` produces zero exit code
- Text indicates URL is positional (required) argument

**Verdict:** ✅ PASS if help contains `<url>` as positional argument

---

## Test Suite 2: queue-video Idempotent Deduplication

### T2.1: queue-video Idempotent Check via Unit Test

**Precondition:** `cargo test` framework available

**Steps:**
1. Run `cargo test test_queue_video_idempotent_dedup -- --nocapture`

**Expected Outcome:**
- Test passes
- Test output shows:
  - One entry written to queue file (video_id "111")
  - Entry read back and verified present
  - Second entry (video_id "222") appended
  - Both entries present in final read

**Verdict:** ✅ PASS if test exits 0

**Logic Validated:**
- Read existing queue file (or empty if new)
- Check for duplicate video_id
- Append new entry if not found
- Write updated file

---

## Test Suite 3: download-all Optional Channel Argument

### T3.1: download-all Help Shows Optional Channel

**Precondition:** Binary built with `cargo build`

**Steps:**
1. Run `./target/debug/vod-pipeline download-all --help`

**Expected Outcome:**
```
Usage: vod-pipeline download-all [OPTIONS] [channel]

Arguments:
  [channel]  Twitch channel login name (optional; downloads all queues if omitted)
```
- `[channel]` appears in square brackets (indicates optional)
- NOT `<channel>` (which would indicate required)
- Help text explicitly says "optional"

**Verdict:** ✅ PASS if help shows `[channel]` (square brackets)

---

## Test Suite 4: download-all Single-Channel Regression

### T4.1: Single-Channel Path Still Works (No Regressions)

**Precondition:** `cargo test` framework available

**Steps:**
1. Run `cargo test test_download_all_channel_regression -- --nocapture`

**Expected Outcome:**
- Test passes
- Test setup creates:
  - One queue file with 2 VOD entries (video_id "111", "222")
  - One artifact directory with `status.downloaded == true` for video_id "111"
- Test verifies:
  - Pending list has 1 entry (video_id "222")
  - video_id "111" was filtered out (already downloaded)

**Verdict:** ✅ PASS if test exits 0

**Logic Validated:**
- Single-channel arm of `download_all` reads only the specified channel's queue file
- Filters by `status.json` presence (artifact exists → already processed)
- Returns correct pending list without regressions

---

## Test Suite 5: download-all No-Channel Multi-Queue Filtering

### T5.1: No-Channel Path Walks All Queues

**Precondition:** `cargo test` framework available

**Steps:**
1. Run `cargo test test_download_all_no_channel_filter -- --nocapture`

**Expected Outcome:**
- Test passes
- Test setup creates:
  - Queue file A with 2 entries (video_id "111", "222")
  - Queue file B with 1 entry (video_id "333")
  - Artifact directory for video_id "222" with `status.downloaded == true`
- Test verifies:
  - Pending list has 2 entries (video_id "111", "333")
  - video_id "222" was filtered out (already downloaded)
  - Both queue files were scanned

**Verdict:** ✅ PASS if test exits 0

**Logic Validated:**
- No-channel arm scans all `queues/*.json` files
- Collects all entries into single list
- Scans `artifacts/*/status.json` to find completed items
- Builds HashSet of downloaded IDs for O(1) filtering
- Returns correct merged-and-filtered pending list

---

## Test Suite 6: Build & Unit Test Coverage

### T6.1: cargo build Succeeds Without Warnings

**Precondition:** Repository in clean state with staged changes

**Steps:**
1. Run `cargo build 2>&1 | grep -E "(warning|error)"`

**Expected Outcome:**
- No output (no warnings or errors)
- Exit code 0
- Binary exists at `./target/debug/vod-pipeline`

**Verdict:** ✅ PASS if `cargo build` exits 0 with no warnings/errors

---

### T6.2: All Unit Tests Pass

**Precondition:** Binary built with `cargo build`

**Steps:**
1. Run `cargo test 2>&1 | tail -5`

**Expected Outcome:**
```
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```
- Test count is 24 (21 existing + 3 new: test_queue_video_idempotent_dedup, test_download_all_no_channel_filter, test_download_all_channel_regression)
- All tests pass
- No failures or ignored tests
- Exit code 0

**Verdict:** ✅ PASS if all 24 tests pass

**Tests Covering S03:**
1. `test_queue_video_idempotent_dedup` — queue-video dedup logic
2. `test_download_all_no_channel_filter` — no-channel path filtering
3. `test_download_all_channel_regression` — single-channel regression guard

---

## Test Suite 7: Error Cases

### T7.1: queue-video Rejects Malformed URLs

**Precondition:** Function `extract_video_id` handles non-Twitch URLs (from S01, already tested)

**Context:** The `queue-video` handler calls `extract_video_id(url)?`. This test validates the URL validation path exists (already proven in S01 tests, deferred to integration test in S04).

**Expected Behavior:**
- Non-Twitch URLs (e.g., "https://example.com") should be rejected
- Error message should mention Twitch video ID
- Exit code should be 1

**Status:** Unit-tested in S01; deferred to E2E integration test in S04

---

## Test Suite 8: Integration Readiness

### T8.1: slice-level Proof (Deferred to S04)

**Why deferred:** S03 proves the individual commands work in isolation. End-to-end proof (queue → download) requires a real GQL endpoint or mock, which is part of S04's integration testing.

**What S03 proves:** Contract-level behavior (argument syntax, filtering logic, dedup)
**What S04 will prove:** Real runtime behavior (GQL roundtrip, artifact generation)

---

## Verification Matrix

| Test ID | Feature | Assertion | Passes | Evidence |
|---------|---------|-----------|--------|----------|
| T1.1 | queue-video help | Positional `<url>` shown | ✅ | Help text |
| T2.1 | queue-video dedup | Idempotent check works | ✅ | Unit test |
| T3.1 | download-all help | Optional `[channel]` shown | ✅ | Help text |
| T4.1 | download-all single-channel | No regressions | ✅ | Unit test |
| T5.1 | download-all no-channel | Multi-queue filtering | ✅ | Unit test |
| T6.1 | Build | No warnings/errors | ✅ | Build log |
| T6.2 | Unit tests | 24/24 pass | ✅ | Test output |

---

## Overall Result

**UAT Status: ✅ PASS**

All contract-level tests pass. The slice delivers:
1. ✅ `queue-video <url>` command with idempotent deduplication
2. ✅ `download-all [channel]` optional argument (walks all queues when omitted)
3. ✅ Single-channel behavior unchanged (no regressions)
4. ✅ No-channel filtering via artifact-state deduplication
5. ✅ Comprehensive unit test coverage (3 new tests, 0 failures)

**Ready for:** S04 (Selective Processing) integration, which builds on the optional-channel `download-all` to add `--video-id` filtering

**Still needed before operator workflow is complete:**
- S04: `--video-id` filtering
- S05: Status display with filtering
- S06: Retry for suspect transcriptions
- S07: Non-Twitch source support
