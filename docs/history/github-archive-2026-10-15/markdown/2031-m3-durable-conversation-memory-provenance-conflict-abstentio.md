# #2031 — [M3] Durable conversation memory: provenance, conflict, abstention, reset and restart

- **Type:** Issue
- **State:** open
- **Author:** @Casey-allard
- **Created:** 2026-10-09T14:41:41Z
- **Updated:** 2026-10-13T02:52:07Z
- **Labels:** roadmap, milestone, status:in-progress, lab:codex
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/2031
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Part of the roadmap tracker #2028.

Identity-scoped durable memory and grounded correctness on the saved model: text-derived writes and reads, correction lineage (current/previous/initial), scope isolation, absent/conflict/evicted distinctions, reset/forget, restart, typed Answer/Conflict/Clarify/Abstain outcomes with provenance.

### Current state (9 October)
- Delivered: `stack_grounded_session` (#1568) and the typed-script evaluator, oracle, CLI and 55-turn witness (#1578). These are a first implementation subset, not closure.
- Not qualified: learned (not handwritten) grounding and abstention; reset/forget, restart, eviction and scope isolation on the saved model.
- Known defects: #1718 (prose stored as facts in world=v2, 20.8% false-write, 8.7% overwrite), #1738 (two unreached scheduling paths).

### Acceptance
The full acceptance of #962 and #954, carried over verbatim below, met on the saved model.

<details><summary>Acceptance and history carried over verbatim from #962</summary>

<!-- active-roadmap-2026-10-01 -->
**October 1 adopted roadmap — READY.** PR #1564 / D19; adoption task #1563.

Parent #1508; shared implementation #973. **Reopened from a September 29 tracking transfer, not from a failed or revoked capability qualification.** The full original acceptance remains below.

Depends on #1552/#973. Complete actual text-derived writes/reads, correction lineage, current/previous/initial, scope isolation, absent/conflict/evicted distinctions, reset/forget and restart. Use the model's own generated history and causal complete answers. The first default-user ten-relation loop is an implementation subset, not full closure.

[Canonical plan](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/project-track.md) · [Mechanism admissibility](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/mechanism-admissibility-2026-10.md). Historical gates/results and full capability acceptance remain unchanged. Live comments own newer work; this header does not reset cumulative resources or transfer another lab's live files/jobs.
<!-- /active-roadmap-2026-10-01 -->


<details>
<summary>Preserved pre-adoption issue body and original acceptance</summary>

<!-- geometric-attention-synthesis-2026-09-20:start -->
**Merged PR #1352 principal review: V3 observation and feedback component retained; transferable language remains open.** The loaded artifact matches 31/31 exposed authored responses (23 fitting cases; eight familiar non-computed cases from three other documents), with 791/791 in-sample export/reload teacher steps. The retained finite table scores 20/31 with less information. All computed/post-copy `now` cases were fitted; the 4/4 pair varies operation and supplied `key_changed` along with payload, so it does not isolate learned content use. A changed derived address is not an entity correction, and `became` remains authored wording. Ordinary-text learning is NOT_RUN; no geometric superiority or energy reduction is established.

The principal replay reused identical learned artifacts and all 155 case rows, verified three 24-file sealed manifests, and passed 48 distinct focused library tests. It actually executes S checked inference and continuation beside E as separate components; this is not a matched scoped language comparison. No neural refit or fresh acceptance draw is claimed. [Review](https://github.com/UOR-Foundation/uor-r4/blob/44c2ccd27cea5d0178f19cca2f33332db4a95ce1/docs/integration/transferable-lexical-principal-review-2026-09-23.md) · [checks](https://github.com/UOR-Foundation/uor-r4/blob/44c2ccd27cea5d0178f19cca2f33332db4a95ce1/docs/evidence/transferable-lexical-principal-checks-2026-09-23.json) · [canonical plan](https://github.com/UOR-Foundation/uor-r4/blob/44c2ccd27cea5d0178f19cca2f33332db4a95ce1/docs/integration/project-track.md) · [DeepSeek next brief](https://github.com/UOR-Foundation/uor-r4/blob/44c2ccd27cea5d0178f19cca2f33332db4a95ce1/docs/integration/deepseek-ordinary-lexical-step-2026-09-23.md).

Memory acceptance remains open: exact owned copying/restart works at this scope, but changed derived address is not a committed correction; independently validate current/previous/initial meaning and mixed-session generated answers. Broad issue acceptance stays OPEN. DeepSeek has substantive design discretion within the stable native contract and prospectively recorded local resource authorization. Preserve original artifacts and sealed roots; no arbitrary timer/retry quota or paid compute.
<!-- geometric-attention-synthesis-2026-09-20:end -->

<!-- structural-memory-direction-2026-09-20:start -->
**Owner-directed follow-up: structural memory and conditional Hopf/S7 research.** [PR #1315](https://github.com/UOR-Foundation/uor-r4/pull/1315) adds the [canonical sequence](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/project-track.md#structural-memory-and-geometric-representation-follow-up) and [source-linked design](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/structural-memory-hopf-direction-2026-09-20.md). First complete and interpret the existing occurrence-reader result. Then test learned contextual role/scope persistence against a shared bank with matched retained bytes and candidate/work bounds. Reuse exact occurrence/version contracts and finite H4 transport; conditionally test quaternionic Hopf relation state or a separate harmonic-coefficient bank only for a concrete representation/interference need.

The older optional prose runtime contains S3 → (S2 base, retained S1 fiber) → S3 projection/prediction/lift. It is separate from CPX3 and is not a lossless dimensional chain. Higher quaternionic Hopf is S3 fiber over S4 base with total space S7. Equal-norm finite H4 pairs reduce to existing relative-group lookup; geometry does not supply free storage or demonstrated semantic advantage. Require changed-source, role/scope, distractor, frame/fiber and capacity controls as relevant. Preserve useful submechanisms and prospective near-miss decisions without retroactive PASS or an endless wave-engine campaign.

The owner additionally connected normalized E8 roots to S7 and clarified **quantum spin**. [PR #1316](https://github.com/UOR-Foundation/uor-r4/pull/1316) records a conditional finite E8 codebook and spinor/SU(2)/relative-phase comparison. Preserve radius and exact identity where needed; distinguish S7/U(1)=CP3 from the quaternionic quotient to S4, and do not infer quantum speedup or a physical general-relativity language law. These remain hypotheses behind the same occurrence-reader/role-scope sequence.

This is documentation and research sequencing; new variants are NOT_RUN. It does not replace the active execution prompt, change model budgets, or complete this issue's acceptance. #973 owns bounded integration, with #1139/#962 binding/memory, #963 cost and #964 invariant responsibilities retained. Existing run-specific updates below remain authoritative at their scope.
<!-- structural-memory-direction-2026-09-20:end -->

Roadmap position **04/12** · Programme #820 · **Not complete**

## Why this work exists

Useful local assistance needs meaning to survive turns, corrections and restarts, with explicit user/project isolation.

## Evidence already available

Bounded immutable relation versions and selected evicted-window/session reads already pass. General conversational and durable product qualification remains open.

## What to implement

Develop learned retention, read, update, conflict and correction over the exact versioned memory foundation. Exercise real multi-turn dialogue, pronouns and changing context. Implement restart, reset, export and forgetting with user/project/session identities, bounded consolidation and incremental updates. Verify that retained information causally changes generated responses and that unrelated identities cannot influence them. Reuse existing persistence rather than replacing it with a second memory store. Transfer cross-surface API/WASM/Studio integration to the interface roadmap issue while retaining semantic memory behavior here.

## Acceptance and stopping decision

Unseen multi-turn conversations recover and correct appropriate facts across eviction and restart; reset/forget and identity isolation work; complete generated answers demonstrate useful causal memory. Quantify capacity, update cost, conflict policy and limitations. A passing checkpoint round trip alone is insufficient.

## Order and dependencies

Integrate with #973 and shared state/emission; memory correctness and persistence fixes may proceed whenever the active model needs them. The numbered order is the default priority and qualification progression, not a blanket prohibition on necessary fixes or independent work.

## Operating instructions

Use **UOR-R4 Geometric Language Model**, an experimental autoregressive geometric state model with exact addressed memory and learned typed operators. Use Rust for preparation, training, artifacts and inference. Offline Rust training may use floating point and matrix multiplication. Owner-adopted [D0-b](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/DECISIONS.md#d0-b--what-no-matmul-at-serving-means-adopted) permits bounded <=4-bit integer/ternary serving maps through additions, subtraction, shifts and table reads, without multiplier or floating numerical kernels; no transformer backbone or hidden provider is adopted. Deterministic geometric address/page selection is allowed. Shared typed operators are current; expert gates remain a conditional future option, not an adopted MoE/sparse-gating architecture.

Refresh [the canonical plan](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/project-track.md), [current evidence/artifact](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/current-state.md) and [source-linked direction](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/model-direction-2026-09.md). Project complete preparation/build/fit/controls/evaluation/retries/checkpoint costs, CPU/threads, wall time, RAM and storage before execution. Cumulative budgets never reset on an issue/session; do not silently raise them. **Standing owner authorization (2026-09-06):** necessary local model/time/storage allowance extensions are already authorized. Record the complete projection, reason, increment and updated cumulative limit before using each extension; retain cumulative charges and the 128 MiB storage stop margin. Do not ask the owner to approve the same class of necessary increase again. This authorizes neither destructive deletion nor paid/external compute, and does not require spending unused allowance. Use focused direct behavior checks and protected PR delivery. Keep positive, negative and NOT_RUN results at their exact scope. Close this issue only when its full acceptance is met; partial PRs say `References`, not `Closes`.

## Reconciliation and preserved history

Owner-directed September 7 reconciliation. Prior issue bodies and the complete 422-issue census are preserved in [the reconciliation record](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/issue-reconciliation-2026-09.json). This rewrite updates ownership and instructions; it claims no new model result. Earlier completed issues remain closed historical evidence.


</details>


</details>

<details><summary>Acceptance and history carried over verbatim from #954</summary>

<!-- active-roadmap-2026-10-01 -->
**October 1 adopted roadmap — READY.** PR #1564 / D19; adoption task #1563.

Parent #1508; shared implementation #973. **Reopened from a September 29 tracking transfer, not from a failed or revoked capability qualification.** The full original acceptance remains below.

Alongside #962/#1552. Preserve source/provenance, unsupported/conflicting cases, query-with-value NoWrite, clarification and causal changed-source/NoRead checks. Report coverage and correctness conditional on answering. A failed candidate search or unresolved compiler is not exact Absent.

[Canonical plan](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/project-track.md) · [Mechanism admissibility](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/mechanism-admissibility-2026-10.md). Historical gates/results and full capability acceptance remain unchanged. Live comments own newer work; this header does not reset cumulative resources or transfer another lab's live files/jobs.
<!-- /active-roadmap-2026-10-01 -->


<details>
<summary>Preserved pre-adoption issue body and original acceptance</summary>

Roadmap position **05/12** · Programme #820 · **Not complete**

## Why this work exists

Fluent output must distinguish supported claims from missing or contradictory evidence before it can support reliable reasoning.

## Evidence already available

Several bounded exact-copy/operator cases pass; C1-SB2 through C1-SB5 negatives and related source-free correctness limits remain scoped historical evidence.

## What to implement

Use independently checkable truth sources and novel supported, conflicting and unsupported cases. Exercise provenance, contradiction resolution, clarification and calibrated abstention through the native model. Preserve NoRead, NoOperation and uncertainty distinctions. Measure response correctness with both whole-population and answered-conditional denominators, coverage and context interventions. Keep open development separate from final evaluation. The historical performance obligations of #954 belong to #963 and must not be silently restored here.

## Acceptance and stopping decision

Grounded complete answers and appropriate clarification/abstention meet declared task-specific criteria on independent final cases. Show causal source dependence and retained exact provenance, including adverse/conflicting cases. A source pointer or perfect copied token does not alone establish correctness.

## Order and dependencies

Consumes model language, memory and operators as available. This is a capability qualification boundary, not a ban on early correctness fixes. The numbered order is the default priority and qualification progression, not a blanket prohibition on necessary fixes or independent work.

## Operating instructions

Use **UOR-R4 Geometric Language Model**, an experimental autoregressive geometric state model with exact addressed memory and learned typed operators. Use Rust for preparation, training, artifacts and inference. Offline Rust training may use floating point and matrix multiplication. Owner-adopted [D0-b](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/DECISIONS.md#d0-b--what-no-matmul-at-serving-means-adopted) permits bounded <=4-bit integer/ternary serving maps through additions, subtraction, shifts and table reads, without multiplier or floating numerical kernels; no transformer backbone or hidden provider is adopted. Deterministic geometric address/page selection is allowed. Shared typed operators are current; expert gates remain a conditional future option, not an adopted MoE/sparse-gating architecture.

Refresh [the canonical plan](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/project-track.md), [current evidence/artifact](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/current-state.md) and [source-linked direction](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/model-direction-2026-09.md). Project complete preparation/build/fit/controls/evaluation/retries/checkpoint costs, CPU/threads, wall time, RAM and storage before execution. Cumulative budgets never reset on an issue/session; do not silently raise them. **Standing owner authorization (2026-09-06):** necessary local model/time/storage allowance extensions are already authorized. Record the complete projection, reason, increment and updated cumulative limit before using each extension; retain cumulative charges and the 128 MiB storage stop margin. Do not ask the owner to approve the same class of necessary increase again. This authorizes neither destructive deletion nor paid/external compute, and does not require spending unused allowance. Use focused direct behavior checks and protected PR delivery. Keep positive, negative and NOT_RUN results at their exact scope. Close this issue only when its full acceptance is met; partial PRs say `References`, not `Closes`.

## Reconciliation and preserved history

Owner-directed September 7 reconciliation. Prior issue bodies and the complete 422-issue census are preserved in [the reconciliation record](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/issue-reconciliation-2026-09.json). This rewrite updates ownership and instructions; it claims no new model result. Earlier completed issues remain closed historical evidence.


</details>


</details>

### How to work on this issue
- Claim with one comment: lab, goal, expected cost; add your `lab:*` label and set `status:in-progress`.
- Post results as short comments with the measured number and the merged PR. No heartbeats or lease renewals here: those go to the compute board.
- Merge to `main` before starting the next piece, then delete the branch and worktree (AGENTS.md, 9 October rule).
- This issue closes only when its acceptance is met on the saved model. A negative result is recorded here and the issue stays open.

## Comments (3)

### comment 6100106227 — 2026-10-10T17:13:26Z — Casey-allard

M3 headline: durable saved-model qualification UNQUALIFIED (target: #2031 acceptance across correction/history, isolation, reset/forget, restart and causal complete answers) · line: durable session integration · count0/3 · next: select one implementation plus saved-model result for the missing reset/forget boundary, preserving the existing exact store/session.

Codex is inspecting M3 while M2's fresh-panel scope awaits owner decision on #2030 comment6100086778. Read #2031 acceptance and open defects #1718/#1738; there are no newer M3 owner directions. Source inspection confirms restart and explicit scope switching already exist, while current snapshot/store APIs do not implement forgetting. The 55-turn #1578 witness was deliberately incapable of accepted name replies under its tiny cap and is not a saved-model capability result. No new headline denominator or qualification criterion has been invented.

Preparation only: locate an existing saved compiler/emitter plus evaluator and identify a concrete operation and complete-answer check before any mutation/model compute. No PR for this reading. Scope being inspected: stack_grounded_session.rs, stack_store.rs, grounded_session_eval.rs and their native-store boundary. No overlapping lab files changed; no GPU lease. Expected initial source assessment <=20min, two workers, <10MiB metadata, charged to the ongoing whole-cycle ledger. Full preregistration with artifact/data identities, bar, steps and cost precedes execution. The M2 constructor mechanism remains open and its numerical source is retained uncompiled/unqualified.


### comment 6100131052 — 2026-10-10T17:16:14Z — Casey-allard

M2 owner scope decision received in chat: use existing128 as transfer diagnostics, reserve new qualification for later. M2 resumes its ordered constructor intervention. This M3 exploration pauses before any source edit, build or model run. Read-only assessment identified learned Op:none supervision for #1718 as a substantive candidate; exact v25 compiler/model cloud index entries located but not yet restored/verified. No M3 result, no mechanism closure and no count increment.

### comment 6137140657 — 2026-10-13T02:52:06Z — Casey-allard

**PM note (kimi/pm1, 13 October).** Status correction only, no scope change: M3 is marked **paused** on the tracker — it waits on the M1/M2 model. The evaluator and session work (#1568, #1578) is delivered but not qualified. No new work is requested here.



---
_Comment/review ids in this file: 6100106227, 6100131052, 6137140657_

