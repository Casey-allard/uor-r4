# Result — one learned geometric exact-occurrence reader

**Principal correction, September 20:** read the [source/receipt audit](occurrence-reader-audit-2026-09-20.md) before this historical report. Raw covered correctness is 5/10, not 10/10; the failure includes ranking and copy strength. Slot attribution, causal controls, paired support, provenance and cost claims are corrected there. The original report below is preserved; its one-abstention-signal next action is superseded by the [memory-utility prompt](deepseek-memory-utility-step-2026-09-20.md).

Date: 2026-09-20. Base source `1dae322ce7770673c0046597c10f4361564cd3a0` (merge of PR #1313),
executing [the occurrence-reader prompt](deepseek-occurrence-reader-step-2026-09-20.md). Isolated
worktree `codex/occurrence-reader` at `.worktrees/geometric-query-read`. Retained root
`.uor-models/realtext-prior-2026-09-20/occurrence-reader-4` (21 files, 3,385,009 bytes, sealed and
verified, 0 unlisted members). Three earlier roots are preserved sealed: `occurrence-reader-1`
(uncorrected threshold/class balance), `occurrence-reader-2` (corrected, but with a mis-specified
altered-source control) and `occurrence-reader-3` (a duplicate of `-2` written by a stale binary
after a failed build). Continuation roots `occurrence-continuation-1/2/3` are preserved.

**Decision: the mechanism works as an exact-occurrence selector and the geometric feature carries
accuracy the exact-identity control cannot — but abstention is uncalibrated, so the reader is not
usable on natural text yet.** Targeted selection passes its predeclared endpoint; both declared
all-position non-regression tolerances fail. No candidate is promoted.

## The mechanism

`z_local(v) = z_E(v) + u_S(b)` where `b` is the frozen separable artifact's query state and the
residual is applied only where S's own `inference_rows` is `Some`, so `z_local` reproduces the
S-query-only baseline exactly and **executes no history fold** (verified: read-disabled equals
`z_local` at all 978 checked positions).

A 128-token causal ring stores observed tokens with a session sequence id. At each position the
reader admits at most 24 past occurrences of the current token, nine 0/1 features are evaluated, a
ten-scalar signed ≤4-bit selector scores each candidate against an explicit `NoRead` threshold, and
the selected occurrence's **exact observed successor** is added as `+2^amp_shift` on that token. No
payload is reconstructed from a geometric code.

Geometric content is `f2`/`f3` (equality of the transported write slot `A(t) = gamma_S[a_codes_S[t]]`)
and the collision feature `f8 = f2 ∧ ¬f0`.

## Construction population (through the real BPE path)

Every role/key/value is a **genuine single-token word** from the real 4,096 vocabulary that
round-trips through `decode`/`encode`. Layout: `M` assignment blocks `(role, key, value)` then a query
`(role, key)` whose successor is the answer. A decoy block repeats the query key with another role and
another value, and is placed after the answer in half the sequences so recency alone cannot solve it.

| Family | Definition | Positions (fresh) |
| --- | --- | --- |
| `exact_role` | the answer block's role token equals the query role | 2,271 |
| `geom_slot_only` | the answer block's role is a **different token with the same transported write slot**; the decoy's role is in another slot | 1,338 |

In `geom_slot_only` the exact-identity feature cannot identify the correct source; only `f2` can. That
is what makes the geometric claim testable rather than decorative.

Fresh panel: seed `0x5A17F2E5`, frozen before scoring, using **12 held-out value ids never used as
payloads during fitting**. 240 sequences / 3,609 positions / 443 reader-relevant.

## Primary endpoint (predeclared)

Hard accuracy on **reader-relevant** positions (local argmax wrong, some admitted candidate covers the
target) of the geometric family, fresh panel, paired by sequence, 2,000 draws:

| Condition | fresh `geom_slot_only`, reader-relevant |
| --- | ---: |
| `z_local` baseline | **0.000** |
| **learned reader** | **0.825** |
| fixed latest-occurrence selector | 0.533 |
| geometry-disabled (same weights, `w2=w3=w7=w8=0`) | 0.550 |
| fixed exact-role-match selector | 0.008 |

**reader − local = +0.825, interval [0.768, 0.880]** — clears the predeclared 0.20 margin with a
positive lower bound. The reader also beats the latest-occurrence control (0.533) and the
geometry-disabled control (0.550) by ≥ 0.20, so the **transported-slot feature contributes +0.275
absolute** over the same weights without it. On `exact_role` positions the reader is 0.138 versus
0.134-0.143 for its controls — as expected, since exact identity already suffices there.

## The negative: abstention is uncalibrated

| Panel | positions | `z_local` | reader | delta | declared tolerance |
| --- | ---: | ---: | ---: | ---: | --- |
| fresh synthetic, all positions | 3,609 | 11.7806 | 12.4519 | **+0.6713** | +0.01 → **FAIL** |
| bounded raw text, all positions | 488 | 6.8002 | 9.5892 | **+2.7891** | +0.02 → **FAIL** |

On raw text the reader admits 88 candidates and **reads at all 88**, with **83 false copies**
(`covering_and_read` = 10). Coverage is only 10/488, so the admitted-but-uncovered stratum dominates
and the selector has no signal distinguishing "a key recurred" from "a key recurred *and* its
successor is the answer". Because the amplitude (2^14, chosen on fit/tune margins) swamps the raw
score range, every false read is catastrophic, not merely wrong.

The failure is **precision, not recall**: on raw text the reader found the covering candidate at 10 of
10 covered positions and copied the *wrong* payload on 83 of the remaining 78 admitted-but-uncovered
positions. A selector trained only on a construction where admitted nearly implies useful cannot learn
that abstention.

## Evolution of the mechanism (preserved near misses)

| Version | Change | fresh `geom_slot_only` relevant accuracy | raw-text delta |
| --- | --- | ---: | ---: |
| `-1` | surrogate fit only; quantized μ = 1 | 0.400 | +0.72 |
| `-2` | declared class balance + tune-only μ calibration (μ = −7) | **0.825** | +2.79 |
| `-4` | corrected altered-source control + corrected decision predicate | 0.825 | +2.79 |

Version 1's failure had a precise cause: the softmax surrogate is not the hard endpoint, and after
scaling to signed 4-bit the threshold landed exactly on the correct candidate's score when the
correct source was the newest (`score = f2 + f8 + f6 = 4 + 1 − 4 = 1`, `μ = 1`, and the comparison is
strict), so the reader abstained exactly when it should have read. Calibrating the threshold on tune
fixed it. Version 2's μ was chosen by a **declared tune-only sweep**; the weights are never changed by
it and the sweep is reported.

## Controls

| Control | Result |
| --- | --- |
| ReadDisabled equals `z_local` exactly | **PASS** (978 positions) |
| Future-token causality: appending/changing later tokens leaves earlier decisions unchanged | **PASS** |
| Stale reference after reset rejected by sequence identity | **PASS** (resolved before reset, rejected after) |
| Altered source payload, query and prefix unchanged | **PASS** (38 changed-source positions, 37 emitted the new payload, 0 abstained) |
| Export/reload decision parity | **0 mismatches** |
| Cross-process continuation (3 processes, 3+3 vs 6 updates) | **PASS** — final parameters, exported selector and checkpoint bytes identical |

The altered-source control was **mis-specified in versions 1–3** (it rewrote the target position
instead of the source occurrence's successor and reported 0/23). It is corrected here; the version-2
number is not evidence of anything.

## Generation and cost

Six retained prompts, 48 greedy tokens through the same integer path: all six repeat, with
pair-cycle periods 13/20/4/1/1/1 and no complete bounded-ring cycle. Repetition is reported, not read
as language quality. The synthetic task is single-token prediction; **no multi-token continuation
integration exists yet** and that boundary is named rather than implied.

Uncached declared serving path (E computed per position, no token-pair cache, ring and selection
included, 32-token real window, one discarded warm-up then five repeats):

| Path | median | per position |
| --- | ---: | ---: |
| `z_local` only | 0.03889 s | 1,215 µs |
| `z_local` + reader | 0.03884 s | 1,214 µs |

The reader adds **no measurable cost** here (ring scan + ≤24 candidate feature vectors). Serialized
bytes: parent E 454,788 + local query artifact 53,555 + occurrence artifact **130** = 508,473.
Resident: local row table 1,966,080 B, parent scratch 16,384 B, ring 512 B, selector 40 B. Whole-run
peak RSS 142,458,880 B; elapsed 110.4 s. **Physical energy UNAVAILABLE.**

## Deviations from the sketch, with reasons

1. **Amplitude is a declared constant, not learned** — the prompt permits a fit/tune-chosen shift and
   requires the two failure modes be separated. The raw-text result shows the constant is the wrong
   *scope*, which is now measured rather than hidden.
2. **All features are 0/1**, so the served score is a sequence of conditional adds: no multiplier.
3. **No reuse of the historical `Model`/`ValueState` path**, only its invariants (relative element plus
   rank table, explicit no-source, selection separated from payload access).
4. **A second, geometry-only family was added** beyond the sketched role/decoy construction. Without
   it the exact-identity feature solves the panel and the geometric hypothesis is untestable.

## Boundaries

A successful synthetic exact-copy panel is an instrument check, not prose, reasoning or instruction
following. `geom_slot_only` accuracy is measured on 120 fresh reader-relevant positions in 240
sequences; the interval is nominal over correlated constructions, not independent semantic domains.
The raw-text probe is 8 open-development windows. No alpha, conversation, coding, frontier or energy
claim follows. Reset/Continue maintenance was not implemented and is not claimed.
