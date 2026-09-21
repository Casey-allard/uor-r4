# Matched relational learning: the refit recovers the relation, the controller's margin does not

Executed September 21, 2026 from base revision `472767dc` (PR #1324 merged). Prospective design and frozen
decision criteria: [relational-learning-design-2026-09-21.md](relational-learning-design-2026-09-21.md),
recorded before implementation. Machine-readable receipt:
[../evidence/native_geometric_relational_learning_2026-09-21.txt](../evidence/native_geometric_relational_learning_2026-09-21.txt).
Delivered sealed root: `.uor-models/realtext-prior-2026-09-20/relational-learning-4` (claimed, sealed,
verified, no unlisted files). Superseded attempts `relational-learning-1..3` are retained and never
resealed; each had one reporting/control defect and no measurement difference (see the design note).

This is a component-learning result on a repaired constructed population plus a document-separated local
reader-text evaluation. It is **not** a promotion, **not** general language capability, and **not** a claim
that ranking, absence handling or geometric uniqueness are solved.

## The constructive change

The parent schedule was scalar fit, then discrete descriptor refinement, then quantize — the scalars were
never refitted after the descriptor map moved, so they were stale with respect to the new assignments. The
run uses a **bounded alternating schedule** with identical dose and selection for every learned arm:

1. scalar fit, 2. descriptor refinement, 3. **scalar refit**, 4. descriptor refinement again,

with the phase chosen by the **exported hard decision quality** on a declared development construction
(`SEED_TUNE`, the fit payload bank, disjoint from the fit and fresh draws), not by the soft surrogate.

## The result, phase by phase

Exported development loss (nats, lower is better) and soft objective by phase:

| Arm | scalar fit | refine 1 | **refit** | refine 2 | selected |
| --- | ---: | ---: | ---: | ---: | --- |
| exact | −1044.84 | −1044.84 | −1044.84 | −1044.84 | scalar fit |
| categorical | −1135.65 | −1220.80 | **−1343.94** | −1408.89 | refine 2 (162 assignments moved, 91,392 evaluations) |
| relational | −1079.49 | **−1040.77** | **−1354.60** | −1403.16 | refine 2 (150 moved, 91,392 evaluations) |

Descriptor refinement alone made the geometric arm's **exported** decision quality *worse* (−1079.49 →
−1040.77) because the rank/feature/strength scores were stale. The refit repaired it and exceeded it
(−1354.60), and a second refinement reached −1403.16. The review's untested optimisation gap is real and
material.

## Fresh construction (1,813 candidate-bearing positions, 140 sequences, 21 absent queries)

| Arm | Hard CE, bits/position | reads | correct reads | covered decision success | correct emitted |
| --- | ---: | ---: | ---: | ---: | ---: |
| local | 12.6845 | 0 | 0 | — | 1 |
| exact | 10.7286 | 672 | 353 | 0.5464 | 280 |
| categorical | **10.0644** | 742 | 462 | 0.7152 | 371 |
| categorical_ctx | 10.0674 | 811 | 465 | 0.7198 | 370 |
| relational | 10.2037 | 856 | 448 | 0.6935 | 360 |
| **relational_ctx** | 10.1916 | 810 | 447 | 0.6920 | 359 |
| relational_ctx lesion | 10.7286 | 672 | 353 | 0.5464 | 280 |

All arms are the **independently reloaded** artifacts; evaluation, generation, interventions and timing all
call the reloaded selectors through the one shared `read_step`/`predict_next` path.

| Comparison (fresh, paired by sequence, 2,000 draws) | point | interval |
| --- | ---: | ---: |
| relational − exact | **−0.524968** | [−0.654447, −0.396747] |
| relational_ctx − relational (all positions) | −0.012093 | [−0.023107, +0.003536] |
| relational_ctx − relational (final present) | **0.0** | [0.0, 0.0] |
| relational_ctx − relational (final absent) | −0.002299 | [−0.005914, 0.0] |
| relational_ctx − relation-channel lesion | −0.537061 | [−0.658622, −0.405202] |
| **categorical_ctx − relational_ctx** | **−0.124149** | [−0.227536, −0.022590] |
| categorical_ctx − relational_ctx (final present) | −0.890533 | [−1.683652, −0.070554] |

### What this means, stated plainly

