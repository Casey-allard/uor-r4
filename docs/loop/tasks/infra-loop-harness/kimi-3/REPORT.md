# REPORT: infra-loop-harness, kimi-3 (repair: scope deltas after round 2)

Lab: kimi. Order: [ORDER.md](ORDER.md).

## What did not work, first

- Round 2 (counted CHANGES at `a52e7e62c`, claude, uor-review v1) found three scopes still missing files their own done-when writes: pr09's onboarding receipts (`docs/harness/onboarding/**`), pr08's moved notes and registries (`docs/integration/NOW.md`, `docs/harness/claims.json`, `docs/harness/milestones.json`), and pr11's watch tests (`crates/uor-r4-harness/tests/loop_gates_watch*.rs`). All three were my omissions, now added.
- `m2-fresh-heldout-panel` still listed `docs/harness/README.md`, one of the six shared registration files Casey exempted (21 Oct 23:43Z, recorded on #2028: "no item scope lists these six"). Dropped; verified none of the six appears in any `scope:` line.
- pr03 and pr08 still collided with `infra-loop-harness` on `docs/roadmap/README.md`. Dropped from both; the umbrella keeps it as the PM's render target, and it stays out of every scope until Claude's exemption PR asks Casey to add it (and `docs/harness/gates.json`) to the shared list.
- The kimi-2 REPORT understated the remaining overlaps: `crates/uor-r4-integer/**` appears in about 8 items. Corrected there, and its "Still open" now records Casey's decision instead of the open question.
- With the round 2 CHANGES at the head, `loop review --check` refused to run at all: the PR added three ORDERs, and the gates resolve the PR's order only when exactly one is added (`loopnext.rs` and `loopprcheck.rs` agree). The repair chain is therefore folded, as the loop page's "the first ORDER is history" directs: kimi-1's and kimi-2's ORDER.md are removed from the branch in a forward commit, their steps already listed in kimi-3's ORDER with their commits, and both REPORTs stay in place with a pointer. Flagged on #2028 for the reviewer.

## What changed

- Five item files: pr09, pr08, pr03, pr11 (scope deltas above) and m2-fresh-heldout-panel (shared-file drop).
- `docs/loop/tasks/infra-loop-harness/kimi-2/REPORT.md`: the two record corrections the reviewer asked for.
- `docs/loop/tasks/infra-loop-harness/kimi-3/`: this ORDER and REPORT.

Size by class: docs 7 files, +6/-4 lines approximately; no code, no tests, no generated files. The ORDER set no estimate (D48).

## Measurements

- `grep '^scope:' docs/roadmap/items/*.md` for the six shared registration files (`src/lib.rs`, `src/main.rs`, `src/loopgates.rs`, `src/doccheck.rs`, `docs/harness/README.md`, `docs/loop/README.md`) and `docs/harness/gates.json`: NONE.
- `docs/roadmap/README.md` appears in exactly one scope: `infra-loop-harness` (the PM umbrella), as the round 2 record directed.
- `cargo run -q --locked -p xtask -- roadmap check` → 87 items OK (ready 51). `render --check` → current.

## Checks run at the head

- `xtask roadmap check` and `xtask roadmap render --check`: pass.
- `uor-harness loop gates --quick`: run before commit; result in the commit message's checks line.
- `uor-harness loop report --check`: PASS for this REPORT.
- `uor-harness loop pr-check --local`: the known failures stand — `scope` (no `scope:` at the merge base until a scoped base lands) and the structural `order`/`commit-order` for a same-branch repair (three ORDERs now); flagged for the architect in round 1, and CI records advisory `policy: fail` while the required checks stay green (P-4).

## Skills used

- scope-guard: the written edge, `git diff --stat` before content. The ORDER and the round 2 record were the opener; each point was applied as directed (this round needed no judgement calls beyond the record).

## Still open

- Claude's exemption build (`loop claim`/`pr-check` honoring the six shared files) and his PR asking Casey to add `docs/roadmap/README.md` and `docs/harness/gates.json` to the shared list. Until then both stay out of every scope, and `pr-check --local` may flag small additive edits to the six — expected, per Casey's recorded decision.
- The one-ORDER/`commit-order` repair gap from round 1 is still the architect's to answer.
- DeepSeek and Antigravity merge these item edits forward.

## Plain summary

The second review found the big problem fixed — the loop setup items no longer block each other — and listed six small leftovers: three file lists that missed files their own tasks must write, one file that belongs to the owner's new shared list, one rendered page that only the umbrella item should touch, and two lines in the earlier report to correct. This repair makes exactly those six changes.

References #2904.
