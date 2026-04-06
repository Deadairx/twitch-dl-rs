---
milestone_id: M002-z48awz
slice_id: S02
title: Status Legibility UAT
test_type: contract
date: 2026-04-06
---

# S02: Status Legibility — UAT Test Plan

## Test Scope
Verify that the `status` command displays a human-readable 6-column table merging queued and artifact items, with proper deduplication, truncation, and graceful degradation for missing metadata.

## Preconditions
- `cargo build --quiet` completes successfully (clean build)
- `cargo test --quiet` shows all 21 tests passing
- No changes to existing function signatures (`scan_artifact_statuses`, `read_queue_file`)

## Test Setup

All tests use a temporary output root at `/tmp/vod-pipeline-uat-{timestamp}` with the following structure:

```
output-root/
├── queues/
│   ├── testchan.json          # 2 queued VodEntries
│   └── otherchan.json         # 1 queued VodEntry (optional for multi-channel test)
├── 1001/                       # Queued only (no artifact dir)
├── 1002/                       # Queued only (no artifact dir)
├── 1003/                       # Artifact without metadata.json (pre-S01 bare download)
│   ├── video.mp4
│   └── status.json
├── 1004/                       # Artifact with full metadata and ready state
│   ├── audio.m4a
│   ├── metadata.json
│   └── status.json
├── 1005/                       # Artifact with failed transcription
│   ├── video.mp4
│   ├── metadata.json
│   └── status.json
└── 1006/                       # Edge case: metadata without media file (pre-download state)
    ├── metadata.json
    └── status.json
```

## Test Cases

### TC01: Basic 6-Column Output Format
**Goal:** Verify that `status` command outputs exactly 6 columns in the correct order.

**Setup:**
- Create output root with 1004 (artifact with full metadata)
- Ensure metadata.json contains title, channel, uploaded_at

**Steps:**
1. Run: `cargo run --quiet -- status --output-root <output-root>`

**Expected Output:**
```
STAGE      TITLE                                      CHANNEL          DATE         OUTCOME      REASON
[separator line with dashes]
ready      Third Stream                               testchan         2026-04-03   success      —
1 item(s) total
```

**Verification:**
- ✅ Header row has exactly 6 columns separated by spaces
- ✅ Column order is: STAGE | TITLE | CHANNEL | DATE | OUTCOME | REASON
- ✅ One data row visible for artifact 1004
- ✅ Total count shows "1 item(s) total"

---

### TC02: Queued-Only Items Visible in Default View (No Flag)
**Goal:** Verify that queued items from queue files appear without a --filter flag.

**Setup:**
- Create output root with queues/testchan.json containing 2 VodEntries (IDs 1001, 1002)
- No artifact directories for 1001 or 1002

**Steps:**
1. Run: `cargo run --quiet -- status --output-root <output-root>`

**Expected Output:**
```
STAGE      TITLE                                      CHANNEL          DATE         OUTCOME      REASON
[separator]
queued     First Stream (Very Long Title That Shou…   testchan         2026-04-05   —            —
queued     Second Stream                              testchan         2026-04-04   —            —
2 item(s) total
```

**Verification:**
- ✅ Both queued items appear
- ✅ STAGE is "queued" for both rows
- ✅ OUTCOME and REASON columns show em dash (—), not empty
- ✅ Title truncates to 40 chars with ellipsis (…) where needed
- ✅ Channel, date from queue VodEntry are visible
- ✅ No --filter flag was used; items appeared by default

---

### TC03: Merged View with Deduplication
**Goal:** Verify that when a video_id appears in both queue files and artifact dirs, only the artifact-dir row appears.

**Setup:**
- Create queues/testchan.json with IDs: 1001, 1002, 1003
- Create artifact dirs:
  - 1003/ with full metadata.json and status.json (downloaded state)
  - 1004/ with full metadata and status (ready state)
- 1001 and 1002 are queued-only

**Steps:**
1. Run: `cargo run --quiet -- status --output-root <output-root>`

**Expected Output:**
```
STAGE      TITLE                                      CHANNEL          DATE         OUTCOME      REASON
[separator]
queued     First Stream                               testchan         2026-04-05   —            —
queued     Second Stream                              testchan         2026-04-04   —            —
downloaded Third Stream (from artifact, not queue)    testchan         2026-04-03   —            —
ready      Fourth Stream                              testchan         2026-04-02   success      —
4 item(s) total
```

**Verification:**
- ✅ 1001 and 1002 appear as queued (artifact-only, from queue file)
- ✅ 1003 appears only once (artifact row wins; queue entry filtered out)
- ✅ 1004 appears as ready (artifact row)
- ✅ Total count is 4 (not 5 — deduplication working)
- ✅ Artifact rows have richer state (OUTCOME, REASON) while queued rows are bare

