# S05: Queue-Aware Filtering — UAT

**Milestone:** M002-z48awz
**Written:** 2026-04-07T04:16:54.549Z

# S05: Queue-Aware Filtering — User Acceptance Tests

## Preconditions

- Binary built: `./target/debug/vod-pipeline` is available and built from the current source
- Test environment: artifacts directory with mixed stage states (queued items in queue files, downloaded/suspect/failed artifacts in artifact directory)
- Expected state: At least 1 item in failed state, 1+ items in downloaded state, 0 items in queued state (based on current test environment)

## Test Cases

### TC01: Filter by Valid Stage — Failed Items

**Objective:** Operator filters status display to see only failed transcriptions

**Steps:**
1. Run: `./target/debug/vod-pipeline status --filter failed`
2. Observe the output

**Expected Outcome:**
- Exit code: 0
- Output contains table header (STAGE, TITLE, CHANNEL, DATE, OUTCOME, REASON)
- All rows have STAGE value = `failed`
- At least 2 rows visible (based on test data with 2 known failed items)
- Examples: "hear exited with status", "Transcription command exited"
- Footer line: "N item(s) total" where N ≥ 2

**Pass Criteria:** All rows show STAGE=`failed`, exit code is 0

---

### TC02: Filter by Valid Stage — Downloaded Items

**Objective:** Operator views all items in downloaded state (includes legacy imports)

**Steps:**
1. Run: `./target/debug/vod-pipeline status --filter downloaded`
2. Observe the output

**Expected Outcome:**
- Exit code: 0
- Output contains table header
- All rows have STAGE value = `downloaded`
- Multiple rows visible (test data includes legacy transcript imports)
- All OUTCOME values are likely "legacy" (legacy transcript imported...)
- Footer: "N item(s) total" where N ≥ 5

**Pass Criteria:** All rows show STAGE=`downloaded`, exit code is 0

---

### TC03: Filter by Valid Stage — Queued Items (No Match)

**Objective:** Operator filters for queued items when none exist (demonstrates not-found path)

**Steps:**
1. Run: `./target/debug/vod-pipeline status --filter queued`
2. Observe the output

**Expected Outcome:**
- Exit code: 0 (not an error)
- stdout: `No items matching filter 'queued'.`
- No error message to stderr
- No table displayed

**Pass Criteria:** Exact message "No items matching filter 'queued'.", exit code 0

---

### TC04: Filter by Valid Stage — Ready Items (No Match)

**Objective:** Operator filters for ready items when none exist (similar to TC03)

**Steps:**
1. Run: `./target/debug/vod-pipeline status --filter ready`
2. Observe the output

**Expected Outcome:**
- Exit code: 0
- stdout: `No items matching filter 'ready'.`
- No error message to stderr
- No table displayed

**Pass Criteria:** Exact message "No items matching filter 'ready'.", exit code 0

---

### TC05: Filter by Valid Stage — Suspect Items (Boundary Case)

**Objective:** Test filtering for suspect transcription state (May or may not have matches)

**Steps:**
1. Run: `./target/debug/vod-pipeline status --filter suspect`
2. Observe the output

**Expected Outcome:**
- Exit code: 0
- If matches exist: table with all rows STAGE=`suspect`
- If no matches: message "No items matching filter 'suspect'."

**Pass Criteria:** Exit code 0; output is either filtered table or not-found message (both valid)

---

### TC06: Invalid Filter Value — Typo

**Objective:** Operator makes a typo in filter value; system provides helpful error

**Steps:**
1. Run: `./target/debug/vod-pipeline status --filter typo`
2. Observe stderr and exit code

**Expected Outcome:**
- Exit code: 1 (non-zero, indicates error)
- stderr: `unknown filter 'typo'; valid values: queued, downloaded, suspect, failed, ready`
- No table displayed

**Pass Criteria:** Exit code 1, stderr contains "unknown filter 'typo'" and lists all valid values

