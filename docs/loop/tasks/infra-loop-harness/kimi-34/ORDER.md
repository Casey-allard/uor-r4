# ORDER: infra-loop-harness, kimi-34 (record the position premise probe)

Lab: kimi. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Base: `742bf59bc`.

## Goal

Record the answered position finite-control probe in PLAN so `loop next` stops offering it again. Close only that premise row, with a decision row preserving the pending registered order test and the exact finite-control scope. This is PM recordkeeping, not a mechanism or milestone completion claim.

## Cancel probe (run first)

At `742bf59bc`, PLAN's position premise is `open`, although `cargo test -p uor-r4-discrete --lib position_mechanism` has run at `b03d08e85` and passed 10/10. The result is recorded on #2843 (`6380719988`). Repeating the same probe adds no information, so the status record still stands.

## Files in scope

- `PLAN.md`
- `docs/integration/DECISIONS.md`
- `docs/loop/tasks/infra-loop-harness/kimi-34/**`

## Done when

- The position premise row is `closed`, dated 2026-10-24, and its probe text states the finite-control answer and the still-pending registered order test.
- A new append-only decision row gives the evidence, exact non-claims and reopening cost.
- No other premise, bar, hold, milestone or roadmap-item status changes.
- `loop next` no longer offers the position probe; report/local PR checks and diff-check pass.

## Stop and ask if

The probe result would be interpreted as language/order-effect evidence, a bar or hold must change, or any file outside scope is needed.

## Owner approval

Owner: Casey Allard (project owner). This ORDER changes `docs/integration/DECISIONS.md`, an owner path, for D61 only.

**Recorded use of the owner's standing direction (deepseek, 2026-10-24T07:37Z).** The owner's own words, quoted: "yo just assume I say yes to this kind of stuff" — scoped by the same instruction to **routine bar/classification cards with a recommendation, with each use recorded**; held training, GPU, spend, charter changes and deletions still need the owner's own words.

D61 is a routine status classification with a recommendation, not a bar/hold/spend/charter change: the finite position-control probe ran and passed 10/10 at `b03d08e85` (#2843 `6380719988`), so the premise's probe has been answered per PLAN's own definition; the row records `closed` while explicitly keeping the registered paired order test PENDING and making no order-effect or language claim, and reopening is low cost. The PM (kimi) made the classification; the owner path only needs the approval to record it.

The approval was recorded at **2026-10-24T07:37Z** on PR #3138 ([comment](https://github.com/UOR-Foundation/uor-r4/pull/3138#issuecomment-6381528519)) and on #2028 ([comment](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6381528708)). If the owner prefers `held` (the D60 pattern) or objects on reading this, the recording is void and the row should be changed or the PR reverted rather than merged.

— folded into this ORDER by deepseek on 2026-10-24T07:38Z under Casey's direct instruction in the deepseek chat: "you have permission to do whatever you need so that you do not sit around doing nothing / resume the loop".
