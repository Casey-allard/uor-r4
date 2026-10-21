# ORDER: infra-loop-harness, kimi-3 (repair: scope deltas after round 2)

Lab: kimi. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2. Base: `a52e7e62c`. Repair of kimi-2 (PR #2929), after the counted CHANGES record at `a52e7e62c9a0b16c5dca39606c4eee2dd45a49e6` (claude, uor-review v1, round 2, fresh context).

## Done in kimi-1 and kimi-2 (history)

- kimi-1: ORDER `add1b4ea7`; 51 scope lines and seven `Closes` steps `952a6e6ba`; REPORT and PR #2929 `fa28f9339`.
- kimi-2: repair ORDER `92e0160ae`; un-serialized scopes, owner-path notes, real issue numbers `f1ae1f70b`; REPORT `4f8314027` and its one-ORDER gate flag `a52e7e62c`. Round 2 confirmed the loop items are now pairwise disjoint and only honest code/data contention remains.

## Goal

Answer the round 2 record, a delta only:

1. pr09: add `docs/harness/onboarding/**` (its done-when's committed receipts; #2932 commits `docs/harness/onboarding/*.json`).
2. pr08: add `docs/integration/NOW.md`, `docs/harness/claims.json`, `docs/harness/milestones.json`; drop `docs/roadmap/README.md`.
3. pr03: drop `docs/roadmap/README.md` (stops the collision with `infra-loop-harness`, which keeps it for PM renders).
4. pr11: add `crates/uor-r4-harness/tests/loop_gates_watch*.rs`.
5. m2-fresh-heldout-panel: drop `docs/harness/README.md` (one of the six shared registration files Casey exempted on 21 Oct, 23:43Z, recorded on #2028; no item scope lists them).
6. Update the kimi-2 REPORT's "Still open" (the decision, and `docs/roadmap/README.md` + `docs/harness/gates.json` staying out of every scope until Claude's exemption PR asks Casey to add them) and its "Known remaining overlaps" (`crates/uor-r4-integer/**` in about 8 items).

## Cancel probe (run first)

Is the round 2 CHANGES record at `a52e7e62c` still the last counted record, points unaddressed? Result at `a52e7e62c`: yes — `loop review --check --pr 2929` reports "the last counted record is CHANGES". The repair stands. Class: instrument.

## Files in scope

- The five item files named above.
- `docs/loop/tasks/infra-loop-harness/kimi-2/REPORT.md` — the two record corrections.
- `docs/loop/tasks/infra-loop-harness/kimi-3/**`

## Done when

- All six deltas are in the branch; `xtask roadmap check` and `render --check` pass; `loop pr-check --local` shows no new failure class beyond the known structural and scope-at-base ones; the kimi-3 REPORT is written; a new review request naming the new head is posted on #2028 (the reviewer expects only a delta check).

## Stop and ask if

- Casey's shared-file list or the exemption build changes what scopes should say mid-repair.
- Any file outside the list above needs to change.
