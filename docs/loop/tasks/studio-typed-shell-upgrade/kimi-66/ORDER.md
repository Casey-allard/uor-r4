# ORDER: complete evidence chain and mutation reproducibility, kimi-66

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Base: `523ecbaac8f88780100e4f9b04b2bd5c1009cca7`, tree `ce1c6cb82dd3a1da33fc5ec7dbda86aaaeb146b5`; merge base: `8688eb1a9b26e8b934c3badc7bd0cc05179ebd1f`. Agent-43 returned CHANGES and blocked ready/queue on PR #3843.

## Findings to repair

- Notebook links the kimi-57 ORDER but not its REPORT and stops the range at kimi-64. Link every report and state the complete range.
- The source-mutation red log predates the final explicit non-null assertion. Repeat the real mutation against that assertion and restore exact source.
- The immediate-external current verifier was edited after its rejection log, removing the probe. Preserve a self-contained current mutation variant with the probe present and rerun it.

## Cancel probe

Temporarily mark `session-new-1` in production, run the final explicit non-null replacement assertion and require failure, restore exact source, and preserve the empty diff. Preserve and run the immediate external mutation variant, requiring rejection of the reserved `.invalid` URL.

## Files in scope

- `docs/research/notebook/m4-serving.md` for the base REPORT link, full range, and final report link.
- `web/studio/src/components/SessionList.tsx` only for the temporary, restored source mutation.
- `docs/loop/tasks/studio-typed-shell-upgrade/kimi-65/REPORT.md` for the final reconciliation pointer only.
- This `kimi-66` repair folder and local mutation evidence; no permanent production behavior change.

No permanent production behavior change, real-data migration, production cutover/publication, native/package work, model/worker/tokenizer/`studio_api.js` change, dependency, external endpoint/credential, Phase B compute, useful-chat, M7/M8, or accessibility-certification claim.

## Done when

- Notebook includes every ORDER/REPORT link and the full range through kimi-66, with 415 and zero unhandled errors.
- The final explicit replacement assertion fails under the real source mutation and passes after exact restoration. The restoration diff is empty.
- The preserved immediate-external current variant contains its probe and rejects it at runtime; normal checks remain green.
- Full checks and fresh exact-head approval precede protected delivery.

## Stop and report

Stop before an incomplete evidence chain, an unreproducible mutation, a leftover source change, or publishing. Continue only after fresh approval.
