# Quaternion-valued projection matrices — October 11

Lab: claude. Milestone: [M4 #2032](https://github.com/UOR-Foundation/uor-r4/issues/2032). Line: geometric weights (spin-native design, rung 0).
Pre-registration: [#2032 comment 6103994742](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6103994742), amended at [6104011708](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6104011708), [6104031189](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6104031189) and [6104037862](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6104037862). `TEST FITNESS: FIT` at [6104042694](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6104042694).

**Result: KEEP (decision row 2).** Quaternion-valued matrices beat a dense model of equal total parameters by 0.018 BPB, six times the noise. They also carry 4× fewer matrix weights than the dense model of equal width, at a cost of 0.088 BPB. The headline is unchanged: softmax at runtime yes, served 0.886838 BPB. This rung is about compute structure, not the read.

## Why this rung

Measured from the M4 model file, the dense projection matrices are 87.6 % of the per-token arithmetic. The quaternion rotation, the model's geometry, is 0.06 %. ([Token map](https://claude.ai/artifact/TvnHn9mZfsvTLi5ZecAxfh), [spin-native design](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6104974064).)

The bridge makes those matrices quaternion-valued: every 4×4 block is the Hamilton left-multiplication matrix of one learned quaternion, the product the recurrence already uses.

## The mechanism (`quaternion_weights=all`)

- **Compact form:** each projection map (`rec.in/gate/out`, `read.query/key/value/out`, `mlp.gate/up/down`) is stored as `[out/4, in/4, 4]`.
- **Dense form:** each forward builds the dense map with `W[4i+r, 4j+s] = L(Q[i,j])[r,s]`, where `L(a,b,c,d) = [[a,-b,-c,-d],[b,a,-d,c],[c,d,a,-b],[d,-c,b,a]]`. CPU, CUDA and bf16 paths are unchanged.
- **Initialization:** variance-matched to the dense initialization.
- **Persistence:** `config.json` records the mode.
- **Refusals:** export, `qat` and `transport_snap` are refused (there is no served form yet).
- **Tests:** Hamilton exactness against an explicit scalar Hamilton product (a flipped basis sign fails it); finite-difference gradients (tolerance at the f32 rounding floor); save/load identity; parameter counts and refusals. The default path is unchanged: the stack suite passes.

## The study

All arms use `geometric-stack train`, geometric `rrarrarr`, read l2, rotation on, chat-v0 (`chat-train.u16`, held-out `heldout.u16` e5f400b0), context 256, batch 16, 2,000 steps, lr 1e-3, warmup 100, on the M1 laptop CPU at \$0.

Scoring is `evaluate max_blocks=320 tune_blocks=64`: the same 256 consecutive held-out blocks × 256 tokens (65,536 targets) for every arm, so per-block differences are paired. Stream basis 2.837427 B/token.

| arm | width / heads / MLP | seeds | held-out BPB (mean) | per seed | map weights | total parameters | float multiply-adds per token | weight bytes per token (f32) |
|---|---|---:|---:|---|---:|---:|---:|---:|
| D256 dense | 256 / 4 / 352 | 2 | **1.407758** | 1.405402, 1.410114 | 4,358,144 | 5,406,720 | 5,537,792 | 21.6 MB |
| **Q256 quaternion** | 256 / 4 / 352 | 3 | **1.495866** | 1.493985, 1.494240, 1.499374 | 1,089,536 | 2,138,112 | 5,537,792 | 8.55 MB |
| D152 dense, matched total parameters | 152 / 2 / 208 | 2 | **1.513594** | 1.515761, 1.511428 | 1,532,768 | 2,155,360 | 2,233,184 | 8.62 MB |
| D128 dense, matched map weights | 128 / 2 / 176 | 3 | **1.552003** | 1.554689, 1.549741, 1.551580 | 1,089,536 | 1,613,824 | 1,679,360 | 6.46 MB |

**Noise (pre-registered):** the larger of the pooled seed SD (0.002929 BPB) and the paired per-block standard error (0.0015–0.0021). It is **0.002929** for every comparison.

**Precondition:** D128 − D256 = **0.144** ≥ 2 × noise, so the panel separates model sizes.

**Curve check:** dev BPB gap Q256 − D152 at steps 1,500 / 1,750 / 2,000 = −0.0155 / −0.0148 / −0.0160. Its range is within the dev noise (0.0035), so the curve is settled.

**Decision rows (in order):**
- Row 1, Q256 ≤ D256 + noise: no (+0.088).
- **Row 2, Q256 ≤ D152 − 2×noise: yes** (−0.0177 against −0.0059) → **KEEP**.

## What it means, and what it does not

- **Structure beats plain sizing at equal weight bytes.** Q256 and D152 read almost the same bytes of weights per token (8.55 MB against 8.62 MB). Serving at this size is limited by memory bandwidth, so the quaternion structure buys 0.018 BPB at the same bandwidth.
- **No compute is saved yet.** A Hamilton 4×4 block is still 16 float multiply-adds, so Q256 does D256's arithmetic. The compute saving needs the next rung: icosian-snapped weights and activations, where a block product is one 120×120 table read.
- **That rung is the spin-snap probe** (pre-registered at [6105073154](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6105073154), FIT), running now on these three Q256 models. Its tools are in this PR: `quaternion-snap` and `evaluate activation_snap=icosian|direction|scale`.
- **4× fewer matrix weights at equal width costs 0.088 BPB at this scale.** No claim is made about larger models or longer training.

## Reproduction

- **Code:** branch `claude/quat-weights`; the study ran the binary built at 6237d7713, and the dense arms ran `main` 7bd2a21ad.
- **Analysis files:** [`score.py`](score.py), the pre-registered scorer, run in the study's run directory; [`result.json`](result.json), its output.
- **Archive:** the trained models and run roots are archived with `cloud-store` as `claude/quaternion-weights-20261011`.

## Next

1. Score the spin-snap probe on the three Q256 models. It decides whether icosian table-lookup products carry the model as they are or need training-in.
2. Then the remaining spin-native rungs on the laptop: the starling read (rung 1 pre-registered and FIT) and the E8 orbit memory in place of the MLP (pre-registered and FIT).
