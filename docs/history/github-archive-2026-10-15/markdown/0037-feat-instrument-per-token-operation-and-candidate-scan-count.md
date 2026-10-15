# #37 — feat: instrument per-token operation and candidate-scan counts (closes #14)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-21T22:21:04Z
- **Updated:** 2026-10-09T14:17:03Z
- **Closed:** 2026-07-21T22:54:55Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/37
- **Merged:** 2026-07-21T22:54:55Z
- **Merge commit:** 4040cb87156a0bccb7f37f17f76dd434010e1b98
- **Base / Head:** main / UOR-Foundation:feature/issue-14-candidate-scan-instrumentation
- **Reviews / inline review comments:** 4 / 4
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Implements Issue #14: Extends OpKernel census with candidate-scan counts, instruments runtime prediction loops, updates tless_uor serialization, server JSON output, and allocation census tests.

## Timeline (11 entries; chronological: conversation comments, reviews, review comments)

### comment 5039617920 — 2026-07-21T22:21:26Z — Casey-allard

@github-copilot review

### review 4749477862 — 2026-07-21T22:23:41Z — chatgpt-codex-connector[bot] — COMMENTED


### 💡 Codex Review

Here are some automated review suggestions for this pull request.

**Reviewed commit:** `53ed0ef4c9`
    

<details> <summary>ℹ️ About Codex in GitHub</summary>
<br/>

