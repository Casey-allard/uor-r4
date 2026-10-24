# REPORT: m1-address-as-read, kimi-34 (pin the fresh-V2 BPB stream)

Lab: kimi. Item: [m1-address-as-read](../../../../roadmap/items/m1-address-as-read.md) (#2833).

References #2833

## What did not work, first

- The address-oracle sheet named the sealed fresh-V2 draw but omitted its full token/mask/seal digests, leaving prerequisite 5 incomplete.
- The raw sealed artifacts were not re-read or re-hashed in this unit. The pins are cross-checked against retained repository evidence; no new draw, upload, disclosure or model result is claimed.
- The checkpoint remains PENDING and the sheet remains NOT_FIT. No compute is cleared.

## Cancel probe

At `742bf59bc`, prerequisite 5 still requested the stream's full digest. Retained records supplied consistent values, so a narrow documentation pin stands.

## What changed

- The data split names `tokens.u16` and `response_mask.u8`, quotes their full SHA-256 values and the seal-report digest, and links to the retained A8 report, dispatch record and packet table.
- Prerequisite 5 is now PINNED. The already-registered `0.01` tolerance is unchanged.
- The current status note records this pin while preserving the checkpoint, panel, bar, FIT verdict and holds.

## Measurements

Retained metadata cross-check:

- tokens: `0b7e754cfd242bf53145fe9cbca563be8503e6cd75df5449abb325b5c51ee1b8`;
- response mask: `a1cda5295547bb641b1ea71ba4ebabb58d58c40e6c048cda91f35a878fecbf5e`;
- seal report: `df5d074ae09c743aa586e3463ffe87faca5703f44f7802bff8ac49ee2990395f`.

The A8 report binds the draw to 44,678 payload tokens and 14,479 eligible response targets. Dispatch and packet records independently quote all three hashes. No GPU work ran.

## Checks run at the head

`loop report --check`, `loop pr-check --local`, `loop gates --full`, and `git diff --check`.

## Skills used

`uor-research-architect` for artifact lineage and exact claim scope; `superpowers:verification-before-completion` for checking the final evidence and delivery gates.

## Still open

The held #2844 checkpoint SHA, fresh-context FIT, PM dispatch and Casey's approval remain required before any run. The PM's panel-v2 Arm B scoring and bar questions are separate and unchanged.

## Plain summary

The sheet now identifies the exact sealed text stream and its three retained hashes. This completes the stream pin only; it does not release the experiment or change its rules.
