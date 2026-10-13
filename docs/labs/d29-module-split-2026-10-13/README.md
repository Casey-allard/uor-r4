# D29 compiled-module split, and the end of NVRTC (2026-10-13)

**Decisions:** [D29](../../integration/DECISIONS.md) (no banned mechanism in any compiled or loaded module;
Claude builds the split, Codex re-qualifies). The owner's amendment "Both: split + no NVRTC" bans runtime
CUDA compilation project-wide
([#2028 6148272002](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6148272002); relayed
to the PM for filing).

**Line:** D29 compliance (governance/infrastructure) · no model line · headlines unchanged. No training, no
model-quality claim; training stays held.

## Question

Every CUDA kernel in the training crate was compiled at run time by NVRTC from three embedded sources. The
largest of them, the 3,300-line geometric-stack source, bundled **33 kernels that carry banned content**:
softmax, sigmoid, softplus, tanh-GELU, exp/log normalisation and tensor-core `wmma`/`mma` contractions, next
to the lawful operators rung 0 runs. D29 makes the compiled module the compliance boundary: presence counts,
not only execution. Can every active module be rebuilt so that it contains only compliant kernels, and be
compiled ahead of time without NVRTC?

## What changed

| Module | Before | After |
|---|---|---|
| Geometric stack, F32 and BF16 storage | NVRTC, 57 kernels | **nvcc ahead of time** (`build.rs`), 24 kernels: Hamilton gate, rate select, RMS norm, quaternion scan (forward and backward), the margin-rank hinge, the clean recurrence scan and tile pieces, AdamW, the norm reductions, the poison diagnostic and SpiralCore/FBS |
| Native context (Line B) | NVRTC, 17 kernels | nvcc ahead of time, 14 kernels; **quarantined, not compiled**, see the correction below |
| Selected potential | NVRTC, 1 kernel | nvcc ahead of time, 1 kernel; **quarantined, not compiled**, see the correction below |
| Starling K1 (rung 0b) | already nvcc ahead of time | unchanged |

- **Excluded families.** The families D29 names are excluded from every compiled module:
  - the fused sigmoid/exp/GELU recurrence (§6, and the banned kernels of §6b/§6c);
  - SwiGLU (§2);
  - cross-entropy (§5);
  - the general fused read with its flash and tensor-core variants (§7);
  - the pointer-copy mixture (§9);
  - in the native context, `whole_path_alias_log`, `emit` and `action_credit` (exp/log normalisation).

  Their source is preserved verbatim in
  [docs/history/d29-excluded-kernels-2026-10-13/](../../history/d29-excluded-kernels-2026-10-13/) and is never
  compiled.
- **Refusal.** `HISTORICAL_KERNELS` (the stack, 33 names; the native context, 3) refuses a launch loudly,
  rather than failing on a missing symbol.
- **Same arithmetic.** nvcc is given the options the NVRTC builds used (`-ftz=false -prec-div=true
  -prec-sqrt=true`, default FMA contraction), so the compliant kernels compute what they computed before.
- **The runtime compiler is gone.** `cudarc`'s `nvrtc` feature is dropped from this crate, and nothing in it
  calls a runtime compiler.

## Interlocks (`crates/uor-r4-training/tests/d29_compiled_modules.rs`, Rust)

1. Every source build.rs compiles carries no exp/log/tanh/erf token, no normalised exponential, no sigmoid,
   softplus or GELU, and no tensor-core token (comments excluded). Proved to fail on a planted `expf`.
2. No excluded historical kernel is defined in any compiled source.
3. Nothing in the crate calls a runtime compiler.
4. **With the `cuda` feature:** every built PTX module carries kernel entries and **no `ex2.`, `lg2.`,
   `tanh.`, `mma.` or `wmma.` instruction**, the instructions those operations compile to.

## Checks

- **Local (Mac):** the crate checks without CUDA; interlocks 1–3 pass, and fail on the planted token;
  `cargo fmt` is clean.
- **Ratchets:** no-softmax measured **1647** against its 1648 ceiling; no-matmul is 93/93. The
  per-file baseline was lowered to 1647 by the Rust ratchet port ([record](../rust-ratchets-2026-10-13/README.md)).
- **GPU (Vast RTX 5090 `vast-55769892`, nvcc 12.8, rustc 1.97.1, source `8db5ccb4a`):**
  - all five ahead-of-time modules build;
  - **D29 interlock 4/4 PASS, including the PTX instruction scan**;
  - **the starling rung-0b controls against the new stack module: 16/16 PASS**, including the whole-trunk
    integration control, which runs the remaining stack kernels.
  - Receipts are in [receipts/](receipts/) with `SHA256SUMS`, and the HF store holds
    `vast-archive/vast-55769892/`. The pod was set up without the bootstrap's CPU-reference parity gate (D24),
    then released and deleted. GPU time was about 15 min at $0.815/h.

## Found in passing (not changed here)

- **`tests/cuda_stack_ops_parity.rs` does not compile on `main`.** Two `StackConfig` initializers lack the
  `rec_gates` field #2293 added. This is the CPU-reference parity test (a D24 problem) that the pod bootstrap
  gates on, so every standard bootstrap of current `main` fails its parity gate. It is reported to DeepSeek
  (harness fix list, item 1) and not repaired here.
- **Vendor kernels.** Candle's own CUDA kernel modules (`candle-kernels`: the unary and reduce modules)
  contain exp, tanh, GELU and normalised-exponential kernels, and the trunk's tensor glue loads those modules.
  D29 names "linked vendor call path" in scope, so this is the next compliance item for the PM to route.

## Re-qualification

D29 assigns re-qualification of the rung-0 path to Codex. This PR is source plus the interlocks. Rung 0
waits on Codex's re-qualification, DeepSeek's bootstrap fix and the owner's hold release.

## Correction: two modules quarantined after Codex's review (same day)

Codex's independent source review (#2028 [6149477404](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6149477404),
[6149949044](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6149949044)) found a learned dense
contraction in two modules this split had listed as compliant:
- **`selected_potential.cu`:** `selected_utilities` applies 16 freely learned coefficients as a 4×4 contraction.
- **`native_context.cu`:** `score()` dots the query with freely learned per-class coefficients over the
  120/120/33 class banks, and `readout_credit`/`transition_credit` train each coefficient independently. That is
  a learned dense 4 → 120/33 projection.

The interlocks above are lexical and instruction-level, so they cannot see a contraction written as scalar
loops. Their PASS did not cover this, and the claim that every module was compliant was wrong. Both modules
are now quarantined:
- removed from `build.rs`;
- sources preserved verbatim as `native_context_quarantined.cu` and `selected_potential_quarantined.cu` in
  [docs/history/d29-excluded-kernels-2026-10-13/](../../history/d29-excluded-kernels-2026-10-13/);
- every launch through either Rust launcher returns a D29 refusal;
- a new interlock (`quarantined_dense_contraction_modules_stay_out_of_the_build`) keeps them out.

Line B (native context) stays **unfinished, not negative** (D22): it needs a typed geometric readout. Rung 0
uses neither module. The compiled surface is now the geometric stack (F32/BF16, 24 kernels plus
SpiralCore/FBS) and starling K1.
