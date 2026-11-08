# REPORT: one-model-end-to-end, deepseek-28

Lab: deepseek. Item:
[one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](./ORDER.md). Laptop preparation only so far; no model,
pod or spend.

## What did not work, first

- The 1,024-step run reached outcome row 6, not the positive row 3: EOS top-1 and
  complete accepted replies remained zero.
- The final 1,024-step checkpoint regressed on free EOS stops and continuation.
- The extension therefore tests dose rather than claiming the recipe already
  works.

## What changed

- Adds the 2,048-step pre-registration and active execution ORDER.
- Parameterizes the reviewed launcher and job so the same code supports the
  legacy 1,024-step unit and this one registered extension.
- Requires a distinct run/store label and reviewed sheet/report paths.

## Measurements

No model or GPU step has run. Empirical calibration from the completed 1,024
run projects about 24 minutes of training and 45-50 minutes of pod wall time,
for roughly $0.45-$0.55 at the prior $0.563/hour offer.

## Checks run at the head

- `gate.sh`: 221 checks passed.
- `test-mutants.sh`: 84 of 84 semantic mutations caught.
- `bash -n`, ShellCheck at warning severity and `git diff --check`: passed.
- `uor-harness check --only prep`: the 2,048-step sheet has every required
  block and no shape error.
- The job rejects an unregistered 1,025-step dose, and the launcher rejects a
  2,048 dose paired with the 1,024 run label before repository or network work.
- Fresh-context FIT: **FIT GO**, no blockers, at
  `a628f4e9ee3f418e5e527a7d8ff34d3ad0bc7d5f`, runtime
  `0964a28196410a44a7d6af28194731a1003d0bb211d890b7a00fbaf2cf7153a3`,
  reviewer session `01a1b8dd-6013-7da2-9d34-53490ddef0e5`.
- Fresh D67: **NO-GO** at `7dd28ece684aabf0dea72f412cbb20769dce6110`.
  The prerequisite sheet was changed without appearing in the ORDER scope, and
  the read-only reviewer sandbox could not perform live pod and store
  admission. The ORDER now names that sheet; live admission is rechecked below.
- Corrected-head FIT: **FIT GO**, no blockers, at
  `5e59d01d9bfa1ce533eeb451b1b9f9cde97c2708`, runtime
  `e69e05e968919f6d4f3f1b282f0af7934a3b0e86cd2e5e1616ca38dbe39b234a`,
  reviewer session `01a1b8eb-6f3d-7dc2-97c7-ebdcb679fc05`.
- Live admission from the writable controller found one running pod,
  `vast-59635959`, at 96% GPU and $0.563/hour, so the project total is one of
  four pods and $0.563 of $8/hour. No uncontested GPU is currently free; the
  verified RTX 5090 offer remains $0.563/hour. The store topic contains only the
  immutable 1,024-step `s1-20261108t005743z-a849820fba9cfbf4` root, so the
  registered `s1-2048` prefix is absent.
- Fresh D67 after correction: **GO**, no blockers, at
  `fa5d7b0874159b17b0486248f5abd239269cfad3`, runtime
  `e69e05e968919f6d4f3f1b282f0af7934a3b0e86cd2e5e1616ca38dbe39b234a`,
  reviewer session `01a1b8f1-79b0-7c53-a3ed-0bda5eee206e`. It independently
  downloaded and re-hashed all 18 remote inputs, verified the prior source
  seal, reran 221 gate checks and 84 mutations, and confirmed admission at two
  of four pods and $1.126 of $8/hour with the new pod counted.

Not run: pod creation, remote build, training, export, panel scoring, store write
or seal readback.

## Skills used

No system-provided Codex skill applied. The repository's loop, compute and
geometry rules were used directly.

## Still open

- Execute and evaluate the 2,048-step dose; no automatic second extension.

## Plain summary

This unit prepares the one registered 2,048-step extension from row 6.
