---
id: cpu-training-program
title: Authorize and sequence systematic CPU training repair
milestone: research
area: training
status: done
open_to: lab
needs_gpu: false
depends_on: []
notebook: docs/research/notebook/cpu-training.md
done_class: host
scope: [CHARTER.md, AGENTS.md, PLAN.md, docs/rules/changes.md, docs/rules/README.md, docs/rules/decision-map.md, docs/integration/DECISIONS.md, docs/integration/project-track.md, docs/contributing/HARD-RULES.md, docs/harness/pm-rules.md, docs/harness/milestones.json, docs/harness/claims.json, docs/harness/skills/cost-and-compute-admission/SKILL.md, docs/harness/skills/context-hydration/SKILL.md, docs/harness/skills/experiment-prereg/SKILL.md, docs/harness/skills/research-plan-cycle/SKILL.md, docs/geometry/README.md, docs/geometry/vsa-and-hamming.md, docs/mechanisms/vsa.md, docs/model/gpu-and-cuda.md, docs/model/pipeline-map.md, docs/labs/README.md, docs/labs/compute.md, docs/labs/session-goal.md, docs/products/desktop-client.md, docs/products/web-client.md, docs/start/faq.md, docs/start/glossary.md, docs/start/where-were-going.md, docs/research/cpu-training-program.md, docs/research/notebook/cpu-training.md, docs/research/notebook/README.md, docs/SUMMARY.md, docs/roadmap/README.md, docs/roadmap/items/cpu-training-program.md, docs/roadmap/items/cpu-discrete-writer-core.md, docs/roadmap/items/cpu-discrete-training-path.md, docs/roadmap/items/cpu-shared-vector-forward.md, docs/roadmap/items/cpu-shared-vector-train.md, docs/roadmap/items/cpu-quantization-exact-memory.md, docs/roadmap/items/cpu-training-pilot.md, docs/roadmap/items/cpu-whole-path-exact-updates.md, docs/loop/tasks/cpu-training-program/**, docs/roadmap/issues.json]
---

## Why it matters

The owner asked on 1 November 2026 for every CPU training method to be repaired systematically, delivered on GitHub `main`, and kept clear of Claude's active serving work. The repository contains ten historical training families with different failures; most are inactive evidence, and at registration no current CPU trainer existed. Treating them as one broken program would erase the reasons each configuration failed.

The project now has stronger mechanisms than the early runs had: address-cell initialization, admission-preserving controls, within-cell reranking, BVG-K, exact Hamming reads, deterministic checkpoints and exact export comparison. Those discoveries make a fresh CPU program worth testing, but they do not reverse the old negatives.

## What is missing

D84 now permits named CPU research, and this item registers the ordered repairs:

1. [`cpu-discrete-writer-core`](cpu-discrete-writer-core.md): exact writer, admission and ranking parity with a benchmark.
2. [`cpu-discrete-training-path`](cpu-discrete-training-path.md): reads, proposals, BVG-K, apply and resume.
3. [`cpu-shared-vector-forward`](cpu-shared-vector-forward.md): the serving-aligned token/vector forward in an isolated crate.
4. [`cpu-shared-vector-train`](cpu-shared-vector-train.md): lawful rank-margin training on that forward.
5. [`cpu-quantization-exact-memory`](cpu-quantization-exact-memory.md): integer-aware continuation, Hamming reads and exact turn memory.
6. [`cpu-whole-path-exact-updates`](cpu-whole-path-exact-updates.md): the proposed exact-update premise was later rejected by its source audit; retain transaction/replay evidence without importing the failed M2 objective.
7. [`cpu-training-pilot`](cpu-training-pilot.md): bounded workstation pilot, performance and complete-reply/fresh-phrasing evidence.

The family-by-family disposition and the rationale for this order are in [the CPU training repair program](../../research/cpu-training-program.md).

## Done when

- D84 is recorded consistently in the charter, onboarding, hard rules, decision index and harness rule registry.
- The seven child items are registered with dependencies and non-overlapping implementation scopes.
- The program preserves the CPU/GPU evidence boundary, historical negative classifications and Claude's active serving paths.
- Roadmap validation/render, owner-path approval checks and a fresh exact-head review pass.

## Resolution

D84 and the seven child items were registered on 1 November 2026 by `deepseek-1`. This item's registration scope is complete; implementation continues in the child items and is not implied by this close.

As of 2 November 2026, #3501, #3502, #3504 and #3510 are done or closed. #3502 now has public trained-codebook parity on all 14,287 rows, and #3501 has exact CPU reads, proposals, apply, BVG-K and child-process restart. #3504 covers snapped/raw FBS gates and transitions with Hamming and exact memory. #3510 was rejected at its premise audit because the retained whole-path source has no parameter-update trainer. The shared-vector path has delivered its tied-token trainer (#3518), exact BVG-K parameter groups (#3522), matched panel controls (#3523), public trained dual-c131 parity, and a strict masked-corpus driver. #3503 has finite Hamming/memory/restart and quantization-shadow mechanisms, but a genuine trained float shadow and complete-reply comparison remain `NOT_RUN`. #3505 remains open for a pre-registered pilot; #3506 is not admitted and the sealed complete-reply panels currently return HTTP 401 without credentials. The durable family matrix is in [the CPU training repair program](../../research/cpu-training-program.md).

## Cancel if

The owner withdraws D84, the current charter still forbids named CPU research at the claimed head, or a proposed scope overlaps an active claim on the shared FBS/integer path. Stop rather than weakening either the CPU authorization or the GPU qualification boundary.

## How to start

Read this item, D84, the program page, and the three 1 November CPU audits. Confirm the child scopes do not overlap an open claim, then take the first dependency-free child in order. Do not start with a corpus run.

## Rules that apply

- **Geometry only, no squashing, no dense matmul** (CHARTER §3.1-§3.3).
- **GPU-first research compute** (CHARTER §3.5, D24 and D84).
- **One row per result; a negative closes a configuration, not a mechanism** (CHARTER §4.1 and §5.6).
- **Isolated delivery:** no edits to Claude's active FBS/integer serving scope in this program's first units.

## Sources

- The owner's direction in `PLAN.md`, recorded 1 November 2026.
- D84 in `docs/integration/DECISIONS.md`.
- The three completed CPU architecture, FBS and historical-family audits recorded in the session on 1 November 2026.
- The current discrete and shared-vector source paths named by the child items.
