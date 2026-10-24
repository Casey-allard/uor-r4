# ORDER: m1-address-as-read, kimi-35 (repair BPB pin status consistency)

Lab: kimi. Item: [m1-address-as-read](../../../../roadmap/items/m1-address-as-read.md) (#2833). Base: `742bf59bc`. Repair for PR #3139.

Finished: ORDER `173d73ed2`; fresh-V2 stream pin and initial REPORT `40f80f8d2`. Independent DeepSeek review at `40f80f8d` returned CHANGES: the new digests are correct, but older live sheet instructions/status lines and the lab README contradict them.

## Goal

Reconcile the address-oracle sheet and lab README with the already-registered/frozen values. Change wording and operator instructions only: do not alter any hash, threshold, tolerance, arm, checkpoint, FIT verdict or hold.

## Cancel probe (run first)

At `40f80f8d`, §3 still says PROPOSED with a two-baseline anchor, §6 points to the wrong panel and redundant bar confirmation, Why NOT_FIT presents its 2026-10-22 snapshot as current, and the README still calls the stream/tolerance pending. The review findings reproduce; repair stands.

## Files in scope

- `docs/harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md`
- `docs/labs/m1-address-as-read/README.md`
- `docs/loop/tasks/m1-address-as-read/kimi-35/**`

## Done when

- §3 records the existing registered bar and single 19/40 baseline without changing the thresholds.
- §6 uses frozen panel-v2 and asks only for the remaining FIT/dispatch/owner approvals.
- The old NOT_FIT reasons are explicitly marked as the 2026-10-22 review snapshot, and the current closing instruction names only the held checkpoint plus fresh FIT/dispatch/owner approval.
- README current state records the stream pin and unchanged registered tolerance; older pending text is clearly historical.
- The kimi-34 status update is a separate bullet, and the REPORT records exact-head check results and size.
- Report/local PR checks, full gates and diff-check pass; another vendor reviews the new exact head.

## Stop and ask if

Any registered value, bar, checkpoint, panel, tolerance, arm, hold or item status would change, or a file outside scope is needed.