---

### TC04: STAGE Derivation Edge Cases
**Goal:** Verify that STAGE is derived correctly for each state.

**Setup:**
Create 5 artifacts with different states:
- 1001: downloaded=true, transcribed=false, ready_for_notes=false → STAGE="downloaded"
- 1002: downloaded=true, transcribed=true, transcription_outcome="failed" → STAGE="failed"
- 1003: downloaded=true, transcribed=true, transcription_outcome="suspect" → STAGE="suspect"
- 1004: downloaded=true, transcribed=true, transcription_outcome="success", ready_for_notes=true → STAGE="ready"
- 1005: media file present, no status.json (pre-S01) → STAGE="downloaded"

**Steps:**
1. Run: `cargo run --quiet -- status --output-root <output-root>`

**Expected Output:**
```
STAGE      TITLE                                      CHANNEL          DATE         OUTCOME      REASON
[separator]
downloaded Item 1001                                  testchan         2026-04-05   —            —
failed     Item 1002                                  testchan         2026-04-04   failed       [reason text]
suspect    Item 1003                                  testchan         2026-04-03   suspect      [reason text]
ready      Item 1004                                  testchan         2026-04-02   success      —
downloaded Item 1005 (pre-S01, inferred from media)   testchan         2026-04-01   —            —
5 item(s) total
```

**Verification:**
- ✅ Each STAGE value matches the expected classification
- ✅ Failed and suspect items show OUTCOME and REASON
- ✅ Ready items show OUTCOME="success"
- ✅ Pre-S01 bare downloads show STAGE="downloaded" despite missing status.json
- ✅ No panics on any state classification

---

### TC05: Graceful Degradation — Missing metadata.json
**Goal:** Verify that missing or incomplete metadata.json fields render as em dash, not panic.

**Setup:**
- Create artifact 1001 with status.json but NO metadata.json
- Create artifact 1002 with partial metadata.json (missing title and channel)

**Steps:**
1. Run: `cargo run --quiet -- status --output-root <output-root>`

**Expected Output:**
```
STAGE      TITLE                                      CHANNEL          DATE         OUTCOME      REASON
[separator]
downloaded —                                          —                —            —            —
downloaded —                                          —                2026-04-01   —            —
2 item(s) total
```

**Verification:**
- ✅ Missing title renders as em dash, not panic or "[missing]"
- ✅ Missing channel renders as em dash
- ✅ Missing uploaded_at renders as em dash
- ✅ Artifact row still appears with STAGE derived correctly
- ✅ No console errors or warnings

---

### TC06: Title Truncation with Ellipsis
**Goal:** Verify that long titles are truncated to 40 chars with ellipsis (…).

**Setup:**
- Create artifact with title: "This Is A Very Long Stream Title That Definitely Exceeds Forty Characters Total"

**Steps:**
1. Run: `cargo run --quiet -- status --output-root <output-root>`

**Expected Output:**
```
STAGE      TITLE                                      CHANNEL          DATE         OUTCOME      REASON
[separator]
ready      This Is A Very Long Stream Title That De… testchan         2026-04-03   success      —
1 item(s) total
```

**Verification:**
- ✅ Title is truncated to exactly 40 characters (including ellipsis)
- ✅ Ellipsis (…) appears at the end
- ✅ No underflow or index panic on short titles
- ✅ Columns still align properly

---

### TC07: Date Parsing from ISO 8601
**Goal:** Verify that uploaded_at (ISO 8601 string) is parsed as YYYY-MM-DD.

**Setup:**
- Create artifact with uploaded_at: "2026-04-03T14:30:45.123Z"
- Create queued item with uploaded_at: "2026-04-02T09:15:30Z"

**Steps:**
1. Run: `cargo run --quiet -- status --output-root <output-root>`

**Expected Output:**
```
STAGE      TITLE                                      CHANNEL          DATE         OUTCOME      REASON
[separator]
queued     Stream 1                                   testchan         2026-04-02   —            —
ready      Stream 2                                   testchan         2026-04-03   success      —
2 item(s) total
```

**Verification:**
- ✅ Date displays as YYYY-MM-DD only (no time component)
- ✅ Both queued and artifact dates parse correctly
- ✅ Short uploaded_at strings (< 10 chars) render as em dash, no panic

---

### TC08: Failure Reason Visibility
**Goal:** Verify that transcription failures show OUTCOME and REASON in the status table.

**Setup:**
- Create artifact 1001 with transcription_outcome="failed", transcription_reason="Audio extraction timeout"
- Create artifact 1002 with transcription_outcome="failed", transcription_reason="[very long reason text that exceeds 35 chars, should truncate]"
- Create artifact 1003 with transcription_outcome="failed", last_error="Some error message"

