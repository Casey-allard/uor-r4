# ORDER: publish the merged Studio while preserving the existing trained catalog

Lab: kimi. Item: [m7-studio-live-model-sources](../../../../roadmap/items/m7-studio-live-model-sources.md) (#3440), follow-up to merged #3639 and #3640. Owner request: "merge your prs and give me the hosted page when its ready". Base: `15c974d133c38301a4e32bc298a2936b7f9883e7`, tree `0bafb1269abc4b6c6cf61b5ff14923f8ab2c8f2c`; live `gh-pages` starts at `454bb30fcab8580db3320717685319a7c80a2d9b`.

## Goal and constraint

Build and publish the current Studio to the existing `/studio/` URL. The full publisher cannot regenerate the older M1 discrete-trunk receipt required for a new trained-model manifest. Do not drop M1, substitute another model, bypass receipt validation, or change publisher code. Use the existing dry-run build, then restore the already-published model catalog byte-for-byte in the local publish commit. This is an incremental Studio update, not a new trained-model release. The final publish diff must contain no change to `studio/models.json` or `studio/models/`.

## Files in scope

- Existing build/publish commands only; no source edits to `scripts/**`, `.github/**`, or Studio runtime code.
- `docs/loop/tasks/m7-studio-live-model-sources/kimi-4/**` for this ORDER and its REPORT.
- Narrow live-record updates after measurement in `web/studio/README.md`, `docs/products/web-client.md`, `docs/roadmap/items/m7-studio-live-model-sources.md`, and `docs/research/notebook/m4-serving.md`.

## Done when

- The existing dry-run build, worker/static/runtime checks, tokenizer check and WASM audit pass at `15c974d13`; no external page-load requests occur.
- The local publish commit is inspected and amended before any push to restore the previous catalog. `git diff HEAD~1 -- studio/models.json studio/models` is empty, both M1 seeds and both research FBS seeds remain listed, and no private artifact is added.
- `origin/gh-pages` has not advanced beyond the expected parent; push is fast-forward with no force. The actual Pages build succeeds.
- The live URL serves the new Studio, the expected WASM hash and the byte-identical previous catalog. Browser inspection covers the dark interface and model sources. No native-loopback-only capability is claimed for static Pages.
- Live records and the REPORT carry exact commit/tree/hashes and all negatives. The record PR uses References, not Closes. Merged worktrees and reproducible build targets are cleaned after verification.

## Stop and report conditions

Stop before dropping or altering a model card, bypassing a receipt check, changing source/publisher/workflow code, publishing private artifacts, force-pushing, or treating a staged build as a live deployment. If another publisher advances `gh-pages`, stop and revalidate instead of overwriting it. Preserve unique evidence and keep at least 30 GiB free after cleanup.
