# #1577 — Restart grounded conversations while retaining exact scoped memory

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-01T15:16:43Z
- **Updated:** 2026-10-09T13:17:33Z
- **Closed:** 2026-10-01T15:28:40Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1577
- **Merged:** 2026-10-01T15:28:40Z
- **Merge commit:** 3144a383c059cb38c1df0aff7a8531f22d77f2a8
- **Base / Head:** main / UOR-Foundation:codex/grounded-conversation-restart-20261001
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

A grounded session has one identity and accumulates its transcript until admission limits are reached. Add a conversation restart over its retained exact memory, with optional explicit caller scope/entity selection, so the next emitter input begins from BOS while durable versions remain accessible at their own address.

`start_conversation` validates the compiler and proposed identity before mutation, clears both prior turns and generated IDs, and starts at the current store commit. Model/compiler/tokenizer/store bytes and limits stay bound; record IDs/commits remain global while new write sources count from the new conversation. `grounded-session restart` saves a new sealed envelope using the existing format. It supplies no authentication or forgetting guarantee; old envelopes and tombstone values remain retained.

Validation: [exact-head CI36883045005](https://github.com/UOR-Foundation/uor-r4/actions/runs/36883045005) passed library/CLI compilation, focused session lifecycle and compatibility tests, unchanged store tests, actual saved learned compiler + geometric/pointer emitter CLI construction, including independent scope/entity switches and reload-equal outcomes/generated IDs. 26parent/integration tests and2explicitfresh-process helpers passed,0failures. Independent nonauthor review approved the same4ff16de9head. The realCLI witness compares14ordinary-text turns and6restart processes, with3refused invalid/existing-output attempts. The inspected advisory publisher NotFound stays separate; D19manualfallback uses actual review/checks and protectedqueue. Initialized emitter and authored fitting rows are construction evidence, not unseen dialogue or selected-trained-emitter qualification.

References #962. Claude's paired fit and DeepSeek's source remain theirs. Internal source work/free GitHub CI only; no local Cargo/model worker or SSD write during the owner storage pause. Full #962 and D11/D5 remain open.


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5934434620 — 2026-10-01T15:17:37Z — Casey-allard

Independent nonauthor source review APPROVE, exact head4ff16de98809570aff226599016c0f908e46ecc6, reviewer /root/adoption_review_evidence (none of4paths authored by reviewer). Session module hash a673281373d0741dbddda8ff54d4b87e4ae3395e49117050e7c9d16318fb0eaf. Compiler/newidentity validation precedes mutation; retained exactstore/model/compiler/limits stay bound; both old turns and generatedcontext clear; global IDs/commits and new conversation-local sources match existing loader. Focused tests target version/conflict/eviction and store-limit preservation, independent scope/entity addressing, atomic refusals, save/reload and priorseal preservation; CLI uses saved learned compiler and actual subprocesses/fulloutcomes. Claims correctly exclude auth/forget/ancestry/trained-emitter qualification. Source approval only; no tests executed by reviewer. Exact-head CI pending.

### comment 5934635684 — 2026-10-01T15:27:42Z — Casey-allard

Exact-head execution SUCCESS: https://github.com/UOR-Foundation/uor-r4/actions/runs/36883045005
Workflow commit243e53493e330be7e9e8d68951b1eb4cd5ef2879 checks out and explicitly verifies reviewed PR head4ff16de98809570aff226599016c0f908e46ecc6.

Executed checks: cargo fmt --check; claim wording and policy JSON; git diff --check; cargo check -p uor-r4-training --lib --example m-world --bin grounded-session --bin temporal-compiler --locked (1m00s).
Focused tests: temporal_compiler::tests4parent PASS plus1explicit fresh-process helper; temporal_cli1PASS; stack_grounded_session::tests12parent PASS plus1explicit fresh-process helper; stack_store::tests9PASS. Total26parent/integration+2explicitchild PASS,0FAIL. The2helpers are ignored by the parent filter and explicitly executed by those tests; no unavailable result inferred. Test compilation4m46s plus31.40s integration compile. Actual CLI execution37.34s; session tests0.23s; temporal tests0.71s; store0.00s. These are test/compile timings, not serving benchmarks.

Actual CLI witness fits/saves/loads the learned composite compiler and uses an initialized geometric/icosian pointer emitter. Nine ordinary-text historical turns plus5fresh-conversation query turns match complete loaded outcomes and generated/history IDs. Scope/entity cross-product and return select independently specified found/absent records;6successfulrestart processes include defaults;3invalid/existing-output attempts refuse. API tests cover actual emitter save/reload, restart global commits/local sources, exact store/conflict/eviction/views and retained record limit, compiler/identity refusal atomicity and old snapshot preservation.

Nonauthor source APPROVE at samehead: https://github.com/UOR-Foundation/uor-r4/pull/1577#issuecomment-5934434620
Advisory delivery publisher36882992284 inspected and failed NotFound; preserved separately, notPASS. Five compatibility status names execute no tests. D19manualfallback permits recorded source review + actual checks + protected queue. No fabricated status, protection bypass, local Cargo/model worker, SSD write or paid compute.

Authored rows/initialized emitter are construction evidence; selected-trained-emitter temporal/scope/reset complete-answer witness remainsNOT_RUN. No authentication, forgetting/erasure, geometric-advantage, D11/D5 or full #962 qualification promoted.



---
_Comment/review ids in this file: 5934434620, 5934635684_