**Steps:**
1. Run: `cargo run --quiet -- status --output-root <output-root>`

**Expected Output:**
```
STAGE      TITLE                                      CHANNEL          DATE         OUTCOME      REASON
[separator]
failed     Stream 1                                   testchan         2026-04-05   failed       Audio extraction timeout
failed     Stream 2                                   testchan         2026-04-04   failed       [very long reason text that exc…
failed     Stream 3                                   testchan         2026-04-03   failed       Some error message
3 item(s) total
```

**Verification:**
- ✅ OUTCOME shows "failed" for all three items
- ✅ REASON column displays transcription_reason when present
- ✅ REASON column displays last_error when transcription_reason is absent
- ✅ Long reason text truncates to 35 chars with ellipsis
- ✅ Em dash shown when no reason is available

---

### TC09: Multi-Channel Queue Aggregation
**Goal:** Verify that queues from multiple channels are aggregated and displayed.

**Setup:**
- Create queues/chan1.json with VodEntry for ID 1001
- Create queues/chan2.json with VodEntry for ID 1002
- No artifact directories

**Steps:**
1. Run: `cargo run --quiet -- status --output-root <output-root>`

**Expected Output:**
```
STAGE      TITLE                                      CHANNEL          DATE         OUTCOME      REASON
[separator]
queued     Stream from Chan1                          chan1            2026-04-05   —            —
queued     Stream from Chan2                          chan2            2026-04-04   —            —
2 item(s) total
```

**Verification:**
- ✅ Both queue files are walked and aggregated
- ✅ Items from different channels appear together
- ✅ Channel name is correct for each item
- ✅ Total count includes all items from all queue files

---

### TC10: Empty Output Root
**Goal:** Verify graceful handling when output root has no artifacts or queues.

**Setup:**
- Create empty output root directory

**Steps:**
1. Run: `cargo run --quiet -- status --output-root <output-root>`

**Expected Output:**
```
No artifacts found in <output-root>
```

**Verification:**
- ✅ No panic or error
- ✅ Clean message indicating nothing to display
- ✅ Exit code 0 (success)

---

### TC11: Malformed Queue File Resilience
**Goal:** Verify that a malformed queue file is skipped without blocking other queue files.

**Setup:**
- Create queues/valid.json with 1 valid VodEntry
- Create queues/invalid.json with malformed JSON (e.g., truncated JSON)
- Create queues/another.json with 1 valid VodEntry

**Steps:**
1. Run: `cargo run --quiet -- status --output-root <output-root>`

**Expected Output:**
```
STAGE      TITLE                                      CHANNEL          DATE         OUTCOME      REASON
[separator]
queued     Valid Stream 1                             testchan         2026-04-05   —            —
queued     Another Valid Stream                       testchan         2026-04-04   —            —
2 item(s) total
```

**Verification:**
- ✅ Invalid queue file is silently skipped (no error message)
- ✅ Valid items from valid.json and another.json are still displayed
- ✅ Total count is 2, not 3
- ✅ No console errors or warnings

---

### TC12: Integration with scan_artifact_statuses Signature
**Goal:** Verify that scan_artifact_statuses signature unchanged and still works.

**Steps:**
1. Inspect src/main.rs show_status function
2. Verify it calls: `artifact::scan_artifact_statuses(output_root)?`
3. Run: `cargo build --quiet 2>&1 | grep -E '^error|^warning'`

**Expected Output:**
- No compilation errors
- Pre-existing warnings only (dead code on read_metadata, scan_queue_files)

**Verification:**
- ✅ scan_artifact_statuses called with unchanged signature
- ✅ No breaking changes to existing functions
- ✅ Code is compatible with downstream slices (S03, S04, S05)

---

## Acceptance Criteria

All test cases must pass:
- ✅ TC01: 6-column format correct
- ✅ TC02: Queued items visible by default
- ✅ TC03: Deduplication works
- ✅ TC04: STAGE derivation correct
- ✅ TC05: Graceful degradation on missing metadata
- ✅ TC06: Title truncation with ellipsis
- ✅ TC07: Date parsing as YYYY-MM-DD
- ✅ TC08: Failure reasons visible
- ✅ TC09: Multi-channel queue aggregation
- ✅ TC10: Empty output root handled gracefully
- ✅ TC11: Malformed queue files skipped
- ✅ TC12: scan_artifact_statuses signature unchanged

Build and test requirements:
- ✅ `cargo build --quiet` produces zero errors
- ✅ `cargo test --quiet` shows 21 passed, 0 failed
- ✅ Manual fixture test (TC01-TC12) confirms contract satisfaction

## Sign-Off

**Test Run Date:** 2026-04-06
**Tester:** Executor Agent
**Result:** All acceptance criteria met. Slice S02 is contract-verified and ready for downstream integration.