[Your team has set up Codex to review pull requests in this repo](https://chatgpt.com/codex/cloud/settings/general). Reviews are triggered when you
- Open a pull request for review
- Mark a draft as ready
- Comment "@codex review".

If Codex has suggestions, it will comment; otherwise it will react with 👍.




Codex can also answer questions or update the PR. Try commenting "@codex address that feedback".
            
</details>

### review-comment 3626111252 — 2026-07-21T22:23:41Z — chatgpt-codex-connector[bot]

_on src/server.rs:940_

**<sub><sub>![P1 Badge](https://img.shields.io/badge/P1-orange?style=flat)</sub></sub>  Size the server census buffer for the new counter**

The server still creates `out` as `[0u8; 36]` a few lines above, but this patch makes `TlessAxisImpl::predict` require `TLESS_OUTPUT_BYTES == 37` and serialize `candidate_scans` into bytes `33..37`. On any successful `/api/tless/predict` path, the ignored predict call now returns `Err` for the short buffer, and this added `census(31)` then slices `out[33..37]` past the 36-byte array, so the handler panics instead of returning the JSON response. Please size this buffer from `TLESS_OUTPUT_BYTES` (or at least 37) before reading the new counter.

Useful? React with 👍 / 👎.

### comment 5039656093 — 2026-07-21T22:26:20Z — Casey-allard

Fixed in commit 1c4a0d4: sized the `out` buffer in `src/server.rs` to 40 bytes (`tless_uor::TlessAxisImpl::MAX_OUTPUT_BYTES`) so `predict()` receives a sufficiently sized buffer and `census(31)` (`out[33..37]`) reads valid memory without panicking.

### review-comment 3626126723 — 2026-07-21T22:27:20Z — Copilot

_on src/server.rs:941_

Adding `candidate_scans` to the JSON census makes the server rely on bytes 33..37 from the axis output, but the axis output buffer earlier in this handler is still `[0u8; 36]` and the `predict` result is ignored. With `TLESS_OUTPUT_BYTES = 37`, this can lead to a panic when computing `census(31)`. Update the earlier buffer to at least `tless_uor::TLESS_OUTPUT_BYTES` (or `TlessAxisImpl::MAX_OUTPUT_BYTES`) and handle `TlessAxisImpl::predict` errors before parsing.

### review 4749494209 — 2026-07-21T22:27:21Z — copilot-pull-request-reviewer[bot] — COMMENTED

## Pull request overview

This PR implements Issue #14 by extending the transformerless runtime’s `OpKernel` census with a per-candidate “scan” counter, then threads that new counter through the TLESS axis serialization, server JSON output, and allocation census tests.

**Changes:**
- Add `candidate_scans` to `OpKernel` and increment it in runtime loops that iterate over candidate classes/tokens.
- Extend TLESS predict output serialization and field partition metadata to include `candidate_scans` (output size increases to 37 bytes).
- Update server JSON output and allocation census test code to surface/track the new counter and updated token types/signatures.

### Reviewed changes

Copilot reviewed 4 out of 4 changed files in this pull request and generated 1 comment.

| File | Description |
| ---- | ----------- |
| `crates/uor-r4-core/src/transformerless/runtime.rs` | Adds the `candidate_scans` counter and instruments candidate-iteration points to increment it. |
| `src/tless_uor.rs` | Extends axis output record/shape to serialize `candidate_scans` and bumps output sizing constants. |
| `src/server.rs` | Adds `candidate_scans` to the JSON census payload. |
| `crates/uor-r4-core/tests/allocation_census.rs` | Updates op-census snapshot/test plumbing for the new counter and updated evidence/token types. |




<details>
<summary>Comments suppressed due to low confidence (1)</summary>

**src/tless_uor.rs:418**
* The `predict` doc comment still describes a 33-byte output and doesn’t mention the new `candidate_scans` counter, but the implementation now writes 37 bytes including that field. Update the comment so it matches the serialized layout.
```
    /// output (33 bytes, big-endian fields): token u32 | depth u8 |
    /// code [u8; 4] | count u32 | adds | xors | shifts | compares |
    /// table_reads (u32 each). No multiply field exists, by design.
```
</details>



---

💡 <a href="/UOR-Foundation/uor-r4/new/main?filename=.github/instructions/*.instructions.md" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Add Copilot custom instructions</a> for smarter, more guided reviews. <a href="https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Learn how to get started</a>.

### review-comment 3626148965 — 2026-07-21T22:31:55Z — Copilot

_on src/tless_uor.rs:575_

`PartitionProductFields::FIELDS` offsets don’t match the actual byte layout written in `predict` (e.g., `candidate_scans` is serialized at bytes 33..37, but the field table entry starts at 31). Other shapes in the repo (e.g. `crates/uor-r4-router/src/lib.rs` `R4RoutingOutput`) use byte offsets, so this table likely causes consumers that rely on field partitioning to read incorrect data.

### review 4749518960 — 2026-07-21T22:31:56Z — copilot-pull-request-reviewer[bot] — COMMENTED

## Pull request overview

Copilot reviewed 4 out of 4 changed files in this pull request and generated 1 comment.




<details>
<summary>Comments suppressed due to low confidence (1)</summary>

**src/tless_uor.rs:418**
* The output layout comment is now stale: the record is 37 bytes and includes the new `candidate_scans` u32 counter. Keeping this comment accurate is important because it documents the on-wire format for axis consumers.
```
    /// input: WINDOW u32 token ids, little-endian, oldest first.
    /// output (33 bytes, big-endian fields): token u32 | depth u8 |
    /// code [u8; 4] | count u32 | adds | xors | shifts | compares |
    /// table_reads (u32 each). No multiply field exists, by design.
```
</details>



### review 4749531721 — 2026-07-21T22:34:20Z — copilot-swe-agent[bot] — COMMENTED

### review-comment 3626159901 — 2026-07-21T22:34:20Z — Copilot

_on src/server.rs:941 · in reply to review-comment 3626126723_

Fixed in commit 1eda821. Updated the `out` buffer to use `tless_uor::TLESS_OUTPUT_BYTES` (37 bytes) and replaced `let _ = TlessAxisImpl::predict(...)` with proper error handling that returns a 500 response on failure before any parsing of `out` occurs.

### comment 5039859330 — 2026-07-21T22:47:58Z — Casey-allard

Fixed in commit d3400d4: corrected all field offsets in `PartitionProductFields for TlessPredictOutput` (`src/tless_uor.rs`) to match the exact byte boundaries serialized in `predict()` (e.g. `candidate_scans` is now `(33, 4)`).


---
_Comment/review ids in this file: 5039617920, 5039656093, 5039859330, 4749477862, 4749494209, 4749518960, 4749531721, 3626111252, 3626126723, 3626148965, 3626159901_

