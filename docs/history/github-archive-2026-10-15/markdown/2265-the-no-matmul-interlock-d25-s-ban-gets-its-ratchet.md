# #2265 — The no-matmul interlock: D25's ban gets its ratchet

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T02:12:31Z
- **Updated:** 2026-10-13T02:18:44Z
- **Closed:** 2026-10-13T02:16:15Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2265
- **Merged:** 2026-10-13T02:16:15Z
- **Merge commit:** 7fc6e0b889ae4b29f8b20105c17c69cbe2aaa243
- **Base / Head:** main / UOR-Foundation:kimi/no-matmul-interlock-2026-10-13
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Line: evaluation integrity · count 1/3 · headline 37/232 → 37/232 (no move claimed)

## The no-matmul interlock: D25's ban gets its ratchet

The owner restated on 2026-10-13 that neither softmax nor matmul may appear anywhere in the executed path, and
that neither may be dressed up behind a helper with a geometric name. D23 has had a machine-checkable guard
(`scripts/check_no_softmax.py`); D25's ban had none. This PR adds it, built to D23's exact mechanics.
Record: `docs/labs/no-matmul-interlock-2026-10-13/README.md`.
Decision: **NOT YET PROMOTED** — a tooling guard, not a capability, and no model result is claimed.

### What landed

- **`scripts/check_no_matmul.py`** — scans `crates/**` for `.rs`/`.cu`/`.cuh`/`.wgsl` (the same selection as
  D23's gate) and counts **call sites in code**: `.matmul(`, `MatMul` (Candle's dense op), `cublas` (any case)
  and word-boundary `gemm` (any case). Comments (`//`, nested `/* */`) and plain string literals are replaced by
  spaces first, so prose about the ban and labels like `"Zero-GEMM"`/`"exact-gemm-float"` cannot trip it. **Raw
  strings are kept as code**, because the CUDA kernels live in one and that text is executable.
- **`scripts/no-matmul-baseline.json`** — repo-relative path → count. A file above its baseline, or a new file
  with any hit, FAILs. `--update-baseline` **refuses to raise** any count (it may only lower it), matching D23's
  guard.
- **`--list`** prints every site as `path:line: [kind] text`, grouped by file, plus the tensor-core sites
  separately. That list is the wrapper audit: a reviewer reads it in one pass and asks of each site whether the
  function around it advertises a dense product or hides one. A text gate cannot decide that; a review can, and
  the record says so explicitly.
- **Tensor cores are visible, not gated** (`wmma`/`mma_sync`, informational): the owner anticipates tensor cores
  for *structured geometric* kernels, while a dense GEMM on tensor cores stays banned. The count keeps that use
  visible so the two can be told apart.

Head `4c98bc35f6882618572b9f1f5912e2c07ff22c2a`, rebased onto `origin/main` @ `862f6363d` (#2264, which
prepended its own `current-state.md` entry); both entries were kept, main's above this one. The rebase
touched no file under `crates/`.

### Baseline at the merged head `19500bf22`

**95 sites in 26 files**, every one of them `.matmul(`:

| Crate | Sites | Files |
|---|---|---|
| `crates/uor-r4-training` | 89 | 25 (largest single file: `src/geometric_stack.rs`, 24 — the dense path that OP1/OP2 are replacing) |
| `crates/uor-r4-model-source` | 6 | 1 |

**0** `MatMul`-op, **0** `cublas` and **0** `gemm` sites in code: every such occurrence in the tree is prose or
a string label, which is exactly why the gate counts call sites instead of the plain word. Informational:
**102** tensor-core intrinsic occurrences in **1** file (`cuda_stack_kernels.rs`, the embedded CUDA read
kernel). Full per-file baseline in `scripts/no-matmul-baseline.json`.

### Checks at this exact head

- [x] Checks run at this exact head, offline, on the owner's laptop
- [x] `python3 scripts/check_no_matmul.py` — **OK, 95 remaining across 26 files (baseline 95)**, exit 0
- [x] `python3 scripts/check_no_softmax.py` — **OK, 1649 remaining across 145 files (baseline 1649)**, untouched
- [x] `python3 scripts/check_claim_wording.py` — PASS
- [x] `python3 scripts/softmax_path_check.py` (D23's path trace, `../check_no_softmax.py`) — runs clean on
      this tree; it lists the remaining `softmax-op`/`softmax-ce`/`logsumexp-norm` sites and this PR changes
      none of them (no Rust source changed)
- [x] `python3 -m py_compile scripts/check_no_matmul.py` — OK
- [x] Positive control: a real `.matmul(` in a scratch file (a helper with a geometric name) — **FAIL**
      `interlock-scratch.rs: 0 -> 1 (+1)`, total 95 → 96, exit 1
- [x] Negative control: the same file with only prose and string labels — **OK**, that file contributes 0 sites
- [x] `--update-baseline` with a raised count — **REFUSED**, exit 1, baseline unchanged on disk
- [x] Lowering path, in a throwaway copy under the gitignored `local/`: a removed site → OK with "1 removed
      since the baseline"; `--update-baseline` lowered it to 0 files; a new site against that empty ratchet then
      FAILs (the zero end state is a valid ratchet, which D23's script reads as "no baseline" — a wart noted in
      the record, not a divergence in mechanics)

The scratch file, the throwaway copy and the `__pycache__` from `py_compile` were removed; `local/` is empty and
**no Rust source changed** (`git status` shows nothing under `crates/`).

### Limits, stated in the record

- It cannot see inside a dependency: `candle_nn::Linear::forward`, or any helper whose internals reach cuBLAS,
  is invisible to a text gate — that is the wrapper-dressing case, and `--list` plus review is the answer.
- Spellings that are not the standalone word (`sgemm`, `my_gemm_kernel`) are not matched; `cublas` covers the
  cuBLAS spellings. Widening the pattern set is a deliberate act with an owner decision, not a silent one.
- It does not decide whether a counted site is legal — it is a ratchet, not a proof; it stops the count growing
  so removal can proceed underneath it.

### Not in this PR (follow-ups, by ownership)

- **CI wiring**: not touched here — `required-transport` and CI are Codex's surface. The follow-up is to run this
  script beside D23's gate.
- **Harness registration**: no `checkrules.py` rule and no `docs/harness/mistakes/<id>.md` entry is written here
  (the harness registry is the lead's), so the `lesson-and-guard` protocol was applied without claiming its
  receipt. The one receipt in this PR is `evidence-record`.
- `scripts/check_no_softmax.py` and `scripts/no-softmax-baseline.json` are **untouched** (Codex's claim territory).
- `docs/integration/DECISIONS.md` is untouched: D25 states the ban and the freeze but claims no interlock, so
  nothing in it is made stale by this PR.

Pre-registration: none is needed for this PR, which runs no compute and changes no Rust source; the counts
above are tooling output (files and call sites), not model numbers. Any model compute this work later
enables is gated by the amended operator card's prospective `TEST FITNESS` verdict and the owner's hold.

Cost: one implementation session on the owner's laptop, no compute, no pod leased.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

