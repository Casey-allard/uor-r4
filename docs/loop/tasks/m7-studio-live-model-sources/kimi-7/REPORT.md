# REPORT: register LexLean Studio repairs, kimi-7

Lab: kimi. Item: [m7-studio-live-model-sources](../../../../roadmap/items/m7-studio-live-model-sources.md) (#3440). Order: [ORDER.md](ORDER.md). References #3440 and #3704.

## What did not work

- The combined implementation PR's local policy showed that a new roadmap item cannot serve as its own merge-base item. This separate registration PR addresses that root cause without rewriting the implementation or weakening the gate.
- No implementation, publication, model change or issue completion is delivered here.
- Generated `docs/roadmap/issues.json` and `docs/roadmap/README.md` are not changed. The previously identified legacy `infra-desktop-client-tauri` status/mapping mismatch blocks canonical sync; this PR does not edit that unrelated record or pretend canonical sync passed.
- An initial local draft used the already occupied `kimi-6` task folder. It was restored before any push, and this `kimi-7` record is the only task history in the final diff.
- Local `loop pr-check` returns **PASS with D76 advisories**, not a claim of clean parent-item scope: the new item path is outside `m7-studio-live-model-sources`'s declared scope, and the restored local draft makes the commit-order check advisory. Both warnings are disclosed here.
- The first fresh exact-head review at `e97b4256c` returned **CHANGES_REQUIRED** for overstated dependency status and review/check claims, an undisclosed scope warning, issue-sync and live-deploy ambiguities, and imprecise measurement wording. Those records are corrected before a fresh review at the new head.

## What changed

Registered `m7-lexlean-studio-tools` as `ready`, depending on the existing Studio source and native-shell items without calling their generated status reconciled. GitHub issue #2826 is closed as completed, while its item file retains the legacy `in_progress` status acknowledged above. The new item cites #3704, whose live body already carries the matching roadmap marker, and inherits the parent's existing owner live-deploy decision. It records the measured LexLean rejection, strict exact-arithmetic behavior, identity errors, two-client evidence, protected delivery and explicit non-capabilities. No other project file changes.

## Measurements

No new measurement is run in this registration unit. The item preserves prior live full-pack probe observations and the implementation's recorded evidence; this PR itself supplies no browser, native or suite measurement and does not publish the implementation record.

## Checks run at the head

- `git diff --check origin/main...HEAD` and `git diff --name-status --stat d416181c1...e97b4256c`: exactly three added files and 118 insertions at the first reviewed head; the corrected diff has 122 insertions across the same three added files.
- `cargo run -q --locked -p xtask -- roadmap check` and `roadmap render --check`: both exit 1 only for the two documented legacy `infra-desktop-client-tauri` errors; the new item's schema/section error was corrected.
- `UOR_LAB=kimi uor-harness loop gates --full`: 11/11 GREEN at the corrected registration head after these record corrections.
- `UOR_LAB=kimi uor-harness loop report --check --order docs/loop/tasks/m7-studio-live-model-sources/kimi-7/ORDER.md`: PASS at the corrected registration head.
- `UOR_LAB=kimi uor-harness loop pr-check --local --base d416181c16923748991a41e2206f5c9ed1de572e`: PASS with the D76 scope and commit-order advisories disclosed above.
- Fresh exact-head review at `e97b4256c`: CHANGES_REQUIRED, not a pass.

## Skills used

Project research/delivery workflow, isolated worktrees, scope discipline, systematic debugging of the merge-base failure, and fresh exact-head review. No model compute, paid resource, new dependency or publication was used.

## Still open

Fresh exact-head review at the corrected head, required GitHub checks and protected queueing remain; their results will be posted on the PR. Then merge this registration, rebase the preserved implementation commit onto the new base, obtain its own exact-head review, and complete protected merge, fresh-main and Pages verification. Close #3704 only when all of its acceptance rows are satisfied. Deferred minor from the first review: the item's “How to start” names the implementation ORDER that arrives with the later implementation PR, not this registration base.

## Plain summary

This PR only registers the LexLean Studio repair item so the implementation can pass the repository's merge-base policy. It changes no runtime behavior and publishes nothing.
