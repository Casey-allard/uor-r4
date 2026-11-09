# ORDER: remove the remaining stale live identity, kimi-13

Lab: kimi. Item: [studio-ide-harness](../../../../roadmap/items/studio-ide-harness.md) (#3569). Base: `c6961374c9a2e0990fc292541dd1e2407468c4f1`, tree `4cb8120baf2a62e2dc712c50233cef4debe4302a`; merged main `5643a45c5c82bc9b72be3a5ac9df89183e4f2c3f`. Agent-74 returned CHANGES on PR #3897 at c696; the verdict is posted and the PR remains draft and unqueued.

## Finding and decision review

`web/studio/README.md`'s publishing section still calls `454bb30fc` the current live app. All other agent-73/74 findings are resolved: explicit staged root and 45/45 tests, direct staged 7/7 WASM audit, distinct failed/clean browser logs, executed live/local models.json identity, and open-list scope. Replace the remaining current claim with actual `ab752d59586c595f9409d8c9ba59026a51d4a613` and label 454bb30fc/d8e67c66 as history. Search all current-identity references so no further stale live claim remains; historical records stay intact.

## Files in scope

- `web/studio/README.md`, the earlier reconciliation REPORT and current roadmap/product pointers where they name the active correction, this task folder, live PR body, and final logs.
- No publisher/application/model/API/engine/worker/tokenizer change, dependency change, `.github/**` edit, ad-hoc gh-pages commit, public service, or deletion of prior evidence.

## Done when

- No current-identity reference presents 454bb30fc or another older publish as live. Current `ab752d595`, 794677-byte artifact `e03b6cd564e37d7ed4832aec2a51d730b2b7a83e373ca170aca0282aa0be3821`, and live/local `models.json` SHA `96e296b7…12554a` agree everywhere.
- The reconciliation report and PR body do not claim correction prematurely; current pointers name kimi-13. Local policy and 11/11 gates pass at the corrected head, and a fresh exact-head reviewer approves before ready/queue.
- Actual protected merge, fresh-main dry run/browser verification, guarded publication and evidence preservation precede cleanup.
