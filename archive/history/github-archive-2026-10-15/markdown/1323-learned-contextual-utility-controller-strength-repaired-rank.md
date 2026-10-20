# #1323 — Learned contextual utility controller: strength repaired, ranking still open (References #973)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-21T03:26:02Z
- **Updated:** 2026-10-09T13:17:12Z
- **Closed:** 2026-09-21T03:27:52Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1323
- **Merged:** 2026-09-21T03:27:52Z
- **Merge commit:** 29f680fb5c80d9d00af9f9f2fb1e4c3e6205303d
- **Base / Head:** main / UOR-Foundation:codex/contextual-utility
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

> **Principal correction, PR #1324:** the contextual primary gain survives. Read the [review](https://github.com/UOR-Foundation/uor-r4/blob/472767dc7cfacba3a3b0622da5fb2f4221d3ddd5/docs/integration/contextual-utility-review-2026-09-21.md) for corrections to residual attribution, duplicate counts, text overlap, control scope, resident bytes and resource rationale. The [next task](https://github.com/UOR-Foundation/uor-r4/blob/472767dc7cfacba3a3b0622da5fb2f4221d3ddd5/docs/integration/deepseek-relational-learning-step-2026-09-21.md) learns the existing signed relation with fair contextual comparators and document holdout. Original report below is preserved.

## What this delivers

A **learned contextual utility controller** for the geometric reader, executed end to end on the repaired construction, with the preregistered comparison, controls, generation and cost measured.

References #973, #820, #963, #964. **No issue is closed**: the broad capability acceptance is unmet.

### Mechanism

The selector factorised `score(c, a) = bias + rank[rel(c)] + sb[a] + w·feats(c)`. `sb[a]` is one positive bias per action, identical everywhere, so the 8-nat action won wherever a read was taken. This PR adds the smallest useful causal interaction:

```text
A_ctx  = { NoRead } U { a_0, a_1, a_2 }                        (unchanged actions)
bucket = f3 | (f1|f2) << 1 | f5 << 2 | (best > second) << 3    (16 frozen causal buckets)
score(c, a | b) = bias + rank[rel(c)] + sb[a] + w·feats(c) + ctx[b][a]   (a > 0)
noread(b)       = noread + ctx[b][0]
```

NoRead is inside the interaction, so the controller learns *when not to read* as well as *how strongly*. Every `ctx` entry is a signed 4-bit integer added by the same integer kernel that already adds `sb[a]`; no multiplier, float or transcendental enters the served selector. The bucket uses only the admitted pool and the frozen ctx-free source scores, never the target, coverage, labels, family ids or future tokens. Fit order is exactly as preregistered: fit trainer, quantize, **freeze**, then fit only `ctx` by bounded discrete coordinate search from zero, so `ctx = 0` recovers the baseline. `RLR2` moves to v3; v2 stays readable (verified against the retained `competitive-reader-1` artifact).

### Preregistered primary result

[Design](docs/integration/contextual-utility-design-2026-09-20.md) frozen before any held-out result. Contextual minus frozen global strength on the fresh construction:

**-0.257644 bits/candidate position, sequence-cluster 95% interval [-0.327088, -0.182968]** (140 sequences, 2,000 draws), so `positive = true` under all five declared conditions, including that the arm **keeps reading** (0.532 vs 0.365) and that the natural-text tradeoff **improves** (+0.2189 vs +0.2443 bits/token).

### Negatives that are equally load-bearing

- **The H4 relation term does not reproduce its old advantage.** Relational minus exact on the repaired fresh panel is **-0.010151 [-0.104470, +0.083675]**, spanning zero. The retired -0.179540 belonged to the construction whose partner/absence defects the principal review found.
- **The nongeometric categorical comparator now learns and is the strongest static arm**: 149 assignments moved, objective -4485.84 to -4973.43, fresh CE **10.3936** vs geometric 10.7185 and contextual 10.4608; the relation-channel lesion (10.7003) fails to hurt. Unique-H4 benefit remains unestablished.
- **Ranking, not strength, is the dominant residual.** Target-using opportunity bound: mean strength loss **0.0933** nats vs mean ranking/admission loss **1.1939** nats (~12.8x). At 646 covered positions the correct source is top-ranked 46.3% and selected 25.2%, while the best finite action belongs to it 100% of the time.
- **Natural text is still harmful in absolute terms** (+0.2189 bits/token), and the panel is inside the fit mixture, so it is development evidence; the old +1.2908 came from a different fit and construction.
- **Generation stays degenerate** (though adjacent-token repetition falls on several prompts). **Served active-candidate cost and energy remain UNAVAILABLE**: the 32-token probe admitted zero candidates. Whole-path D0-b compliance is not claimed.

### Instruments

All controls pass: one shared `read_step`/`predict_next` path; independent S query-only reconstruction (0 mismatches / 1,813); future-token invariance required at every candidate-bearing predecessor **including** the one immediately before the changed token; stale reference after reset; per-arm reload parity; and a **behavioural** zero-interaction baseline check. The superseded attempt `contextual-utility-1` is retained (never resealed); its only defect was a struct-equality self-check, and repairing it left the measurement bit-identical.

### Tests run

- `cargo test -p uor-r4-core --lib native_geometric::learner::relational` -> **20 passed** (4 new)
- `cargo test -p uor-r4-core --bin competitive-reader` -> **4 passed**, 1 ignored
- the ignored retained-artifact smoke run -> **passed** (real generation matches teacher forcing; v2 artifact loads)
- `cargo fmt --check` clean, `cargo check --offline` clean, `python3 scripts/check_claim_wording.py` passed

Retained sealed root: `.uor-models/realtext-prior-2026-09-20/contextual-utility-2`. All prior roots preserved untouched, including the unlisted helper in `competitive-reader-1`.

**Next step (one):** a prospective matched-cost comparison of the source term at higher resolution (the shared scalar `rank[rel]` over the 120-root descriptor versus a learned directed `(query role, source role)` compatibility) on a fresh construction draw with the utility controller frozen on both sides, judged on covered-position selection success as well as hard-action CE.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

