# ORDER: register the browser-owned Studio item, kimi-19

Lab: kimi. Item: [studio-ide-harness](../../../../roadmap/items/studio-ide-harness.md) (#3569, registration parent only). New item: `studio-cursor-hosted`. Base: `d4f1e3470ed2681847f33028de3258b5053b63f8`. References #3569; never closes it or changes its acceptance.

## Owner approval and goal

The owner said verbatim: **"I approve of this plan, please implement it end to end autonomously"** (2026-11-09, Kimi chat). Register the item; the top session decides acceptance and scope, never a subagent.

## Cancel probe (ran first)

Instrument/docs only: #3569 is OPEN and unchanged, with no competing item/PR. A new item cannot anchor its own registration ORDER at the merge base, so use this parent route.

## Scope decision and files

Top-session D76 review accepts only the new-item advisory outside the parent's scope, not an implementation waiver or parent acceptance change. Touch only the new item, this task folder and valid official roadmap output. Later code uses the merged item's ORDER. No service, model/API changes, `.github`, rendered STATE/HANDOFF, publication, credentials, real R4 generation, or owner-checkout changes.

## Binding implementation rulings

1. Add `ComposerSurface.tsx` in Task 7. Typed mode has one visible task list and one `AgentRuntime` send path; no parallel legacy `SessionList` or composer send.
2. Give the typed rail independent 1024-aware state. Test `1023_defaults_closed`, `1024_defaults_open`, close/reopen and manual close at 1024; do not change legacy AppShell.
3. Retarget the one skip link to `#harness-main` typed and `#mainColumn` legacy. Scope protected-selector assertions to visible elements under `#studio-typed-root`.
4. Before choosing transport, test concurrent NDJSON-read + POST-intent with flushed frames and real browser headers. If blocked, use `agent_host_poll`; no second listener. Require Host + token; validate Origin when supplied, not assumed on same-origin GET.
5. Native approval is authoritative. No page HMAC key; receipts are opaque host capabilities or asymmetrically verified. Test `page_cannot_approve_its_own_valid_intent` and `receipt_is_opaque_or_asymmetrically_verified`.
6. Resume emits `run.resumed` in a new generation at the next sequence; never repeat start markers. Relax the fixture-only invariant for resume and test continuation without duplicate output.
7. "No tasks yet" is normal; `runtime_unavailable` is initialization failure. Preserve the provider-key exception without copying it to IndexedDB. Keep `run.expired`/`quota_exceeded` out of local UI as NOT_RUN.
8. Test parser duplicate keys or narrow that claim. Define quota refusal/last-verified-snapshot recovery, multi-tab lease races or enforced single-tab refusal, and rejection of float/negative/non-ASCII tool arguments.
9. Each task uses red-first tests, fresh implementer/reviewer, serial writers and declared scope. Replay fixtures are not model evidence.

## Done when

- Record the exact four unchanged base roadmap errors and demonstrate no new item errors. Global roadmap check/render/sync is BLOCKED/NOT_RUN in this PR until base inputs validate; no unrelated repair or hand-generated output. Once the official path works, issue sync/render precedes Task 1 code.
- The item preserves #3569 and excludes hosted service, real R4 generation, useful-model evaluation, production deployment and full-parity claims. PR/commit text uses only `References #3569`, never a closing keyword for it.
- ORDER/REPORT, loop policy/full gates and required GitHub checks pass; fresh exact-head review and protected merge complete. Verify the reviewed source/tree on fresh `main`, confirm #3569 remains open with unchanged acceptance, then clean up this unit's branch/worktree. Later code uses the new item at its merged base.

## Stop and re-scope if

A check requires editing #3569 acceptance, an undeclared source, a service/credential/model dependency, hand-edited generated output, or an unsupported capability claim.
