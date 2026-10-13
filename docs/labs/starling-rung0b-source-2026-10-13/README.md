# Rung 0b source: the lawful native starling / murmuration read (2026-10-13)

**Milestone:** M4 [#2032](https://github.com/UOR-Foundation/uor-r4/issues/2032). **Line:** starling read ·
count **3/3** (this source PR; the next action on the line is a pivot card that names the decisive §5 run).
**Headline:** "softmax at runtime: yes" → unchanged. This is training-side source; the served form is the
named later item "starling D11 export".

## Question

The starling read keeps about 7 neighbours from the query's 600-cell vertex and its 12 neighbours, with no
scan of the whole context and no normalisation. Rungs 1–3 showed that q·k never learns to rank the true
source among the kept slots without competition. The only thing that taught it was a training-only softmax
surrogate, which D23 bans. Can the read be rebuilt so that **every executed operation is lawful** and the
kept-slot ranking still gets a competitive learning signal? Lawful here means:
- no softmax, no squashing, no dense map;
- GPU-only;
- no NVRTC.

## Design (approved)

- Card: [6146209361](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6146209361).
- Amendment A1: [6147797508](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6147797508).
- PM first pass, FIT-recommended: [6147268508](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6147268508).
- **Codex second pass: TEST FITNESS: FIT for the §4 finite controls**, with binding fixture corrections:
  [6148141029](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6148141029).
- Sheet: [m4-starling-rung0b-finite-2026-10-13.md](../../harness/prep/m4-starling-rung0b-finite-2026-10-13.md).

### Forward
1. **K1 selection** (`starling_native.cu`): one GPU thread per (batch row, head) keeps a ring of depth M per
   icosian vertex.
   - Rings clear at every BOS.
   - Key t enters before query t reads.
   - Query t takes its home vertex plus the 12 neighbours and keeps the K + E best by address dot, ties to
     the later position.
2. **Hard rank read:** the K kept slots and a learned per-head **null score** `b` are ranked by content
   score `s = <q, k>/sqrt(d)`.
   - Real slots ranked below the null are dropped.
   - Survivors read the rank table `T[h, r]`.
   - The E exploration slots carry zero forward weight.
3. **Maps:** query, key, value and output maps, plus two new address maps, are all artifact-bound
   SpiralCore/FBS projections. The read layer's dense NoRead map, age table and Lorentz scalars are removed.

### Backward
- **Exact:** the value and rank-table gradients are the exact derivatives of the forward.
- **`CreditCapture`:** the identity on the read. Its backward receives `g` once and injects the declared
  credit:
  - utility `u = -<g, v>`, with the null's fixed at 0;
  - **surrogate A**, a pairwise hinge `max(0, mu - (s_a - s_b))` over pairs with `u_a - u_b > delta`,
    normalised per query by `max(1, pairs)`, which trains content q/k and the null score;
  - **surrogate B**, the address margin `max(0, mu_a - (<a, rho_best> - <a, rho_home>))` when the best
    candidate is helpful and lies off-home, which trains the query address only.
- **Neither surrogate is the derivative of hard selection or a descent guarantee.**

### The owner's NVRTC ban (13 October)
K1 is compiled ahead of time by `nvcc -ptx` in `crates/uor-r4-training/build.rs` (the `cuda` feature) into
its own module, `uor_r4_starling_native`. The project's NVRTC stack module bundles historical softmax, sigmoid
and exp kernels; K1 never joins it. The project-wide migration is Kimi's to assign
([#2028 6148272002](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6148272002)).

## What landed

| Piece | Where |
|---|---|
| K1 kernel, nvcc-built | `crates/uor-r4-training/src/starling_native.cu`, `build.rs`; launcher `starling_native_select` in `src/geometric_stack/cuda_ops.rs` |
| Spec, selection op, rank read, credit capture, unit address | `crates/uor-r4-training/src/starling_native.rs` |
| Trunk wiring | `src/geometric_stack.rs`: `set_native_starling_read`, `set_native_starling_credit`, `native_starling_layer`, native admission of `a` layers, no context bound, `config.json` field `native_starling_read`, save/load through the shared shape helper |
| CLI | `examples/geometric-stack.rs`: `native_read=starling_native:M:K:E` with a **mandatory** `starling_credit=MU:DELTA:MU_A:RANK:ADDRESS` (no silent fixture default); read layers are admitted only through it |

## Checks

**Local (Mac; the `cuda` feature cannot build here):**
- `cargo check -p uor-r4-training --lib --tests --examples`: passes.
- Every CUDA-gated test type-checks with its gate temporarily removed.
- Host controls, 5/5: route parsing, the BOS reset mask, the codebook (120 vertices, 12 neighbours at phi/2),
  CPU refusal, and the source interlock (no exp, log, normalised-exponential or squashing token in K1, and no
  runtime-compiler call). The interlock was proved to fail on a planted `expf`.
- `geometric-stack` CLI contract test passes, extended for the native read.
- Ratchets: no-softmax **1648/1648**, no-matmul **93/93**, claim wording passes, fmt is clean.
- Training-crate lib suite without CUDA: 948 passed, 12 failed. **All 12 fail identically on clean
  `origin/main`** (snap, `joint_campaign`, `whole_path_learning`) and predate this change.

**GPU: 16/16 PASS** at source **`91788550aa0c07e50def382c9a0f048c58cdb57d`**, on one Vast RTX 5090
(`vast-55762148`, driver 595.84, compute capability 12.0).
- **Build:** nvcc 12.8.93 and Rust 1.97.1, with `cargo test --release -p uor-r4-training --features cuda`;
  K1 is built ahead of time by nvcc and no NVRTC is on the K1 path.
- **Run:** `test result: ok. 16 passed; 0 failed` in 2.97 s.
- **Controls passed:** every K1 case (ring, ties, neighbours, far cell; document reset; batch/head isolation;
  non-finite refusal), the corrected K3a–K3d fixtures (including the composed normalisation gradient), the
  bit-exact read pass-through, the whole-trunk integration control and the five host controls.

The sheet's stop rule re-pinned the source three times, each a BLOCKER repaired, never a mechanism result:
1. At `87fe3ea57`, one raw `unsafe` launch was refused by the crate's `deny(unsafe_code)`. K1 now launches
   through the audited `launch_prebuilt` in `cuda_stack_kernels::cuda`.
2. At `8a84b2f9e`, 15/16 passed; the integration control hit a reshape of the heads × 4 address maps by the
   model width. The maps are now split with their own width.
3. At `9e185c88d`, 15/16 passed; the integration control wrongly expected an exact gradient on the
   key-address map, which A1 leaves untrained by design. The control now asserts no gradient there.

**Receipts.** The four job logs are in [receipts/](receipts/) with `SHA256SUMS`, and are archived in the
HF store at `vast-archive/vast-55762148/`. The pod was released and deleted.
- **GPU time:** about 20 min at $0.815/h, roughly $0.27; projected $1.7.
- **Set up without the CPU-reference parity gate:** the bootstrap was stopped before its
  `cuda_stack_ops_parity` step, which is reported as a D24 problem on
  [#2028 6148671105](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6148671105).

## Limits

- No training, optimizer step, corpus or quality measurement: this is source plus finite controls only.
  Training stays held.
- No D11 serving claim; the served integer/table form is the later "starling D11 export".
- `shared` is refused in the native scope, which is per-head K only.
- **Design gap found by the integration control:** A1 trains the query address only. The key-address map
  feeds only the hard selection, so it never learns, and keys move between cells only as the hidden state
  moves. This is proposed for §5's sheet: a symmetric address margin that also pulls the best candidate's
  key toward the query's cell.
- Whole-model value/table gradients agree across credit on and off to F32 tolerance. Bit identity is the
  per-op guarantee, because CUDA gather adjoints accumulate atomically in nondeterministic order.

**Next:**
- The pivot card on #2032, naming the §5 decisive run: in-context recall top-1 against rung 0 at matched
  seeds and dose, with its own sheet after rung 0 reports.
- The deterministic reply-grader check set (D28).
