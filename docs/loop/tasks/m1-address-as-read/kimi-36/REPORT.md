# REPORT: m1-address-as-read, kimi-36 (finish BPB status consistency repair)

Lab: kimi. Item: [m1-address-as-read](../../../../roadmap/items/m1-address-as-read.md) (#2833). Second repair for successor PR #3142.

References #2833

## What did not work, first

- The second independent DeepSeek review at `d8acb05b3` returned CHANGES: sheet status/registration history still carried stale FIT/prerequisite wording, README registration/panel-freeze history was not explicitly superseded, and the then-active REPORT overclaimed consistency.
- DeepSeek #3140 landed overlapping FIT prose repairs during review. It was integrated at `f7952a350`; no duplicate or competing rule was introduced.
- `docs/roadmap/items/m1-address-as-read.md` still says the oracle bar is not registered. That file is outside this ORDER and is **not repaired here**; it needs a separate scoped PM follow-up. This PR claims consistency only for the sheet and README.
- Raw sealed artifacts were not re-read or re-hashed. No new draw, upload, model result or compute release is claimed.

## What changed

- The sheet now separates current status from dated update history. Current guidance is checkpoint pin → fresh-context non-author FIT → PM dispatch → Casey's approval in chat.
- Registration history is explicitly a snapshot; historical NOT_FIT reasons remain preserved and marked. Obsolete typed-release wording in live guidance is corrected.
- README registration and panel-v1 sections are explicitly historical snapshots; current panel-v2/BPB/approval guidance remains below them.
- All stream hashes, panel-v2, 0.01, 19/40 and other registered values are unchanged. NOT_FIT and every hold stand.

## Measurements

No new measurements or GPU work. Previously verified retained metadata pins remain:

- tokens `0b7e754cfd242bf53145fe9cbca563be8503e6cd75df5449abb325b5c51ee1b8`;
- response mask `a1cda5295547bb641b1ea71ba4ebabb58d58c40e6c048cda91f35a878fecbf5e`;
- seal report `df5d074ae09c743aa586e3463ffe87faca5703f44f7802bff8ac49ee2990395f`.

## Checks run at the head

- Size: tests 0 files +0/−0; code 0 files +0/−0; docs 6 files +191/−18; generated/other 0 files. The ORDER gave no numeric estimate, so no ratio applies.
- `loop report --check`: PASS.
- `loop pr-check --local`: PASS, including commit-order on the fresh successor branch.
- `loop gates --full`: 11/11 GREEN.
- `git diff --check`: PASS. Output is retained under `local/kimi-address-bpb-status/`.

## Skills used

`uor-research-architect`, `superpowers:systematic-debugging`, `superpowers:requesting-code-review`, and `superpowers:verification-before-completion` for conflict integration, exact claim scope, independent review and fresh final evidence.

## Still open

The held #2844 checkpoint SHA, fresh-context FIT, PM dispatch and Casey's approval remain required before any run. The roadmap item's unregistered-bar sentence requires its own scoped PM PR. The absolute-versus-measured-gain bar card is separate; no registered value changes here.

## Plain summary

This repair marks old status text as history and makes the current sheet/README instructions agree. It does not change the experiment, clear compute, or claim the separate roadmap page is fixed.
