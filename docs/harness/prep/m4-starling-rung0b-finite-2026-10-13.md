# Pre-registration - M4 (starling-read): rung-0b finite CUDA qualification

- date: 2026-10-13
- milestone: M4 #2032 - One served model: the chosen model through D11 integer serving and a CLI
- mechanism: starling-read (the native starling / murmuration read)
- mechanism brief: docs/mechanisms/starling-read.md
- line: starling read - count 2/3 (the implementing source PR will be 3/3; D21 count is owned by the milestone)
- kind: **finite kernel/source qualification only.** No optimizer step, no corpus, no training, no
  quality panel, no fresh draw. This is not the section-5 campaign, which needs its own sheet after rung 0
  reports, and it lifts no owner hold.

## Approved specification and review

- design card: [#2032 6146209361](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6146209361)
- amendment A1: [#2032 6147797508](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6147797508)
- PM first pass, FIT-recommended: [#2032 6147268508](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6147268508)
- **Codex final second pass: TEST FITNESS: FIT for section 4 finite controls only**, including binding
  fixture corrections ([#2032 6148141029](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6148141029));
  PM record [6148179884](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6148179884). The corrections:
  - K3a: delta = 0.125 and v_C = -0.625, so u_C - u_B = delta exactly and (C, B) is excluded. Loss 1.12,
    score gradients [+0.2, -0.4, -0.4], null gradient +0.6, query gradient -0.06.
  - K3b: both real utilities 0, so there are no real/real or real/null pairs.
  - K3c: the composed normalisation gradient (I - a^ a^T)(rho_home - rho_adj)/|a|.
  - Exact value and rank-table bit identity is checked per op with the same incoming g.
- owner decision honoured: **no NVRTC** (owner, 13 October 2026; relayed on
  [#2028 6148272002](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6148272002)).
  K1 is compiled ahead of time by nvcc (`crates/uor-r4-training/build.rs`) into its own PTX module
  `uor_r4_starling_native`.

## Pinned source

- branch `claude/starling-rank-read-2026-10-13`, source commit
  `91788550aa0c07e50def382c9a0f048c58cdb57d`. Any repair re-pins the commit here before it is rerun. Re-pinned once: the first pod build at `87fe3ea57` failed to compile one raw `unsafe` launch under the crate's `deny(unsafe_code)` (a BLOCKER, repaired by routing K1 through the audited `launch_prebuilt`). Re-pinned twice: at `8a84b2f9e` 15 of 16 controls passed and the integration control hit a reshape of the heads x 4 address maps by the model width (a BLOCKER, repaired by splitting them with their own width). Re-pinned a third time: at `9e185c88d` the integration control's own classification wrongly expected an exact gradient on the key-address map, which A1 leaves untrained by design; the control now asserts no gradient there.
- files under qualification:
  - `crates/uor-r4-training/src/starling_native.{rs,cu}`, `build.rs`;
  - the `starling_native_select` launcher in `src/geometric_stack/cuda_ops.rs`;
  - the native read wiring in `src/geometric_stack.rs`;
  - the CLI admission in `examples/geometric-stack.rs`.

## Bar (pre-registered)

**Kernel/source KEEP iff every named control passes, on CUDA, at the pinned source**, after a clean
`cargo build -p uor-r4-training --features cuda` (nvcc present; no NVRTC call on the K1 path). Any failure,
compile error or refusal mismatch is a **BLOCKER to repair**: never a mechanism negative, and never a
quality or language claim (D22 section 1).

CUDA controls (`starling_native::cuda_tests`, `geometric_stack::tests`):

| Control | What it fixes |
|---|---|
| `k1_ring_ties_neighbours_and_far_cells` | ring eviction at depth M, ties to the later position, the 12-neighbour union, far cell excluded, home and key cell columns |
| `k1_document_reset_isolates_documents` | rings clear at BOS before its key enters |
| `k1_batch_and_head_isolation` | each (batch, head) row of a joint call equals that row selected alone |
| `k1_refuses_non_finite_addresses` | NaN addresses raise the device error flag |
| `k3a_active_hinge_known_answers` | the corrected K3a: 5 pairs, loss 1.12, gradients into k, null and q; identity forward |
| `credit_capture_passes_the_read_gradient_bit_exactly` | per-op bit-exact pass-through of an arbitrary g |
| `k3b_tie_and_empty_pair_set_give_zero` | zero utilities: no pairs, exactly zero credit |
| `k3c_address_credit_off_home` | unit-address adjoint -(rho_adj - rho_home); margin 0.1 + (1 - phi/2) |
| `k3c_composed_normalisation_gradient` | from raw a = 2 rho_home: ((phi/2) rho_home - rho_adj)/2 |
| `k3d_home_or_unhelpful_gives_no_address_credit` | home or unhelpful best candidate: no address credit |
| `native_starling_read_cuda_integration_control` | the actual native trunk with an `a` layer: forward and backward with credit off and on (identical forward; whole-model value/table gradients within F32 tolerance; credit reaches only content, null and address maps), E = 0 vs E = 2 identical forward, past-context forward, historical-route refusal, save/load bit-identical |

Host metadata controls (`starling_native::spec_tests`, also run locally): route parsing and refusals;
the BOS reset mask; the codebook (120 vertices, 12 neighbours at phi/2); the source interlock (no exp,
log, normalised-exponential or squashing token in K1, and no runtime-compiler call in the module); and
CPU refusal before any arithmetic.

## Minimum test power (D22 section 3)
- seeds / fresh rows: **not applicable.** These are fixed analytic fixtures with no sampling and no
  panel rows. This run is not decisive under D22 section 3 and makes no claim that needs to be.
- trained in / fresh transfer: not applicable (no training; section 5 carries both).
- noise: exact comparisons and an absolute tolerance of 1e-6 (F32 arithmetic) on the fixture answers;
  whole-model gradient agreement to a relative 1e-5.

## Base artifact
- base: none. Fresh-initialized fixture models only.

## Data split
- split: none. Fixture token IDs only, with no corpus and no panel rows.

## Steps
- steps: 0 optimizer steps; one forward/backward per fixture.

## Cost (charged cumulatively, before execution)
- preparation: local source, compile without CUDA and the host controls are done (no GPU).
- run: one Vast pod through `scripts/pod/uor-pod` (D26), 1 GPU from the ladder (RTX 5090 first). A
  release build with the `cuda` feature (`CARGO_TARGET_DIR` on `/workspace`), then the named tests:
  **<= 2 h, about 1.7 USD projected at the quoted 0.815 USD/GPU-h (UNVERIFIED)**. Retries stay within the same lease and projection.
- storage: build cache on the pod; receipts (test output, source commit, nvcc and driver versions,
  executable hash) to the HF store `results/claude/starling-rung0b-finite-2026-10-13/`. No local copy is
  kept after the round-trip.
- placement (D24): every model operation runs on the GPU; the host controls touch metadata only.
- pod/lease plan:
  - `uor-pod status`;
  - `uor-pod up --lab claude --session starling-0b --purpose "rung-0b finite CUDA qualification" --hours 3`, or a free leased GPU;
  - one job through `uor-pod run`;
  - renew if the build runs long;
  - release, then `uor-pod down`.
- owner caps: <= 4 running pods and <= $8.00/h across all labs and sessions; no new spending class.

## Stop rule
- stop: each control runs once. On any failure: diagnose, repair, re-pin the source, rerun within the
  lease. Hard stops: the owner caps and the disk floor.

## Owner rules that apply
- D21, D22, D23 (no softmax anywhere), D24 (GPU only, no host fallback), D25 (no squashing, no dense
  matmul), D26 (Vast), D27 (Rust), D28 (no transformer comparisons or neural graders), and the 13
  October NVRTC ban.

## Verdict
TEST FITNESS: FIT (Codex final second pass 6148141029; section 4 finite controls only; PM record 6148179884)
