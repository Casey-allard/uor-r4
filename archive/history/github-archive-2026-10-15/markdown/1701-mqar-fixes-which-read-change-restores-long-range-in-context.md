# #1701 — MQAR fixes: which read change restores long-range in-context recall (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-04T23:02:43Z
- **Updated:** 2026-10-09T13:08:46Z
- **Closed:** 2026-10-04T23:07:18Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1701
- **Merged:** 2026-10-04T23:07:18Z
- **Merge commit:** 3aa8ba16a38c7c9832f58e25fcbd2763d582883d
- **Base / Head:** main / UOR-Foundation:codex/mqar-fixes
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #820. This PR builds on the MQAR bench merged in #1698 and is rebased onto `main` (`2e458888`). Its bench and stack sources are byte-identical to commit `30502cd7`, which produced every run below, and the 8 tests were rerun at this head.

## Result

One change restores long-range in-context recall completely: **F2, a previous-token key channel**. Every read key also carries the previous position's key, turned by the fixed unit quaternion `j`.

On the all-read l2 stack (arm A), F2 lifts recall from 0.81 / 0.18 / 0.01 / 0.03 to **1.000 at every distance (d16 / d64 / d200 / d400)**, by step 400. It also scores 1024/1024 on the adversarial held-out pairing class, where arm A scored 0/1024.

The age-slope spread (F1) does not help; it hurts. Rotation (F3) has no effect on the reads at all.

## What changed

**`StackModel::set_read_key_shift(bool)`** in `geometric_stack.rs`. It is off by default, and the default path is unchanged.
- At every geometric read, the key becomes `k_t + j·k_{t-1}`, with zero before position 0.
- `k = W_key u` is the ordinary projection, so no new weight is involved.
- `j·` is left multiplication of each four-channel lane by the unit quaternion `j`. That is the exact signed permutation `(a,b,c,d) → (−c, d, a, −b)` (`quaternion_j_left`), a quarter turn in S³.
- The cost is one causal shift, one signed permutation and one add. There are no parameters and no matrix product.
- Queries, values, NoRead and age are unchanged.
- It has no saved or served form yet, so `save` refuses a model with it enabled.

**Bench changes** in `examples/mqar-bench.rs`:
- `age=spread` (F1) and `key_shift=true` (F2).
- The read probe: `probe_steps=0,600,final` and `probe_sequences=16`. For each read layer and head, it records the softmax weight on the true key position `q−d` and on the value position `q−d+1`, by bucket. It observes through `read_binding_masses` and changes no logits. Probe time is reported separately and not charged to the training cap.

**Tests:** `cargo fmt --all -- --check` passes. `cargo test --release -p uor-r4-training --example mqar-bench` passes 8 tests. The new ones check:
- the `j` turn is the exact quaternion product and has order 4;
- the key shift is causal, leaves position 0 unchanged, changes the read, and is refused by `save`;
- rotation has no effect on an all-read stack (identical shapes and logits), while a recurrence layer does take rotation rows;
- the probe returns one weight in [0, 1] per head and bucket.

## Runs

All arms use the same settings as #1698: CPU at 4 threads, context 512, width 128, 4 heads, MLP 384, 6 layers, 1800 steps of batch 8, lr 1e-3 with 100 warmup steps and cosine decay, seed 1.

- Accuracy is over 512 fresh-pairing queries per bucket (64 sequences).
- The binary is from commit `30502cd7`.
- Report roots are sealed under `~/uor-r4-local/mqar-bench/fixes-ctx512-30502cd7/`.
- The arm A rerun reproduces #1698's arm A exactly (0.2554), so the probe is observation-only.

| arm | change from A | d16 | d64 | d200 | d400 | all | held-out class | positions scored / query token | first curve point ≥ 0.9 | wall (train) |
|---|---|---|---|---|---|---|---|---|---|---|
| A | all-read l2 | 0.807 | 0.176 | 0.010 | 0.029 | 0.255 | 0/1024 | 8,564 | never | 1041 s (870) |
| F1 | age slopes spread `2^(−2−4h)`, last head 0 | 0.398 | 0.041 | 0.014 | 0.014 | 0.117 | 0/1024 | 8,564 | never | 965 s (819) |
| **F2** | **key `k_t + j·k_{t−1}`** | **1.000** | **1.000** | **1.000** | **1.000** | **1.000** | **1024/1024** | 8,564 | **step 400** | 1195 s (1005) |
| F1+F2 | both | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1024/1024 | 8,564 | step 600 | 1208 s (1025) |
| F3 | `rotation=false` | identical to A (see below) | | | | | | | | 30-step check |

F1+F2 was run because the grid was launched sequentially, before F1's result was known. F1 alone does not help.

### The deciding probe: weight on the true key / true value position

Each cell is the best single read head's mean weight, over 16 queries per bucket.

| arm | step | d16 | d64 | d200 | d400 |
|---|---|---|---|---|---|
| A | 0 | 0.022 / 0.024 | 0.010 / 0.010 | 0.003 / 0.003 | 0.001 / 0.001 |
| A | 600 | 0.016 / 0.192 | 0.012 / 0.109 | 0.007 / 0.043 | 0.003 / 0.022 |
| A | 1800 | 0.021 / 0.232 | 0.011 / 0.125 | 0.005 / 0.045 | 0.008 / 0.027 |
| F1 | 1800 | 0.124 / 0.100 | 0.029 / 0.062 | 0.008 / 0.038 | 0.006 / 0.031 |
| F2 | 600 | 0.006 / 1.000 | 0.003 / 1.000 | 0.002 / 0.999 | 0.000 / 0.987 |
| F2 | 1800 | 0.001 / 1.000 | 0.001 / 1.000 | 0.001 / 1.000 | 0.000 / 0.994 |
| F1+F2 | 1800 | 0.005 / 0.975 | 0.005 / 0.970 | 0.004 / 0.964 | 0.001 / 0.955 |
| uniform reference | – | 0.0163 | 0.0100 | 0.0040 | 0.0025 |

