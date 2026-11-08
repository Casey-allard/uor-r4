# REPORT: close agent/session verification and evidence defects, kimi-58

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Base: `7e9b83be0700a897f042241d13591b0ee70967bc`, tree `8c57b7f9ec19a7588f33fc2e333fbe69e8e08d03`; original merge base: `8903c5551a5e9c3953bd6a2eab1322c7e6c276c2`. Current merge-base/HEAD/tree are preserved in `merge-base.log` and `identity.log`, not embedded self-referentially. Agent-36 returned CHANGES at the base and blocked ready/queue on PR #3843.

## What did not work

- Create accepted any existing active summary after a no-op boolean, duplicate accepted the source ID, and remove accepted an unchanged target. These were false operation-success paths, not merely incomplete tests.
- Legacy rename truncates to 80 code units; exact comparison against an 81-character input returned failure after the real write. Direct active-ID reads could throw from Send. A synchronous stopped event cleared `activeGeneration` before interruption, leaving identity stale.
- Browser request arrays were computed before typed interactions, and a generic `42` satisfied the response check. Re-sampling MCP after returning to Agent initially produced an instrument-only zero; Models and post-workflow counts are now separate and truthful.
- Remaining doubles encoded impossible creation/deletion behavior, the reducer accepted an active ID absent from its catalog, failed-start session identity lacked a direct assertion, and REPORT wording implied fresh-main evidence existed. Bulk Settings clearing was described too broadly as a controller mutation. At this repair stage the command-group assessment was wrong: base comparison had not been performed, and the test checked only Agent. kimi-59 later proved five commands had moved out of the base `Navigation` group and restored all six; the claim of no product change is superseded.

## What changed

- Create requires a different newly cataloged active session absent from the before snapshot. Duplicate requires a new non-source entry and unchanged active ID. Remove requires target absence and a verified resulting active session, preserving unrelated active identity where required. Resume still permits the current session.
- Rename normalizes to the legacy 80-code-unit bound before calling and verifies the stored bounded title. The input exposes `maxlength=80`. Send and generation active-ID reads fail closed with no model invocation or uncaught exception.
- The reducer rejects absent active IDs and accepts interruption from either the active or just-stopped generation snapshot when its prior identity still matches. Synchronous stop completion now synchronizes the verified new identity and preserves the old session's stopped snapshot.
- Request/external/raw-TypeScript capture occurs after typed evaluation. Exact responses require `17 + 25 = 42`, `R4 geometric exact arithmetic`, and the `p0: 7 + 5 + 0 -> 2, carry 1` trace. Models records `typedMcp=1`; returning to Agent records `typedMcpAfterWorkflow=0` as expected unmounting, not a duplicate/missing-panel defect.
- Documentation narrows named per-session operations versus bulk Settings clearing, corrects fresh-main NOT_RUN, updates totals, and cites this repair chain. Impossible doubles were replaced with real catalog/active-ID behavior.

## Measurements before the final committed repair head

- Red-first focused result: **9 failures / 91 passes**. Failures included no-op create, source-ID duplicate/no-op deletion, long rename, throwing active-ID reads, synchronous interruption, absent catalog identity, and missing rename bound. After repairs: **100/100 focused tests** pass.
- Complete typed suite: **405/405 tests across 47 files**. TypeScript and Vite build pass; typed main JS **88.34 kB** (gzip **28.47 kB**). Plain-JS remains **112/112**. All four legacy checks pass.
- Normal built browser: `created=true`, all resume ID/marker pairs true, exact response present, absent in the other transcript, restored on return. Post-workflow typed requests **12**, all local and no raw TypeScript; default **23**, all local and no raw TypeScript; zero page runtime errors. `typedMcp=1`, `typedMcpAfterWorkflow=0`.
- Mutation evidence: the old verifier accepted a planted `https://interaction.example.invalid/probe.png` request and generic `42`. The repaired verifier rejects the URL explicitly. With the URL removed but the same one-call generic transcript, it rejects `answerReady=false` even though the subsequent real transcript contains the exact trace. These are instrument proofs, not external endpoint use.
- No real browser data, credential, external endpoint, dependency, second provider, or Phase B compute was used. Exact arithmetic remains protocol evidence, not useful conversation.

## Checks run at the head

Red/focused/full logs are under `local/kimi/studio-agent-session/`; mutation copies/results are under `verifier-mutation/`. Exact-head identity, full-suite, four-legacy, harness/report/local-PR/full-gate, and built-browser logs are preserved in a head-specific directory after commit. Fresh-main logs and fresh review are **NOT_RUN** at this writing. Actual advisories are the notebook scope and existing FILEMAP rows; read-path passes. Agent-37 subsequently returned CHANGES for command grouping, repeated interruption, fresh-main wording, and smaller evidence/double gaps; kimi-59 owns those repairs.

Agent-36 returned CHANGES at the base. The follow-up fresh exact-head review, PR update, GitHub checks, ready/queue/merge, and fresh-main verification are **NOT_RUN**. PR #3843 stays draft and uses `References #3500`; issue #3500 remains open. No production publisher was invoked.

## Skills used

Receiving review with source verification, systematic debugging, red-first TDD and honest-test guidance, solution integrity, accessibility/native-form constraints, and verification before completion. No dependency or production cutover was added.

## Still open

This repairs the mounted agent/session authority and its evidence. Real browser-data migration, final lifecycle ownership for other subsystems, final visual acceptance, native/shared packaging, packaged macOS acceptance, and production publication remain later Phase A units. No useful-chat, M7/M8, accessibility-certification, or Phase B claim is made.

## Plain summary

Session operations now prove their effects rather than trusting return values. Long renames match legacy behavior, reads fail closed, synchronous stops preserve identity, and browser checks catch post-workflow requests and generic responses. The full product and verifier evidence is green, but the new exact-head review and protected delivery have not run.
