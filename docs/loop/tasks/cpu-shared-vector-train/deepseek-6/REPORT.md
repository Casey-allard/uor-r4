# REPORT: write and verify the trained corpus artifact, deepseek-6

Lab: deepseek. Item: [cpu-shared-vector-train](../../../../roadmap/items/cpu-shared-vector-train.md) (#3505). Base: `45540f256`.

## What did not work, first

- **The merged corpus driver saved a trainer checkpoint but not a model artifact.** That checkpoint contains the artifact bytes, but `uor-chat --stack` and serving-side evaluators require a standalone `UORLUT01` file.
- **Writing the trainer bytes alone would not prove the handoff.** The implementation now reads the file back, compares the complete byte vector, and reparses it with `IntegerStackModel` before a successful report.
- **The final checkpoint was only written, not independently reloaded.** The driver now resumes it and requires the resumed artifact to equal the current trainer artifact.
- **The driver printed its JSON report before checking exact CPU/serving parity.** A mismatch could therefore print a success-shaped report and then exit nonzero. The parity refusal now runs before report construction.

## What changed

- `cpu-corpus-driver` writes `<checkpoint-dir>/final.uorlut` through a temporary file and rename.
- The final artifact is read back byte-for-byte and parsed by the serving engine.
- The final checkpoint is independently resumed and its artifact must match the current trainer artifact.
- A zero-step run must reproduce the source artifact byte-for-byte.
- The JSON report records the final artifact path, SHA-256, successful reload and zero-step source identity.
- Added a temporary-directory control that writes, reloads and parses a synthetic `UORLUT01` artifact.
- Added a corrupt-byte control that refuses before creating either the final artifact or its partial file.

## Measurements

- Focused artifact-export test: PASS.
- Zero-update public seed-1 check (`a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`): the emitted `final.uorlut` has the same SHA-256, equals the source byte-for-byte, and reloads through `IntegerStackModel`.
- No nonzero training, corpus pilot, complete-reply, fresh-phrasing or language result occurred.

## Checks run at the head

- Full release suite: PASS after the known duplicate-artifact retry, 111 library tests plus 1 artifact-export test, with 4 release benchmarks ignored.
- Strict clippy: PASS.
- Formatting and `git diff --check`: PASS.
- D23/D25/D32 ratchets: PASS, 4,451 / 93 / 31 sites and baselines unchanged.
- Local PR check: PASS.
- Full loop gates: PASS, all 11 green.
- Public zero-update fixture: PASS with artifact SHA-256 `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd` and final checkpoint SHA-256 `cbe9b44a35fd48fec15601cdf82fc08a326542ed9bd1d21647c2b43e11221bae`.
- Fresh exact-head adversarial review.

## Skills used

No repository skill was invoked. The work follows the merged corpus-driver and serving-artifact contracts.

## Still open

- The pre-registered two-seed bounded pilot.
- A trained float shadow and private/labeled complete-reply panels.

## Plain summary

The CPU corpus driver now leaves behind a reloadable trained model artifact, not only a private trainer checkpoint, and refuses success unless the checkpoint, trainer and serving parser agree byte-for-byte.