- **Arm A at the final step:** the best d200 head is layer 0, head 3, with 0.045 on the value and 0 of 16 queries above 0.5. Weight on the key stays at about uniform at every distance; no head ever locates the key.
- **F2 at the final step:** layer 0, head 2 puts 1.000 (d200) and 0.994 (d400) on the value position, with 16 of 16 queries above 0.5.

## Interpretation

These are measured results at this scale and settings, not proofs.

1. **The probe rejects the narrow form of my hypothesis.** In arm A the key match does not fail by being merely too flat against about 400 distractors; it never forms.
   - No head puts more than uniform weight on the key position at any distance or step.
   - The best long-range head (layer 0, head 3, which has the smallest default slope) puts 0.045 on the value at d200 and 0.027 at d400. That is roughly 1/24, which is what a head spreading over all of the window's value tokens would give. It is the "bag of values" read that #1698's wrong-answer breakdown pointed to: wrong answers are mostly other pairs' values.
   - Layer 0 cannot do better. Its keys know only the current token, and the value position's own token says nothing about which key preceded it.
2. **The missing ingredient is the previous-token binding, not range.**
   - To find "the position whose predecessor is my query key", a read needs predecessor identity in its keys.
   - Arm A must build that with a two-layer circuit: a local read copies `k` into position `p+1`, then a later read matches it. Its only position signal is the per-distance age table (reads carry no RoPE and A has no recurrence). In 1800 steps that circuit forms only at short range.
   - F2 supplies the predecessor identity exactly in the key path. A single layer-0 head then lands on the value with 0.99–1.00 weight at every distance.
   - The phase transition is sharp: in-class accuracy is 0.15 at step 300 and 0.98 at step 400.
   - The turn by `j` keeps the predecessor channel in a separate subspace from the current-token channel, so a query can address "previous = k" without also matching "current = k".
3. **The recency prior was not the limit.**
   - Under the default slopes, F2's solving head is head 2 (initial slope 2^−6). It still reached d400 with 0.994 weight, by learning its free age table.
   - F1, which spreads the slopes and adds a zero-slope head, weakened the short-range bias that previous-token circuits use. It halved A's d16 recall (0.40) and slowed F1+F2 (≥ 0.9 at step 600 instead of 400).
4. **Held-out pairing class.** F2 scores 1024/1024 where A scored 0/1024. Once the read retrieves exactly, the model follows the context over the training class prior.
5. **For the chat model**, two parts follow.
   - Measured: the production pattern's reads lack predecessor identity in their keys, and on this task that is enough to block long-range recall.
   - Hypothesis, not yet measured: F2 might also lift the D19 MQAR cell with the sieve off. The next steps are the production mix `rrarra` with `key_shift=true` on this bench, then the D19 grounded cell.

## F3: what `rotation=true` does

`rotation` parameterizes only the recurrence's quaternion transport `u_t` in `r` layers (`rec.gate` rotation rows). The read layers have no rotation and no relative-position term other than the learned age table.
- For an all-read stack, rotation on and off give identical shapes and identical logits; a unit test checks this.
- Two 30-step runs give bit-identical losses (6.26352, 6.15312, 6.03775). They are sealed as `F3-rotation-on-30steps` and `F3-rotation-off-30steps`.
- A full 1800-step F3 run would duplicate arm A, so it was not run.

**What relative-position rotation of q/k would need** (described here, not built):
- Choose a fixed unit quaternion `ρ_l` per four-channel lane, for example `exp(θ_l n_l)` with `θ_l` from the artifact's fixed zeta phases and axis `n_l`.
- Rotate the query at `t` and the key at `s` lane by lane: `q_t ↦ ρ_l^t q_t` and `k_s ↦ ρ_l^s k_s`.
- Left multiplication by a unit quaternion preserves inner products and Euclidean distance. So `⟨ρ^t q, ρ^s k⟩ = ⟨q, ρ^{s−t} k⟩` and `|ρ^t q − ρ^s k| = |q − ρ^{s−t} k|`: the dot and L2 scores both depend only on `s − t`.
- `ρ^t` is a lookup table indexed by `t`, and the product is a fixed 4×4 signed-coefficient map.
- This is the quaternion generalization of rotary encoding: the U(1) special case is RoPE. If the owner's "no RoPE" excludes that whole family, it should stay unbuilt.
- F2 shows that this task does not need it: predecessor identity alone gives exact recall at every tested distance.

## Cost

- Agent wall time: about 1 h 25 min (17:33–18:58 ET).
- Model compute, all on CPU at 4 threads (`RAYON_NUM_THREADS=4`):
  - four 1800-step arms: 4,409 s wall, including 3 probes of about 58 s each per arm;
  - F3 short runs: about 50 s;
  - smoke run: about 70 s;
  - builds: about 6 min.
- Peak RSS was under 1 GB.
- New storage is 992 KB of sealed report roots.
- No GPU or external compute was used.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

