# #514 — Corpus-scale law + recommend-scale calculator: model-weight -> required observation scale

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-08T23:41:52Z
- **Updated:** 2026-08-09T07:31:05Z
- **Closed:** 2026-08-09T07:31:05Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/514
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Goal
Turn "how much corpus does a teacher of a given size need" from a guess into a calibrated, shippable calculator: `r4 recommend-scale --model <hf-dir>` reads a model config and prints the recommended observation scale (records + articles + expected regions / EXCT-miss), so anyone can size an observe run for any weight class automatically.

## What the runs already reveal
EXCT full-code coverage vs corpus size (the saturation signal):

| records | EXCT full-code resolution | source |
|---|---|---|
| 21k (176 wiki articles) | 37.5% (62.5% miss) | #509 |
| 500k (2,507 stories) | 85.4% | #432 |
| 2.11M (10k wiki articles) | 97.8% | #432 |

Coverage knee ~1-2M records; past ~2M the cover *capacity* keeps paying but only with scaled capacity on: `regions ~ (N/ref_n)^0.45` grew regions 50->104 and held-out top-1 **+5.0pp** at 400k (#460).

## The law (form grounded, exponent to calibrate)
Config-only capacity proxy `S = d_model * n_layers * log2(vocab)`:
stories15M ~19k, SmolLM2-135M ~190k (10x), SmolLM2-360M ~338k (18x).

Needed corpus scales sub-linearly with S (n-gram coverage saturates): **`N_needed = N_ref * (S/S_ref)^beta`**, beta bounded by evidence in [0.45 (measured capacity alpha), 1.0]. beta is **not yet pinned** - one clean per-teacher saturation sweep pins it.

## Cheap calibration (no repeated observes)
Observe once at the top scale, **sub-sample the record stream** to {50k, 200k, 800k, 2M}, measure where held-out top-1 and EXCT-miss flatten. One big observe per teacher yields its whole saturation curve; 3 teachers pin beta. The empirical knee always overrides the closed form.

## Deliverables
1. `recommend-scale` estimator (closed-form from `(N_ref, S_ref, beta)` + config reader).
2. Sub-sample saturation-sweep harness (turns one observe into a curve).
3. `docs/scaling_law.md` capturing anchors + calibration protocol.
4. Calibrate beta once the 360M / 135M / 15M sweeps exist (needs the big observes on dev hardware).

## Approved near-term scale
360M baseline: **~10k-15k Simple-Wiki articles (~2-3M records)** with scaled capacity on and `capacity_ref_n` retuned to the new corpus size.


## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

