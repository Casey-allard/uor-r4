# ORDER: cpu-training-pilot, deepseek-151 (copy-key depth-age router)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `1b501c2f1`.

## Goal

Fit a discrete copy-key router from match depth and candidate age on one new
m-world-v2 panel, then test whether it beats longest-first exact matching on a
second new panel. This turns the deepseek-149 exact-match screen into a small
trainable read surrogate.

This is D84 CPU development research. It changes no float training, integer
serving, CUDA or Claude-owned source.

## Fixed construction

At each exact copy candidate, record its suffix depth `1..=4` and age
`t - s`. Fit an empirical correctness table over depth and these fixed age
uppers:

```text
1, 2, 4, 8, 16, 32, 64, 128, 192
```

At inference, among available exact candidates, choose the depth whose learned
`right / matches` is highest. Equal precision chooses the deeper key. A depth
with no training match is unavailable. This is a lookup table, not a softmax,
normalization or matrix operation.

Controls on the held-out panel are longest-first `4->1`, `k2>k1`,
fixed-depth-1, and a seeded random depth.

## Data split

Fit on new `cell=train,train` panel seed `2601`, 64 conversations. Evaluate only on
new `cell=dev,dev` panel seed `2602`, 64 conversations. Each panel must contain
at least 200 eligible positions. Public DEV remains instrument-only and must
reproduce deepseek-149's parity counts with the existing depth screen.

Exploratory use of the earlier opened seed-2501/2502 panels produced a
router/longest result of 813/815 on train and 826/826 on dev. This is **NOT A
DECISION RESULT**; the new panels above alone decide.

## Bar

The router passes only if, on seed 2602, it beats longest-first, `k2>k1`, and
random depth by the paired two-sigma rule. Each comparison requires a greater
candidate sum and at least `max(2, ceil(2*sqrt(control-only)))` candidate-only
disagreements. The random control must change at least one copyable decision.

## Stop rules

Stop on public-DEV parity drift, input/hash drift, nondeterminism, a vacuous
random control, malformed router tables, fewer than 200 evaluation rows,
resource breach or any need to change serving or Claude-owned source.

## Done when

1. Router fitting, validation, deterministic routing and malformed-table tests
   pass.
2. Fresh FIT and independent GO precede the two newly drawn panels.
3. Evidence and REPORT classify only the tested router.
4. Exact-head review, protected merge and fresh-main verification complete.
