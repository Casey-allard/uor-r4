# Typed Studio live publication design

## Intent

Update the existing public `/studio/` page from the November 7 legacy build to the reviewed neutral workbench on merged `main`, without publishing an unmerged branch, changing the legacy publisher's default behavior, or implying hosted terminal/agent capability.

## Goals and non-goals

- Add an explicit `--typed` publication path to `scripts/studio/build-and-publish.sh` and `scripts/stage-studio-site.sh`.
- Build the current Vite dist after generating the WASM/tokenizer artifacts, stage exactly that dist through `runtime-files.sh --typed`, regenerate the shared catalog, and verify the staged output before any push.
- Preserve the current legacy command, allowlists, public model/trained-model inputs, Pages limits, dry-run default, main-only live guard, and rollback output.
- Do not change `.github/**`, model/API/engine/worker/tokenizer code, the R4 API contract, native terminal behavior, or production agent/hosting claims. Static web terminal and native workflows remain explicitly unsupported.
- Do not add a dependency or ad-hoc direct-to-pages upload.

## Approaches

1. **Chosen: opt-in typed mode.** Reuse the current dist, typed runtime allowlist, catalog, publisher staging, verification and rollback. It isolates the new behavior and leaves legacy defaults reversible.
2. Change the default publication to typed. Less command input but silently changes the established legacy command and rollback expectations.
3. Manually copy dist to gh-pages. Rejected because it bypasses the standard allowlist, catalog, staged verification and protected delivery evidence.

This approach is the lead's implementation choice under auto permission and the owner's explicit request to update the existing WebUI; the product direction and typed-publication requirement already exist in the approved harness roadmap.

## Implementation parts

- `stage-studio-site.sh`: parse `--typed`; select `web/studio/dist` as the source; call typed copy/verify while retaining the same catalog and trained/public-model checks.
- `build-and-publish.sh`: parse and document `--typed`; after the existing artifact build and legacy verification, run `npm ci` and `npm run build` in typed mode; pass `--typed` to staging and print the mode. No `--skip-verify` weakening.
- Tests: red-first contract tests prove the current publisher has no typed path, then prove typed staging selects dist, includes the neutral bundled assets, rejects legacy/source/map files, and preserves default legacy staging. A staged browser check must mount the neutral workbench with zero external/raw-TypeScript/runtime errors before any live push.
- Records: new ORDER/REPORT under `studio-ide-harness`, product capability text and PR evidence distinguish static preview publication from native/hosted agent capability.

## Verification and rollout

Run focused script tests, typed runtime tests, complete frontend/browser suites, repository gates and local policy. Obtain fresh exact-head review and protected merge. From a fresh detached `origin/main` checkout, run the typed publisher in dry-run mode with the publish worktree retained, inspect and browser-test its staged output, then push only the reviewed local publish commit to `gh-pages` using the script's printed path. Watch Pages deployment and hard-reload the public URL to verify the neutral workbench. Preserve source/staged/published identities and rollback head before cleaning generated outputs.
