# ORDER: correct final record discrepancies, kimi-9

Lab: kimi. Item: [studio-ide-harness](../../../../roadmap/items/studio-ide-harness.md) (#3569). Base: `b8caa6a8b9dec35619606537a1874bef3c023a94`, tree `9595f44866fd179f41541ad5ae14f14ed9b8159e`; merged main `fbf15e56f3ab1961b30610893392e7a97fd0ab18`. PR #3890 remains draft and unqueued. Fresh reviewer agent-70 returned CHANGES on records only; the terminal lifecycle, bounds, executed native/loopback evidence, and complete application-input identity passed.

## Findings and decision review

- `kimi-6/REPORT.md` still labels the supported integer-library run `499/499 PASS` although the same report and actual log say 499 passed, 1 ignored. Correct the remaining label without erasing the historical ignored test or the separately disclosed failed unsupported/all-target attempts.
- The PR body names the full-scope and FILEMAP advisories but omits the commit-order advisory that `kimi-8/REPORT.md` says it discloses. Add the missing advisory name; protected queue acceptance remains a separate outstanding check.
- The README and historical typed-shell current-state paragraph use a generic `studio-final-acceptance/<final-head>/identity.json` pointer that does not identify the workbench/PTY record-only delivery head. Distinguish the c663 application-acceptance identity from the record-only delivery identity and logs bound by PR #3890; do not invent an identity file for b8c or later record-only heads.

## Files in scope

- `docs/loop/tasks/studio-ide-harness/kimi-6/REPORT.md`, `docs/integration/current-state.md`, `docs/research/notebook/m4-serving.md`, `docs/products/desktop-client.md`, `docs/roadmap/items/studio-{ide-harness,typed-shell-upgrade}.md`, `web/studio/README.md`, and this task folder.
- `local/kimi/pr-cowork-body.md`, live PR body, preserved agent-70 review output, and exact-head record-only check logs under `local/kimi/harness-parity/`.

No application/model/API/engine/worker/tokenizer change, dependency change, test change, native rebuild solely for these record corrections, scope waiver, or deletion of historical artifacts/other labs' work.

## Done when

- All integer labels agree with the actual 499 passed, 1 ignored result; the PR body names scope, commit-order and FILEMAP advisories; acceptance and record-only delivery evidence pointers are explicit and existing.
- The complete application/artifact input comparison against c663 remains empty except the excluded README record, with actual quiet/name-only results recorded.
- Terminal regression 21/21, repository gates 11/11, local policy, formatting and diff checks pass at the corrected head. Evidence and the agent-70 review are preserved and uploaded/read back.
- A fresh exact-head reviewer accepts the corrected delivery before the PR becomes ready or enters the protected queue. Required GitHub checks, protected queue acceptance, actual merge and fresh-main verification precede cleanup. Do not queue on agent-70's CHANGES.
