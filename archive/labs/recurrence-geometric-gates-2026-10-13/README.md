# The geometric recurrence gates: dependency 3 of the rung-0 sheet (2026-10-13)

**Question.** The [rung-0 pre-registration](../../harness/prep/m1-rank-margin-rung0-2026-10-13.md) named four
executed squashing sites in the recurrence (a sigmoid opening, an `exp`/`softplus` decay rate and a tanh-GELU
output gate) as dependency 3, with the replacements pre-declared. Can those four sites be replaced by exact
geometric mechanisms — a clamped-linear opening, an exact dyadic rate table and the Hamilton rotation — so the
recurrence the compliant trunk trains contains no exponential-family operation in either direction?

**State: implemented and locally checked. No run, no device receipt, no model numbers.**

**Decision: NOT YET PROMOTED.** Source only. Device qualification is dependency 4 and still open; the training
hold is untouched, and this record moves no headline (reply 37/232, v5 memory 20/40).

## What landed

A new `RecGates` mode on `StackConfig` (`historical` default, `geometric`), following the exact
pattern `Gate`/`Objective`/`QuaternionWeights` established for the earlier operator work:

- **`rec_gates=geometric`** makes `composed_recurrence` — the model's actual forward path — compose:
  `score = clamp(gate opening + rec.decay, 0, 1)` (clamped-linear, exact subgradient), then
  **`rate_select`** (an exact table read), then the unchanged `quaternion_scan` carry, then **`hamilton_gate`**
  (the OP1 rotation gate, `#2260`) on the recurrence output. *The per-lane decay parameter is read raw as the
  rate-selector score* — no softplus transform — and the table's entry is chosen by threshold, so all four
  sites change at once.
- **The fixed dyadic table** (`RATE_TABLE`/`KEEP_TABLE`, `RATE_ENTRIES = 16`): `lambda = 2^-(s+1)` for
  `s = 0..16`, exact f32 powers of two, with `keep = sqrt(1 - lambda^2)` precomputed beside each entry
  (`|keep^2 + lambda^2 - 1| <= 6e-8`; the last four keeps are the documented f32 plateau at `1`). This is the
  D11-compatible form: serving is table reads.
- **The declared geometric secant surrogate** (the pattern `#2279` landed for the native projection, labelled
  the same way in code and here): the forward's hard selection has zero derivative almost everywhere, so the
  backward pairs the incoming adjoints `(d lambda, d keep)` with the selected entry's lawful neighbours'
  **complete counterfactual table displacement**, resolving that loss contrast along the fixed edge
  `1 / RATE_ENTRIES` — one half per side, the whole weight to the single neighbour at either end of the table.
  No probability, no exponential and no learned coefficient is involved.
- **Device path**: `rate_select_fwd`/`rate_select_bwd` CUDA kernels (`act_t`-typed for the f32 and bf16 builds;
  `lambda` built from its exact f32 bit pattern, `keep` by `sqrtf`, the secant rule identical to the host op)
  with f32 and bf16 launchers.
- **Refusals, following the existing patterns**: a model in `rec_gates=geometric` **refuses any non-CUDA
  device loudly at the model level** (`params()`; D24 — the ops keep host references for unit tests only);
  `validate()` refuses the mode on a transformer control or a width that is not whole 4-blocks; and
  **export/D11 and `qat=true` refuse the mode** because the integer engine binds the *historical* decay
  parameter — exporting would silently reinterpret `rec.decay` values as rate scores, and a stale or
  undeclared binding is refused, never converted.

## Deliberate deviation: the historical form is retained in-tree, not deleted

