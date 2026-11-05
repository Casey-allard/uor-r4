# REPORT: cpu-training-pilot, deepseek-176

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `1b78a0652`.

## Status

Complete bookkeeping release for the merged deepseek-175 reference-prefix
copy-candidate ranker.

## What did not work

Nothing in this bookkeeping unit. The released experiment remains
**INSUFFICIENT_TRAINING_DATA**: reference-prefix supervision exposed 85
supervised groups against the registered 640-group floor, with no perceptron
fit or reply evaluation.

## What changed

- Recorded reviewed head
  `b35b19c802430bfeeb9cba28908a4048f3174704`, PR #3723 and protected merge
  `1b78a0652fa419c0200788e0ba82b15f480c9229`.
- Recorded fresh-main CPU library and binary verification.
- Removed the completed `deepseek/copy-ranker-reference` claim.
- Changed no model, serving, CUDA, float-inference, termination or Claude-owned
  source.

## Measurements

- experiment reviewed head:
  `b35b19c802430bfeeb9cba28908a4048f3174704`;
- protected merge:
  `1b78a0652fa419c0200788e0ba82b15f480c9229`;
- fresh-main CPU library: **315 passed**, 4 expected ignores;
- fresh-main CPU binary test: **1 passed**;
- fresh-main raw local manifest and published evidence: verified;
- fresh-main no-corpus boundary: pass.

## Checks run at the head

- Fresh-main CPU tests at the protected merge.
- Raw local and published evidence manifests.
- Claims JSON parsing and absence of the released claim.
- Report and local PR policy: pass.
- `loop gates --full`: 11/11 green.
- `uor-harness check`: 0 errors, warnings only.
- `xtask docs-catalog --check`: pass.
- `git diff --check`: pass. The release changes documentation only.
- The global `xtask links` scan still reports 1,271 pre-existing broken links;
  every relative link added by this release resolves.
- `xtask roadmap check` still fails on the pre-existing
  `infra-desktop-client-tauri.md` status and stale `issues.json` entry. Neither
  file is changed by this release.

## Size

This release changes four documentation paths against base `1b78a0652`.

## Skills used

No system-provided Codex skill applies. Project delivery procedures were used.

## Still open

Useful local chat remains open. The next ranker attempt needs a broader
candidate construction or denser objective, a new seed and a new registration.

## Plain summary

The reference-prefix ranker implementation and its fit-stop evidence are
merged and verified on main. This PR records delivery and releases the
completed claim.
