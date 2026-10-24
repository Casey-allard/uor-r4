# REPORT: m1-address-as-read, kimi-35 (repair BPB pin status consistency)

Lab: kimi. Item: [m1-address-as-read](../../../../roadmap/items/m1-address-as-read.md) (#2833). Repair for PR #3139.

References #2833

## What did not work, first

- Independent DeepSeek review at `40f80f8d` accepted every new digest and metadata claim but returned CHANGES: §3 still called the registered bar PROPOSED, §6 named the wrong panel and redundant bar confirmation, the 2026-10-22 NOT_FIT reasons read as current, and the README had no current stream-pin record.
- The original ORDER omitted the README from its narrowed scope. The repair ORDER adds it before editing; no out-of-scope file changed.
- Raw sealed artifacts were not re-read or re-hashed. No new draw, upload, model result or compute release is claimed.

## What changed

- §3 now quotes the existing registered bar, single 19/40 baseline and 27+7+20=54/80 derivation without changing any threshold.
- §6 verifies frozen panel-v2 and leaves only checkpoint pinning, fresh-context FIT, PM dispatch and owner approval before a run.
- The old NOT_FIT reasons are explicitly a dated snapshot; the current instruction names the remaining checkpoint and approval path.
- The kimi-34/35 status update is a separate top-level bullet.
- README has a current BPB pin section with all three hashes and the unchanged registered tolerance; its dated PENDING sections remain as history.

## Measurements

No new measurements. The previously verified retained metadata pins are unchanged:

- tokens `0b7e754cfd242bf53145fe9cbca563be8503e6cd75df5449abb325b5c51ee1b8`;
- response mask `a1cda5295547bb641b1ea71ba4ebabb58d58c40e6c048cda91f35a878fecbf5e`;
- seal report `df5d074ae09c743aa586e3463ffe87faca5703f44f7802bff8ac49ee2990395f`.

No GPU work ran. The held checkpoint, NOT_FIT, panel, bar, tolerance and holds are unchanged.

## Checks run at the head

- Size: tests 0 files +0/−0; code 0 files +0/−0; docs 6 files +173/−9; generated/other 0 files. The ORDER gave no numeric estimate, so no ratio applies.
- `loop report --check`: PASS.
- `loop pr-check --local`: PASS (scope, ORDER-first and repair-ORDER checks included).
- `loop gates --full`: 11/11 GREEN.
- `git diff --check`: PASS.

## Skills used

`uor-research-architect`, `superpowers:requesting-code-review`, and `superpowers:verification-before-completion` for independent review, exact claim scope and fresh final evidence.

## Still open

The held #2844 checkpoint SHA, fresh-context FIT, PM dispatch and Casey's approval remain required before any run. The absolute-versus-measured-gain bar card is separate; no registered value is changed here.

## Plain summary

The hashes were right, but nearby instructions still described old prerequisites. This repair makes the sheet and README consistent while leaving every experiment rule and hold untouched.
