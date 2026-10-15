# #34 — Evaluation-report tooling for HF-compiled (SmolLM2) models

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-21T21:25:35Z
- **Updated:** 2026-07-22T05:53:18Z
- **Closed:** 2026-07-22T05:53:18Z
- **Labels:** graph-compiler
- **Milestone:** Phase 0: Baseline and contracts
- **Assignees:** afflom, Copilot
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/34
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Gate C and the Phase 0 baseline need a fidelity certificate for the CURRENT default model path (HuggingFace Llama-family compile, .uor-models/compiled/smollm2-135m-instruct). Today only the legacy llama2.c stories15M path has one (certify.rs; PROOF.md P2: 28.9% top-1 / 31.7% agreement / 6.54 bits/token). README.md:203-208 flags this as the remaining end-to-end workflow gap (ask/chat refuse non-evaluated instruction-chat manifests; no command creates the evaluation report). Deliverable: a command that runs the HF teacher over a pinned held-out set (D3) and emits a certificate with top-1 agreement, teacher-argmax agreement, bits/token vs teacher floor, and CIDs. Blocks Gate C harness (Phase 2 exit). Plan: docs/r4_graph_compiler_implementation_plan.md §5 Phase 0, §8 Gate C.

## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

