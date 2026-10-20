# The local prose deficit is in the output layer, and the state does not retain the second-most-recent token — September 23, 2026

> **Correction — 2026-09-23 (K3 architecture review, follow-up to PR #1368).** The title's second clause is
> **contested and refuted for the training-free observer class**. An unmerged controlled diagnostic
> (`codex/observer-transport-20260923`) run on **byte-identical dev states** (SHA `c5334071…992f`, matching this
> record's sealed `olx-probe-4` `recorded_states_sha256.dev`) recovers `prev = x_(t-2)` at **75.90 %**
> (3,209/4,228) with a **transport-aware** decoder, against this record's **raw-embedding** **2.06 %** (87/4,228)
> on the same denominator, with rotated-label (0.67 %) and wrong-frame (0.14 %) controls and an
> uncentered-transport step still at 70.4 %. The recurrence writes `E[x_(t-1)]` at full scale and contracts
> `x_(t-2)` by `Wh >> 3`, so `prev` **is** present in the state in contracted, `Wh`-mixed coordinates —
> invisible to a nearest-*embedding* probe (and to any **linear** readout) but recoverable through the model's
> own map.
>
> **What stands unchanged:** `prev` is not available to the served **linear readout class** (the `P − C = +0.729`
> gap stands), and the **raw-embedding** decode of `prev` is 2.1 %. **Withdrawn as general statements:** the title
> clause "the state does not retain the second-most-recent token" and §3's "the token before it is not
> [available]". The measured numbers in this record are unchanged; this is a claim-scope correction. The
> contradicting evidence is **unmerged and not yet reproduced on main**; its independent adjudication (with a
> transport-decode depth ladder) is the ordered next milestone in [current state](current-state.md).

This record answers one consequential uncertainty that the previous milestone left open and that
decides which remedy to build next: is the shared model's ~1.5-bit deficit against a tuned two-token
count reference a limit of the **served recurrent state** or of its **trained output layer**?

It is executed on `codex/lexical-formulation-20260923` at `67f2e350` — the head of open,
unmerged PR [#1363](https://github.com/UOR-Foundation/uor-r4/pull/1363); protected main is `79b5d50c`.
The model source `learner/transferable_lexical.rs` is **unchanged**
(`sha256 03f83eb8…b250`). Only the runner gained a diagnostic mode, `--state-probe`. Evidence:
[`docs/evidence/ordinary-lexical-state-probe-2026-09-23.json`](../evidence/ordinary-lexical-state-probe-2026-09-23.json).
New sealed roots: `olx-probe-3` and `olx-probe-4` (under
`/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/`); `olx-probe-1/2` and the
`/tmp/olx-sp-*` smokes are preserved as earlier attempts. `olx-form-2` and every other preserved root
are unmodified.

## 1. The instrument, and the two corrections an independent review forced

`--state-probe` walks every fit and dev window **exactly as `TlModel::score_example` walks it** and
records, at every scored prose position, the precise state vector `h` the served readout receives.
It then:

- fits a **float** multinomial logistic readout of that state (closed-form ridge first, then softmax
  refinement) on the fit split and evaluates it on development;
- reports the artifact's own served readout loss, a tuned **unigram + bigram(`cur`)** count model
  (`E1`) and the tuned **`(prev, cur)`** count model (`C`) on the identical target population;
- decodes `cur = x_(t-1)`, `prev = x_(t-2)` and the generated target `x_(t+1)` from the recorded state;
- re-prices the fitted readout into the two servable alphabets (ternary, and 4-bit as D0-b permits).

Three independent controls run every time: the recorded per-position bits must equal
`score_example`'s own bits and the recorded state count must equal its scored Generate targets (passed:
334,796 fit and 5,376 dev states); every recorded state must carry `(cur, prev) = (tokens[k-1],
tokens[k-2])` of its window (passed on all 340,172 positions); and a **null probe** fitted on permuted
labels must sit at the restricted unigram (`olx-probe-3`: 8.5662 versus 8.5264, gap +0.040;
K=64 smoke: +0.008; `leaking: false` in both).

An independent K3 architecture review of the instrument changed it in two ways that matter for reading
these numbers, and both corrections are load-bearing:

1. **A linear readout cannot in general reach the count reference.** Any readout linear in `h` scores
   `w_next · h(context)`, so its score matrix over (target × context) has rank at most `h_dim = 64`,
   while `C` is a full interpolated 4096×4096 table. The decision rule must therefore be
   **`P` versus `A`** (a converged readout of the same input versus the artifact's own readout), not
   "does `P` approach `C`". `C` is a context anchor.
2. **The comparison needs the same normalisation on both sides.** The probe's softmax is over the
   retained target classes, which hands it the probability mass the restriction removes and makes its
   loss look better than a full-set loss by up to ~0.14 bits at K=1024 and ~1.1 bits at K=64. The first
   full-population run compared across the two conventions; the artifact-initialised run was added
   specifically to remove that confound (and, as §6 records, it did not fully).

## 2. Decision-population result (K = 1024, 4,954 of 5,376 development targets = 92.15% coverage)

All figures in bits/target. `A`, `E1`, `C`, `U` share the artifact/count normalisation (the full legal
set or vocabulary), restricted to those targets, and are directly comparable with each other. `P` is
the probe's own K-class convention and is comparable with `A` only after allowing for that convention.

| quantity | value |
| --- | ---: |
| `A` artifact's own served ternary readout | **6.055956** |
| `P_softmax` converged float linear readout of the same state | **5.5136** (top-1 0.2618) |
| `P_ridge` closed-form least-squares readout | 7.7575 |
| `E1` unigram + bigram(`cur`) count model | **5.565123** |
| `C` tuned `(prev, cur)` count model | **4.784593** |
| `U` fit-only unigram | 8.669572 |
| `P_softmax` − `A` | **−0.5423** |
| `P_softmax` − `E1` | −0.0515 |
| `P_softmax` − `C` | +0.7290 |

The softmax refinement is at its floor: the last epoch moved the fit loss by **0.0119 bits/target**
(`fit_converged: false` by the declared 0.01-bit rule, i.e. converged for practical purposes). This is
not the under-trained probe of the first run; the artifact-initialised run reached 5.5136 where the
ridge-initialised 12-epoch run had reached 5.9284.

## 3. The state carries the last token and does not expose the one before it to a raw-embedding probe

> **Correction (see the banner at the top):** the *retention* reading of this section is withdrawn — `prev` is
> recoverable through the model's own transport at 75.90 % on these same states. The *raw-embedding and linear
> readout* results below stand exactly as measured.

On the same 4,228 decodable dev positions (`cur`, `prev` and the target all inside the class set), a
**training-free nearest-embedding rule** over the artifact's own 4,096 embedding rows gives:

| decoded quantity | nearest-row top-1 (L2) | nearest-row top-1 (cos) | majority class | chance (Σp²) |
| --- | ---: | ---: | ---: | ---: |
| `cur = x_(t-1)` | **0.9674** | **0.9681** | 0.0381 | 0.0087 |
| `prev = x_(t-2)` | 0.0206 | 0.0206 | 0.0367 | 0.0084 |
| `x_(t+1)` (the generated target, as a control) | 0.0201 | 0.0203 | 0.0350 | 0.0083 |

A ridge decode from `h` agrees on the ordering with the same estimator applied to all three targets:
`cur` 4.0495 bits / 0.5333 top-1, `prev` 6.4473 / 0.2838, `x_(t+1)` 7.6746 / 0.1781. The estimator is
weak in absolute terms at 1,024 classes, but the **relative** ordering is what matters and both rules
agree: the immediately preceding token is available from the state almost exactly, and the token before
it is not.

**The state-side hypothesis is therefore rejected for the *first*-order structure and supported for the
*second*-order structure.** This matches the recurrence: `E[x_(t-1)]` enters `h` as a verbatim add,
while `x_(t-2)` reaches `h` only after a ternary matrix product and a right-shift by three.

## 4. The served output layer realises about half of what its own state supports

`P_softmax` − `A` = **−0.5423 bits/target**. Allowing the probe's convention at most the ~0.14-bit
advantage the restriction can buy it, the artifact's served readout is losing **at least ~0.40
bits/target** that a converged readout of the very same input recovers. And the converged float readout
lands within 0.05 bits of `E1`, the one-token count model: the state plus a readout that fits it is
essentially a bigram, while the artifact's served ternary readout is 0.54 bits behind that — and 0.49
bits behind `E1` is exactly where the artifact sits (6.055956 vs 5.565123).

So the local deficit decomposes, on this population:

- **~0.54 bits (≥0.40 after the convention allowance): output-side.** Recoverable by fitting the served
  readout better, or widening its alphabet, on the *same* state.
- **~0.73 bits: second-order structure.** `P` − `C` = +0.7290. This part is *not* available from the
  current state through a linear readout, consistent with §3: the state does not retain `prev` well
  enough, and a rank-≤68 linear map of it cannot implement a pair table.

## 5. What the two repaired training regimes plateau on

The previous milestone measured two structurally different repaired regimes at ~6.69 unrestricted
(`olx-form-2` 6.6841; `olx-form-5` 6.6903). This result reframes that plateau: **it is a shared
output-layer ceiling at this state**, and the plateau is *not* evidence of a representational limit of
the recurrence. That correction matters because the two arms shared the trainer, the output block and
the objective family; they were one budget point, not two independent ceilings.

## 6. Instrument negatives, recorded

- **Post-hoc re-pricing of a fitted readout into the servable alphabets is destructive and is not a
  valid class ceiling.** On the same states, the converged float readout at 5.5136 re-prices to
  **8.6058** (ternary, fitted per-row scales), **7.6181** (ternary with fitted offsets) and **8.9683**
  (4-bit) — all far *worse* than the artifact's own in-class-trained 6.055956. Naive quantisation of a
  float solution is not evidence about what the class can express; only in-class training is.
- **An unexplained initialisation discrepancy.** The artifact-initialised probe's epoch-0 dev loss is
  **6.5954**, which *exceeds* `A` = 6.055956. Renormalising from the artifact's full legal set onto the
  probe's class set must make the loss *lower*, so that inequality is impossible under the intended
  mapping, even though the initialisation's logits were verified to match the artifact's readout
  (`worst absolute deviation 0.0` on 128 fit + 128 dev positions). The cause was not isolated inside
  this run. **Nothing in this record is built on the epoch-0 value or on the epoch-0-to-refined drop**;
  the conclusions rest on the converged refined value versus `A`, on `E1`/`C`, and on the decode ladder.
  Reconciling the mapping is a declared obligation for the next probe run.
- The K=64 run covers only 45.9% of development targets (the frequent-token subpopulation) and is
  reported as directional context only: there `A` = 4.5599, `P` = 3.8515, `E1` = 4.5222, `C` = 3.7902,
  `U` = 6.7077 — the same ordering.

## Limits and claim boundaries

Measured: the state's local content by a training-free decode; the headroom of a converged float linear
readout over the artifact's own readout on the same input; the artifact's standing against a one-token
and a two-token count model on the same population; the failure of post-hoc re-pricing as a class
ceiling. **Scoped to** `olx-form-2` (the prose-only window-objective arm), the served window
conditioning, teacher-forced states, one corpus, one split, one seed, and the top-1024 target
population. The probe is a *linear* readout of the state, so its loss is an upper bound on what a linear
readout extracts and no bound on nonlinear readouts; no per-document cluster interval is computed for
the probe quantities (the paired-by-position comparison and the measured matched-arm noise scale of
0.03–0.08 bits are the uncertainty argument, and a ≥0.40-bit effect is several times that). The
epoch-0 discrepancy above is unresolved. Greedy generation still collapses, sampling is NOT_RUN, and
energy is UNAVAILABLE; the probe is a float diagnostic and is not a serving or product dependency. No
general-language, reasoning or geometric-advantage claim follows, and the state-side negative applies
to *first-order* local retention only.

## Resources

The probe mode plus one intermediate extension, across three implementation rounds and one independent
K3 review. Measured machine time: `olx-probe-2` 1,052 s, `olx-probe-3` 1,292 s, `olx-probe-4` 1,160 s,
four sealed smoke runs ≈ 1,120 s, the interrupted `olx-probe-1` attempt ≈ 840 s, builds and tests
≈ 400 s — about **5,900 s ≈ 1.6 h** of machine time. One process at a time, two threads, peak RAM well
under 1 GiB; new retained sealed evidence a few megabytes. Physical free space stayed above the 24 GiB
working reserve plus the 128 MiB stop margin. The complete projection, the two prospective extensions
and the final charge are in the [resource ledger](resource-ledger-2026-09-19.md). No paid or external
compute; no unique artifact or preserved root was modified.

## One evidence-supported next milestone

**Realise the output-side headroom in class, on the same state.** Freeze the recurrence and the
embedding table, and train the served readout — in the served alphabet, with the frozen-state data —
to the level the converged float readout already demonstrates is available, keeping the grounded
Copy/Stop responsibility and the served window conditioning. Pre-declare the acceptance *before* the
fit: served development loss below `olx-form-2`'s **6.6841** by at least **0.30 bits** with preflight A
32/32, held-out 3/3 and class 4/4 retained, and `--probe-init artifact` epoch-0 reconciled. In-class
training, not post-hoc re-pricing, is the only valid route (§6); if in-class training fails to recover
the headroom, the next branch is the second-order one — giving the state or the readout explicit access
to the two-token context, since §3 shows the state does not retain `prev`. Signed H4/shared geometric
transport remains conditional on a witnessed order/role/distant-interference failure against an
information- and compute-matched ordinary control, and nothing measured here witnesses one.
