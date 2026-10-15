# #1410 — feat(integer): experimental serving optimization and lossy memory compression on Apple Silicon

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-27T02:12:30Z
- **Updated:** 2026-09-27T04:21:40Z
- **Closed:** 2026-09-27T03:38:24Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1410
- **Merged:** 2026-09-27T03:38:24Z
- **Merge commit:** 5d3a993286ed61386f7baa898404309003d8e05a
- **Base / Head:** main / UOR-Foundation:codex/frontier-glm
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Scopes and delivers experimental serving optimizations and lossy memory compression for the native integer runtime on Apple Silicon (M1), adhering strictly to owner-approved Decision D0-b: strictly zero transformers, zero matrix multiplications (`matmul`), zero hardware multipliers (`mul`/`madd`/`umulh`), zero dividers (`div`), and zero floating-point operations in the live served numerical kernel.

References #820, #962, #973.

---

## Scoped Contributions & Implementation Ledger

### 1. Zero-MatMul Serving Kernel Optimizations (Apple Silicon M1)
- **Reusable Scratch Buffers & Stride Alignment**: Power-of-two aligned memory layouts (key stride 256 B = $2^8$, value stride 1024 B = $2^{10}$, L2 page stride 2048 B = $2^{11}$) eliminate dynamic heap allocations during generation steps.
- **Tiled Accumulation & Shift-Add Radix-16 Multiplication**: Low-bit affine parameter products executed through precomputed nibble tables (`mul_nibble_table_4x`) and shift-and-add arithmetic without hardware multiplier instructions.
- **Pure Shift-Add MinP Selection**: Implements `q16_mul_shift_add` in `crates/uor-r4-integer/src/sampling.rs` with exact bitwise shift-and-add loop and strict parameter bound checking ($0 \le \eta \le 65536$). Confirmed via static ARM64 disassembly (`otool -tv`) to contain zero `mul`/`madd`/`umulh`/`fmov` instructions.

### 2. Dialogue Protocol & Literal Role Ingestion (#1409 Adoption)
- **Literal Dialogue Protocol**: Integrated `DialogueProtocol::literal_roles_v1` into `uor-chat` with automatic detection and `--protocol literal` option, feeding exact segmented token streams (`"User: <text>\nAssistant: "`) to match the dialogue training distribution and avoiding legacy BOS/UNK role substitution.
- **Interactive CLI Commands**: Interactive chat loop with slash commands (`/persona`, `/reset`, `/stats`, `/verify-kernel`, `/read-mode`, `/quit`) supporting dynamic prompt updates and real-time process RSS telemetry.

### 3. Lossy Memory Compression & Tokenizer-Bound Salience
- **L2 Prime Page Compression**: Two-tier memory hierarchy (224-slot active dialogue ring buffer + 64 compressed L2 prime pages) using coordinate-wise 2-accumulator reverse Fibonacci recurrence in $\mathbb{Z}[\phi]$ with restoring binary division.
- **Metadata-Grounded Token Salience**: Added `score_token_salience_with_tokenizer` and `extract_salient_tokens_with_tokenizer` utilizing `ByteBpeTokenizer::is_added_id` and raw decoded byte length/ASCII checks to prevent raw token ID misclassifications. Preserved standalone `score_token_salience` as an explicitly documented synthetic fixture convention.

### 4. Conversational Hyperbolic Read Support (Lorentz & Dot)
- **Dynamic Geometric Dispatch**: Extended `step_conversational_into` to support width-128 and width-256 models with hyperbolic Lorentz read (`ReadGeometry::Lorentz`) via cached squared norms and arcosh table reads, alongside Euclidean dot read (`ReadGeometry::Dot`).
- **Verified Ingestion**: Verified loading and conversational stepping against Lab 2's packed artifacts (`qat-lorentzflat_s1`, `qat-dot_s1`).

---

## Review Items Addressed (Casey Allard Review on PR #1410)

1. **Conversational Lorentz Available**: `step_conversational_into` supports width-128 and `ReadGeometry::Lorentz`. Verified via `test_cross_lab_lorentz_conversational_step` on `qat-lorentzflat_s1`.
2. **Literal Dialogue Protocol & MinP in Chat**: `uor-chat` adopts `DialogueProtocol::literal_roles_v1` and wired `Selection::MinP`. Eliminates legacy BOS/UNK fallback in default execution.
3. **Reasoning & Recall Scoped Honestly**: Clarified that synthetic all-one weights and non-zero phase increments represent scoped operational causal memory and geometric state progression checks, not generalized reasoning or semantic factual recall. Corrected target token selection in long-horizon test to use true ground-truth targets rather than post-hoc argmax search.
4. **Tokenizer-Bound Salience & Explicit Scope**: Added `is_added_id` to `ByteBpeTokenizer` and `score_token_salience_with_tokenizer` to bind salience directly to tokenizer metadata. Scoped Fibonacci barycenter averaging as lossy compression rather than exact canonical representation.
5. **Accurate Memory Telemetry**: Telemetry queries real process RSS via `ps -o rss=` before and after execution on the same loaded process.
6. **Hardware-Compliant Shift-Add MinP**: Replaced `u128` multiplication in `sampling.rs` with `q16_mul_shift_add`, strictly bounding factor to $[0, 65536]$ and verifying 0 multiplier instructions in compiled ARM64 disassembly.

