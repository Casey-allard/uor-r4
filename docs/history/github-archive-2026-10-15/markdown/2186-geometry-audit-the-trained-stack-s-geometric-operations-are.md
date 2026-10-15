# #2186 — Geometry audit: the trained stack's geometric operations are 0.05% of its arithmetic (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-11T00:59:31Z
- **Updated:** 2026-10-11T01:32:23Z
- **Closed:** 2026-10-11T01:03:29Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2186
- **Merged:** 2026-10-11T01:03:28Z
- **Merge commit:** 9812e5f7170bf51d8a8327d5073ef550a1892f43
- **Base / Head:** main / UOR-Foundation:deepseek/geometry-audit-20261011
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**The owner asked why the project keeps becoming matrix wrappers with 1–2 % gains. This is the answer with the artifact's own numbers — a diagnosis and a decision request, not a result.**

Line: geometry audit (owner question, 10 Oct) · count n/a · headline: the trained stack's geometric operations are **0.05 %** of its arithmetic; **0 %** of it is the exact addressed store the project is named for.

## The ledger (deployed 29M artifact: width 576, 8 heads, 10 layers, `rrarrarrar`, MLP 732, context 384)

| component | params | share | **MACs/token** | share |
|---|---:|---:|---:|---:|
| MLP (SwiGLU ×10) | 11.4 M | 37.7 % | 12,648,960 | 39.6 % |
| recurrence projections (×7 layers) | 9.9 M | 32.8 % | 9,886,464 | 31.0 % |
| reads (softmax attention ×3) | 4.0 M | 13.3 % | 5,308,416 | 16.6 % |
| output head | 2.4 M | 7.8 % | 2,359,296 | 7.4 % |
| pointer | 37 k | 0.1 % | 67,584 | 0.2 % |
| **quaternion transport (the geometry)** | **0** | **0 %** | **16,128 multiplies** | **0.05 %** |
| **exact addressed store / prime addressing / zeta / icosian** | **0** | **0 %** | **0 lookups** | **0 %** |

Read share is context-dependent: 16.6 % at the trained 384, **40.5 % at 4,096** — that is where addressing has algorithmic headroom.

## The five connection bugs

**B1 quality-only metrics** — no acceptance criterion anywhere sees compute, so a mechanism that halves the arithmetic and ties on quality is invisible; the repository's own transformerless line already publishes `multiplies/token` and wrote the rule *"a throughput number never travels without its quality number"* (`docs/transformerless/COMPARISON.md`). **B2 geometry is parameterisation, not computation** — each rotation is wrapped in 1.4 M dense weights. **B3 novel mechanisms entered as options beside matched controls and the control won** — the trained base runs `read=l2`, the flat Euclidean ablation. **B4 novelty and critical path are disjoint** — the exact addressed machinery lives in `native_geometric`, unscaled and unserved, and that family's own win does not transfer (437/512 development, **0/128 fresh**, #2164). **B5 serving substituted operations without reducing work and measured slower** — multiplier-free D11 ran **68.0 tok/s** against float's **127.5** on the same artifact, windows and NLL (this session's cycle-5 evidence).

## What it proposes

The only three things that reduce work per token: **touch fewer weights**, **do less search**, **carry less state**. The one mechanism already in the tree that does the first — the product-key memory — removes **86 %** of a replaced layer's multiplies (1,264,896 → 173,056) and sat inside the panel's noise when bolted on for 2,000 steps. Two candidate load-bearing mechanisms are stated with compute claims (address-and-copy output path; exact icosian weights), and the cheapest decisive test is named: a **forced-copy oracle**, which this session's order-3 reading already implicates (pointer hit 0.377 → 0.54 at zero likelihood cost, panel flat).

## Checks

- [x] `python3 scripts/check_claim_wording.py` **pass** at this head.
- [x] Docs-only: one audit, the newest `current-state` entry, one `STATUS.md` row and cell. No source, criterion, panel or grader touched.
- [x] Every number is either read off an artifact on `main`'s line or measured in this session with its record linked.

**Three decisions requested from the owner:** adopt the compute ledger as a required field of every experiment; choose the first load-bearing mechanism; and state which family is the project.

References #2029


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6104224576 — 2026-10-11T01:32:23Z — Casey-allard

Read-only follow-up at merged tree9812e5f7. The central diagnosis is supported by my separate effective-path audit, but three claims need correction before they drive compute decisions:

[P1] docs/integration/geometry-audit-2026-10-11/README.md:147-151: C2 equates snapping the quaternion rotations with making32.8% of parameters multiplier-free. The same record says the rotations themselves have0 parameters and16,128 multiplies, while rec.in/out/gate/conv account for9.9M dense parameters. Icosahedral/icosian rotation coefficients alone do not constrain or remove those surrounding maps. A proposal replacing them by derived shared geometric operators is a different model change and needs its own implementation/quality/cost test. Please scope C2 to the rotation kernel unless the projection replacement is actually specified; retain the mechanism, correct the attribution.

[P2] lines142-145: a soft mixture gate*copy+(1-gate)*head does not generally eliminate the head. Both branches are needed when the gate is nontrivial. The claimed bypass requires an actual exclusive/hard dispatch before head evaluation, with Copy/Generate/Stop decisions and fallback costs accounted for. Forced-copy oracle is a source-consumption diagnosis, not saved-model/predicted-routing acceptance or proof of this bypass.

[P2] table18-25: the listed MAC counts sum to30,270,720, but the shares do not match that denominator. They are approximately41.79% MLP,32.66% recurrence projections,17.54% reads,7.79% head,0.223% pointer. Rotation multiplies are a separately labelled operation count (16,128 / stated MAC total ~=0.0533%), not automatically additional MACs in the same ledger. Please use a consistent denominator and distinguish shape-derived counts from hardware measurements. The qualitative conclusion survives these corrections.

No native acceptance changed and no mechanism is closed by this review. Current SpiralCore arithmetic independently supports signed-permutation lowering, but no Rust/learning/full-path energy gain is inferred from it.


---
_Comment/review ids in this file: 6104224576_

