# #1288 — Testbed exhausted as an instrument; real-text projection recorded (2h, unused)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-19T20:29:55Z
- **Updated:** 2026-09-19T20:42:55Z
- **Closed:** 2026-09-19T20:30:58Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1288
- **Merged:** 2026-09-19T20:30:58Z
- **Merge commit:** 037e5b8f2493dc1053e4b6b2bcbcd86a383756a0
- **Base / Head:** main / UOR-Foundation:codex/testbed-exhaustion
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## The synthetic-task well is exhausted as an instrument; the real-text path is next

Documentation only — no code change, no model time spent.

Three consecutive mechanisms converge on the same conclusion, and it's a property of the **testbed**, not the mechanisms:

- The **bounded shortlist** (the compute lever: readout is 99.9748% of per-token work; 480× projected) **cannot have its accuracy cost measured** — on synthetic tasks the address *determines* the answer, so a shortlist keyed on the address is trivially exact.
- The **hyperbolic substrate** can only be judged on whether it represents a *hierarchy* at equal fidelity with fewer dimensions, which needs real structure.
- **Curvature typing** has nothing to predict while address collisions are zero.

**Further mechanism measurement on this testbed cannot change a decision.** The instrument has done its work: it found the recurrence instability, the address collision, the read-activation defect, the identity-class defect and the objective-level reason the learned filter fails; and it established that composition carries capability (**0.42 vs 0.00** on unseen facts) and that abstention converts hallucination into honesty (**0 ok / 64 abstain / 0 wrong**). What it cannot do is discriminate **cost-versus-accuracy** trades, because its answers are structural rather than statistical.

## Projection recorded before use (standing authorization, 2026-09-06)

| Item | Estimate |
|---|---|
| Tokenizer derivation + corpus + packing (engineering, not model time) | ~40 min wall |
| Training: ~2,000 steps, batch 32, ~64-token sequences (~4.1M tokens), `V=4096`, `dv=128` | ~1,800 s |
| Held-out evaluation + raw generation reading | ~300 s |
| Contingency (diagnose and re-run, no blind repeats) | ~1,800 s |
| **Total** | **~3,900 s ≈ 65 min** |

**Increment: 7,200,000 ms (2 h); updated limit 154,400,000 ms; remaining 8,221,435 ms (~137 min).** Recorded but **not spent** — so the next block can draw on it without re-approval. Storage: tokenizer (<1 MB) + packed corpus (~10 MB), recreatable; 128 MiB stop margin untouched.

## Also reverted from the roadmap on this evidence

Hyperbolic arithmetic at serving (costs *more* per operation than an add); R8 expansion *for accuracy* (precision is measured as not the accuracy constraint); Hamming as a semantic similarity metric (already falsified in-project). The hierarchy survives in its useful form — **bounding the readout** — still a projection with an unmeasured accuracy cost, labelled as such in the receipt, the test, `EVIDENCE.md` and `current-state.md`.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

