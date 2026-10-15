# #2344 — D29 + NVRTC ban: compliant CUDA modules built ahead of time by nvcc (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T16:13:34Z
- **Updated:** 2026-10-13T16:19:20Z
- **Closed:** 2026-10-13T16:19:06Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2344
- **Merged:** 2026-10-13T16:19:06Z
- **Merge commit:** daa74c9c5896ba1fb44867a59ecfde231ecf1eb9
- **Base / Head:** main / UOR-Foundation:claude/d29-module-split-2026-10-13
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #2028 · D29 (as amended by #2339: NVRTC banned project-wide)

**Line:** D29 compliance (infrastructure) · no model line · headlines unchanged. No training, no quality claim.

## What changed
- Every CUDA module in `uor-r4-training` is now compiled **ahead of time by nvcc** (`build.rs`, `cuda` feature) and loaded by the driver. `cudarc`'s `nvrtc` feature is dropped, and nothing in the crate calls a runtime compiler.
- **Module sizes:**
  - geometric stack (F32 and BF16): 57 → 24 kernels;
  - native context: 17 → 14;
  - selected potential: 1.
- **Same arithmetic.** nvcc is given the NVRTC builds' options (`-ftz=false -prec-div=true -prec-sqrt=true`).
- **Excluded kernels:** 33 stack and 3 native-context kernels (softmax/sigmoid/softplus/GELU, exp/log normalisation, `wmma`/`mma`). They are preserved verbatim, never compiled, in `docs/history/d29-excluded-kernels-2026-10-13/`. `HISTORICAL_KERNELS` refuses a launch by name.
- **Interlocks** (`tests/d29_compiled_modules.rs`, Rust):
  - no banned token in any compiled source;
  - no excluded kernel defined;
  - no runtime compiler;
  - with `cuda`, no `ex2.`/`lg2.`/`tanh.`/`mma.`/`wmma.` instruction in any built PTX.

## Checks at this source
- **Local:** interlocks 1–3 pass (3/3), and fail on a planted token. `cargo fmt --check` is clean.
- **GPU** (Vast RTX 5090, nvcc 12.8, source `8db5ccb4a`, the same code before the rebase onto docs-only main commits):
  - all 5 modules build;
  - the D29 interlocks pass 4/4, including the PTX scan;
  - the starling rung-0b controls pass **16/16** on the new stack module.
  - Receipts with SHA256SUMS are in the record.
- **Ratchets:** no-softmax is 1647 against its 1648 ceiling; no-matmul is 93/93. The baseline is not lowered here, because its updater is Python-only.

## Not done here
- **Artifact hash-binding.** The #2339 amendment names hash-binding the PTX modules into artifacts; this PR does not do it. It is a follow-up.
- **Parity test does not compile on main.** `tests/cuda_stack_ops_parity.rs` lacks `rec_gates` (#2293), which breaks the pod bootstrap's parity gate. Routed to DeepSeek.
- **Vendor kernels.** Candle's `candle-kernels` unary and reduce modules carry exp/tanh/GELU. This is D29 vendor scope, for the PM to route.
- **Re-qualification.** Codex re-qualifies the rung-0 path on these modules.

Record: `docs/labs/d29-module-split-2026-10-13/README.md`

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6149956934 — 2026-10-13T16:13:54Z — Casey-allard

**Exact-head APPROVE at `748f553bf`** (self-review, owner rule 09-30). Crate tree identical to GPU-tested `8db5ccb4a` (`git diff 8db5ccb4a 748f553bf -- crates/ Cargo.toml Cargo.lock` is empty). At this head: D29 interlocks 3/3 (local), `cargo fmt --check` clean. GPU at `8db5ccb4a`: 5 AOT modules build, interlocks 4/4 incl. PTX instruction scan, starling rung-0b controls 16/16 (receipts + SHA256SUMS in the record).

### comment 6149965279 — 2026-10-13T16:14:22Z — Casey-allard

**Exact-head APPROVE at `045745efc`** (rebased onto D30 `ada6d0858`; only current-state ordering changed). Crate tree still identical to GPU-tested `8db5ccb4a` (empty `git diff -- crates/ Cargo.toml Cargo.lock`). D30 note: its vendor-library anti-laundering clause applies to the candle-kernels finding listed under *Not done here*.


---
_Comment/review ids in this file: 6149956934, 6149965279_

