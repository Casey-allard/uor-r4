# #608 — docs: plan open-weight behavioral recompilation into R4

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-10T22:05:21Z
- **Updated:** 2026-10-09T14:17:03Z
- **Closed:** 2026-08-11T00:11:41Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/608
- **Merged:** 2026-08-11T00:11:41Z
- **Merge commit:** 24533d6a8194e838862800219f7aba29cffde86d
- **Base / Head:** main / UOR-Foundation:docs/open-weight-recompilation-plan
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Purpose

Add the repository plan for compiling pinned permissively licensed open model snapshots into the R4 behavioral compiler/runtime path.

## What this documents

- Audit of current source ingestion, tokenizer, geometry, attention, trace, compiler, runtime, and certification surfaces.
- Deduplication matrix covering complete work, existing ownership, new issues, deferrals, and blockers.
- Truthful finding that the current `r4_attention` branch is chunked scalar dot-product attention followed by softmax, with a remainder-dimension defect; it is not the R4-native target operator.
- Dependency-aware phases from source snapshot identity through source conformance, trace/operator boundaries, target fitting, certification, architecture canaries, and future upstream integration.

## Tracker outcomes

Created:

- #597 source snapshot manifest
- #598 indexed Safetensors shards and BF16/F16/F32 codecs
- #599 source-executor parity and adapter conformance
- #600 explicit source/compiled geometry and projection versioning
- #601 versioned tokenizer adapters
- #602 source/target operator specification
- #603 deterministic versioned trace profiles
- #604 R4RouteAttentionV1 reference/lowering (**blocked by #515**)
- #605 target-operator fitting (**blocked by #515**, consumes #531)
- #606 target-operator certification (**blocked by #515**)
- #607 first non-Llama adapter

Updated:

- #515 with verified attention formula/control-flow evidence and the proposed activation-gate follow-up.
- #531 with a concise dependency note; corpus scale remains its existing scope.

#589 remains explicitly blocked. PRs #593, #594, and #596 remain the owners of the #515 preserve-and-gate tranche.

## Dependency sequence

`#597 -> (#598, #599, #601) -> #600 -> #602 -> #603 -> #604 -> #605 -> #606 -> #607`

The plan includes the full Mermaid graph and records #531 and #589 as separate ownership/blocker edges.

## Scope boundary

This PR changes documentation only. It implements no compiler, runtime, tokenizer, model-source, artifact, or certification code and does not re-pin fixtures.

## Verification

- `git diff --cached --check`
- `python3 scripts/check_claim_wording.py`
- Exact-file staging review (`docs/open_weight_recompilation_plan.md` only)

Refs #515, #531, #589, #597, #598, #599, #600, #601, #602, #603, #604, #605, #606, #607.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

