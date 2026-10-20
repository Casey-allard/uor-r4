# Starling read, rung 2: the addresses learn, but the readout does not use them (11 October 2026)

**Milestone:** M4 [#2032](https://github.com/UOR-Foundation/uor-r4/issues/2032).
**Line:** starling read · count 2/3 · headline "softmax at runtime: yes" → yes.
**Pre-registration:** [6107021590](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6107021590), amended in [6107070727](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6107070727) and [6107090319](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6107090319).
**Test fitness:** [FIT](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6107098703).
**Device amendment:** [6110389805](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6110389805).

## Question

Rung 1 ([record](../starling-read-2026-10-11/README.md)) put the pure starling read at chance. A hard 7-neighbour selection gives keys outside the kept set no gradient. Does the address learn once every key gets one?

## Change

`starling_dense` (training only) adds α × a causal mean over every earlier position of `sigmoid(β⟨a_t,b_j⟩+γ)·v_j`. It uses no softmax.

- α falls linearly from 1 to 0 over the first half of training (`starling_anneal=0.5`). The second half trains the pure starling read.
- Evaluation, save and export always read the pure form.
- Tests: α = 0 is bit-equal to the pure read, and the helper's key-address gradient matches finite differences.
- `mqar-bench` logs the **address hit rate** at step 0 and every 100 steps. The hit rate is the share of queries whose true source position is among the positions some head kept.

## Runs

- **Bench:** rung 1's, unchanged: context 512, 1,800 steps, `pattern=rarr key_shift=true`, width 128, 4 heads.
- **Arms:** SA7 (`starling:4:7`) and SA7w (`starling:8:7`), 3 seeds each.
- **Where:** one RTX 4090 on `uor-pod`, `device=cuda`. All six runs took 2 min 52 s in total, versus about 6 h projected on the laptop CPU.
- **Reused from rung 1:** L (L2 read) and S7.

## Result

| Arm | Recall per seed | Mean | Hit rate: step 0 → peak → final | Hit-rate bar H | Row |
|---|---|---:|---|---:|---:|
| L, L2 softmax read (rung 1) | 0.669, 0.373, 0.809 | 0.617 | — | — | — |
| S7, pure starling (rung 1) | 0.013, 0.008, 0.006 | 0.009 | — | — | — |
| **SA7** | 0.0127, 0.0127, 0.0083 | **0.011** | 0.100 → **0.725** (step 900) → **0.432** | 0.201 | 3 |
| **SA7w** | 0.0117, 0.0127, 0.0078 | **0.011** | 0.099 → **0.727** (step 1000) → **0.556** | 0.198 | 3 |

Held-out-class recall is 0.000–0.003 for both arms. Every run reached step 1,800.

**Decision: row 3** for both arms. Recall is below row 2's bar (0.059). The final hit rate is well above H.

- **The addresses learn.** With dense gradients, the true source moves from 10% to 73% of the kept 7-neighbour sets.
- **They keep much of it on the pure hard read.** Once the helper is gone, the final hit rate is 43% for SA7 and 56% for SA7w.
- **Recall still does not rise.** The fault has moved from finding the source to reading it. The rank table `R[h][m][r]` and the gate `σ(β⟨a,b⟩+γ)` do not single out the true source among the 7 kept positions, so the mix of 7 values does not decode to the answer.

This is a configuration result (D22 §1). It does not close the mechanism.

## Next (pre-registered row 3)

Fix the readout. Candidates:
- a supervised or agreement term on the rank table and gate;
- reading values through the helper's top keys;
- a sharper gate, so the right one of the 7 dominates.

Each gets a synthetic check before any panel run.

## Files

- [`score.py`](score.py) reproduces every number from the run logs (cloud-store `claude/starling-anneal-20261011`).
- [`result.json`](result.json) holds the scored output.