The brief asked for the sigmoid/`exp`/tanh recurrence to be preserved as an indexed patch in
`docs/history/branch-archive/`. **Nothing was deleted, so nothing needed archiving.** `RecGates::Historical`
remains the default and the exact historical code path, so every existing config, checkpoint and test
reproduces bit for bit — the same choice the project made for `Gate::Swiglu`, `Objective::Ce` and the dense
quaternion path's *callers*. It is documented in code as prohibited for new runs, and the geometric form is
what rung 0 declares. Had the historical path been deleted, every existing artifact would have stopped
reproducing, which preservation policy forbids. If the owner prefers deletion-with-archive instead, it is a
mechanical follow-up (the patch would be `RATE_TABLE`-adjacent: `composed_recurrence`'s else branch).

## Evidence

**The geometric path is squash-free — verified by grep, per arm, after the change** (`exp|expf|exp2f|sigmoid|tanh|tanhf|softplus|log1p|gelu`,
word-boundaried):

| Region | Squash-family tokens |
| --- | --- |
| geometric decay arm (`composed_recurrence`, host) | **0** |
| geometric output-gate arm (host) | **0** |
| `rate_select` host op — *code* | **0** (its documentation names `exp`/`softplus` twice, to state they are absent) |
| `rate_select` CUDA kernels (`cuda_stack_kernels.rs` § 2d) — code | **0** |

The historical `else` branches in the same function retain 8 such tokens by design (above).

**The new kernels were executed on the host from their own source text** (the technique used for OP1/OP2) and
checked against an independent reference: rate table **5e-13** max abs error, keep table **4.6e-10** max
relative, secant credit **5.5e-8** max relative. That covers arithmetic and indexing only; nothing about device
launch (dependency 4).

## Checks

| Check | Result |
| --- | --- |
| `cargo check -p uor-r4-training --all-targets --offline` | clean (0 errors) |
| `cargo test -p uor-r4-training --lib --offline` | see the PR body for the exact count; **the three `*snap*` failures are pre-existing** — they exercise the dense quaternion path `#2279` retired, and the refusal that fails them is present in this branch's base commit (`9a20ca953`, verified by `git show`) |
| `cargo fmt -p uor-r4-training --check` | clean |
| `python3 scripts/check_no_softmax.py` | **OK 1649** (baseline 1649, unchanged — nothing was removed, and no new occurrence was added) |
| `python3 scripts/check_no_matmul.py` | **OK 93** (baseline 93, unchanged) |
| `python3 scripts/check_claim_wording.py` | PASS |

The new tests are host fixtures of the arithmetic: the table's exactness/monotonicity and the
`keep^2 + lambda^2 = 1` identity; hand-computed selection on fixed scores including both table ends; the
secant credit against hand values for an interior and a boundary entry; the clamp's exact subgradient; the
model-level CUDA refusal and the two validation refusals; a composed one-step case (entry by score, the
rotation gate's norm preservation); and the mode's serde round-trip with byte-identical legacy configs.

## Limitations

- **No device receipt**: the kernels are host-executed, not device-qualified; that is dependency 4's job, and
  the geometric mode's CUDA requirement means the rung cannot run before it.
- The geometric recurrence runs as an **op composition** rather than the fused `RecurrenceCore` kernels; the
  fused path is retained for the historical mode and is not on the geometric path. Fusing the geometric form
  is a later performance item, not a correctness one.
- The **read path's `exp`** (`cuda_stack_kernels.rs:992` region) is untouched: rung 0 has no read layer, and
  that site is rung 0b's item per the sheet.
- The geometric mode is f32 only (matching the landed margin kernel); a bf16 margin kernel is out of scope.
- Main carries pre-existing test failures from other labs' recent merges (the three `*snap*` cases above, and
  a flaky `joint_campaign` checkpoint-binding family); this PR adds none and fixes none — they are reported,
  not silently absorbed.

## Next

Dependency 4 (lawful device qualification: stored/analytic expected values, no CPU reference) after Codex's
projection and OP3a; then the owner's hold release and rung 0 itself. The FIT review of the rung-0 sheet
checks this dependency's delivery against the sheet's own terms.

**Cost:** one source session on the owner's laptop; no compute, no pod.
