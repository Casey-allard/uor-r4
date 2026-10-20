# #1107 — Freeze the native Four-fact workbench source candidate

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-09-03T17:24:35Z
- **Updated:** 2026-09-04T04:30:09Z
- **Closed:** 2026-09-04T04:29:22Z
- **Milestone:** Native Rust geometric intelligence — conversation, memory, reasoning and coding
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/1107
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Parent and authority

Bounded implementation child of #1084, following the protected delivery of #1105 / ADR-0006 at `8ba5887e067e4cc33a1a3c9a6459fcc4b4077a11`.

The accepted authority is `docs/r4_service_contract_1105.json`. The accepted #1105 source audit and manifest preserve the original NEMESIS, W33, UOR, donor UI, and R4 source identities. Those sources motivate typed state and receipt boundaries; they do not prove or measure this host.

## Frozen decision

Implement the final-candidate source for the opt-in native `uor-r4-workbench` crate and `r4-workbench` executable:

- one loopback parent owns HTTP, lifecycle, configuration, and verified static assets;
- one same-executable `--internal-worker` child owns the loaded `LoadedResearchReference`;
- a separately invoked private comparison mode is present in the same exact executable before build freeze and exposes no listener;
- the only public operation is `answer_four_fact_raw_text/v1`, preserving exact raw bytes, the accepted reader/core, known vocabulary and query form, and the fixed four-fact context;
- the first web shell truthfully displays the service-returned model, qualification, readiness, request, refusal/token, progress, cancel, and error state;
- the seven frozen `/uor/v1/workbench` routes, strict same-origin/framing/config/file limits, one-slot state model, worker correlation, termination/reaping rules, and no-fallback behavior are implemented from ADR-0006.

No build may start until the complete source, lockfile, toolchain, exact commands, environment, empty output root, resource caps, and supervising receipt format are independently reviewed and accepted.

## Named decision and terminal

If the accepted source builds under the one admitted offline verification/build sequence, publish **WORKBENCH_CANDIDATE_BUILT_UNQUALIFIED** with exact source, asset, build, binary, and runtime identities.

This terminal means only that the frozen candidate source and static shell exist and the named source/unit/build checks passed. It does not mean the executable is numerically qualified, the artifact was loaded, an answer was produced, HTTP/browser behavior was accepted, or general chat/coding/context/final integer-kernel capability exists.

If any source, dependency, flags, target, comparison mode, assets, or build command changes after admission, the admission is consumed and a fresh independent source/build review is required before another compiler invocation.

## Required evidence

- implementation, source/asset manifest, and exact contract conformance map;
- independent prebuild acceptance bound to the frozen tree and admitted command sequence;
- supervised receipts with elapsed time, output-byte accounting, exit status, toolchain, environment, source/lock hashes, and final executable identity;
- independent postbuild review that distinguishes source inspection, compile/test observations, and all unrun behavior;
- current roadmap, claim/evidence records, knowledge index, storage review, GitHub issue/PR state, and protected merge evidence.

## Explicit exclusions

This child performs zero model fitting, export, artifact/model deserialization, model load, forward pass, comparison, qualification, service launch, socket request, or browser run. The old #1102 binary and receipt are historical only and cannot qualify this host. The #1102 and #1094 execution envelopes remain consumed. #973 remains open and #954 remains blocked.

## One next action after closure

Freeze and independently accept the exact built candidate's fresh private-comparison release before any artifact access or model work.


## Comments (2)

### comment 5535521748 — 2026-09-04T04:09:34Z — Casey-allard

## Source-only execution amendment — 2026-09-04

The repository-wide `deterministic_source_only` policy delivered by #1108 / PR #1109 supersedes this issue's earlier agent-run build, test, probe, supervisor and retry requirements. The owner has explicitly instructed that the next issue proceed under that mechanism.

The frozen terminal for this issue is now **`WORKBENCH_CANDIDATE_SOURCE_FROZEN_UNBUILT`**.

Acceptance is limited to:

- carry the previously prepared source into a complete Git worktree from refreshed `origin/main`;
- inspect it against ADR-0006, `docs/r4_service_contract_1105.json`, the current roadmap, the project knowledge index and the pinned NEMESIS/W33/UOR source boundaries;
- resolve concrete source-review findings only;
- record the exact source boundary and preserve all prior negative, unavailable and measured results without relabeling them;
- deliver the source and records through a protected pull request and merge queue.

Project compilation, tests, model/artifact access, fitting, comparison, service launch, HTTP/browser behavior, operating-system diagnostics, supervisors and retries are **`NOT_RUN_BY_POLICY`**. Consequently this terminal establishes source presence and source-level contract mapping only. It does not establish that the crate compiles, that the executable exists, or that any numerical, lifecycle, HTTP, browser, security, portability, performance or model behavior is correct.

After closure, the sole concrete next action is an owner decision on whether to activate a separately scoped manual qualification path. Automated agents do not dispatch or run it.


### comment 5535669118 — 2026-09-04T04:30:09Z — Casey-allard

Delivered through protected PR #1110, merge `b50d5f901fc717fee9838d361cf7cc29159889f5`.

Terminal: `WORKBENCH_CANDIDATE_SOURCE_FROZEN_UNBUILT`.

The merged tree contains the opt-in `uor-r4-workbench` crate, private same-executable worker and comparison entries, and the first Four-fact shell defined by ADR-0006. Independent static review accepted the final source after correcting five contract mismatches: mutation error precedence, incomplete host-acceptance discovery, private-comparison completion, inherited descriptor handling, and POST media-type precedence. The roadmap, continuation handoff, source/claim ledger, and exact reviewed source tree `1ca9723348f11a42d7245504dc2da11497c1338c` are updated.

Limits: compilation, tests, model operations, numerical comparison, qualification, HTTP/service execution, browser acceptance, target behavior, portability, performance, security behavior, and local knowledge-index ingestion are `NOT_RUN_BY_POLICY`. No mathematical proof or new measured model/product behavior follows. #1102/#1094 and every prior negative/unavailable result retain their original scope.

Closure: #1107 is complete; #1084 remains open and unassigned.

One next action: the owner decides whether to authorize a separate manual qualification workflow. Automated agents do not dispatch or execute it.



---
_Comment/review ids in this file: 5535521748, 5535669118_

