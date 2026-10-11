# E8 orbit memory in place of the MLP, rung 1: half the MLPs go within the bar, but learned keys beat fixed E8 addresses (11 October 2026)

**Milestone:** M4 [#2032](https://github.com/UOR-Foundation/uor-r4/issues/2032).
**Line:** E8 orbit memory · count 1/3 · headline "softmax at runtime: yes" → yes.
**Pre-registration:** [6105777437](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6105777437), amended in [6105824579](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6105824579). The test-fitness review is recorded on #2032.
**Brief:** [E8 orbit memory](../../mechanisms/e8-orbit-memory.md).

## Question

The MLPs are the largest compute slice: 44% of a token's multiply-adds at context 384. Can E8 orbit memories replace them without losing held-out quality at matched training?

## The memory

`e8_memory=LAYERS:HEADS:K` (`crates/uor-r4-training/src/e8_memory.rs`). Per head, the memory works in four steps:
1. **Exact decode.** Each 8-d query half is decoded to its nearest E8 root in O(8). This is the D8 rule of the two largest |x_i|, or signs with a parity fix for the half-integer roots; no 240-way scan is needed. It is tested against a full scan on 20,000 queries, with a negative gate.
2. **Candidates.** A half's candidates are the 16 roots of that root's orbit under SpiralCore's 64-element Cl(0,6) group, which splits the 240 roots into 15 orbits of 16.
3. **Selection.** The top k of the 240² value slots are kept.
4. **Read.** Each kept slot is weighted by `R[h][r]·σ(β·s+γ)`, a plain rank table and a gate. No softmax is used.

## Runs

- **Recipe:** the quaternion study's. Geometric `rrarrarr`, width 256, 4 heads, `stack_mlp=352`, context 256, batch 16, 2,000 steps, lr 1e-3.
- **Panel:** the same 256 held-out blocks (`evaluate max_blocks=320 tune_blocks=64`).
- **Hardware:** laptop CPU, as pre-registered. This was the last rung of that kind: geometric-stack training now goes to `uor-pod` GPUs (amendment [6110389805](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6110389805)).
- **Optimizer fix used by this rung:** the CPU optimizer now updates in place, bit-identical to before (it was 62% of each step). With E8 slot selection running in parallel, this makes an E8 step 1.8× faster. It is part of this PR.

## Result

| Arm | BPB per seed | Mean | vs D256 |
|---|---|---:|---:|
| D256, dense MLPs (reused) | 1.4054, 1.4101 | 1.4078 | — |
| P, learned product-key memory, equal capacity (control) | 1.4143, 1.4207 | 1.4175 | +0.010 |
| **E8h**, layers 1, 3, 5, 7 replaced | 1.4406, 1.4406 | **1.4406** | **+0.033** |
| E8a, all 8 layers replaced | 1.5172, 1.5302 | 1.5237 | +0.116 |

The noise is 0.0054, the pooled seed SD; the paired per-block SE is smaller.

**Decision: row 3, keep and iterate** (E8h ≤ D256 + max(0.05, 2×noise)).
- Rows 1 and 2 are not met: E8a is +0.116 and E8h is +0.033.
- **The E8 geometry is not credited.** P beats E8h by 0.023, about 4× the noise. Learned sub-keys address the slots better than fixed E8 roots and orbits at this scale.

**Routing is healthy.** It is not a collapse.

| Arm | Orbit entropy per half (max 3.91 bits) | Share of 240² slots read |
|---|---:|---:|
| E8h | 3.85, 3.87 | 83%, 85% |
| E8a | 3.86, 3.85 | 87%, 86% |

So the gap to P is not routing failure. It is what fixed addresses cost against learned ones, or the value capacity per slot.

**Cost:**
- Per token per replaced layer, by design arithmetic: about 27k multiply-adds against the MLP's 270k, so **about 10× fewer**.
- Each E8 layer stores a 59 MB value table (240² × 256 f32), of which a token reads 32 rows. With Adam state that is 3 × 59 MB per layer in training.
- On the laptop CPU, E8 training ran at 1,330 tok/s for E8h and 405 tok/s for E8a, against 2,140 for P. The per-token arithmetic saving is not a training-speed saving yet.

## Next (row 3)

Iterate the configuration with GPU runs only, each pre-registered and adversarially reviewed for completion and value. Candidates:
- **k 8 → 16**;
- **learned residual addresses on the E8 frame**: an E8 decode plus a learned per-orbit offset, which tests whether the gap to P is addressing.

E8a's +0.116 says replacing every MLP is a separate, later step.

## Files

- [`score.py`](score.py) reproduces the numbers. Evaluations are in cloud-store `claude/e8-rung1-done-20261011` and `claude/e8a-rung1-20261011`; D256 is in `quaternion-study-20261011`.
- [`result.json`](result.json) holds the scored output.
