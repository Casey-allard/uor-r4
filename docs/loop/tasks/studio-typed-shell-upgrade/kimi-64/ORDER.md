# ORDER: correct evidence lineage and finish deletion/mutation checks, kimi-64

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Base: `f252285d67d4d4cb7f772dc944c52875f21de10e`, tree `250047b3e34042a5cc7fe94defd5ed697792371c`; merge base: `efee4049e7a9745b5ddecb3a1c2955c4bcddb861`. Agent-41 returned CHANGES and blocked ready/queue on PR #3843.

## Findings to repair

- kimi-62 REPORT claims the later 414/remount/disposal results at its own 411-test head. Restore that head's actual evidence and move later results to kimi-63/kimi-64.
- Notebook omits the direct kimi-62 link. Include the complete chain and current totals.
- The wrong-row log demonstrates DOM attributes but not a production source mutation. Run a real source mutation that marks the wrong row and confirm the replacement assertion fails, then restore the source.
- The generation-specific delete-error branch is untested. Open the modal before generation and prove the stop-first error branch.

## Cancel probe

Run the generation-specific delete test before changing evidence. Temporarily change the row's production active-ID comparison to mark `session-new-1`, run the replacement assertion and require failure, then restore the exact source and rerun it. Preserve both logs and confirm the final tree is unchanged.

## Files in scope

- `web/studio/src/components/SessionList.test.tsx` for generation-specific deletion and source-mutation evidence only; no permanent production behavior change.
- `web/studio/src/components/SessionList.tsx` only for the temporary, restored source mutation used by the probe.
- `docs/loop/tasks/studio-typed-shell-upgrade/kimi-62/REPORT.md` for its actual 411-test head and later-repair pointer.
- `docs/loop/tasks/studio-typed-shell-upgrade/kimi-63/REPORT.md` for current totals.
- `docs/research/notebook/m4-serving.md`, `web/studio/README.md`, and `docs/products/web-client.md` for complete links/current totals.
- This `kimi-64` repair folder.

No permanent production behavior change, real-data migration, production cutover/publication, native/package work, model/worker/tokenizer/`studio_api.js` change, dependency, external endpoint/credential, Phase B compute, useful-chat, M7/M8, or accessibility-certification claim.

## Done when

- kimi-62 reports 411 at its actual head; 414 belongs to kimi-63 and the new total to kimi-64. All direct report links are present and no later behavior is claimed at an older head.
- The generation-specific delete error is asserted. The real source mutation fails the replacement assertion and is restored byte-identically before final checks.
- Full checks and fresh exact-head review pass before protected delivery.

## Stop and report

Stop before claiming later evidence at an older head, leaving a mutation in source, weakening an assertion, or publishing. Continue only after fresh approval.
