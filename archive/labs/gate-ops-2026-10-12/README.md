# OP1 (G2 only) — the Hamilton rotation gate, implemented (2026-10-12; owner amendment 2026-10-13)

**Amendment this record supersedes its own first version with.** The owner ruled on 2026-10-13T00:46:07Z
([#2029 comment 6135511346](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6135511346),
the same comment the [operator card](operator-phase-2026-10-12/README.md) cites): **OP1 runs G2 only. The
rectified-square arm G1 is withdrawn as non-geometric; there is no non-geometric fallback.** Build order is
**OP2 first** (the fused Hamilton-product CUDA kernel, because G2 is itself a Hamilton product), then OP1 (G2)
→ OP3a → rung 0 → OP4, with OP3b later research. A geometric feed-forward block is permitted. The constraint
set is frozen through the first compliant rung-0 result, and **training remains held**. The first version of
this branch contained a G1 arm; it was removed in the rebase onto `8773ac8dd` and **no G1 source ships**.

**Question.** Can the trunk's MLP gate be the geometry's own Hamilton rotation — the gate projection read as
one quaternion per 4-lane block and the up block rotated by it, `out = q(g) ⊗ u` — with exact gradients, CUDA
kernels, a trainer flag and a saved-config field, and no exponential anywhere, so the amended OP1 can be
qualified and later compared against the frozen pre-ban artifact?

**State: implemented and locally checked. No run, no device receipt, no model numbers.**

**Decision: NOT YET PROMOTED.** This record carries a mechanism and its checks, not a capability, a headline
or an acceptance result. Under the amended card, OP1's own bar and any language fit wait on OP2 and on the
lawful path existing; the amended card is also **not FIT clearance** and does not release the training hold.

## What landed

- **`StackConfig::gate`** (`crates/uor-r4-training/src/geometric_stack.rs`): `Swiglu` (default) and
  `Hamilton` only. `#[serde(default, skip_serializing_if = "Gate::is_swiglu")]`, so a config JSON written
  without the field stays byte-identical, loads as the historical gate, and every existing artifact
  reproduces exactly. `Hamilton` requires `mlp_hidden % 4 == 0` (`validate()`).
- **The gate op** (`HamiltonGate`, `CustomOp2`, beside `SwiGlu`): CPU forward and backward, CUDA dispatch,
  Metal refused (the margin objective's documented-gap pattern). No exponential, sigmoid or tanh in either
  direction. The backward is `du = conj(q) ⊗ d`, `dq = d ⊗ conj(u)`, `dg = (dq − q(q·dq)) / r` with
  `r = sqrt(|g|² + 1e-12)`.
- **CUDA C kernels** (`cuda_stack_kernels.rs`): `hamilton_gate_fwd/bwd`, one thread per 4-block, `act_t`-typed
  so the f32 and bf16 builds share one source, with f32 and bf16 launchers in `cuda_ops.rs` /
  `cuda_ops_bf16.rs`. The kernel is the same Hamilton product the amended OP2 rung is to fuse for the
  projections — this PR is its first instance, not a second mechanism.
- **The trunk's dispatch** (`StackModel::mlp`): the gate runs as the op itself, not through the f32-contract
  `pub fn` wrapper, so `precision=bf16` reaches the bf16 kernels exactly as the historical gate does. This
  was a real gap: the wrapper refuses non-f32 activations, which left the bf16 kernel unreachable and would
  have failed every bf16 run at its first MLP.
- **Trainer** (`examples/geometric-stack.rs`): `gate=swiglu|hamilton` for `train` and `dialogue-train`,
  recorded in the report and the resume lineage through `config`.
- **Refusals.** A gate other than `swiglu` refuses `qat=true` and the export/D11 path
  (`stack_export::check_export_config_weighted`, `StackModel::set_served_representation`, the trainer's
  `check_qat_config`): the integer engines implement the historical gate's mix only, so their forward would
  not compute the model that was trained. The Hamilton gate's serving form is a later rung.

## The amended card's OP1 specification, item by item

The card asks OP1 to "specify unit-normalization, zero/near-zero behavior, icosian snap, exact hard-forward
geometry and the actual gradient bridge".

| Card item | State in this PR |
|---|---|
| unit-normalization | **Delivered**: `r = sqrt(Σg² + 1e-12)`, `q = g/r`, shared verbatim by CPU, f32 kernel and bf16 kernel |
| zero/near-zero behavior | **Delivered, with a named hazard**: a zero block gives `q = 0`, forward `0`, finite result; its `d_gate` is `~1/r = 1e6` (see Findings) |
| icosian snap | **Open** — not implemented, not specified beyond "later rung"; the card's snap requirement is unmet |
| exact hard-forward geometry | **Open** — no snap, so no straight-through bridge and no hard forward exists yet |
| the actual gradient bridge | **Delivered (floating point)**: the exact adjoint plus the normalization Jacobian, checked against central differences; nothing is snapped, so no STE bridge is in play |

## Checks actually executed (owner's laptop, 2026-10-13, on the rebased tree)

| Check | Result |
|---|---|
| `cargo test -p uor-r4-training --lib --offline` | **944 passed / 0 failed / 7 ignored (base 938 + 6 Hamilton-gate tests)** (base 938 + the Hamilton gate's tests) |
| `cargo check -p uor-r4-training --lib --offline` | clean, pre-existing warnings only |
| `cargo check -p uor-r4-training --examples --bins --offline` | clean |
| `cargo check -p uor-r4-training --all-targets --offline` | 0 errors |
| `cargo fmt -p uor-r4-training --check` | clean |
| `python3 scripts/check_no_softmax.py` | OK: 1649 remaining (baseline 1649) |
| `python3 scripts/check_claim_wording.py` | PASS |
| `python3 scripts/harness/cli.py check` / `lint check` | no new rule failure; PR body lints with 0 errors |

The gate's tests are fixtures of the op arithmetic, not model evidence: a hand computation on one 4-block
(the forward, the norm-preserving rotation and the hand quaternion product), a save/load round trip proving a
non-default gate cannot silently revert on reload, central-difference gradient checks, the partial-4-block
refusal, the wire-name/validation round trip, and a tiny model's forward and backward step under the gate.

Device parity tests are in place and **skipped here** (no CUDA device on the laptop):
`test_hamilton_gate_parity`, `test_hamilton_gate_model_parity` and `test_bf16_hamilton_gate_model_parity` in
`crates/uor-r4-training/tests/cuda_stack_ops_parity.rs`. `UOR_REQUIRE_CUDA=1` on the pod turns a missing
device into a failure.

## What the local checks do and do not cover

There is no CUDA toolkit on this host (`cudarc`'s build script fails on `nvcc`), so device qualification is
pod work. The substitute used here, with its exact scope: the embedded CUDA C source type-checks under
`clang++ -fsyntax-only` with CUDA builtin stubs, and the same source, executed in a desktop build with stubbed
thread indices, reproduces the Rust CPU op and an independent from-spec implementation on a 64-element f32
vector to <= 9.3e-7 peak-scaled, exactly on the hand-computed blocks (`k ⊗ (1,2,3,4) = (-4,-3,2,1)`,
`du = (0,0,0,-1)`, `dg = (1,-2,-3,0)`). That is evidence about kernel arithmetic only; it says nothing about
device launch, grid indexing, the device bf16 path, orientation/identity conventions under a device build, or
model behavior, and it is not device qualification.

## Findings for the lead

1. **The bf16 dispatch gap was real and is fixed** (above); without it the bf16 kernel was unreachable.
2. **The amended card needed no edit from this PR.** Its OP1 section already carries the owner's G2-only,
   G1-withdrawn wording and the OP2-first order, repaired under D25 by #2259 on `origin/main`; duplicating it
   here would have been churn on an owner-authored record. This table above is the PR's contribution to that
   card's checklist. If the lead wants a pointer line from the card to this record, say so and it is one line.
3. **The icosian snap and the exact hard-forward bridge remain open** (table above). The card requires them to
   be *specified*; the snap is also the piece that would make the gate servable under D11, so it is a real
   dependency, not documentation debt.
4. **A near-zero gate block gives a large `d_gate`** (`1/r`); the 1e-12 guard is the specified mechanism and
   was not changed. Worth watching in a real run: weight decay can drive a block toward zero.
5. **Fixture churn is mechanical**: the new `StackConfig` field needed one line in 24 fixture files (bins,
   examples and test modules) plus imports — no behavior change, every literal keeps the historical gate.

## Next

1. **OP2 first**, per the amended order: the fused Hamilton-product kernel for the projections. The gate's
   kernel above is a per-4-block Hamilton product with unit normalization, so OP2 should share its core
   rather than write a second one.
2. Then the pod parity receipt for this gate's tests (no model compute), and OP1(G2) TEST FITNESS.
3. Training stays held until the whole executed path is lawful, device-qualified, FIT and explicitly released.

**Cost:** one implementation session on the owner's laptop; no paid compute, no pod leased.