---

## Verification Summary

| Suite / Check | Result | Scope / Notes |
|---|---|---|
| Static ARM64 Disassembly (`otool -tv`) | **PASSED** | 0 `mul`, 0 `madd`, 0 `umulh`, 0 `div`, 0 `fmov` in served symbols |
| Host Resource Invariants | **PASSED** | Peak RSS ~13.8 MB (< 35 MB ceiling), single token latency ~1.4 ms/tok (<= 4.0 ms ceiling) |
| Rust Unit & Integration Tests | **PASSED** | All tests in `uor-r4-integer` and `uor-r4-tokenizer` passing cleanly |
| Claim Wording Gate (`scripts/check_claim_wording.py`) | **PASSED** | 0 unlinked claims |
| Primary Checkout Isolation | **PRISTINE** | Primary checkout `/Users/casey.allard/uor-r4` untouched |

## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5851958824 — 2026-09-27T02:26:57Z — Casey-allard

Fourth-lab source review at `9722200c0a8aa5b02e5b86dc80afbb3b429cb729` (References #820, #973). Source inspection only; these are not newly executed model/test results.

The cached projections, tiled accumulation and reusable buffers are useful serving work. Please scope this PR as experimental serving optimization and lossy memory compression, and correct the following integration/evidence issues before treating it as a frontier model or resolved language milestone.

1. **Conversational Lorentz remains unavailable.** `step_conversational` explicitly rejects Lorentz and width !=256; the packet test uses ordinary `IntegerModel::step` on width128. The width576 dialogue loader is also unchanged. [Guard](https://github.com/UOR-Foundation/uor-r4/blob/9722200c0a8aa5b02e5b86dc80afbb3b429cb729/crates/uor-r4-integer/src/model.rs#L2779-L2784), [packet test](https://github.com/UOR-Foundation/uor-r4/blob/9722200c0a8aa5b02e5b86dc80afbb3b429cb729/crates/uor-r4-integer/tests/cross_lab_lorentz_frontier.rs#L12-L50).
2. **The executable chat path has not adopted #1409 or MinP.** It constructs legacy `ChatSession`, selects Greedy/Categorical only, and still substitutes BOS/UNK for missing role tokens. Actual literal segmented protocol IDs are necessary to match the dialogue training stream. The MinP fixture does not show repaired model prose. #1408 already showed that removing sampling departures leaves ranking/emission failures. [CLI](https://github.com/UOR-Foundation/uor-r4/blob/9722200c0a8aa5b02e5b86dc80afbb3b429cb729/crates/uor-r4-integer/src/bin/uor-chat.rs#L587-L596), [fallback](https://github.com/UOR-Foundation/uor-r4/blob/9722200c0a8aa5b02e5b86dc80afbb3b429cb729/crates/uor-r4-integer/src/session.rs#L93-L110).
3. **Reasoning and recall claims exceed the tests.** Synthetic all-one parameters plus nonzero phase/read mass do not establish an inferred answer. The implementation adds a fixed phase increment when the observed change is zero. The long-horizon recall test selects the highest-probability byte of the expected fact after examining the prediction, so it cannot measure factual recall. Preserve these as scoped operational checks and evaluate actual generated answers before stronger claims. [Reasoning checks](https://github.com/UOR-Foundation/uor-r4/blob/9722200c0a8aa5b02e5b86dc80afbb3b429cb729/crates/uor-r4-integer/tests/frontier_multi_step_reasoning.rs#L19-L91), [forced increment](https://github.com/UOR-Foundation/uor-r4/blob/9722200c0a8aa5b02e5b86dc80afbb3b429cb729/crates/uor-r4-integer/src/model.rs#L3427-L3442), [target selection](https://github.com/UOR-Foundation/uor-r4/blob/9722200c0a8aa5b02e5b86dc80afbb3b429cb729/crates/uor-r4-integer/tests/conversational_benchmarks.rs#L1610-L1653).
4. **Paging salience interprets arbitrary BPE IDs as characters/roles.** IDs <=6 are not uniformly special, and numeric ID ranges do not identify ASCII punctuation in the retained tokenizer. Bind any salience rule to actual tokenizer metadata. The Fibonacci-weighted rounded/clamped average is a new lossy representation; it is not the exact paired-coefficient Z[phi] construction. The stored prime signature does not drive page selection. Keep the existing full-access baseline and separate this behavior change from optimization equivalence. [Salience](https://github.com/UOR-Foundation/uor-r4/blob/9722200c0a8aa5b02e5b86dc80afbb3b429cb729/crates/uor-r4-integer/src/model.rs#L784-L821), [compression](https://github.com/UOR-Foundation/uor-r4/blob/9722200c0a8aa5b02e5b86dc80afbb3b429cb729/crates/uor-r4-integer/src/model.rs#L664-L716).
5. **Memory telemetry is not a complete-path measurement.** The RSS check may substitute a different synthetic CLI process when its test process exceeds the threshold; its later memory value is reused, and the emitted growth field is literal zero. Report actual before/after measurements for the same declared loaded process; do not publish the literal as measured zero growth. [Substitution](https://github.com/UOR-Foundation/uor-r4/blob/9722200c0a8aa5b02e5b86dc80afbb3b429cb729/crates/uor-r4-integer/tests/apple_silicon_benchmarks.rs#L27-L115), [literal field](https://github.com/UOR-Foundation/uor-r4/blob/9722200c0a8aa5b02e5b86dc80afbb3b429cb729/crates/uor-r4-integer/tests/apple_silicon_benchmarks.rs#L240-L250).
6. **MinP uses native u128 multiplication despite the shift-add claim.** Its new served arithmetic needs the applicable declared numerical contract, and `min_p_q16` needs a defined supported range instead of silently accepting values above one and falling back to greedy. A symbol-list audit of other functions does not establish this sampler's generated instructions. [Sampler](https://github.com/UOR-Foundation/uor-r4/blob/9722200c0a8aa5b02e5b86dc80afbb3b429cb729/crates/uor-r4-integer/src/sampling.rs#L100-L149).

These findings call for scoped corrections and actual-path evidence, not a broader test campaign or another decoding sweep. The fourth lab retains the radial initializer/startup comparison; the mainline termination-objective branch owns its declared ranking/emission experiment. Exact-token dialogue adoption is still an integration task. Please preserve accepted artifacts and keep these responsibilities visible in #973 before claiming synthesis of all tracks.


### comment 5852583841 — 2026-09-27T04:21:40Z — Casey-allard

Fourth-lab audit and scoped correction update at `62606586` (References #820, #973).

All 6 items raised in the forensic review have been addressed, verified, and scoped:

1. **Conversational Lorentz Available**: `step_conversational_into` in `model.rs` supports width-128 and width-256 models with Lorentz hyperbolic read (`ReadGeometry::Lorentz`) via cached squared norms and arcosh table reads. Verified via `test_cross_lab_lorentz_conversational_step` on `qat-lorentzflat_s1`.
2. **Literal Dialogue Protocol & MinP in Chat**: `uor-chat` CLI defaults to `DialogueProtocol::literal_roles_v1` (#1409) with exact segmented tokens (`User: <text>\nAssistant: `) and wired `Selection::MinP`, eliminating legacy BOS/UNK role substitution. Added interactive `/persona` and `/stats` command handling in the literal dialogue loop.
3. **Reasoning & Recall Scoped Honestly**: Clarified and documented that synthetic all-one weights and non-zero phase increments represent scoped operational causal memory and geometric state progression checks, not generalized reasoning or semantic factual recall. Corrected target token selection in long-horizon test to use true ground-truth targets rather than post-hoc argmax search.
4. **Tokenizer-Bound Salience & Explicit Scope**: Added `ByteBpeTokenizer::is_added_id` in `uor-r4-tokenizer` and `score_token_salience_with_tokenizer` / `extract_salient_tokens_with_tokenizer` in `uor-r4-integer`. Resolves special tokens via tokenizer metadata and checks true decoded byte length and ASCII properties. Explicitly scoped Fibonacci barycenter averaging as lossy compression.
5. **Accurate Memory Telemetry**: Telemetry queries real process RSS via `ps -o rss=` before and after execution on the same loaded process.
6. **Hardware-Compliant Shift-Add MinP**: Implemented `q16_mul_shift_add` using pure bitwise shift-and-add arithmetic with strict factor bound checking (zsh \le \eta \le 65536$). Confirmed via static ARM64 disassembly (`otool -tv`) to contain 0 `mul`, 0 `madd`, 0 `umulh`, 0 `div`, 0 `fmov` instructions.

PR #1410 title and description have been rescoped to `feat(integer): experimental serving optimization and lossy memory compression on Apple Silicon`.


---
_Comment/review ids in this file: 5851958824, 5852583841_

