# #1096 — Repair isolated runtime readiness after the #1094 preflight stop

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-09-03T05:31:58Z
- **Updated:** 2026-09-03T12:05:49Z
- **Closed:** 2026-09-03T12:05:49Z
- **Milestone:** Native Rust geometric intelligence — conversation, memory, reasoning and coding
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/1096
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Delivered readiness decision — 2026-09-03

#1096 completed **ISOLATED_RUNTIME_READY** through protected [PR #1098](https://github.com/UOR-Foundation/uor-r4/pull/1098), merge `6f21fc5f4c40b9620c9fec5e95a39097f812ae73`. The delivered tree matches the independently reviewed head. The one actual readiness-only attempt completed in 2.058100333 seconds with all four harmless probes denied, verified fixed runtime/source/artifact identities, null states and zero model loads/row or batch forwards/optimizer updates. Conservative combined peak RSS was 704,806,912 bytes. [Complete record and exact receipts](https://github.com/UOR-Foundation/uor-r4/blob/6f21fc5f4c40b9620c9fec5e95a39097f812ae73/docs/r4_isolated_runtime_readiness_1096.md); [independent delivery review](https://github.com/UOR-Foundation/uor-r4/blob/6f21fc5f4c40b9620c9fec5e95a39097f812ae73/docs/r4_isolated_runtime_readiness_1096_delivery_review.md).

This qualifies one pinned runtime startup and the tested harmless-read restrictions. It does not identify the precise original denial cause, prove general isolation or qualify raw-text model behavior. #1094's original UNAVAILABLE_REFERENCE_REPLAY, exact authoring results (320/320 valid, 16/16 refusal), source/binding/profile and sealed mode-000 population remain preserved. No corpus preparation, withheld access, fit, model comparison or replay occurred under #1096. #1079 LANGUAGE_R4_PRESERVED_CONTROL_WEAK and #1082 descriptive limits remain unchanged; #973 stays open and #954 blocked.

The one next action is **#1094's separate frozen preparation contract**, using the accepted source/profile/runtime handoff, explicit accounting for its prior consumed budget and independent release review before withheld access or comparison. #1094 remains open, parked and unassigned until activated. No further task was executed in this delivery.

## Outcome

Repair the isolated interpreter launch that stopped #1094 before Python startup. Demonstrate the accepted runtime can start while the corpus/reference/result paths remain unavailable, using a separate zero-forward readiness contract. This child does not launch another adapter comparison.

## Preserved evidence

#1094's sole preparation at `ff925481fcb290e8f91442a28a1b43b51b28dd26` returned `UNAVAILABLE_REFERENCE_REPLAY`: `sandbox-exec: execvp() ... failed: Operation not permitted`. The immutable stop receipt SHA256 is `87bd3082ce9b4da5e5227a3b82f6515773cf5f113de689ff6251a2a45340fad5`; bindings are `baeb56a632c0e75c0574d9bad503e7849cf434039490e1fe92e3b8f0114b1f0b`.

Authoring preflight separately recovered all 320 valid rows and refused all 16 cases exactly, plus two schema probes, with zero model loads/forwards. Its SHA256 is `f79df8623038961d899ba727d99bb69b39754b1878d10bbed9da0bfe03e5ee82`. Withheld files remain sealed and unobserved. No worker runtime/isolation, full-output preservation, answer accuracy or replay was qualified. Preserve that unavailable terminal and all original receipts.

## Bounded contract and DoD

- Trace the denied launch from the actual pinned executable, resolved interpreter and library paths, and source policy. Do not infer the precise denial cause from the one execvp message.
- Freeze the minimal executable/runtime access policy and independent source review before executing a new readiness attempt. Keep the raw-only inference boundary, denied corpus/reference/history/results, denied network and read-only model assets. Do not grant broad home-directory access or bypass an external access control.
- One separately recorded readiness attempt, at most 60 seconds and 16 MiB new receipts, reusing installed Python 3.12.14/Torch 2.7.1 on the bound Apple M1. No model construction, model forwards, optimizer updates, downloads, corpus regeneration, withheld reads or broad QA.
- Require the actual worker to verify source/artifact identities and exact runtime settings, demonstrate `PermissionError` for harmless sentinels at forbidden path classes, and emit initial/final zero-forward receipts. Expected-denied probes must contain no research payload or answers.
- Positive: record `ISOLATED_RUNTIME_READY` and hand the exact source/profile/runtime bindings back to #1094 for separately frozen preparation and independent release review. This does not itself authorize withheld comparison.
- Negative/unavailable/resource stop: retain the specific failure and keep #1094 comparison `NOT_RUN`. No automatic retry, broader allowlist or renewed budget.
- Independent review, protected PR delivery, current-map/claim/evidence/knowledge updates and storage preservation are required. #1079's weak-control result, #1082's descriptive limits, #973 open and #954 blocked remain unchanged.

