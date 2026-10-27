---
id: pipeline-map
title: Map the model end to end: each stage's domain, size, hardware and cost
milestone: research
area: architecture
status: done
open_to: lab
needs_gpu: false
depends_on: []
done_class: host
scope: [docs/model/pipeline-map.md, docs/model/architecture.md, docs/SUMMARY.md, PLAN.md, docs/loop/tasks/pipeline-map/**, docs/roadmap/items/pipeline-map.md]
---

## Why it matters

Casey's direction of 27 Oct 2026, in claude's chat, verbatim:

> "we may have to pause for a moment and try to refocus on exactly what mechanisms we need to make this work, what we have, what order they need to be in, what geometry/mathematical domain they have to be in for downstream so that the mechanisms works cleanly from first trained "token" (or whatever we end up using) to last emitted work in the answer to a user's question. If we cannot define all of the steps in that process, we need to sit down and work those out. We need to know what sizes are required for each steps, what hardware, (CPU/GPU) and what end mechanism delivers the best speed for the least amount of hardware resources while still completing the mechanism.
>
> the goal is a local model equivalent to probably gpt4 in ability that can be trained on less resources than a datacenter, and that can be served on a normal laptop with specs about equivalent to an apply m1 (this hardware)."

The same day, claude-146's look-back read went through three forms that sat at chance on its recall probe. On the research note's reading, the first could not learn its match. The other two copied a token into a domain the next stage did not read:
- **Keys from the trunk's own state:** with Hamming attention keyed on the state, every arm was at chance in every bucket of the recall probe, and openings sat at the 161 floor through step 8,192. The research note's reading is a credit fault: hard top-K gives credit only to the positions it picks, and at the start the right position is no nearer than any other (`docs/loop/tasks/in-space-trunk-to-text/claude-146/RESEARCH.md`, on branch `claude/hamming-read`, #3332). claude-144's state-keyed Flock read did not clear its pre-registered openings bar at either seed ([claude-144 REPORT](../../loop/tasks/in-space-trunk-to-text/claude-144/REPORT.md)).
- **The copied position's residual as payload:** in the commit's words, "a copied token reads as the trunk's own input, not its answer" (`87ade4920`, on branch `claude/hamming-read`, #3332).
- **The token's point added before the final norm:** next to the stream's larger parts, it "barely moves where the prediction lands" (`6d8bac253`, on the same branch).

The form claude-146 then moved to (`6d8bac253`) moves the readout itself toward the copied token's table point, the point the decoder compares against. Its result is not yet recorded in the repository (#3332). A map that names each stage's input and output domain is how the next stage is checked before it is built.

## What is missing

- No page defines every stage from the first trained token to the last word of a reply, with each stage's exact input and output domain.
  - [The model, end to end](../../model/architecture.md) covers the discrete path only.
  - It predates the float FBS trunk, the float-trunk exporter (#2820) and the Hamming read (#3332).
- The repository has two model paths:
  - the discrete path, which carries the M1 bar;
  - the float in-space trunk, which writes the replies.
  
  No page says how, or whether, they meet.
- No page gives the sizes, the hardware or the cost per token that the owner's goal needs, stage by stage. Nor does any page give the scale arithmetic that compares that goal with standard models.

## Done when

1. One page, `docs/model/pipeline-map.md`, gives every stage, in order, from the training data to the last word served on an M1 laptop. For each stage it gives seven things:
   - the mechanism the repository has, with its file and whether it is merged or on a branch;
   - its exact input and output domains, and whether the next stage consumes that output in that domain;
   - its size now and the size the goal needs;
   - its hardware for training and for serving, naming the M1 unit (NEON CPU, GPU or ANE);
   - its measured cost per token, with the source of each figure;
   - its status, measured or hypothesis;
   - what it depends on.
2. The same page gives:
   - the scale arithmetic for the goal, marking which of the geometry's claims are measured and which are hoped for;
   - the gaps, ranked by what blocks the goal first;
   - an order of work, each unit named with what a pass and a fail would unlock.
   
   Every number has a source.
3. The page is merged on main. One `[<lab>, for Casey]` note on #2028 carries:
   - the page's link;
   - the five biggest gaps;
   - the first unit to run.

## Cancel if

A page that maps every stage with its input and output domain is already on main, or the owner withdraws the 27 Oct direction.

## How to start

Read:
- [the model, end to end](../../model/architecture.md) and [serving and export](../../model/serving-and-export.md);
- the 25–27 Oct rows of [the float-training notebook](../../research/notebook/float-training.md);
- the REPORTs under `docs/loop/tasks/in-space-trunk-to-text/` and `docs/loop/tasks/d11-exporter-float-trunk/`;
- the [mechanism board](../../mechanisms/README.md) and [object store](../../mechanisms/object-store.md);
- claude-146's research note on branch `claude/hamming-read`.

This is document work: no GPU run and no pre-registration (CHARTER §4.5).

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Claims** (§4.6): each number is labelled as a measurement with its source, a proof from code, or a hypothesis; NOT_RUN is not PASS.
- **Changes to §3 are the owner's** (§5.1): where the map finds a constraint that costs speed, it proposes; it never relaxes.

## Sources

- Casey's direction, 27 Oct 2026, in claude's chat (quoted above).
- [`docs/model/architecture.md`](../../model/architecture.md)
- [`docs/research/notebook/float-training.md`](../../research/notebook/float-training.md)
- [`docs/loop/tasks/in-space-trunk-to-text/claude-145/REPORT.md`](../../loop/tasks/in-space-trunk-to-text/claude-145/REPORT.md)
