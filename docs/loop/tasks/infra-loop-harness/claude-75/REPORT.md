# REPORT: infra-loop-harness, claude-75

Lab: claude. Order: [ORDER.md](ORDER.md).

## What did not work, first

- The ORDER named only `loop_review.rs` and `loop_prcheck.rs` as tests that encode D46. The narrow run then found a third, in `loop_next.rs`: `an_own_doc_only_pr_owes_a_fresh_self_review_and_an_own_code_pr_does_not` asserted that an own code PR is never offered for review. The file is in the item's scope, so the ORDER was re-scoped to name it (the stop-and-re-scope rule of D65), and the test now asserts the D66 behaviour.
- Seven `loop_review` tests, not the three first found, encoded the old rule. Each now asserts the D66 behaviour; none was deleted or weakened. The four found late:
  - `the_switch_ends_d59_only_when_listed_and_ended`;
  - `a_claude_record_on_a_setup_pr_under_d59_needs_fresh_context` (its D59-off half);
  - `cli::a_same_lab_approve_on_a_code_pr_is_refused`;
  - `cli::a_fresh_context_self_review_counts_on_a_doc_only_pr_only`.
- The first negative-gate run stopped at the first failing test binary, so only `loop_next` was shown failing. It was re-run with `--no-fail-fast` (below).
- The first full suite gave 1,134 passed and 2 failed. The two failures were `loop_review`'s CLI tests, which printed the old "(D46)" message. Cause: Claude's own `loop close --verify --pr 3160` ran `loop gates --full` on `origin/main` into the same `CARGO_TARGET_DIR` at 21:25Z, mid-suite, and rebuilt the `uor-harness` binary from the pre-change source. This was an instrument collision, not a defect: the same two tests had passed in the narrow run just before. The suite was re-run alone (below).
- The in-unit review round 1, at `9a007ef0a`, returned CHANGES.
  - Blocking: `docs/loop/README.md` still stated D46 in the review-record paragraph. Round 1's fix also updated the opening principle and the setup-era teams line. Round 2, at `e77002a10`, found steps 10 and 12 of the step table and the build-plan intro still stating the old rule as current. Those are updated too, so the step table names the in-unit review and post-merge audits. A final check at `2e4e6c1dd` found two more current-tense lines (the kit-mapping row and the policy-gate bullet), and they are updated; the build-plan PR A line is now marked as the rule of its time.
  - Not blocking: no test pinned that another lab of the ORDER lab's vendor is still refused. Added `another_lab_of_the_order_labs_vendor_is_still_refused_d66`. With the condition widened from "same lab" to "same vendor" it fails, and it passes on the fix.
  - Also noted: `loop next` offers the self-review only once a PR is non-draft, so after ready rather than before. That stays for the `loop next` unit.

## What changed

- `loopreview::refusal`:
  - The ORDER lab's own record now counts on every PR when it carries `context: fresh`. Before, that held only on a doc-only PR.
  - Without `context: fresh` it is refused with "a same-lab record needs `context: fresh` (D66)".
  - Another vendor's record counts as before. A record from another lab of the ORDER lab's vendor is still refused ("same vendor … (D66)").
  - D59's path and the CHANGES-blocks rule are unchanged.
- The same gate feeds the rest, so all of these follow:
  - `loop review --verdict` now posts the author's fresh record on a code PR;
  - `loop queue` and pr-check's `review-record` rule count it;
  - `loop next`'s `reviews` source offers an own code PR to its author until it holds a counted record;
  - the bench lane mapping counts it.
- Tests:
  - `loop_review.rs`: the row-6 planted fault is now "a same-lab record without `context: fresh` is refused". The doc-only test now covers any PR. A fresh same-lab CHANGES on a code PR now blocks. Four more expectations were updated.
  - `loop_prcheck.rs`: the code-PR case asserts that the author's fresh record counts and the same record without `context: fresh` fails "(D66)".
  - `loop_next.rs`: as above.
- Docs: the gate rule, the refusal list and `loop next`'s `reviews` source in `docs/harness/README.md`, and row 6's planted fault on the loop page.

Size: code 1 file, tests 3 files, docs 4 files (line counts in pr-check's size line at the top of the PR body). Against the ORDER's estimate (two test files): 1.5, because of the third test file.

## Measurements

Nothing was measured; this is a rule change in the harness.

## Checks run at the head

- Narrow: `cargo test -q --locked -p uor-r4-harness --test loop_review --test loop_prcheck --test loop_next` gives 36, 47 and 33 passed, 0 failed.
- Negative gate: the `loopreview.rs` change was set aside as a patch against `6ecb40c45` (`git diff 6ecb40c45 -- crates/uor-r4-harness/src/loopreview.rs`, 31 lines, then `git apply -R`). Run with `--no-fail-fast`, `loop_review` gives 7 failed, `loop_prcheck` 1 failed (`review_record_counts_exact_head_records_from_allowed_labs`) and `loop_next` 1 failed (`an_own_pr_owes_a_fresh_self_review_doc_only_or_code_d66`). After restoring, all pass again and the working tree is clean.
- `cargo fmt -p uor-r4-harness -- --check`: clean. `cargo clippy -q --locked -p uor-r4-harness --all-targets --all-features -- -D warnings`: exit 0.
- The full harness suite, `cargo test -q --locked --no-fail-fast -p uor-r4-harness`, re-run alone after the collision above, on the branch merged forward onto `8a646977c`, gave 1,136 passed and 1 failed.
  - The failure was `loop_prcheck`'s `an_inner_gate_is_respected`, an existing test this unit does not touch. git reported "unable to create temporary file: No such file or directory" mid-commit: its temp directory vanished, most likely an environment flake (inferred, not proven; the clean reruns below support it).
  - Run alone twice, `cargo test -q --locked -p uor-r4-harness --test loop_prcheck` gives 47 passed, 0 failed both times.
- `loop pr-check --local`, `loop report --check` and the quick gates: results are in the PR's review record at the head.

## Skills used

None.

## Still open

- `loop next` still emits WAIT and offers premise probes that have already run, and does not rank work by unblock value (phase 2).
- `loop close --session` still writes "Await the exact-head review" into the PICKUP, and only `References #N` into the PR body (D65, D70; phase 1).
- The owner's queue view and the supervisor are not built (D69, D71).
- The shared `~/.cargo/bin/uor-harness` must be rebuilt from `main` after this merges, or the labs' binary keeps the old gate.

## Plain summary

The harness has a gate that decides whether a pull request has been reviewed well enough to merge. It used to accept only a review from a lab of a different AI vendor, except on documentation-only changes. That left every code change waiting for another lab to notice it. The owner ruled that the review is part of the work: each lab reviews its own change with a fresh, adversarial reviewer before merging, and other labs check it afterwards.

This change makes the gate accept that fresh self-review on every pull request. A self-review without the fresh-context marker is still refused.