1. **The earlier "H4 relation is not better than exact recurrence" finding was an optimisation artefact.**
   On the same repaired construction the parent measured −0.010151 [−0.104470, +0.083675]; after the refit
   the geometric arm is **−0.524968 [−0.654447, −0.396747]** better than exact recurrence. The relation was
   never represented badly here; the scalars reading it were stale. The retired capacity diagnosis is not
   revived, and no capacity increase is justified by this run.
2. **Final-query outcomes improve sharply.** On the 119 present final queries `relational_ctx` reaches
   **4.0912 bits/query** with **103/119** correct payload reads and **83/119** correct emitted next tokens
   (parent: 9.08857 bits, 49 correct reads, 35 correct emitted). Absent queries remain wrong: 0 correct
   reads and 0 correct emissions at 21 positions, hard CE 14.186 bits.
3. **The contextual interaction's marginal value is no longer resolved.** `relational_ctx − relational` is
   −0.012093 [−0.023107, +0.003536], spanning zero, and the two arms make **identical decisions at all 119
   present final queries** (0 differing payload/strength/emitted triples). So the parent's
   −0.257644 strength win was largely the stale-scalar artefact surfacing as a strength correction. The
   preregistered margin (−0.05) is **not** met: `retained_component_positive = false`. The controller is
   retained as a working, correctly-fitted component; it is not promoted on this evidence.
4. **No unique geometric benefit.** With the *same* contextual procedure fitted to each source arm, the
   nongeometric categorical arm is significantly **better**: −0.124149 [−0.227536, −0.022590] overall and
   −0.890533 [−1.683652, −0.070554] on present final queries. A claim of unique H4 advantage is therefore
   not supported, again, and this time under a matched comparison rather than a lesion.
5. **The relation channel is load-bearing inside the geometric arm.** Removing it costs −0.537061
   [−0.658622, −0.405202], so the geometric arm's advantage over exact recurrence really is carried by
   `rank[rel]`; it is simply not better than the learned categorical code map.

## Attribution, repaired

Every position records the **ungated top source** (argmax of the ctx-free source score), its exact
`(seq, abs)` reference, the served reference, payload, bucket, action and emitted token. **Any correct
payload counts as correct**; no first-matching occurrence index is used anywhere. Per-position verification
found **0 of 1,813** read actions where the served source differed from the ungated top source, so the
factorized regret identity applies and is exact:

| Arm | ranking | gate | dose | actual | Lpool | covered |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| relational | 0.5999 | 0.2092 | 0.1212 | −1.7196 | −2.6500 | 646 |
| relational_ctx | 0.5999 | 0.2136 | 0.1086 | −1.7279 | −2.6500 | 646 |
| categorical_ctx | 0.5379 | 0.1997 | 0.0984 | −1.8140 | −2.6500 | 646 |

(nats per candidate-bearing position; `ranking + gate + dose = actual − Lpool` exactly, asserted by a
focused test and verified per position.) Ranking regret is **identical** between `relational` and
`relational_ctx`, which independently confirms that the interaction cannot reorder sources. `Lpool` is
identical across arms because it depends only on the admitted pool and the target, and only **646 of 1,813**
positions admit the correct payload — so **coverage, not ordering, is the largest single loss**. `Lpool`
cannot see a source excluded by admission, and is reported as a bound, never as a serving feature.

## Document-separated reader text

Reader text fit and evaluation use **disjoint documents** (path and content-hash disjointness asserted
before fitting), one 64-token window each, 8 documents per side, 512 fit tokens / 214 fit candidate
positions.

| Panel | Arm | Δ bits/token vs local | document interval |
| --- | --- | ---: | ---: |
| **held out** | relational | +0.2709 | [0.1695, 0.4048] |
| **held out** | relational_ctx | +0.2742 | [0.1747, 0.4060] |
| **held out** | categorical_ctx | +0.2422 | [0.1197, 0.3809] |
| fit-document regression | categorical_ctx | +0.0577 | [−0.0167, 0.1239] |

All held-out deltas are **positive and above the 0.05 tolerance**: the reader still harms held-out prose, and
the contextual arm is not better than its global-strength parent there (+0.2742 vs +0.2709, within the
declared 0.05 tradeoff). The frozen local prior's exposure to this pinned corpus is disclosed separately and
is not removed by the split; this is a reader-held-out local evaluation, not an externally unseen claim.