## Original ownership and next action

Child of #1094. Casey-allard coordinates; parked and unassigned until active. Next action: inspect the recorded launch path and sandbox policy, specify the minimal correction, and independently review the zero-forward readiness attempt before running it.


## Comments (4)

### comment 5525313289 — 2026-09-03T11:53:34Z — Casey-allard

#1096 is active and assigned to Casey-allard in isolated branch `issue-1096-isolated-runtime-readiness`, based on current `origin/main` `df2c4cb8ef47e35b6d4083d4b8da135c7676fc19`. Parent #1094 remains open; no comparison or withheld release is part of this task.

The reviewed source and decision contract are frozen at `443b6482bc728ef751b8e52a48e94bc8ed704abe`. The identity-only manifest is 9,341 bytes, SHA256 `704b78e285a1ba5b1ee24f5c9e3a82205ac15c3d31081fc942c1a0dcbcb140ea`, CID `blake3:cc319cdd7033cfba6dc4df00016b718d841b2fcc8868f63194e06543cfb5c90b`. The independent reviewer must approve those exact bytes before the sole launch.

Static inspection found that the venv launcher passes through `cpython-3.12-macos-aarch64-none`, while the original profile allowed only the resolved `cpython-3.12.14-macos-aarch64-none` tree. The correction adds literal interpreter symlink reads and metadata on exact ancestors of already permitted paths. It retains the existing read trees and five exact assets, denies network/home writes, and does not allow the old editable source. The original execvp error does not identify its precise cause; this is one combined correction, not a causal attribution experiment.

metric to move: actual isolated worker readiness, currently unavailable; zero language/model decisions.
reachability ceiling: this can qualify only startup, fixed runtime/source/asset identity and denied harmless reads. It cannot qualify model behavior or repair #1079's weak control.
instrument + verdict: invoke the actual worker once with `--readiness-only`, stdin closed, frozen source/profile/runtime and four harmless corpus/reference/history/results stand-ins. Require exact ready/done identities, four PermissionErrors, null states, zero model loads/row and batch forwards/optimizer updates and exit zero.
exit rule: all declared readiness criteria and resource bounds pass => ISOLATED_RUNTIME_READY; any missing/failing identity, launch or probe => UNAVAILABLE_ISOLATED_RUNTIME; resource overrun => INCOMPLETE_RESOURCE.
if positive: deliver #1096 through protected PR, close its readiness scope and return exact bindings to still-open #1094 for separately frozen preparation and independent release review.
if negative: preserve the specific failure and original receipts, keep #1094 comparison NOT_RUN and require a separate repair decision. No retry or broader allowlist under this contract.
cost estimate: seconds for hashing/import; one attempt, hard 60-second parent deadline, 16 MiB new receipts/probes, 1 MiB worker-output sublimit and conservative 3 GiB combined peak RSS. Same installed Python 3.12.14/Torch 2.7.1, Apple M1, four intra-op/one inter-op CPU threads. No optimizer steps, model construction, downloads, actual corpus reads, withheld access, regeneration or broad QA.

Independent source/contract review corrected global deadline coverage and preservation of captured output on timeout. Exact-manifest approval is still required; identity authoring performed no sandbox launch or model/Torch import. Named checks are source review, changed-Python syntax, the sole readiness decision, independent result/digest review, claim wording, document/reference integrity and diff whitespace. Queue statuses are transport acknowledgements only.

The original #1094 stop/bindings/profile remain byte-identical, the withheld directory remains mode 000, and #1079/#1082 findings are preserved. #973 remains open and #954 blocked. Storage before work was about 51 GiB free; the isolated checkout is about 1.11 GiB, with no new runtime, model or build cache and no deletion.


### comment 5525355337 — 2026-09-03T11:56:58Z — Casey-allard

Independent pre-execution review is complete. The sole admitted freeze is `/Users/casey.allard/.codex/uor/issue-1096-readiness-freeze02/manifest.json` at source `79c674c8f6179a68878a12ee86e664f1435c3ebf`: 9,395 bytes, SHA256 `4acd2b7ec00ac8874573e2d6e52e5087b376bc0ebcd52aedd4464aa28979c644`, CID `blake3:b8877f377463308ead6879eb6da01f5f95505cada0a0becee2c63cc91486b171`.

Exact profile SHA256: `d508d3ec50b78e3bbd3cb1fe9468788f457f4e47f713f9a793f03079df48416e`. Independent review receipt SHA256: `e092e4bc3d3a989d37868c5530ff3483d80f4a64e53b3cf609aa1774a5c43759`, status `APPROVED_FOR_SINGLE_READINESS_ATTEMPT`. The reviewer independently verified 169 committed source records, 18 installed runtime files, two interpreter symlinks, four harmless sentinels, exact profile reconstruction and unchanged five original asset identity records. The parent and actual worker will separately rehash the accepted asset bytes; neither constructs a model.

