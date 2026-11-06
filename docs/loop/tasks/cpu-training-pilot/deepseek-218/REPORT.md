# REPORT: cpu-training-pilot, deepseek-218

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `76fcbae7c`.

## Status

Complete as release bookkeeping.

## What did not work

No further source work was needed. Deepseek-217 remains a
**REJECT_TERMINATION_LIMITED**: retrieved prefixes improve body alignment but
do not produce BOS completion.

## What changed

Recorded protected delivery and removed `deepseek/prompt-prefix`.

## Measurements

- reviewed head:
  `a1e45f9574f2bbebad111cb113d0a0326c4d2067`;
- protected merge:
  `76fcbae7cb0bed3ad5479968074436f09cbe3e23`;
- reply-eval source:
  `ce2d7fb0f13d936a9167e9ccb0a91bf46456d22c4f871d133225f71e2a32c04f`;
- compact evidence:
  `5ed98fb14329e0facb77659f546485397f5c9d6177690f2ff57ca6e6501e032b`.

Fresh-main source and published checksum verification passed.

## Checks run at the head

- fresh-main source and evidence hash checks;
- release policy and claim checks.

## Skills used

No system-provided Codex skill applied.

## Still open

- termination from forced-prefix continuation state;
- fresh generation beyond stored replies;
- Claude's independent human-text track.

## Plain summary

The retrieval-prefix rejection is on `main` and its claim is released.
