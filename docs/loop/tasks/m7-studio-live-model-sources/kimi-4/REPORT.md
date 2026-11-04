# REPORT: publish the merged Studio while preserving the existing trained catalog

Lab: kimi. Item: [m7-studio-live-model-sources](../../../../roadmap/items/m7-studio-live-model-sources.md) (#3440), follow-up to merged #3639 and #3640. Order: [ORDER.md](ORDER.md). Base: `15c974d133c38301a4e32bc298a2936b7f9883e7`; previous live `gh-pages`: `454bb30fcab8580db3320717685319a7c80a2d9b`. Publish candidate: `233b97fb9780699db3409fe3ce1fa67aa0408e6d`, tree `eceaab6fd6fa1e5bbadfe90e0197d10dbba2abdf`. Actual push, Pages build and live records are added only after they exist.

## What did not work

- The full publisher cannot regenerate the older M1 discrete-trunk receipt required for a new trained-model manifest. The temporary original manifest and M1 receipt were unavailable; the FBS seed-1 receipt passed independently, but that does not authorize substituting or dropping M1. The existing published catalog is therefore restored byte-for-byte in the local publish commit. This is an incremental Studio update, not a new trained-model release or a bypass of receipt validation for a new artifact.
- The first amended publish candidate included development source, tests, package/build configuration, declarations and a nested verifier. Independent review returned CHANGES for that publication-hygiene defect. Those paths were removed before any push; the final diff is only 11 runtime changes and deletion of non-runtime `pkg/README.md`.
- No production source or publisher code was changed. No public push occurred during the dry-run or either review. Actual live/Pages verification is not preclaimed.

## What changed

Built the Studio at source base `15c974d13` plus the publish ORDER `9847357ab`. The local publish commit restores the previous `models.json` exactly and retains both M1 seeds and both research FBS seeds. The final cleaned candidate has only these runtime changes: `composer_input.js`, `image_input.js`, `model_sources.js`, `stream_metrics.js`, `studio_api.js`, `studio_files.js`, `studio_ui.js`, `index.css`, `index.html`, `pkg/uor_r4_integer_bg.wasm`, and removal of non-runtime `pkg/README.md`. `studio/models.json` and `studio/models/` have no diff. No private artifact, credential, retired router WASM, source library, tests, package/build configuration, declarations or verifier is published.

## Measurements

- Build/worker: ALL STUDIO WORKER CHECKS PASSED, including save/restore identity and zero remote requests.
- Static and runtime page traps: PASS with `external: []`; the planted external trap fires.
- Tokenizer: `TOKENIZER ARTIFACT MATCHES THE STACK`, seed-5 SHA-256 `d60f88d6…`.
- WASM: 700,628 bytes, SHA-256 `b4ab83f124907b4e4e9120999ef7b70356c5e7ab710629aa7136bd1b36b70d42`; instruction audit 7/7 with zero floats/unclassified instructions.
- All eight public model/tokenizer URLs returned HTTP 200 before publish. Local browser inspection showed the new dark interface, Connections navigation, and all five model cards (synthetic, two M1, two FBS).
- `git diff --quiet HEAD~1 -- studio/models.json studio/models` passed; catalog SHA-256 is `96e296b74530e8488a6c525c59b4dfbd9f980f519ce39c1c338ed7a5cd12554a` before and after.
- `git merge-base --is-ancestor 454bb30fc… HEAD` and `git diff --check` passed; `origin/gh-pages` remained `454bb30fc` at the final pre-push check.

## Checks run at the head

Implementation base `15c974d13`; ORDER head `9847357ab`; cleaned publish head `233b97fb9`. The independent final publish review returned **APPROVE** with no Critical or Important findings. `gh-pages` advanced by fast-forward `454bb30fc → 233b97fb9` without force. Pages build `1339751024` reports **built** at `233b97fb9` with no error. The live `models.json` is byte-identical to the previous catalog (SHA-256 `96e296b7…12554a`), the live WASM is 700,628 bytes with SHA-256 `b4ab83f1…70d42`, and the hosted browser shows the new dark interface and all five model cards. The browser tab was closed.

## Skills used

Existing build/publish workflow inspection, independent publication review, exact catalog/artifact identity checks, local built-in-browser preflight, and fresh verification after every local amendment. No GPU, paid compute, new dependency, new trained model, or publisher-code change was used.

## Still open

Only the live-record PR and cleanup remain. Static Pages does not provide the native loopback service or trained visual understanding. No useful-chat or M7/M8 qualification claim is made.

## Plain summary

The merged Studio is live at https://uor-foundation.github.io/uor-r4/studio/ with the polished dark interface and every previous model card preserved. The Pages build and hosted file/browser checks pass; development-only files were removed before publishing.
