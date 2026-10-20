# #2260 — OP1 (G2 only): the Hamilton rotation gate, with CUDA kernels

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T01:11:42Z
- **Updated:** 2026-10-13T01:39:55Z
- **Closed:** 2026-10-13T01:39:17Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2260
- **Merged:** 2026-10-13T01:39:17Z
- **Merge commit:** 7567cf725c74242d3eec3aa3d85405f4cbc0203f
- **Base / Head:** main / UOR-Foundation:kimi/gate-ops-2026-10-12
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Line: evaluation integrity · count 1/3 · headline 37/232 → 37/232 (no move claimed)

## OP1 (G2 only): the Hamilton rotation gate, with CUDA kernels

Head `501a13525e6d6cb3a52b1f783ea309fe646e725b`, rebased onto `origin/main` @ `8773ac8dd` (#2259, which
includes `0b9977323` #2258). Branch `kimi/gate-ops-2026-10-12`.
Record: `docs/labs/gate-ops-2026-10-12/README.md`.
Decision: **NOT YET PROMOTED** — this lands a mechanism with its checks, not a capability or acceptance
result, and no run has been made.

### The owner amendment this PR follows

[#2029 comment 6135511346](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6135511346)
(2026-10-13T00:46:07Z, cited by [D25](docs/integration/DECISIONS.md) and the operator card): **OP1 runs G2
only; the rectified-square arm G1 is withdrawn as non-geometric, with no non-geometric fallback**, and
**OP2 (the fused Hamilton-product kernel) comes first**. A G2 miss closes that configuration only (D22).
The first version of this branch contained G1; the rebase removed every trace of it — no G1 source,
kernel, launcher, flag value or test ships.

### What landed

- `StackConfig::gate` = `Swiglu` (default) or `Hamilton`, serialized only when non-default, so a config
  JSON without the field stays byte-identical and loads as the historical gate; `Hamilton` requires
  `mlp_hidden % 4 == 0`.
- `hamilton_gate` (`CustomOp2`, beside `SwiGlu`): the gate block `g` is unit-normalized and rotates the up
  block, `out = (g/|g|) ⊗ u`, with the exact adjoint backward `du = conj(q) ⊗ d`, `dq = d ⊗ conj(u)`,
  `dg = (dq − q(q·dq))/r`, `r = sqrt(|g|² + 1e-12)`. CPU forward/backward, CUDA dispatch, Metal refused
  (the margin objective's documented-gap pattern). No exponential of any kind in either direction.
- CUDA C kernels `hamilton_gate_fwd/bwd` (one thread per 4-block, `act_t`-typed so f32 and bf16 share one
  source) with f32 and bf16 launchers. This is a per-4-block Hamilton product with unit normalization —
  **the same core the amended OP2 rung is to fuse for the projections**.
- The trunk applies the gate as the op itself, not through the f32-contract `pub fn` wrapper, so bf16
  activations reach the bf16 kernel exactly as the historical gate does. This was a real gap in the first
  version: the wrapper refuses non-f32 activations, which left the bf16 kernel unreachable.
- Trainer: `gate=swiglu|hamilton` for `train` and `dialogue-train`, carried into the report and the resume
  lineage through `config`. `relu2`/`relu_square` are gone.
- Refusals (no served form yet): a gate other than `swiglu` refuses `qat=true`, export and D11 in
  `stack_export::check_export_config_weighted`, `StackModel::set_served_representation` and the trainer's
  `check_qat_config`.
- One line of fixture churn in 24 bins/examples/test modules for the new config field, no behavior change.

### The operator card's OP1 specification, item by item

| Card item | State |
|---|---|
| unit-normalization | delivered (`r = sqrt(Σg² + 1e-12)`, identical in CPU, f32 kernel, bf16 kernel) |
| zero/near-zero behavior | delivered with a named hazard (zero block → `q = 0`, forward 0; its `d_gate ≈ 1/r`) |
| icosian snap | **open** — not implemented, not specified beyond "later rung" |
| exact hard-forward geometry | **open** — no snap, so no straight-through bridge and no hard forward |
| the actual gradient bridge | delivered in floating point (exact adjoint + normalization Jacobian, checked against central differences) |

The card itself already carries the owner's amended wording (repaired by #2259 on `origin/main`), so this PR
does not edit it; this table is the PR's contribution to that checklist.

### Checks at this exact head (offline, owner's laptop)

- [x] Checks run on the rebased tree, committed unchanged, at `501a13525`
- [x] `cargo test -p uor-r4-training --lib --offline` — 944 passed, 0 failed, 7 ignored (base 938 + 6 tests)
- [x] `cargo check -p uor-r4-training --lib --offline` — clean (pre-existing warnings only)
- [x] `cargo check -p uor-r4-training --examples --bins --offline` — clean
- [x] `cargo check -p uor-r4-training --all-targets --offline` — 0 errors
- [x] `cargo fmt -p uor-r4-training --check` — clean
- [x] `python3 scripts/check_no_softmax.py` — OK, 1649 remaining (baseline 1649)
- [x] `python3 scripts/check_claim_wording.py` — PASS
- [x] `python3 scripts/harness/cli.py check` / `lint check` — no new rule failure; this body lints with 0 errors

The banned word appears **zero** times in this diff's added lines under `crates/**` (`.rs`/`.cu`): verified by
`git diff origin/main -- 'crates/**/*.rs' 'crates/**/*.cu' | grep "^+" | grep -ci <word>` → 0 of 997 added
lines. The only occurrence anywhere in the diff is the filename `scripts/check_no_softmax.py` in the record's
checks table (documentation, exempt).

The six new lib tests are fixtures of the op arithmetic, not model evidence: a hand computation on one 4-block
(forward, norm-preserving rotation, hand quaternion product), a save/load round trip proving a non-default gate
cannot silently revert, a central-difference gradient check, the partial-4-block refusal, the wire-name and
validation round trip, and a tiny model's forward and backward step under the gate.

Device parity tests are in place and skipped on this host (no CUDA device): `test_hamilton_gate_parity`,
`test_hamilton_gate_model_parity`, `test_bf16_hamilton_gate_model_parity`. `UOR_REQUIRE_CUDA=1` makes a
missing device fail on the pod.

### Scope of the local evidence (no GPU here)

No CUDA toolkit on this host (`cudarc` fails on `nvcc`), so device qualification is pod work. Substitute with
a stated scope: the embedded CUDA C source type-checks under `clang++ -fsyntax-only` with builtin stubs, and
the same source, executed in a desktop build with stubbed thread indices, reproduces the CPU op and an
independent from-spec implementation on a 64-element f32 vector to <= 9.3e-7 peak-scaled, exactly on the
hand-computed blocks. Kernel arithmetic only — not device launch, grid indexing, the device bf16 path,
orientation/identity conventions under a device build, or model behavior.

### Limitations and what the pod run should watch

- No device receipt and no run: the kernels and their launchers have never executed on a GPU, and
  `precision=bf16` with the gate has never run at all.
- The icosian snap and the exact hard-forward bridge are open (above); the snap is also what would make the
  gate servable under D11, so it is a dependency, not documentation debt.
- A gate block driven to ~0 gives `~1e6` `d_gate` (`1/r` guard); worth watching in a real run.
- If `test_bf16_hamilton_gate_model_parity` fails with "requires F32 tensors", the dispatch fix did not reach
  that build.
- A tolerance trip in the Hamilton kernels is more likely nvcc FMA contraction than a formula error (the
  formulas were hand-derived and host-executed).
- No capability, serving or model-quality claim; the gate is training-only until a served form exists.

This PR also carries the lead's five harness receipts (`docs/harness/ledger/2026-10.jsonl`).

Pre-registration: none is needed for this PR, which runs no compute. The rung this source serves is
pre-registered by `docs/labs/operator-phase-2026-10-12/README.md` as amended; its `TEST FITNESS` verdict is
still pending and gates the run, not this source.

Cost: one implementation session on the owner's laptop, no paid compute, no pod leased.

References #2029


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

