# ORDER: infra-loop-harness, claude-81 (trim the merge gates to what protects the project)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 5. Base: `460d1e413`.

## Goal

The owner asked for this on 24 Oct: "please dont let us get locked down in CI hell, if something doesn't matter to the project it can be removed or skipped." Since the loop started on 21 Oct, gates that guard no hard rule have repeatedly cost a unit an extra PR: scope widening, read-path trims, render refreshes and heading rewords. 25 of the last 30 CI failures were documentation and records gates.

1. **`loop pr-check` and `loop queue` demote six bookkeeping rules from FAIL to WARN:** `scope`, `commit-order`, `rule-sources`, `rendered-lock`, `read-path` and `filemap`. Their findings still print. These stay hard: `order`, `banned-presence`, `report`, `owner-paths`, `review-record` and `required-checks`, the rules guarding §3 and the review.
2. **`docs.yml`: the link, claim-wording and catalog/figure steps become advisory** (`continue-on-error`). Archive verify stays hard, because nothing is deleted.
3. **Delete `relative-query-check.yml`.** It is tied to a September branch.
4. **Append D76** with the owner's words.
5. **The ruleset change** (no up-to-date rule, queue wait 0) is a repo setting made after merge. It is recorded on #2904.

The CI policy job and the five acknowledgements stay. The acknowledgements carry the policy verdict that `loop queue` requires, so that wait is the real gate.

## Cancel probe (run first)

At `460d1e413`, are the six rules still hard? Yes. `RULES` in `loopprcheck.rs` runs all twelve, and `cli` fails on any `Verdict::Fail`. No advisory list exists, and `docs.yml` has no `continue-on-error`. The unit stands. Class: instrument (gates).

## Files in scope

- `crates/uor-r4-harness/src/loopprcheck.rs`, `crates/uor-r4-harness/tests/loop_prcheck.rs`, and any test asserting a demoted rule's FAIL
- `.github/workflows/docs.yml`, `.github/workflows/relative-query-check.yml` (deleted)
- `docs/integration/DECISIONS.md` (D76), `docs/loop/README.md` (the rule list)
- `docs/loop/tasks/infra-loop-harness/claude-81/**`

## Done when

- A planted scope violation prints `WARN scope` and the verdict is PASS. A planted owner-path or banned-operator violation still FAILs. Both are pinned by tests that fail with the demotion reverted.
- The harness tests, `loop report --check`, `loop pr-check --local` and the quick gates pass.
- A fresh-context review at the exact head counts (D66).
- The PR says `References #2904`. Because this PR changes the gate, it is posted and queued with its own head's binary.

## Stop and ask if

A demoted rule turns out to guard a §3 constraint or the review.

## Owner approval

Owner paths changed: `.github/**` and `docs/integration/DECISIONS.md`.

Casey, in Claude's chat by 2026-10-25T00:03:59Z, answered "Yes, change them (Recommended)" to: "May I change the GitHub merge settings so only gates that protect the project remain (one real fast check replacing the five placeholders, no 'branch up to date', queue wait 0, docs/link checks advisory)?" The five acknowledgements stay because they carry the policy verdict.
