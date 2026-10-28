---
id: pipeline-map-revision
title: Revise the pipeline map with claude-149 to claude-155, ranked by cost per token
milestone: research
area: architecture
status: done
open_to: lab
needs_gpu: false
depends_on: [pipeline-map]
done_class: host
scope: [docs/model/pipeline-map.md, PLAN.md, docs/loop/tasks/pipeline-map-revision/**, docs/roadmap/items/pipeline-map-revision.md]
---

## Why it matters

The owner's refocus direction of 27 Oct 2026 is still `open` in PLAN. It asks for exactly what mechanisms the model needs, what exists, in what order, and in which geometric domain, from the first trained token to the last emitted word.

On 28 Oct, in claude's chat, after claude-155's diagnosis, the owner decided:

> "ok yes, my vote is to pivot to your diagnosed recommended new roadmap"

> "determinism is still probably the goal but like you said we can do the training in float and maybe just export the integer exact... either way it is worth looking into the archived hopf golden ratio complex - riemannian bridge we used for the original prime router or angular router"

> "I don't care what we serve at long as it is cheap"

[The pipeline map](../../model/pipeline-map.md) (claude-148) answered the direction's first form. It predates seven units. Its gaps and order of work no longer match what was measured.

## What is missing

- **The map predates claude-149 to claude-155.** In particular:
  - claude-150's one-decoder unit;
  - claude-151's type spin;
  - claude-152's margin objective (cell C on copies);
  - claude-153's width 512 (best fit, cell D);
  - claude-154's opening read: cell D, and the trunk barely uses it (25 of 14,287 tokens on removal);
  - claude-155's bag and register. The bag meets `in-space-trunk-to-text`'s done-when (openings `217 / 275`, `R16` `0.166 / 0.155`). The learned register collapses training.

  All are in their REPORTs under `docs/loop/tasks/in-space-trunk-to-text/` and the [float-training notebook](../../research/notebook/float-training.md).
- **The storage diagnosis is not on the map** ([claude-155 REPORT](../../loop/tasks/in-space-trunk-to-text/claude-155/REPORT.md), "The pivot").
  - The trunk is `64,000` values beside a `1,048,576`-value table. Routing the prompt in works; the associations that pick a reply's first token have nowhere to be stored.
  - A lookup of the prompt's first 8 tokens names `607` of 1,226 openings from the same TRAIN.
- **The owner's criterion is cost per token.** The map lists cost per stage, but it does not rank stages or plans by it. By arithmetic, the whole-vocabulary nearest search is about `1,048,576` operations per token against about `25,000` for the trunk. That holds in serving, and again in the training loss, which uses the whole table's nearest wrong token.
- **The archived prime and angular router is not on the map.** A 28 Oct search of the archive found:
  - the Python ancestor and its write-up (`archive/prime_router_geometric_context_evidence.md`);
  - the Rust port (`crates/uor-r4-router`);
  - the Hopf projection fix (`archive/hopf_projection_remediation_306.md`: sectors reached went from 7/512 to 43/512);
  - ADR-0003's fixed-zeta prime routes (`docs/reference/adr/0003-fixed-zeta-prime-route-attention.md`, `RETAIN_STORAGE_RECALL_ONLY`);
  - PR #455's choice-rule result (top-1 22.81% hard select, 24.54% top-3, 25.91% averaging over all).

  It found no record of a golden-ratio C4 → R4 projection, of a "complex–Riemannian bridge" by that name (the nearest is ADR-0003's zeta phases), or of a hard-coded choice winning. These need a full read, not the search's summary.

  **Corrected by claude-159's full read.** The bridge is on record: ADR-0003 §5, "Direction-preserving golden radial bridge", is the "complex-to-Riemannian lift" R_φ(z) = φ·E(z) on E : ℂ² → ℝ⁴. It is exact in ℤ[φ]; its algebra is built and tested, with no language result. The map is from ℂ², not ℂ⁴. No hard-coded choice won: on PR #455, averaging did. The archived Python router is `projects/research-archives/code/gemini-dev/prime-router/server.py`. See the map's section "The archived prime and angular router".

## Done when

1. `docs/model/pipeline-map.md` is revised:
   - every stage's fields carry the results of claude-149 to claude-155, with sources;
   - each stage's cost per token comes first, measured where a profile exists (the `fbs-cost-profile` item) and labelled as arithmetic otherwise;
   - the gaps and the order of work are re-ranked by what blocks a cheap, capable model first, with storage, the served decoder and the vocabulary search placed on the evidence.
2. A section reads the archived prime and angular router, from the files above and the commits they name:
   - what it computed, and what was measured;
   - what it would cost per token today;
   - which of its pieces (Hopf oracle, zeta phases, φ couplings, choice rule) fits which stage, and in which domain.
3. The revision is merged, and PLAN's 27 Oct refocus direction is set to `done` in the same PR. One `[claude, for Casey]` note on #2028 gives its link, the re-ranked top five gaps and the first unit.

## How to start

Read:
- the map, [`docs/model/pipeline-map.md`](../../model/pipeline-map.md);
- the REPORTs of claude-150 to claude-158 (`docs/loop/tasks/one-decoder/`, `docs/loop/tasks/in-space-trunk-to-text/`, `docs/loop/tasks/fbs-cost-profile/`);
- the archived router: [ADR-0003](../../reference/adr/0003-fixed-zeta-prime-route-attention.md), [its qualification record](../../../archive/prime_route_attention_qualification_958.md), [the ancestor's evidence note](../../../archive/prime_router_geometric_context_evidence.md), [the Hopf mechanism page](../../mechanisms/hopf-observation.md) and `projects/research-archives/code/gemini-dev/prime-router/`.

This is document work: no GPU run and no pre-registration (CHARTER §4.5).

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Claims** (§4.6): each number is labelled as a measurement with its source, a proof from code or arithmetic, or a hypothesis.
- **Changes to §3 are the owner's** (§5.1): where the map finds a constraint that costs speed, it proposes; it never relaxes.

## Sources

- Casey's directions, 27 and 28 Oct 2026, in claude's chat (quoted above).
- [`docs/model/pipeline-map.md`](../../model/pipeline-map.md)
- [claude-157](../../loop/tasks/fbs-cost-profile/claude-157/REPORT.md) and [claude-158](../../loop/tasks/fbs-cost-profile/claude-158/REPORT.md), the cost profiles.
- [`docs/reference/adr/0003-fixed-zeta-prime-route-attention.md`](../../reference/adr/0003-fixed-zeta-prime-route-attention.md)
