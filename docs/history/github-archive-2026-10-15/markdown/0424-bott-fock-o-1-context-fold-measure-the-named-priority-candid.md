# #424 — Bott-Fock O(1) context fold: measure the named priority candidate for the long-context gap

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-05T01:04:48Z
- **Updated:** 2026-08-07T07:34:04Z
- **Closed:** 2026-08-07T07:34:04Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/424
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The long-range context gap is measured and named (graph slice 1.23% top-1, generalization below unigram null #390) but has NO live carrier: FMM measured dead (#290 RESULT-52), session lane inert (#247), window+penalty pure cost on fixture (#381). The #249 deferral record names bott_fock 'the priority candidate' but its tracking pointer (#234) has closed without the comparison ever running — stale-pointer fix + revival (completeness review 2026-08-05). First measurement: feature-flagged certifier-side A/B of a Bott-Fock-folded context signal vs the 32-token window on the #375 window-on Gate C surface.

## Comments (3)

### comment 5212676024 — 2026-08-07T05:09:46Z — Casey-allard

Level-set (no work started, definition sharpened so this is pick-up-able cold).

**Still genuinely unmeasured.** `crates/uor-r4-core/src/transformerless/bott_fock.rs` exists (from #107) and `docs/r4_furey_quantum_geometric_plan.md` names it "the priority candidate" for the long-context gap; `docs/deferral_record_2026_08_05.md` records it as implemented, unit-tested, and unused by the shipped runtime, whose context is still the hard 8-token window. The existing tests measure **cost only** — `context_scaling_benchmarks.rs` asserts state length 256 and under 50 microseconds per token — plus determinism, convergence, order-sensitivity, and the P-4 opcode scan. **No accuracy number for bott_fock exists anywhere in the tree.**

**Concrete remaining work:** activate `BottFockContextStore` behind a feature flag as an alternate context carrier and run a pinned A/B against the hard 8-token window baseline (top-1, top-8, teacher bits) with a pre-declared exit rule — the same activation-condition pattern the deferral record uses for the Spin(4) engine.

Worth stating plainly for whoever picks it up: the long-context gap this targets has no live carrier at all (FMM measured dead in #290, session lane inert in #247, window+penalty pure cost in #381), so a negative here closes the gap question rather than merely one candidate.

### comment 5213662629 — 2026-08-07T07:02:23Z — Casey-allard

Reachability gate run (AGENTS.md gate one). **The A/B this issue specifies is not reachable and was not launched.** PR #475 ships the arithmetic as two instruments so the finding is re-checkable rather than a claim in a comment.

```
metric to move:       Gate C top-1 via a long-range context carrier
reachability ceiling: +1.02pp for ANY carrier on this corpus;
                      +0.16pp for the fold at its shipped decay constant
instrument + verdict: long_range_ceiling.rs -> "MECHANISM SOUND, CONSTANT WRONG"
exit rule:            ceiling >= 0.5pp AND shipped decay retains >= half -> A/B
if positive:          activate behind the flag, run the Gate C A/B
if negative:          retune the decay first, or park the gap question
cost estimate:        41s, vs hours for the A/B it replaced
```

**Two questions, separated, because they have different answers.**

**1. Is there long-range signal here?** Yes, and it is worth about **+1.02pp** of top-1 — real but small. On the pinned bundle `blake3:194db0ee…` (3,000 stories / 361,693 positions, document-level split), against a backoff n-gram baseline at 21.62% ± 0.16pp:

| Arm | best λ | top-1 | GAIN | null gain |
|---|---:|---:|---:|---:|
| BASE (window only) | — | 21.62% | — | — |
| CACHE (order-free bag of distant tokens) | 0.05 | 21.99% | +0.36pp | +0.01pp |
| INDUCTION (order-sensitive, unbounded horizon) | 1.0 | 22.64% | **+1.02pp** | −1.29pp |

A stranger's document history does not reproduce the gain, so it is genuinely this document's context. **Two thirds of it is order-carried** — an order-free cache cannot collect it. That is a point in `bott_fock`'s favour and the reason this does not close the mechanism out.

**2. Can the fold as shipped collect it?** No. `cell <- cell - (cell >> 2)` is a 3/4 geometric decay, and the influence horizon it produces is hard-bounded:

| lag k | 0 | 8 | 16 | 32 | 48 | **64** |
|---|---:|---:|---:|---:|---:|---:|
| L1 influence on the 256-cell state | 1,363,342 | 134,203 | 14,514 | 141 | 2 | **0** |

A token 64 positions back leaves a **bit-identical** state. ~90% of the fold's representational mass sits inside the eight most recent tokens — the window the runtime already has. (A control test confirms this is decay, not state collapse: 32 distinct long streams still fold to 32 distinct states.)

**Bounding the fold without building it.** Replaying its inductive bias *losslessly* — exact counts, exact distances, no 256-cell bottleneck — is an upper bound on the fold, since anything the fold can express this arm can express and not conversely. Sweeping the decay then separates "wrong mechanism" from "wrong constant":

| decay | horizon (tok) | GAIN | share of ceiling |
|---:|---:|---:|---:|
| **0.75 (shipped)** | 24 | **+0.16pp** | **16%** |
| 0.90 | 66 | +0.32pp | 31% |
| 0.95 | 135 | +0.61pp | 60% |
| 0.97 | 227 | +0.83pp | 82% |
| 0.99 | 687 | +1.06pp | 104% |

Monotone in the horizon, saturating near 0.99. **The mechanism is sound and the constant is wrong.** At the shipped constant the lossless upper bound is one standard error — no A/B could resolve it.

`>> 2` was chosen for the P-4 shift discipline, and any `>> n` satisfies that discipline equally. **`>> 7`** (127/128 ≈ 0.992) lands on the saturation point. The constant that forfeits 84% of the available signal was never a measured choice — it was the first shift that kept the state bounded.

**A number I am retracting, on purpose.** An earlier pass of the harness reported the ceiling as `observed − null` and got **+3.13pp**. That is wrong for this decision: a wrong donor actively damages the baseline, so the margin grows while achievable gain does not, and a deployed system's alternative is *no* distant evidence rather than *wrong* distant evidence. Reporting it would have promised three points no Gate C run could confirm. The harness now keys on `observed − base` with the null as an admission gate. Recorded in `docs/context_horizon_424.md` because the failure mode — a null that flatters by degrading — is not specific to this issue.

**Scope limits.** Documents in this bundle cap at 128 tokens, so this measures long-range dependency *within 128 tokens* and licenses no claim about book-length context. The baseline is weaker than the full serving stack, which makes +1.02pp generous rather than conservative. Two independent implementations agree to 2dp on every arm.

**Disposition — proposing this issue closes on merge of #475, and the long-context gap question closes with it:**

- Do not activate `BottFockContextStore` at the shipped constant.
- The gap the #290 negative left open is real but worth **~1pp**, below the ±2pp thresholds recent exit rules use. It ranks below the open evidence-quality work.
- If it is ever reopened, the first move is one character (`>> 2` → `>> 7`) plus a re-run of `long_range_ceiling.rs`, and only then a flagged A/B. A corpus with documents longer than 128 tokens is the other named follow-up.
- `context_horizon.rs` pins the shipped horizon by assertion, so retuning the constant cannot silently change the mechanism's reach without updating the record.

Leaving assigned until #475 merges, then closing with this as the record unless you want the `>> 7` retune done now as a follow-on.

### comment 5213950651 — 2026-08-07T07:34:03Z — Casey-allard

Closing with the record. PR #475 merged as 8403d49.

**Outcome against the run contract:** the A/B this issue specified was not reachable and was not run. Long-range context on this corpus is worth **+1.02pp** of top-1 to any carrier (two thirds order-carried); the shipped decay constant retains **16%** of that, so the lossless upper bound on `BottFockContextStore` as shipped is **+0.16pp — one standard error**. Mechanism sound, constant wrong.

The long-context gap question that #290's negative left open closes here too: the gap is real but ~1pp, below the ±2pp thresholds recent exit rules use, and it ranks below the open evidence-quality work. `bott_fock.rs` stays implemented, unit-tested, and unused — but no longer 'awaiting a comparison'.

Two named follow-ups if it is ever reopened, neither filed as an issue because neither has an owner or a next action today:
1. retune the decay `>> 2` → `>> 7` (one character, keeps the P-4 shift discipline, lands on the ceiling), re-run `long_range_ceiling.rs`, and only then consider a flagged A/B;
2. re-measure on a corpus whose documents exceed 128 tokens — this bundle caps there, so the ceiling is scoped to long-range *within 128 tokens*.

Records: `docs/context_horizon_424.md`; instruments `crates/uor-r4-core/tests/context_horizon.rs` (always-on, pins the horizon by assertion) and `crates/uor-r4-graph-certify/tests/long_range_ceiling.rs` (`--ignored`, reusable against any corpus via R4_CORPUS_META/RECS).


---
_Comment/review ids in this file: 5212676024, 5213662629, 5213950651_

