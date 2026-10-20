# The context curve: served tokens/s at 384 / 1,024 / 4,096 on the D11 product path — and the O(n) prize is bounded at ~1.7×

References #2029 (M1 acceptance; measurement cycle, no capability claim). Lab: DeepSeek, session `deepseek/ctx2`.
Run 2026-10-11 19:27–20:05 UTC: **one RTX PRO 6000 pod, three short fits, three exports, three evaluations ≈ $2.00**,
released and deleted at the last evaluation. Scoring and the stream build were local.

> **One line.** Every evaluation this project has run sits at context ≤ 384, where the quadratic read is cheapest, so the
> O(n)-versus-O(n²) question had no number. Measured on the product path (`d11-evaluate`, both integer engines, same
> architecture, only the served window differing): **D11 63.41 tok/s at 384 → 56.19 at 1,024 → 30.43 at 4,096**, a
> **2.1× fall for 10.7× the context**, with the wall per four windows rising **24 s → 73 s → 538 s**. The engines stay
> **bit-exact at every length** (`d11 − d10 nll = 0.0`, `max |Δlogit| = 0`, `top1_agreement 1.0`). The honest reading:
> **the quadratic read is real but bounded** — even at 4,096, where the audit puts it at 40.5 % of the MACs, an
> infinitely cheap read could buy at most ≈ **1.7× the served rate** — and the multiplier-free engine's *relative*
> disadvantage **shrinks** with context (D10/D11 = 1.66× at 384, 1.24× at 4,096).

## The measurement

| context | D11 tokens/s | D10 tokens/s | D10/D11 | wall, 4 windows | targets | bit-exact |
|---:|---:|---:|---:|---:|---:|---|
| **384** | **63.41** | 105.49 | 1.66× | 24.2 s | 1,536 | ✓ `0.0` / `0` |
| **1,024** | **56.19** | 77.40 | 1.38× | 72.9 s | 4,096 | ✓ `0.0` / `0` |
| **4,096** | **30.43** | 37.79 | 1.24× | 538.4 s | 16,384 | ✓ `0.0` / `0` |

| | |
|---|---|
| models | three, **one architecture** (width 576, 8 heads, 10 layers `rrarrarrar`, `mlp_hidden 732`, read `l2`, rotation on), differing **only** in `context`; 300 steps each on the same stream |
| why three models | the served window *is* the model's `context` (`StackShape::context` sizes the read's age table), so one model cannot be served at three lengths — and with the architecture held fixed, the rate difference **is** the context scaling |
| stream | a 4 M-token recall stream **built locally** and pushed to the pod (8 MB), so the run could not fail on a volume that lacked this lab's data — the failure mode that had blocked the previous attempt |
| the pod | one RTX PRO 6000, inputs pushed **before** the run, all three fits and evaluations on one GPU, **released and deleted at the last evaluation** ($0/h since) |

## What the curve means

1. **The read's quadratic term is not the dominant cost, even at 4 K.** The rate falls 2.1× while the context grows 10.7×:
   the per-window cost rises ≈ **n^1.31**, not n². The MLP (41.9 % of MACs at 384) and the recurrence (32.7 %) are paid at
   every length, and they set the floor. **An O(n) read is worth ≤ ≈ 1.7× at 4,096** — and less at every length we
   currently serve.
2. **The multiplier-free engine gets relatively better with context.** D10/D11 falls from **1.66×** to **1.24×**, so D11's
   cost — the price of serving with no multiplier instruction — is *smallest* exactly where the windows are longest.
   That is a favourable fact for the serving contract and it was not known before this run.
3. **Bit-exactness holds at every length**, including a 4,096-token session: the integer engine reproduces the reference
   exactly (`0.0` NLL difference, `0` logit difference, `top1_agreement 1.0`). Any future port must preserve this.

## The decision this measurement was for

The owner asked for O(n) instead of O(n²). The measurement says the quadratic term is worth **~1.7× at 4 K and less
below it**, while the O(n)-per-token components (MLP + recurrence + head ≈ **82 % of the arithmetic at 384**, and still
~55 % at 4,096) are where the remaining time is. So the highest-value compute move is **not** replacing the read with an
address — it is **making the per-token operators cheaper** (the MLP dominates at every length we serve), and the read is
a *long-context* optimisation whose ceiling is now known rather than assumed. That reframes the port: still worth doing
for long-context serving, no longer the first thing to do.

## Limitations

- **Three short fits (300 steps)**, trained for a **rate** measurement, not for quality: **no panel, no capability claim**,
  and the models are not candidates. Pre-registered as such.
- **One pod, one GPU model, four windows per context**; the rates are means over a small number of windows (the engine's
  own wall-clock accounting), so the *ratios* are the result, not the absolute values.
- **The stream is synthetic recall data** (4 M tokens), which is sufficient for rate and is not a quality statement.
- **Context 4,096 is 10.7× the length every capability evaluation in this project uses**; nothing here says quality would
  hold there.

## Cost

| | |
|---|---|
| pod | one RTX PRO 6000, ~38 minutes: three 300-step fits, three exports, three evaluations — **≈ $2.00**, deleted at the last evaluation |
| local | the stream build and all scoring (cheap work kept off the pod) |
| pod discipline | inputs pushed before the run; two invocation errors (`missing arch=`, stale export roots) cost ~2 minutes and are recorded; **$0/h since** |

## Evidence

- Reports: `~/uor-r4-local/ctxcurve-20261011/rate-c{384,1024,4096}/evaluation.json` and the three run reports, bundled to
  cloud-store as `[BUNDLE]`; the stream builder and its manifest are in the same bundle.
- Pre-registration: [#2029 comment 6112639827](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6112639827).
