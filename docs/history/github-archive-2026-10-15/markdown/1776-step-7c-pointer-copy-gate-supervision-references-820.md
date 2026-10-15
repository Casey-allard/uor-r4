# #1776 — Step 7c: pointer copy-gate supervision (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-06T01:20:15Z
- **Updated:** 2026-10-09T13:09:06Z
- **Closed:** 2026-10-06T01:27:25Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1776
- **Merged:** 2026-10-06T01:27:24Z
- **Merge commit:** 12f5caa50ebd5dad11962db0d018afab793b87cf
- **Base / Head:** main / UOR-Foundation:claude/step7c-gate-supervision
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #820. Step 7c, pre-registered at issuecomment-6003816899. The weight-selection rule was frozen before the sweep at issuecomment-6006413126.

**Result: REJECT G.** Copy-gate supervision opens the gate, but the pointer then copies the wrong value. v4 memory: G 21/21 vs C 24/21.

## What changes

`dialogue-train pointer_gate_supervision=W` is opt-in. The default is 0, which leaves the existing run unchanged; the evidence is below.

**The loss.** On each scored response target whose token id is held by an input position `0..=t` of its window (the soft pointer's whole reachable range, and causal), the run adds `W·(BCE(g,1) − log p_copy(target))`. `p_copy(target)` is the pointer's mass summed over every position holding that id. On a scored target that no position holds, it adds `W·BCE(g,0)`. Both terms are weighted means over the scored targets, like the mixture NLL.

**One deviation from the comment's wording.** The comment says "elsewhere no gate target". The owner's Step 7c task instead specifies `BCE(g,0)` on absent targets, and that is what runs. It is stated in the weight-rule comment.

**Where it lives.**
- `PointerMixture` gains `supervise`, which outputs `[mixture, gate_bce, pointer_nll]`. The backward reads the three upstream gradients.
- CPU and CUDA f32 are both covered. The CUDA `pointer_rows` kernel uses the free `PointerDims` slot, a warp `__any_sync` for "present", and per-row terms in the forward-unused buffers.
- `StackModel::gate_supervised_loss` and `pointer_mixture_loss_supervised` are the entry points.
- The two terms are reported in the train curve (`train_gate_bce`, `train_pointer_nll`, `train_objective`; `train_response_nll` stays the mixture NLL) and in the dev pointer diagnostics (`gate_bce`, `pointer_nll`).

**Refused cases.** `precision=bf16` is refused with "no bf16 pointer gate supervision kernel; use precision=f32". So are a pointer selection or route, a missing pointer head, and a nonpositive or nonfinite W.

## Checks (exact head a0747e97 unless noted)

**Unit tests:** `cargo test -p uor-r4-training --lib -- gate_supervision pointer` passes, 22 + 1 tests (CPU, run on b9fe4272; the a0747e97 fix touches only the CUDA host file):
- `gate_supervision_matches_the_reference_and_its_finite_differences`: an independent f64 reference of all four parts, plus central differences for every logit, side and beta element, for Dot and Lorentz, with present, absent and zero-weight rows.
- `gate_supervision_without_its_terms_is_the_unsupervised_gradient`: backpropagating only the mixture part gives exactly the unsupervised gradients.
- `gate_supervision_is_refused_where_it_is_not_defined`: covers the refusals, including bf16. The mixture part equals `weighted_loss` bit for bit.

**geometric-stack example tests:** 11/11 pass.

**CUDA parity:** `test_supervised_pointer_mixture_parity` (new) and the whole `cuda_stack_ops_parity` suite (32 passed) pass on 1× RTX 5090 at a0747e97 (uor-pod bootstrap `parity: PASS`). It compares CPU and CUDA values and gradients, and checks that the CUDA mixture part is bit-identical to the unsupervised CUDA loss.

**Bit identity of the off path on real data (2× RTX 5090):**
- The Step 6c C-s1 run was repeated with the new binary at default, and again with `pointer_gate_supervision=0`.
- Both produce `model_sha256` b754d656…, which equals the archived Step 6c C-s1 model.

**Determinism:**
- G-s1 equals the sweep run with the same W (98dab16d…).
- Each sweep W gives the same sha whichever dev set it is evaluated on.

**Data:** the store regenerated with Step 6c's exact generate and mix commands has `tokens.u16` sha256 b0770fbd…, the same as Step 6c.

`cargo fmt --check` fails on main because of `relation_compiler.rs`. That file is untouched here, and the touched files are formatted.

## Weight choice (frozen rule; seed 1; no v3, v4, open or D19 output read before the pick)

| W | generator-dev response NLL | ft/dev NLL (guard ≤ 0.0916) | generator-dev gate | generator-dev hit |
|---|---|---|---|---|
| 0 (C) | 0.2072 | 0.0816 | 0.086 | 0.49 |
| **0.1** | **0.2069** | **0.0896 ✓** | 0.75 | 0.81 |
| 0.3 | 0.2270 | 0.0923 ✗ | 0.79 | 0.81 |
| 1.0 | 0.2433 | 0.0940 ✗ | 0.81 | 0.80 |

**Chosen: W = 0.1**, the only weight that passes the ft/dev guard.

## Result: v4 is held out; v3 is development

| arm | v4 memory | v4 unknowable | v3 memory | D19 sieve | D19 off | sieve-off Mqar | Instr (sieve) | open /232 |
|---|---|---|---|---|---|---|---|---|
| A-s1 | 12/40 | 11/24 | 6/30 | 1014 | 1013 | 104/109 | 78 | 22 |
| C-s1 | 24/40 | 17/24 | 15/30 | 1004 | 1002 | 106/109 | 74 | 19 |
| **G-s1** | **21/40** | 15/24 | 18/30 | 1000 | 999 | 105/109 | 70 | 20 |
| A-s2 | 13/40 | 8/24 | 10/30 | 1022 | 1020 | 106/109 | 74 | 20 |
| C-s2 | 21/40 | 19/24 | 20/30 | 985 | 983 | 106/109 | 67 | 27 |
| **G-s2** | **21/40** | 17/24 | 21/30 | 991 | 990 | 105/109 | 64 | 25 |

Frozen criteria for G, both seeds:

| criterion | G-s1 | G-s2 | met? |
|---|---|---|---|
| v4 memory ≥ 28/40 and ≥ C + 4 | 21 (C − 3) | 21 (C + 0) | no |
| unknowable ≥ C − 3 | 15 vs C 17 | 17 vs C 19 | yes |
| sieve-off Mqar ≥ 95 | 105 | 105 | yes |
| Instruction within 5 of A | 70 vs A 78 | 64 vs A 74 | no |
| open panel not significantly worse than A (pooled McNemar) | G-only 24 vs A-only 21, p = 0.77 | (pooled) | yes |

**Decision: REJECT G.**

## Binding probe (PR #1764 rule, τ = 0.10), v4 memory misses

| model | pass | miss | (a) swap | (b) unreached | (c) readout | misses naming distractor | pointer gate on misses | pointer on expected / distractor (misses) |
|---|---|---|---|---|---|---|---|---|
| C-s1 | 24 | 16 | 7 | 0 | 9 | 2 | 0.029 | 0.09 / 0.20 |
| **G-s1** | 21 | 19 | **10** | 1 | 8 | **9** | **0.545** | 0.06 / 0.46 |
| C-s2 | 21 | 19 | 9 | 0 | 10 | 4 | 0.083 | 0.12 / 0.17 |
| **G-s2** | 21 | 19 | **12** | 0 | 7 | **5** | **0.338** | 0.06 / 0.30 |

- **Passes.** The gate rose from 0.21–0.27 to 0.94–0.96, with pointer mass on the expected value at 0.86–0.92.
- **(c) readout misses fell less than expected:** from 9/10 to 8/7.

**Reading:** supervision does what it was trained to do. The gate now opens where a value is copyable. On the rows the model had wrong, though, the pointer's mass sits on the distractor or on other tokens rather than on the expected value. Opening the gate converts "no value" misses into wrong-value copies, so (a) binding-swap becomes the majority on both seeds. The prediction "(c) → pass, (a) unchanged" is not borne out.

This is measured behaviour on one 96M configuration, one weight and two seeds. It is not a claim about gate supervision in general. The retrieval and binding limit (which value the read or pointer selects) is now the binding constraint, and gate readout alone does not lift v4 memory. Instruction also drops by 4–10 against A.

## Cost and artifacts

- **Run pod.** y2puhyt947w8s0 (2× RTX 5090, EUR-NO-1, $1.98/h), 00:21–01:19 UTC, ≈ 58 min ≈ $1.9. This covered data regeneration, 2 bit-identity runs, 7 sweep runs, G-s1/G-s2, D19 sessions, replies, the binding probe and Qwen grading.
- **Failed attempts before it.** 12 pods were created and deleted between 22:41 and 00:17. The first built and parity-tested a0747e97; each then failed in the bootstrap's `--with-ollama` step, whose download URL now returns 404. That is ≈ 30 min ≈ $1.0. **Total ≈ $2.9.**
- **Archive.** `/workspace/uor-r4/step7c/step7c-a0747e97.tar` (MD5-verified; G models, reports, replies, grades, sessions, probes, sweep reports), plus `step7c-decision.txt`, `weight-choice.json` and `step7c.done`.
- **iCloud.** Small results are under `step7c-results-20261006` (claude).

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6007426041 — 2026-10-06T01:26:30Z — Casey-allard

Exact-head check at a0747e97 (Claude lab): rustfmt clean on all 7 touched .rs files; `cargo test --release -p uor-r4-training --lib pointer` 20 passed (incl. finite-difference gradient check and off-path bit identity). Author evidence: CUDA parity suite on RTX 5090 at this head, default/0-weight runs reproduce archived Step 6c C-s1 model sha b754d656, data hash b0770fbd matches. Result is a pre-registered REJECT (gate opens, swaps rise); merging as an opt-in default-off negative candidate (D12: preserve negative candidates). Approve at this head.


---
_Comment/review ids in this file: 6007426041_

