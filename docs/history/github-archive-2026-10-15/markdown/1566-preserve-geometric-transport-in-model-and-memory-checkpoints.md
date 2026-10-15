# #1566 — Preserve geometric transport in model-and-memory checkpoints

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-01T06:33:28Z
- **Updated:** 2026-10-09T13:17:33Z
- **Closed:** 2026-10-01T06:42:07Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1566
- **Merged:** 2026-10-01T06:42:07Z
- **Merge commit:** bbdecac23e7e9bc44c8e2a410a3c679ce2435c41
- **Base / Head:** main / UOR-Foundation:codex/grounded-checkpoint-snap-20261001
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Saving an inference checkpoint currently rejects every model with a transport snap, even though the model serializer now preserves it. Add checkpoint schema 2 that binds the saved transport identity and digest, so a snapped geometric model can reload alongside its exact store. Unsnapped saves retain schema 1 and its existing fields. Unsupported served/QAT state is rejected both before saving and on loading, preventing silent fallback to float variables.

Focused tests cover loaded logit bits, four greedy steps on two causal prefixes, memory updates and a second checkpoint; schema compatibility; file, digest, root-set and configuration mismatches; and served-state refusals. Exact-head [CI36825407561](https://github.com/UOR-Foundation/uor-r4/actions/runs/36825407561) passed formatting, claim/JSON/whitespace checks, the training-library and m-world compile, and all 10 focused checkpoint tests. Two independent nonauthor reviews approved head2d5cfeac with no blockers; their scope and the advisory publisher Not Found error are recorded in PR comments.

This repairs the library checkpoint boundary. A trained R1/S4 artifact witness is NOT_RUN; the format does not contain the learned compiler, transcript or live session cache. No integrated dialogue, D11 export or model-quality result is claimed. No local Cargo or model worker was started.

References #1565, #962, #964.


## Timeline (4 entries; chronological: conversation comments, reviews, review comments)

### comment 5926078572 — 2026-10-01T06:35:47Z — Casey-allard

# Independent checkpoint architecture and compatibility review

Reviewer: Codex subagent `/root/adoption_review_architecture`.
Authorship: nonauthor. I authored neither changed repository file and made no repository edits, GitHub mutations, Cargo invocations, or model-process launches in this review.

Reviewed head: `2d5cfeacd38369d5e61e5f91aa1944fd2959c202`.
Reviewed base: `c877b2bb06ec3a9f33526abfd671a08d2c2dca41`.
Verdict: **APPROVE this exact head. No material issues or unresolved blockers found.** This is a source and architecture approval; the author's actual exact-head compile/focused-test results remain the separate execution requirement before protected delivery.

## Scope and evidence

I read live issue #1565's work card, current AGENTS, both changed files, the existing `StackModel` save/load and transport/QAT implementation, the prior schema-1 checkpoint reader, and the checkpoint's repository callers. I inspected all changed tests without executing them. The checkout remained at the reviewed head with no working-tree changes. `git diff --check` between the reviewed base and head passed.

## Reviewed invariants

- **Compatible schema-1 wire format.** `stack_checkpoint.rs:305-307` defaults missing transport to `None` and omits it during serialization. `:666-686` retains schema 1 and its original restore scope for unsnapped models. `:719-733` rejects a transport field even when null in schema 1, requires a non-null declaration in schema 2, and rejects unknown schemas. The prior reader's exact file allowlist/schema check refuses new snapped checkpoints. The retained unsnapped round-trip test now checks the original field set at `:973-1004`.
- **Snap identity survives save/load.** `:644-658` delegates transport construction to `StackModel::save`, re-reads the saved snap, and binds the file digest. `:534-553` couples declaration presence to the exact sealed file set and verifies SHA-256. `:561-565` additionally compares the actually loaded snap to the checkpoint declaration. The existing dependency at `geometric_stack.rs:1730-1748` checks the transport schema, root count, and ordered-root digest; `:1784` restores the snap through the configuration-checking setter. The new save precheck at `stack_checkpoint.rs:467-469` refuses incompatible public configuration changes before claiming a root.
- **QAT does not silently become float inference.** Active served mode is refused before claiming a save root at `:464-465`. A saved served representation is refused before `StackModel::load` at `:555-560`; this matters because that loader intentionally restores transport but not served/QAT state. The tests at `:1236-1287` cover free and snapped variants, including a resealed training save.
- **Behavioral witness is appropriately scoped.** `:1291-1353` checks that the snap changes the tiny model's logits, then compares exact loaded logits and four greedy autoregressive steps for original and changed prefixes; it also checks all existing store views, post-load updates, a second save/load, and no-store operation. This is a same-CPU construction witness, not a trained R1/S4 or dialogue qualification. `:1356-1518` covers sealed loss, absent/extra/null declarations, schema downgrade, stale/recomputed digests, invalid roots/schema/snap, malformed transport, and incompatible configuration.
- **Existing lifecycle guarantees remain.** Exclusive report-root claim, failed-attempt preservation, complete file-set verification, protocol identity, tensor/configuration validation, parameter count, and store lineage/history/vocabulary checks remain in the load/save path. This change adds no new runtime or model mechanism and alters no other lab's source.
- **Documentation respects the boundary.** `current-state.md:43-57` identifies the implementation as a library persistence interface, explicitly marks the trained R1/S4 witness `NOT_RUN`, and excludes a saved compiler, transcript, live session cache, generated-history conversation, and D11 export.

## Limits

No compile or test execution was performed by this reviewer. I did not replay an actual trained artifact, validate cross-backend or cross-version bitwise reproducibility, or inspect live memory/session behavior. The code does not claim training resume and does not add a dialogue CLI consumer. Those limits are explicit and do not require a new proof campaign for this scoped serialization change.

The new public field and error variant alter Rust source-level construction/matching of `CheckpointRecord` and `StackCheckpointError`, while preserving the old schema-1 file format. Repository search found no external construction sites or matches for the removed error variant; there is no in-tree migration blocker.

Final verdict: **APPROVE `2d5cfeacd38369d5e61e5f91aa1944fd2959c202`**, with actual scoped CI results recorded separately before merge.


### comment 5926078863 — 2026-10-01T06:35:48Z — Casey-allard

# Independent exact-head review — #1565 checkpoint transport

**Verdict: APPROVE. No blocking source or test-design finding.** Executed compile/test evidence remains the delivery owner's separate merge requirement; I did not run Cargo or model inference.

- Reviewer: Codex subagent `/root/adoption_review_evidence`, independent nonauthor of this change.
- Exact reviewed head: `2d5cfeacd38369d5e61e5f91aa1944fd2959c202`.
- Base: `c877b2bb06ec3a9f33526abfd671a08d2c2dca41`.
- Scope: the two-file diff (`stack_checkpoint.rs`, `current-state.md`), the live [#1565 work card](https://github.com/UOR-Foundation/uor-r4/issues/1565), and the unchanged `StackModel` transport/save/load and store-validation interfaces on which the change relies.

## Review findings

**The schema transition is explicit and preserves old writer output.** Schema 1 retains its constant, original scope text and original serialized identity fields because the new optional transport field is omitted when absent. Schema 1 rejects even an explicit null transport field; schema 2 requires a non-null transport identity. The file-set check then requires `transport.json` if and only if that declaration exists. An old reader rejects schema 2/its additional file rather than loading an unsnapped model. The existing schema-1 round trip now asserts the exact old field set, absence of the transport file, and model/store equality. See [record shape](https://github.com/UOR-Foundation/uor-r4/blob/2d5cfeacd38369d5e61e5f91aa1944fd2959c202/crates/uor-r4-training/src/stack_checkpoint.rs#L271-L309), [record validation](https://github.com/UOR-Foundation/uor-r4/blob/2d5cfeacd38369d5e61e5f91aa1944fd2959c202/crates/uor-r4-training/src/stack_checkpoint.rs#L718-L738), and the old-format test at lines 970–1057.

**The snap is bound through independent checks, not merely a declared flag.** Save validates the snap/config before claiming a root and checks that the model's saved transport equals its active snap. Load verifies the complete seal, exact file set, lowercase transport SHA-256, the file digest, and then delegates root count/digest/schema validation and snap/config compatibility to the existing model loader. It compares the loaded snap against the checkpoint declaration. These checks reject a missing transport, an extra transport in a free checkpoint, malformed or wrong-root metadata, incompatible configuration and unilateral schema changes. See [save and load](https://github.com/UOR-Foundation/uor-r4/blob/2d5cfeacd38369d5e61e5f91aa1944fd2959c202/crates/uor-r4-training/src/stack_checkpoint.rs#L451-L579) and [underlying transport loader](https://github.com/UOR-Foundation/uor-r4/blob/2d5cfeacd38369d5e61e5f91aa1944fd2959c202/crates/uor-r4-training/src/geometric_stack.rs#L1726-L1785). As the module correctly states, consistent rewriting and resealing is not authentication.

**QAT refusal is retained and strengthened on reload.** Active served mode is rejected before save claims a root. The loader now checks saved served-representation metadata before `StackModel::load`, preventing that loader's intentional float-only behavior from silently changing a resealed inference checkpoint's declared computation. The focused test covers both free and snapped models, both save refusal/no created root and a resealed training save rejected on load. Malformed metadata propagates a structured error. See [QAT test](https://github.com/UOR-Foundation/uor-r4/blob/2d5cfeacd38369d5e61e5f91aa1944fd2959c202/crates/uor-r4-training/src/stack_checkpoint.rs#L1236-L1288).

**The behavioral fixture is meaningful at its stated persistence scope.** It first asserts that snapping changes the original fixture's logits, then requires bitwise equality after reload across two full-context rows. Four greedy steps feed their generated tokens back, recording every deciding logit row. Both original and changed-prefix traces must round-trip, and the changed prefix must alter the initial logits. This would catch an accidental unsnap and verifies numerical continuation of this initialized CPU fixture; it does not require the changed prefix to change the chosen token. The store is checked independently for full equality, every existing address/history view, Assert/Correct update results and serialized bytes, then a second model/store save-load. The model-only snapped case is also exercised. See [snapped round trip](https://github.com/UOR-Foundation/uor-r4/blob/2d5cfeacd38369d5e61e5f91aa1944fd2959c202/crates/uor-r4-training/src/stack_checkpoint.rs#L1291-L1353).

**Adversarial tests pass the outer seal deliberately to exercise inner validation.** Missing/extra files, null/absent declarations, schema downgrade, transport digest divergence, recomputed-digest wrong roots/schema/snap, malformed JSON and incompatible rotation are separately constructed. Existing model, tokenizer, protocol, memory lineage and store-vocabulary checks remain in the common load path. No model/store invariant was removed. See [presence tests](https://github.com/UOR-Foundation/uor-r4/blob/2d5cfeacd38369d5e61e5f91aa1944fd2959c202/crates/uor-r4-training/src/stack_checkpoint.rs#L1356-L1426) and [content tests](https://github.com/UOR-Foundation/uor-r4/blob/2d5cfeacd38369d5e61e5f91aa1944fd2959c202/crates/uor-r4-training/src/stack_checkpoint.rs#L1429-L1518).

## Evidence limits and execution

This review establishes source coverage, not passing test execution. `git diff --check` passed; the final HEAD check matched the reviewed SHA and the worktree was clean. No Cargo process, model worker, trained-artifact reload, GitHub write or worktree mutation was performed by this reviewer.

The fixture is initialized and executes within one test process. Its store does not feed the model's logits. It is therefore not a trained R1/S4 witness, fresh-process conversation, generated-history grounding, learned compiler, session-cache persistence or D11 export. The [current-state addition](https://github.com/UOR-Foundation/uor-r4/blob/2d5cfeacd38369d5e61e5f91aa1944fd2959c202/docs/integration/current-state.md#L43-L57) accurately marks the trained-artifact witness `NOT_RUN` and identifies the absent CLI/compiler/history consumer. These limitations do not block this scoped persistence implementation; they must remain explicit in the delivery record.


### comment 5926142415 — 2026-10-01T06:41:12Z — Casey-allard

**Exact-head validation and delivery review: PASS for checkpoint scope.**

- Head `2d5cfeacd38369d5e61e5f91aa1944fd2959c202`; base `c877b2bb06ec3a9f33526abfd671a08d2c2dca41`.
- Actual [CI run36825407561](https://github.com/UOR-Foundation/uor-r4/actions/runs/36825407561) checks out that precise head and binds `UOR_BUILD_SOURCE_COMMIT`. Claim wording, policy JSON, diff whitespace and `cargo fmt --check` pass.
- `cargo check -p uor-r4-training --lib --example m-world --locked` passes (1m04s).
- `cargo test -p uor-r4-training --lib stack_checkpoint::tests --locked` passes: **10 passed,0 failed,0 ignored**,0.16s execution after4m58s test compilation. Tests include actual newly loaded CPU model logits/short generated-token continuations and exact-store reads/updates; this is an initialized fixture, not a trained language result or fresh-process dialogue.
- Independent nonauthor [architecture review](https://github.com/UOR-Foundation/uor-r4/pull/1566#issuecomment-5926078572) and [evidence review](https://github.com/UOR-Foundation/uor-r4/pull/1566#issuecomment-5926078863) approve this exact head with no unresolved blockers.
- Root delivery review confirms declared file ownership, schema1 compatibility, bound transport identity, unsupported QAT refusal and scoped documentation. A trained R1/S4 witness remains NOT_RUN; the checkpoint has no dialogue CLI consumer or saved compiler/transcript.

The historical queue checks are acknowledgements, separate from this actual execution. Advisory `mission-delivery-gate`/publisher run36825412071 failed with **Not Found**; it is not claimed PASS. The owner-adopted D19 manual fallback permits exact-head review plus actual checks and the protected queue while deployment remains unverified. No branch protection is changed or bypassed.

Proceeding through protected merge under the owner's standing instruction to merge our reviewed, validated PRs. No local Cargo/model worker, paid compute or unique-material deletion occurred.


### comment 5926169741 — 2026-10-01T06:43:25Z — Casey-allard

**Delivered and verified:** protected PR #1566 merged at `bbdecac23e7e9bc44c8e2a410a3c679ce2435c41` on2026-10-01T06:42:07Z. Refreshed `origin/main` contains that merge, and the **entire merged tree equals** reviewed/tested head `2d5cfeacd38369d5e61e5f91aa1944fd2959c202`. No protection bypass or direct main push.

Actual [CI36825407561](https://github.com/UOR-Foundation/uor-r4/actions/runs/36825407561) passed formatting/claim/JSON/whitespace checks, library+m-world compilation and **10 checkpoint tests,0 failures,0 ignored**. Two nonauthor reviewers approved the exact head. The advisory publisher's Not Found error is preserved separately in the validation comment; it was not claimed PASS.

The completed scope is the versioned checkpoint API: snapped model plus optional exact store, unchanged schema1 output for unsnapped models, transport/root/digest validation and explicit QAT refusal on both save and load. Tests actually exercised newly loaded CPU models, exact logits, short greedy continuation and store updates. They use an initialized fixture. **Trained R1/S4 witness and fresh-process learned dialogue are NOT_RUN**; the issue's optional artifact witness was unavailable through the existing consumer and no stand-in model-quality claim is made. This API has no dialogue CLI consumer and does not retain compiler/transcript/cache state.

This satisfies #1565's scoped persistence deliverable. Parent #962/#964/#973 and the overall model objective remain open. The next integrated #962 work card consumes the real saved compiler with exact store, trained emitter and generated-history/fresh-process continuation. Claude's compiler/data work and jobs remain theirs.

Accounting: local Cargo/model workers0. Free GitHub compile1m04s; test compilation4m58s; tests0.16s. Complete orchestration/source/review/delivery elapsed is charged cumulatively through the internal immutable resource ledger, not reset from the stale external model-time view. Receipts/reviews/logs are retained in the task's `uor-r4-plan-adoption/checkpoint-snap` archive; no unique material was removed.



---
_Comment/review ids in this file: 5926078572, 5926078863, 5926142415, 5926169741_

