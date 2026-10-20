# #1325 — Matched relational learning: the refit recovers the relation, the controller margin does not (References #973)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-21T04:34:39Z
- **Updated:** 2026-10-09T13:17:52Z
- **Closed:** 2026-09-21T04:36:27Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1325
- **Merged:** 2026-09-21T04:36:27Z
- **Merge commit:** f80fdbdf56a18df848311b5924573396fb25c1a7
- **Base / Head:** main / UOR-Foundation:codex/relational-learning
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

> **Principal correction, PR #1327:** the source-learning improvement survives. The [review](https://github.com/UOR-Foundation/uor-r4/blob/aac25404dc0ff0a3da7c2c821ab71609bb0c2e39/docs/integration/relational-learning-review-2026-09-21.md) corrects admission dominance, reused construction, exclusive refit attribution, control scope and artifact/continuation claims. The [successor](https://github.com/UOR-Foundation/uor-r4/blob/aac25404dc0ff0a3da7c2c821ab71609bb0c2e39/docs/integration/deepseek-reader-utility-transfer-step-2026-09-21.md) tests useful language read influence; admission is conditional on full-stream opportunity. Original report below is preserved.

## What this delivers

One constructive learning step: **learn the useful source relation already expressible in signed H4**, keep the contextual utility controller, add an **equally trained categorical+contextual comparator**, and use **real document-separated reader text**. The attribution, artifact and causal-control repairs the principal review asked for are integrated into this run.

References #973, #820, #963, #964. **No issue is closed.**

### The constructive change

The parent schedule fitted scalars, refined descriptors, then quantized — the scalars were never refitted after the descriptor map moved. This PR uses a **bounded alternating schedule** with identical dose and selection for every learned arm: scalar fit, descriptor refinement, **scalar refit**, descriptor refinement, with the phase chosen by the **exported hard decision quality** on a declared development construction (`SEED_TUNE`, fit payload bank, disjoint from fit and fresh).

### The decisive result

| Exported development loss (nats) | scalar fit | refine 1 | **refit** | refine 2 |
| --- | ---: | ---: | ---: | ---: |
| categorical | −1135.65 | −1220.80 | **−1343.94** | −1408.89 |
| relational | −1079.49 | **−1040.77** | **−1354.60** | −1403.16 |

Descriptor refinement alone made the geometric arm's exported quality **worse**; the refit repaired and surpassed it. Consequences on the fresh construction:

- geometric global-strength loss **10.7185 → 10.2037 bits/position**
- **relational − exact = −0.524968 [−0.654447, −0.396747]** — where the pre-refit parent measured **−0.010151 [−0.104470, +0.083675]** on the same repaired construction. The earlier "not better than exact recurrence" reading was an **optimisation artefact, not a representation limit**. The retired capacity diagnosis is not revived and no capacity increase is licensed.
- present final queries: **4.0912 bits/query**, **103/119** correct payload reads, **83/119** correct emitted tokens (parent: 9.08857, 49, 35).

### Both preregistered margins fail

- `relational_ctx − relational` = **−0.012093 [−0.023107, +0.003536]**, spanning zero, with **zero** differing decisions across all 119 present final queries. `retained_component_positive = false`: the parent's −0.257644 strength gain was largely the stale-scalar artefact. The controller is retained as a working, correctly-fitted component — not promoted.
- The matched nongeometric `categorical_ctx` is **significantly better**: −0.124149 [−0.227536, −0.022590] overall and −0.890533 [−1.683652, −0.070554] on present queries, so `unique_geometric_benefit = false`. The relation channel is still load-bearing *inside* the geometric arm (lesion −0.537061 [−0.658622, −0.405202]).

### Repaired attribution, and what it says

Per position the run saves the **ungated top source**, its exact `(seq, abs)` reference, served source, payload, bucket, action and emitted token. **Any correct payload counts**; no first-matching occurrence index anywhere. **0 of 1,813** read actions had a served source differing from the ungated top source, so the factorized identity is exact and asserted by a focused test: `ranking + gate + dose = actual − Lpool`.

`relational`: ranking 0.5999, gate 0.2092, dose 0.1212, Lpool −2.6500 nats/position. Ranking regret is **identical** with and without the interaction, independently confirming the interaction cannot reorder sources. Only **646 of 1,813** positions admit the correct payload, so **coverage, not ordering, is the largest measured loss**.

### Honest populations and controls

- **Document-separated reader text**: disjoint fit vs held-out documents asserted by path and content hash before fitting. Held-out harm is +0.2709 / +0.2742 / +0.2422 bits/token for global / contextual / categorical-contextual — all above the 0.05 tolerance. The frozen local prior's exposure to this corpus is disclosed and not removed.
- **Selected-source intervention**: 113 attempts, 112 selected the new payload, 95 emitted it, **0 lost support, 0 NoRead**; unrelated-source control 1/102; read-disabled changed 90/113.
- Shared path, future-token invariance (876 compared incl. 131 changed-token predecessors), stale reference, per-arm reload parity, zero-table parity all pass; evaluation/generation/interventions/timing all use the **independently reloaded** artifacts.
- A test-only **expressivity witness** confirms the binary icosahedral centre is the unique non-identity involution, `g^-1(−g) = −1` both ways, and the reviewed signed-4-bit selector separates the authored present/absent final queries with strict tie behaviour. **No artifact parameter comes from it.**
- **Cost**: active-candidate probe with 1.67 candidates admitted/prediction and 5–7 reads, plus a matched no-reader-work baseline. The reader increment is **not resolved** — the uncached local E path dominates (~1.4 ms/prediction) and the spread is within noise; energy `UNAVAILABLE`. Resident 256 table entry bytes vs 64 serialized bytes.
- `binding.json` binds the full observation digest, splits, seeds, configuration, geometry, parent and format versions.

### Tests run

- `cargo test -p uor-r4-core --lib native_geometric::learner::relational` → **23 passed** (3 new)
- `cargo test -p uor-r4-core --bin competitive-reader` → **5 passed**, 1 ignored (1 new)
- `cargo fmt --check`, `cargo check --offline`, `python3 scripts/check_claim_wording.py` clean

Delivered sealed root `.uor-models/realtext-prior-2026-09-20/relational-learning-4` (27 files, 0 unlisted). Attempts 1–3 are preserved superseded reporting/control defects with a **bit-identical** measurement: (1) a shared-path control compared the contextual arm against the global arm's `choose`; (2) the cost timings vector was omitted; (3) the no-reader-work baseline used a different stream. All prior roots preserved untouched.

**Next (one):** a prospective, matched comparison of **one bounded broadening of the causal admission rule** — same 128-token ring, same ≤24 candidates, same shared path, same alternating fitting schedule — against the current pool, scored on the admitted-correct rate and held-out document loss.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