---

### TC07: Invalid Filter Value — Unrecognized Stage Name

**Objective:** Test filtering by a stage name that doesn't exist in the model

**Steps:**
1. Run: `./target/debug/vod-pipeline status --filter transcribed`
2. Observe stderr and exit code

**Expected Outcome:**
- Exit code: 1
- stderr: `unknown filter 'transcribed'; valid values: queued, downloaded, suspect, failed, ready`

**Pass Criteria:** Exit code 1, valid values list in error message

---

### TC08: Case Sensitivity — Uppercase Stage

**Objective:** Verify filter values are case-sensitive (no automatic folding)

**Steps:**
1. Run: `./target/debug/vod-pipeline status --filter Failed`
2. Observe stderr and exit code

**Expected Outcome:**
- Exit code: 1 (not treated as valid "failed")
- stderr: `unknown filter 'Failed'; valid values: ...`

**Pass Criteria:** Exit code 1, uppercase variant is rejected as invalid

---

### TC09: Case Sensitivity — Mixed Case

**Objective:** Additional case-sensitivity test

**Steps:**
1. Run: `./target/debug/vod-pipeline status --filter Queued`
2. Observe stderr and exit code

**Expected Outcome:**
- Exit code: 1
- stderr: `unknown filter 'Queued'; valid values: ...`

**Pass Criteria:** Exit code 1, mixed-case variant is rejected

---

### TC10: Help Text — Flag Documentation

**Objective:** Verify --filter flag is documented in help

**Steps:**
1. Run: `./target/debug/vod-pipeline status --help`
2. Search output for "--filter"

**Expected Outcome:**
- help output includes: `--filter <STAGE>` with description mentioning valid stages
- Example: "Show only items in the given stage: queued, downloaded, suspect, failed, ready"

**Pass Criteria:** Help text includes --filter flag and list of valid stages

---

### TC11: No Filter Flag (Backward Compatibility)

**Objective:** Verify existing `status` command behavior is preserved when no --filter is provided

**Steps:**
1. Run: `./target/debug/vod-pipeline status`
2. Observe output

**Expected Outcome:**
- Exit code: 0
- Output displays merged table of all items (queued + artifacts) with all stages
- Multiple rows visible (both failed, downloaded, etc.)
- No filter applied, so all stages represented

**Pass Criteria:** All items displayed, no filtering applied, backward-compatible behavior

---

### TC12: Help for Status Subcommand

**Objective:** Verify status subcommand shows --filter in its own help

**Steps:**
1. Run: `./target/debug/vod-pipeline status -h`
2. Search for "--filter"

**Expected Outcome:**
- Same documentation as TC10 (--filter flag with description)

**Pass Criteria:** --filter flag documented in status subcommand help

---

### TC13: Filter Matches All Items (Edge Case)

**Objective:** Filter applies but matches all items (no reduction)

**Steps:**
1. Run: `./target/debug/vod-pipeline status --filter downloaded` (assuming many downloaded items exist)
2. Count rows in output
3. Run `./target/debug/vod-pipeline status` (no filter)
4. Count rows in output
5. Note total lines (compare counts)

**Expected Outcome:**
- Filtered output shows fewer or equal rows
- If all items are in downloaded state, counts would be equal
- Footer "N item(s) total" reflects the filtered set size

**Pass Criteria:** Filtered count ≤ unfiltered count; counts are consistent

---

## Summary

**Total Test Cases:** 13
- **Positive Cases (filtering works):** TC01, TC02, TC05, TC11 (4 cases)
- **Not-Found Cases (valid filter, no matches):** TC03, TC04 (2 cases)
- **Invalid Input Cases (error handling):** TC06, TC07, TC08, TC09 (4 cases)
- **Documentation Cases (help/backward-compat):** TC10, TC12, TC13 (3 cases)

**Success Criteria:** All 13 test cases pass with expected exit codes, output messages, and filtering behavior.
