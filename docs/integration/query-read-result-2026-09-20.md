# Result — matched geometric query read (Q/S/L) against the frozen corrected E

Date: 2026-09-20. Source `92240a8619927d287bb90ee50aeebf0c944bc597` (merge of PR #1309).
Executed in the isolated worktree `codex/geometric-query-read` at
`.worktrees/geometric-query-read`. Sealed report root:
`.uor-models/realtext-prior-2026-09-20/query-read-2` (39 files, 35,737,680 bytes, sealed and
verified). The superseded first complete run is retained sealed at
`.uor-models/realtext-prior-2026-09-20/query-read-1`.

**Decision: the query-conditioned multiplicative read is a valid NEGATIVE at this fixed
architecture and dose.** All instrument checks pass, all three arms improve over E, and the
*separable* arm improves most; the joint `q*b` transport adds no demonstrated value over its
equally parameterized controls. No candidate is promoted.

## What was executed

Three arms add a bounded residual to the frozen corrected empirical head E
(`565cf98a01273af38425687adb522704b09a89bd7ab976b58b20bfbdaccfa0bf`). Identical shape,
initialization, optimizer and dose for every arm; two reader-row accesses and residual bound 4096
each.

```text
Q: (sum_j W[v,j] * (R[q*b, j]      + R[e, j])) << 7
S: (sum_j W[v,j] * (R[q, j]        + R[b, j])) << 7
L: (sum_j W[v,j] * (R[q_tail*b, j] + R[e, j])) << 7
```

`A[4096,8]` write map, `B[4096,8]` query map, `R` 120×16 ternary reader, `W` 4096×16 ternary
output. Reader shift 4, output shift 3, total 7 — a new `CPX3` artifact; the old `CPX2` reader
shift 5/output shift 3 (total 8) is never reinterpreted and a `CPX2` file is rejected by magic.

Settings exactly as prescribed: V=4096, F=10, H=64, at most 62 older tokens, 512 batch-8 updates
per arm with 64 warm-up updates (`R/W` only), one fixed seed-13 Fisher–Yates pass over the same
4096 consumed windows, Adam β=(0.9, 0.999), ε=1e−8, decay 0, R/W lr 0.03, A/B lr 0.003, R/W and
post-Jacobian A/B blocks clipped at 1, master clamp [−1, 1], fixed ternary threshold |master| ≥ 0.5.
`W = 0` at step 0, so every arm reproduces E exactly before the first update.

Population reproduction is exact: **429 eligible documents**, 38,987 fit windows, the recovered
**4096 consumed windows / 257,113 n−1 targets**, and the **36-document / 288-window / 17,342-target**
open-dev panel. Frozen E dev micro CE reproduces to 1e−15:
**7.170815717556** bits/target (expected 7.170815717556).

## Final reloaded results

Every reported number comes from a reloaded `CPX3` artifact after a full-panel integer parity check
(0 mismatches over all 17,342 positions, rows *and* integer logits).

| Arm | Dev micro CE | Dev macro CE | Gain over E, nominal 95% paired interval |
| --- | ---: | ---: | --- |
| Corrected empirical incumbent E | 7.170815718 | — | — |
| Q, query-conditioned older read | 7.141505532 | 7.138598406 | **+0.029310 [0.022866, 0.036005]** |
| S, separable older + query read | 7.032227740 | 7.029358957 | **+0.138588 [0.125831, 0.150844]** |
| L, local-only matched read | 7.101049449 | 7.098201142 | **+0.069766 [0.058944, 0.080294]** |

Paired document bootstrap, the retained 2,000 draws and seed `0x12345678`, ratio of resampled loss
sums over counts. These are nominal repeated open-development comparisons on correlated repository
documents, not fresh final qualification.

## Predeclared screens

| Screen | Rule | Result |
| --- | --- | --- |
| Practical | `CE_E − CE_Q ≥ 0.10` and paired lower bound > 0 | **FAIL** — 0.029310 < 0.10 (lower bound +0.022866) |
| Matched, versus S | `CE_S − CE_Q > 0` and lower bound > 0 | **FAIL** — S is **0.109278** *better* than Q |
| Matched, versus L | `CE_L − CE_Q > 0` and lower bound > 0 | **FAIL** — L is **0.040456** *better* than Q |
| Older content | Q's exact-donor penalty `≥ 0.10` bits/target with lower bound > 0 | **FAIL** — **−0.003830 [−0.012235, +0.004366]** |
| Query-route use | Neutralising Q's query to `b = e` gives a positive penalty with lower bound > 0 | **FAIL** — +0.000322 [−0.002913, +0.003358] |
| E reproduced | within 1e−8 of the recorded value | **PASS** |
| Instrument checks | reload parity, read-disabled ≡ E, and L exact invariance | **PASS** |

Matched controls, all on the reloaded artifact:

- **Read disabled** — every arm equals E exactly (per-occurrence loss vectors byte-identical).
- **Older-prefix donor** — the fixed map grouped by exact `(previous, current, older_length)`,
  recomputed with the *recipient arm's own* learned A: 1,177 eligible observations, 576
  no-history positions excluded, 16,583 strata, 558 (Q) / 564 (S) / 0 (L) changed reads.
- **L exact invariance** — the local-only arm is byte-identical under both the donor map and
  older-order reversal.
- **Query neutralisation** — S's query channel is strongly used (**+0.190660 [0.173466, 0.207857]**)
  and L's is used (**+0.019431 [0.014554, 0.024397]**), while Q's is not distinguishable from zero.

## Interpretation

The project's declared hypothesis was that combining ordered content-dependent transport with a
query would beat an *equally parameterized separable* read. It does not at this dose. The
separable arm already has an active query channel and identical storage; measured on the same
panel it is the strongest of the three, and its query channel is demonstrably used. Q's residual
does beat E (+0.0293, interval clear of zero), so the joint transport is not inert — but the effect
is below the predeclared 0.10-bit practical margin, smaller than both fitted controls, and Q's own
older-content and query-route diagnostics are inconclusive.

The measured behavioural pattern is consistent with the algebra: `q*b` is a right multiplication of
the older register by the query element, so for fixed `b` it permutes 120 addresses and adds no
information to the register; the extra step only changes *which* reader row is addressed. A model
that benefits from older content can obtain that content from the separable `R[q]` row without
conjugating it, and this run shows it did.

Learning moved the hard maps substantially and similarly for all three arms: 2,173/2,241/1,549 of
4,096 write codes and 1,544/1,571/1,581 query codes changed from the seeded initialization. So the
negative is a representation/arrangement result, not an optimizer stall.

## Generation

All six retained prompts, 64 greedy tokens from the reloaded artifacts, no sampler, repetition
penalty, blacklist or output override. Every arm degenerates into a short cycle: five of six prompts
enter a length-1 pair cycle; prompt index 3 repeats its 64-token ring at offset 48 for Q, S and L.
Cycle certificates use the complete bounded token ring plus availability for the context arms, not
the repeated pair alone. **Generation remains degenerate for every arm; the CE improvement does not
produce usable text.**

## Cost

| Quantity | Value |
| --- | ---: |
| `CPX3` artifact bytes per arm | 53,555 |
| Learned parameter bytes per arm (R/W/A/B masters + moments) | 1,595,904 |
| 64-token greedy generation, single prompt, 8 repeats | 0.099 – 0.123 s |
| Older-prefix state fold (62 tokens) | 88.9 – 122.9 ns |
| Local-pair state fold | 2.4 – 2.5 ns |
| Fit-time parent-score cache | bounded at 32,768 entries, peak 536,412,160 B, 2–3 resets |
| Peak RSS, complete run under `/usr/bin/time -l` | 763,002,880 B |
| Complete run wall time | 992.7 s |

The older fold costs roughly 36–51× the local fold at the same prediction; that is a measured
state-fold cost difference, not a serving-latency optimisation. The fit-time parent-score cache is
**not** a serving optimisation: it is explicitly bounded and its recomputation is charged. Whole-run
RSS is not standalone serving RSS. **Physical energy remains UNAVAILABLE** — no compliant numerical
path is an energy measurement.

## Continuation

On-disk split-versus-uninterrupted check: 8 updates from four different initial processes, split at
4 with an intervening `CPQK` checkpoint and resume, compared against an uninterrupted run. Next
batch loss, all eight batch losses and the resulting artifact bytes are equal
(`35209523ac…`). Loading an inference artifact is not training continuation and is not claimed as such.

## Corrections made in this step

1. **`learner/head_projection.rs::project_row` skipped a prescribed seed.** The nearest-code seed
   was swept only when `s != s0`, so the untouched Q0 candidate was returned without coordinate
   optimisation at its own scale. The sweep now runs at **every** admissible shift including `s0`.
   Covered by `nearest_seed_is_swept_at_s0_on_a_correlated_gram`: correlated `G = [[1,.9],[.9,1]]`,
   `w = [.6,.6]`, `s0 = 0` — untouched `[1,1]` has `J = .608`, the sweep reaches `[0,1]` with
   `J = .088`. Restoring the old guard makes the test fail with `[1,1]` versus `[0,1]`, which was
   confirmed by temporarily reverting the fix. **The previously reported full-panel measurements
   remain valid for the reduced search actually executed; the complete originally specified
   projection is still NOT_RUN, and no full QG campaign was rerun.**
2. **`bin/head-projection.rs` wrote `singular: true` literally.** It now writes
   `singularity: "NOT_MEASURED"` and `singular_inputs_supported: true`. No rank was computed.
   Old report bytes are untouched.

## Instrument defect found and corrected before the retained run

The superseded `query-read-1` run mis-specified the local-only arm's *reversal* control: it dropped
that arm's query transport and returned `(q_tail, e)` where the arm's own read is
`(q_tail*b, e)`. The consequence was a spurious `instrument_checks: false`. The model, losses,
artifacts and every other control were unaffected. Fixed in `bin/query-read.rs`, covered by the
existing `local_arm_is_invariant_under_an_older_prefix_change` unit test plus the corrected
control, and re-executed into the fresh root `query-read-2`; the numerical results are identical to
`query-read-1` (E/Q/S/L CE agree to the last printed digit). `query-read-1` is preserved sealed and
is superseded only on this reporting defect.

## Boundaries

No alpha, fluent conversation, coding, frontier-capability, general-prose or energy claim follows.
One 120-state register holds at most `log2(120) ≈ 6.91` bits; fixed-`b` right multiplication only
permutes it. This is a query-conditioned **read**, not a selective write, not a query-conditioned
recurrent update, not a transformer replacement, and not evidence for any particular advantage of
the 2I group over other finite groups. The panel is nominal repeated open development. Prime
identities, zeta phases and paired-H4 capacity are untouched by this experiment.