The first identity-only freeze named above is superseded and retained unattempted. Review found an asset ordering mismatch after canonical binding serialization; the source now sorts asset names before profile rendering. Neither freeze has been executed. Both directories together contained 153,632 bytes before the 2,452-byte review receipt; this aggregate is included in the same 16 MiB task allowance. This is a pre-execution correction, not a runtime retry.

The decision, fixed runtime, harmless denied probes, 60-second deadline, zero model load/forward/update requirements and positive/negative actions in the prior contract remain binding. Exactly one invocation of the actual readiness-only worker is now admitted against this second manifest. Failure consumes that attempt; no retry, broader profile, corpus preparation, withheld release, model evaluation or budget reset follows. #1094 remains open and its comparison remains NOT_RUN.


### comment 5525378943 — 2026-09-03T11:58:44Z — Casey-allard

The one independently admitted readiness attempt completed: **ISOLATED_RUNTIME_READY**. Source `79c674c8f6179a68878a12ee86e664f1435c3ebf`, manifest SHA256 `4acd2b7ec00ac8874573e2d6e52e5087b376bc0ebcd52aedd4464aa28979c644`; result SHA256 `439aa149d6f128844490c4a9002bfe2ffb52fdeeaad067e8e3cb16447b24b930`, CID `blake3:9e62fa112632d3fac636008000086bed0827ea933f0df49b367f86c99461f147`.

The actual worker emitted exactly ready/done and exited successfully. Bound Python 3.12.14/Torch 2.7.1, Apple M1 CPU/Accelerate, four intra-op/one inter-op thread and deterministic execution settings were verified, including 18 runtime files, two interpreter symlinks and source/accepted artifact byte identities. All four harmless corpus/reference/history/results stand-ins raised PermissionError. Initial/final model states stayed null; model loads, row/batch forwards and optimizer updates were zero. Parent elapsed 2.058100333 seconds; conservative combined peak RSS 704,806,912 bytes. Both original freeze directories total 165,740 receipt/probe bytes after execution. No model or corpus payload was copied into evidence.

The original freeze remains unattempted; exactly one started receipt exists in the admitted second freeze. Exact raw stdout/stderr, result/resources, binding/profile/manifest and approval receipts are retained for independent result review and protected PR delivery. The original #1094 stop/binding/profile remain unchanged and the withheld directory remains mode 000.

This qualifies this pinned startup and the tested harmless-read restrictions only. It is not a model-output result, general isolation proof or separate causal identification of the original execvp error. #1094 comparison/replay and any new preparation remain NOT_RUN. After protected delivery, the next action is a separately frozen #1094 preparation contract with prior-budget accounting and independent release review. #1096 remains active until that delivery is verified; #973 stays open and #954 blocked.


### comment 5525465279 — 2026-09-03T12:05:48Z — Casey-allard

#1096 completed **ISOLATED_RUNTIME_READY** through protected [PR #1098](https://github.com/UOR-Foundation/uor-r4/pull/1098), merge `6f21fc5f4c40b9620c9fec5e95a39097f812ae73`. The delivered tree matches the independently reviewed head. The one actual readiness-only attempt completed in 2.058100333 seconds with all four harmless probes denied, verified fixed runtime/source/artifact identities, null states and zero model loads/row or batch forwards/optimizer updates. Conservative combined peak RSS was 704,806,912 bytes. [Complete record and exact receipts](https://github.com/UOR-Foundation/uor-r4/blob/6f21fc5f4c40b9620c9fec5e95a39097f812ae73/docs/r4_isolated_runtime_readiness_1096.md); [independent delivery review](https://github.com/UOR-Foundation/uor-r4/blob/6f21fc5f4c40b9620c9fec5e95a39097f812ae73/docs/r4_isolated_runtime_readiness_1096_delivery_review.md).

This qualifies one pinned runtime startup and the tested harmless-read restrictions. It does not identify the precise original denial cause, prove general isolation or qualify raw-text model behavior. #1094's original UNAVAILABLE_REFERENCE_REPLAY, exact authoring results (320/320 valid, 16/16 refusal), source/binding/profile and sealed mode-000 population remain preserved. No corpus preparation, withheld access, fit, model comparison or replay occurred under #1096. #1079 LANGUAGE_R4_PRESERVED_CONTROL_WEAK and #1082 descriptive limits remain unchanged; #973 stays open and #954 blocked.

The one next action is **#1094's separate frozen preparation contract**, using the accepted source/profile/runtime handoff, explicit accounting for its prior consumed budget and independent release review before withheld access or comparison. #1094 remains open, parked and unassigned until activated. No further task was executed in this delivery.



---
_Comment/review ids in this file: 5525313289, 5525355337, 5525378943, 5525465279_

