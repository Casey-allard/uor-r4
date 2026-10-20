# The no-matmul interlock: D25's ban gets its ratchet (2026-10-13)

**Question.** D25 bans dense matrix multiplication in training and in serving (owner restatement, 2026-10-13:
neither softmax nor matmul anywhere, and neither may be dressed up behind a helper with a geometric name), but
only D23 had a machine-checkable guard. Can the ban get one that fails on a new call site, cannot be raised to
bless one, and is auditable in one read for wrapper dressing?

**Artifacts.** `scripts/check_no_matmul.py` and `scripts/no-matmul-baseline.json` (new, in this PR);
`scripts/check_no_softmax.py` + `scripts/no-softmax-baseline.json` are D23's, read for their mechanics and
**not modified** (Codex's claim territory). No Rust source changes. No compute.

**What was run (and what failed first).**

| Step | Result |
|---|---|
| `python3 scripts/check_no_matmul.py` before the baseline existed | FAIL: "no baseline" — the ratchet has to be set deliberately |
| `--update-baseline` (first run) | baseline written: **26 files, 95 sites** |
| `--list` | 95 sites, every one `[matmul-method]`; ledger below |
| clean-tree check | **OK: 95 remaining across 26 files (baseline 95)**; informational line: 102 tensor-core intrinsic sites in 1 file |
| positive control — a real `.matmul(` inside a scratch file, in a helper with a geometric name | **FAIL** `crates/uor-r4-training/interlock-scratch.rs: 0 -> 1 (+1)`, total 95 → 96, exit 1 |
| negative control — the *same* scratch file with only prose and string labels (`"Zero-GEMM"`, `"exact-gemm-float"`, a sentence containing `.matmul(`) | **OK**, that file contributes **0** sites: comments and plain strings are stripped before counting |
| `--update-baseline` with a raised count | **REFUSED**, exit 1, baseline unchanged on disk |
| lowering path (throwaway copy of the script + a fixture, under the gitignored `local/`) | site removed → OK with "1 removed since the baseline"; `--update-baseline` lowered it to 0 files; a new site against that empty ratchet then FAILs |

The scratch file and the throwaway copy were removed; `local/` is empty and nothing outside `scripts/` and
`docs/` was touched.

**Numbers, with scope.** Measured on the tree at `19500bf22` (origin/main, the merged OP2):

- **95 call sites in 26 files**, all of them `.matmul(` method calls: `crates/uor-r4-training` 89 (25 files,
  `src/geometric_stack.rs` alone 24), `crates/uor-r4-model-source` 6. Full per-file baseline:
  `scripts/no-matmul-baseline.json`.
- **0** `MatMul`-op, **0** `cublas` and **0** `gemm` sites in code. Every such occurrence in the tree is prose
  or a string label ("Zero-GEMM", "exact-gemm-float", `MatMul-free` in banner text), which is exactly why the
  gate counts call sites rather than the plain word.
- **102** tensor-core intrinsic occurrences (`wmma`/`mma_sync`) in **1** file
  (`cuda_stack_kernels.rs`, the embedded CUDA read kernel). Informational only: they never fail the gate.

**Decision: NOT YET PROMOTED** (a tooling guard, not a capability or a model result). The ban itself is
unchanged: D25 stands whether or not the count is zero.

## What it counts, exactly

In **code** — comments (`//`, nested `/* */`) and plain string literals are replaced by spaces first, so prose
about the ban cannot trip it — the patterns are the ones D25's interlock defines:

| Pattern | What it catches |
|---|---|
| `.matmul(` | a call to the dense product |
| `MatMul` | Candle's dense-product op, applied by name |
| `cublas` (any case) | a cuBLAS handle, binding or entry point |
| `gemm` (word boundary, any case) | a BLAS dense product by its own name |

**Raw strings are kept as code**, not stripped: the CUDA kernels live in one (`CUDA_STACK_SOURCE`), and that
text is the executable kernel source. A dense product written there is counted.

**What it does not count (stated, not hidden):**

- **Anything inside a dependency.** A call to `candle_nn::Linear::forward`, or any helper whose *internals*
  reach cuBLAS, is invisible to a text gate. This is the wrapper-dressing case the owner named, and it is why
  the gate ships with `--list`: the reviewer reads every site in one pass and asks of each whether the name
  before it is honest. A gate cannot answer that; a review can.
- **`gemm` spellings that are not the standalone word** — `sgemm`, `my_gemm_kernel`. The pattern is the
  word-boundary one D25's interlock defines; the `cublas` pattern covers the cuBLAS spellings (`cublasGemmEx`,
  `cublas_gemm`). Widening the pattern set is a deliberate act, not a silent one.
- **A `"..."` string that names a dense product** (e.g. a kernel name in a launch table): plain strings are
  stripped, including this one.
- **Prose in `docs/`**: not scanned at all — dated records are evidence, not plans (same as D23's gate).
- **Whether a counted site is legal.** The gate is a ratchet, not a proof: it stops the count from growing so
  removal can proceed without new violations landing underneath it.

## The wrapper audit, in one read

`python3 scripts/check_no_matmul.py --list` prints every site as `path:line: [kind] text`, grouped by file, then
the tensor-core sites separately. The workflow for a reviewer: read the list; for each `[matmul-method]` site,
look at the function it sits in and ask whether that function's name advertises a dense product or hides one; a
name like `geometric_*`/`hamilton_*`/`lattice_*` wrapping one of these patterns is the violation the owner
described, and it is a review finding, not something the text gate can decide.

## Tensor cores: visible, not gated

The owner's distinction is recorded here so nobody has to re-derive it: **tensor cores are anticipated for
structured geometric kernels** — a Hamilton product or an icosian table lookup that happens to use fragment
intrinsics is a legitimate implementation of a geometric mechanism, and `wmma`/`mma_sync` are therefore listed
informationally and never fail the gate. **A dense GEMM on tensor cores is still banned**: the ban is on the
dense contraction, not on the hardware unit. The informational count keeps that use visible so a reviewer can
tell the two apart.

## Line of the ratchet, and the follow-up

- The gate is **not wired into CI in this PR**: `required-transport` and CI ownership are Codex's, and the
  harness registry is the lead's. The follow-up is to wire `python3 scripts/check_no_matmul.py` into the same
  place D23's gate runs (and, if the harness should carry it, to register it as a `checkrules.py` rule — this
  record deliberately does not touch either surface).
- The `lesson-and-guard` protocol was applied to design this guard (trigger: "a lab adds a helper whose body
  calls a dense product"; symptom: a green suite and a growing count; lesson: name the pattern set and ratchet
  it; guard: `scripts/check_no_matmul.py`). Registering it as a `docs/harness/mistakes/<id>.md` entry with a
  `checkrules.py` rule id is left to the harness owner: this PR does not write into that registry, so no
  `lesson-and-guard` receipt is claimed.
- The receipt in `docs/harness/ledger/2026-10.jsonl` logs the `evidence-record` skill for this record.

**Cost:** one implementation session on the owner's laptop; no compute, no pod.
