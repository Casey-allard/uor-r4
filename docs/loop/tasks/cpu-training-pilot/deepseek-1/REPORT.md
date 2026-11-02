# REPORT: pre-register the bounded public CPU rank-margin pilot, deepseek-1

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `02767ac73`.

## What did not work, first

- **The first sheet called its public evaluation “held-out.”** The manifest says `split=train`, and the published base artifacts' historical exposure is not established. The final sheet calls it a document-disjoint internal-development set and limits the claim to the 256 subsequent CPU updates.
- **The first sheet required a public-run restart that the CLI cannot execute.** `cpu-corpus-driver` emits and reloads a final checkpoint but has no resume argument. The final sheet relies only on the merged finite trainer-level resume evidence and records public-corpus restart as `NOT_RUN`.
- **The first sheet named source, binary, OS and hardware identity without a capture step.** The final sheet writes those identities to `<run>/identity` before either arm.
- **A fresh non-author review returned `NOT FIT` for those three points.** The revised sheet received `TEST FITNESS: FIT`; no nonzero-step run occurred before that verdict.

## What changed

- Added a pre-registered zero-update versus 256-step pilot on both public trained FBS seeds.
- Fixed the public corpus at its exact token, mask, manifest, tokenizer, response-target and horizon identities.
- Used even/odd BOS-document partitioning: 200 training documents / 7,080 targets and a deterministic 1,024-row document-disjoint internal evaluation.
- Defined per-seed three-standard-deviation same-metric ranking gains, no regression in the other ranking metric, non-decreasing target admission and exact CPU/serving export.
- Added a separate 16-step timing probe from fresh state, 64-step/final checkpoints, final artifact write/reload and explicit machine identity capture.
- Recorded complete replies, genuinely fresh phrasing and public-run restart as `NOT_RUN`; this unit does not close the full #3506 contract.

## Measurements

- No training or evaluation compute ran after the FIT.
- No parameter was updated.
- No corpus-training, complete-reply, fresh-phrasing or language result is claimed.

## Checks run at the head

- `uor-harness prep-check`: this sheet is complete and `FIT`; the repository-wide report separately identifies the pre-existing malformed Kimi sheet.
- `sha256sum` over both public FBS artifacts and the public token/mask files matches the identities in the sheet.
- `git diff --check`.

## Skills used

No repository skill was invoked. The work used the owner-authorized D84 CPU program and the merged CPU corpus-driver/REPORT evidence.

## Still open

- The pre-registered nonzero-step pilot itself.
- Public-corpus checkpoint restart.
- A trained FBS float shadow.
- Complete-reply and genuinely fresh-phrasing evaluation with credentialed sealed inputs.

## Plain summary

The public CPU rank-margin pilot now has a FIT pre-registration with a narrow, falsifiable development bar. No training has started, and the sheet deliberately does not turn this public internal-development test into a complete-reply or fresh-phrasing claim.