## Causal controls and instruments

- **Selected-source intervention** (reloaded contextual arm): 113 attempts, **112** selected the new payload,
  **95** emitted it, **0 lost support**, **0 NoRead**; unrelated-source control selected it **1/102**;
  read-disabled emission changed on 90 of the same prefixes. The occurrence mutated is the one the exported
  policy actually selected, outside the four-token query context, never a first target-bearing substitute.
- **One shared inference path**: `predict_next`/`read_step` matches the independent score construction on 203
  positions of the reloaded contextual arm. **Future-token invariance**: 876 positions compared including 131
  changed-token predecessors, all unchanged. **Stale reference after reset**: resolves before, not after.
  **Independent S query-only reconstruction**: 0 mismatches / 1,813. **Per-arm reload parity**: 0 failures.
  **Zero-table parity**: an all-zero interaction table chooses identically to each arm's baseline on every
  fit and fresh position.
- **Expressivity witness** (test-only; no artifact parameter comes from it): the binary icosahedral centre is
  the unique non-identity involution, `g^-1(−g) = −1` in both partner directions, and the reviewed
  `rank[−1]=+7` / exact-key `+7` / NoRead `7` selector separates the authored present and absent final
  queries with strict tie behaviour, selecting the correct payload with the weakest tied strength.
- **Regret identity** asserted on 400 random positions plus real constructed observations.

## Cost

Active-candidate probe with representative admission (fresh construction stream, 15 predictions,
**1.67 candidates admitted per prediction**, 5–7 reads):

| Row | predictions | median | µs/prediction | admitted/pred | reads |
| --- | ---: | ---: | ---: | ---: | ---: |
| local, no reader work (same stream) | 15 | 22.10 ms | 1473.3 | 0.00 | 0 |
| reader, global strength | 15 | 20.41 ms | 1360.7 | 1.67 | 5 |
| reader, contextual | 15 | 20.86 ms | 1390.8 | 1.67 | 5 |
| reader, categorical+contextual | 15 | 20.67 ms | 1378.0 | 1.67 | 7 |
| local, no reader work (text window) | 29 | 44.98 ms | 1551.1 | 0.00 | 0 |

**The reader increment is not resolved by this probe.** The uncached local E computation dominates every row
(~1.3–1.5 ms/prediction), the spread between rows is within repeat noise, and the reader rows measured
*slightly faster* than the matched local row, which cannot be a real saving and is reported as noise. This is
a real total-path measurement with real admission, not an optimised serving comparison; a token-pair cache is
the obvious prerequisite for a matched measurement. Resident vs serialized: the contextual table is 256
resident entry bytes (16 `[i32;4]` rows plus Vec metadata/capacity) versus **64 serialized bytes**; the
per-arm artifacts are 4,344 (relational), 4,408 (relational_ctx), 8,440 (categorical), 8,504
(categorical_ctx). Physical energy `UNAVAILABLE`.

## Generation

Adjacent-token repetition falls for the geometric arms (e.g. 35 → 7 on prompt 0) but all four arms remain
degenerate; `categorical_ctx` collapses to the local repetition on several prompts. A teacher-forced CE gain
is not general language capability.

## Decision

Under the design note's frozen rule: `retained_component_positive = false` (the contextual interaction does
not clear its margin and changes no present-final-query decision); `unique_geometric_benefit = false`
(categorical+contextual is significantly better); `all_noread_collapse = false` (read rate 0.447);
instrument checks pass. The constructive learning result is the **alternating refit**, which recovers a
significant relation-versus-recurrence advantage and a large final-query improvement, and it is retained.
Held-out prose harm, absent-query handling, admission coverage and geometric uniqueness all remain open.

## Next step

One justified successor: **raise admission coverage before touching capacity or geometry** — on the fresh
draw only 646 of 1,813 positions admit the correct payload while `ranking + gate + dose` account for at most
~0.93 nats/position within the pool. A prospective, matched comparison of one bounded broadening of the
causal admission rule (same ring, same ≤24 candidates, same shared path, same fitting schedule) against the
current pool, scored on the admitted-correct rate and on held-out document loss, would test the largest
remaining measured loss rather than the smallest.
