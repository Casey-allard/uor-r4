# REPORT: remove the remaining stale live identity, kimi-13

Lab: kimi. Item: [studio-ide-harness](../../../../roadmap/items/studio-ide-harness.md) (#3569). Order: [ORDER.md](ORDER.md). Base: `dca1327c135ccec374e79eab3574e74636a7c63b`; merged main `5643a45c5c82bc9b72be3a5ac9df89183e4f2c3f`. Agent-74 returned CHANGES on PR #3897 at c696 and the verdict is posted. The PR remains draft and unqueued.

## What did not work

The README publishing section still called `454bb30fc` the current live app, contradicting the actual `ab752d59586c595f9409d8c9ba59026a51d4a613` identity used by the corrected top-level README and product records. This made the earlier reconciliation claims incomplete even though the explicit root/browser/catalog evidence was valid.

## What changed

- The publishing section now identifies `ab752d595`, its 794,677-byte artifact and `e03b6cd564e37d7ed4832aec2a51d730b2b7a83e373ca170aca0282aa0be3821`, and labels 454bb30fc/d8e67c66 as history.
- Current roadmap, desktop, typed-shell and current-state pointers name kimi-13. The earlier REPORT records agent-74's third finding and leaves only outstanding review/publication work.
- The PR body no longer claims the identity correction was complete prematurely; it records agent-74 and the final fix.
- No publisher/application/model/API/engine/worker/tokenizer behavior, dependency, public service, or native consent changed.

## Measurements

- Current identity is consistently `ab752d595`, 794,677 bytes, SHA-256 `e03b6cd564e37d7ed4832aec2a51d730b2b7a83e373ca170aca0282aa0be3821`.
- Existing explicit-stage evidence remains **45/45** on port 4174, direct staged WASM **7/7**, and clean browser-load 14 requests with zero external/raw-TypeScript/runtime errors. The failed cleanup is separately labeled.
- Live/local `models.json` remain 4,365 bytes with SHA-256 `96e296b74530e8488a6c525c59b4dfbd9f980f519ce39c1c338ed7a5cd12554a`.
- Existing typed **453/453**, plain-JS **113/113**, runtime **15/15**, and default/GUI matrices remain green; final report-head local policy and **11/11** gates are rerun after this REPORT is committed.

## Checks run at the head

- Searched all current-identity references: the remaining `454bb30fc` statement in this README is replaced; older identities elsewhere are explicitly historical. No current `454bb30fc` live claim remains.
- `uor-harness loop pr-check --local`: final rerun after the REPORT commit. `uor-harness loop gates --full`: final **11/11** rerun. Fresh exact-head review and required GitHub checks remain outstanding.
- Public gh-pages is unchanged at `ab752d595`; no typed publication, remote push or fresh native exit-0 claim is made.

## Skills used

Adversarial review reception, current-identity audit, documentation synchronization, and verification before completion.

## Still open

Fresh exact-head review, required checks, protected queue acceptance and actual merge, fresh-main typed dry run and explicit staged-browser verification, guarded publication, Pages verification, evidence preservation, then branch/worktree/build-output cleanup. Full agent runtime and live R4 API remain unbuilt.

## Plain summary

The remaining stale live claim is removed. The publisher and explicit staged evidence are unchanged; only the identity record and active pointers were corrected before a fresh review.
