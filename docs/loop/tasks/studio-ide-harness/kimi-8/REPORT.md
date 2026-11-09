# REPORT: reconcile final evidence labels and complete input identity, kimi-8

Lab: kimi. Item: [studio-ide-harness](../../../../roadmap/items/studio-ide-harness.md) (#3569). Order: [ORDER.md](ORDER.md). Base: `0f9a4838ae4b96a64e58f8e279d93d3cfff86fdc`, tree `48097619e69918925635f7e68f72a1cd39638c6e`; merged main `fbf15e56f3ab1961b30610893392e7a97fd0ab18`. Fresh reviewer agent-69 returned CHANGES on records only; lifecycle and executed native/loopback evidence passed. PR #3890 remains draft and unqueued.

## What did not work

- The notebook labelled final kimi-7 counts, evidence and report as kimi-6, obscuring the different repairs and measurements.
- `kimi-6/REPORT.md` still said 499/499 although the actual log says 499 passed, 1 ignored. `kimi-7/ORDER.md` still named obsolete 35117/kimi-6 rather than c663/kimi-7. `kimi-7/REPORT.md` listed completed final gates as open.
- Scope prose narrowed the historical warning to two manifests even though the actual log spans the full change surface and also includes FILEMAP. Protected queue acceptance was not distinguished from the local warning.
- The first broad input check included the changed README and returned 1, so it could not prove identity. A second narrow list omitted assets/tests/verifiers/launcher paths.

## What changed

- The notebook now has a correctly attributed current kimi-7 row (21/453, c663) and a separate historical kimi-6 row (18/450, 35117). The historical report says 499 passed, 1 ignored and is explicitly historical.
- `kimi-7/ORDER.md` names c663/kimi-7. Its REPORT leaves only fresh review, required checks, protected queue acceptance, merge and fresh-main verification open; completed final gates are not relisted as pending.
- Final REPORT and PR body now describe the full historical scope/commit-order and FILEMAP advisories. The manifest exception is not a general waiver, and protected queue acceptance remains outstanding until actually checked.
- The application/artifact input check now covers the full Tauri tree, every Studio source/test/verifier/asset, launcher/site scripts, typed-runtime test and integer/tokenizer/vision crates. Only the README record is excluded. Both the quiet diff and name-only check returned no application-input differences. No application rebuild was needed for these record-only corrections.

## Measurements

- `git diff --quiet c66377b8fa77bbd225e52acba6539885dedc0c4a..HEAD -- Cargo.toml Cargo.lock .gitignore src-tauri web/studio scripts/studio scripts/tests/test_typed_runtime.py scripts/stage-studio-site.sh crates/uor-r4-integer crates/uor-r4-tokenizer crates/uor-r4-vision pkg ':(exclude)web/studio/README.md'`: **exit 0, no application/artifact input differences**.
- The equivalent `--name-only` check produced **zero paths**. The initial broad check's result of 1 was caused by `web/studio/README.md` only and was not represented as an application-code change.
- Existing evidence remains 21 focused/453 typed tests, 499 passed/1 ignored, default **11/3/21/4/7/13/8**, GUI **13/3/25/4/8/13/9**, plain-JS **113/113**, Playwright **45/45**, launcher **6/6**, **7/7** WASM and **11/11** gates. Final record-only head gates are rerun below rather than inferred from the earlier head.

## Checks run at the head

- `npm test --prefix web/studio -- src/components/TerminalSession.test.tsx`: **21/21 PASS** at the reconciliation head.
- `uor-harness loop gates --full`: **11/11 GREEN** at the reconciliation head.
- `uor-harness loop pr-check --local`: PASS with disclosed historical scope/commit-order and FILEMAP advisories; protected queue acceptance is still a separate required check.
- Evidence is uploaded and read back before cleanup, including `application-input-identity.txt`, `delivery.diff`, native screenshots and actual red logs. The final head/tree belongs in the PR and durable identity, not an invented self-hash here.

## Skills used

Adversarial review reception, source/evidence identity checking, complete application-input comparison, documentation synchronization, artifact preservation, and verification before completion.

## Still open

Fresh exact-head review, required GitHub checks, protected queue acceptance of head scope, actual merge, and fresh-main verification. Full persistent runtime, parallel tasks, memory, Dispatch, browser/computer workflows, live R4 API, production publication, signed release and useful-model capability remain unestablished.

## Plain summary

The records now distinguish historical and current repairs, disclose the real advisory scope, and prove that only records changed after the c663 application acceptance. The implementation itself was not changed by this reconciliation; fresh review and protected delivery remain pending.
