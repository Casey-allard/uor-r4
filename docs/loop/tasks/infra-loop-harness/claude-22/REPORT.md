# REPORT: infra-loop-harness, claude-22 (repair ORDERs in one PR; shared registration files)

Lab: claude. Base: `9f5894d85`. Order: [ORDER.md](ORDER.md), committed first (`43339b915`). The cancel probe found that every tool refused a PR adding two ORDERs, and that no file was exempt from `scope`.

Three fresh-context builders wrote one commit each: the repair-ORDER rule, its tool tests, and the shared-files exemption. The main session folded them in, re-ran every builder gate (the negative gates included), and wrote the docs and this REPORT.

## What did not work, first

- **My own review records led the labs into the gap.** The loop page's step 11 gives a repair a new `<lab>-<n+1>` ORDER, so my CHANGES records on #2929, #2930 and #2932 named repair orders. But `pr-check`, `loop review`, `loop next`, `loop report` and `loop close` were all built for exactly one ORDER per PR. Once Kimi's #2929 carried `kimi-1` to `kimi-3`, `loop review` refused to post a record. Kimi and DeepSeek worked around it by folding their chains into one ORDER. This PR makes the documented shape legal.
- **Kimi's narrowed scopes did not fit real diffs.** DeepSeek showed on #2028 that loop PR 11 legitimately changes `looparrive.rs` and `loopprcheck.rs`, which pr11's scope did not list, and that PR 12 needs hook scripts and its own test file. Casey made `loopprcheck.rs`, `gates.json` and the roadmap page shared (the second decision below). Kimi's #2933 adds the remaining item-scope lines.
- **Two builders stopped at their context mark.** The repair-ORDER builder committed the rule and the `pr-check` tests but left `report`, `next`, `review` and `close` untested. The test builder then covered `report` and `next`, but not `review` and `close`. Those two are covered through shared code: `loop review` resolves its ORDER with `loopnext::added_order` (`loopreview.rs:775`), and `loop close` with `loopreport::find_order` (`loopclose.rs:1016`). Both functions now have chain tests.
- **Review round 1 found that a repair ORDER could re-point the PR.** Two fresh lenses at `33f3631f7` each returned APPROVE WITH LOWS. One medium: each chain ORDER was compared only with its own version at the head, so a later ORDER could name a different `Item:` (a wider scope), keep a history ORDER's wrong `Lab:`, or drop the `Setup row:`, which changes who may review. Since `115cfeaf8`, every chain ORDER must match the active ORDER's `Lab:`, `Item:` and `Setup row:`.
- **My chain tests first landed outside the item scope.** Once `main` brought in the real scopes, `pr-check` flagged `tests/loop_prcheck.rs`. That is the test file of a shared module, and it is not shared itself. `c906a9913` moved the four new tests into `tests/loop_repair_orders.rs`, following the rule I gave the labs. The one existing case that encoded the old rule had to stay; Kimi's #2936 added that file to the umbrella scope.
- **One existing test case changed meaning.** `order_fails_for_each_broken_condition` expected `claude-1` + `claude-2` to be refused. That is now a legal chain, so the case uses `claude-1` + `codex-2` to show that two labs are refused.

## What changed

| Commit | What |
|---|---|
| `cc49e031d` | `active_order` and `order_chain` in `loopprcheck.rs`: the highest numeric `n` of one lab in one folder wins, while mixed labs or folders and a repeated `n` are refused. `the_order` uses it, so `order`, `report`, `owner-paths`, `read-path`, `review-record` and `loop queue` all judge the active ORDER. `commit-order`: the lowest ORDER is the first commit, alone, and each later ORDER is added in a commit touching only its own folder. `scope` allows every chain folder. `loop next`, `loop review`, `loop report` and `loop close` resolve through the same helper. |
| `aa6a3bd7a` | Chain tests for `loop report` (the highest REPORT is checked, and an invalid lower one is ignored) and `loop next` (a record for the superseded ORDER leaves the review owed; two labs are refused). |
| `cfe10dc59` | `loopclaim::SHARED_FILES`, the nine paths, with Casey's two decisions in its doc comment. `pr-check`'s `scope` always allows them, `loop claim` refuses an item scope that covers one (judged with the same `glob_match` that `scope` uses), and the overlap check ignores them. Tests are in a new `tests/loop_shared_files.rs`. |
| `9058ecb73` | Docs. Step 11 on the loop page states the chain rule, a new "Shared registration files" section lists the nine with Casey's words, and the harness README's `order`, `scope`, `commit-order`, `report` and `loop claim` text matches the code. |
| `47f1b1427` | The ORDER gains Casey's second decision. |
| `115cfeaf8` | The review fixes:<br>- `chain_identity_problems`: every chain ORDER keeps one `Lab:` (matching its folder), one `Item:` and one `Setup row:`;<br>- a test for a repeated `n` (`kimi-01` with `kimi-1`);<br>- `loopnext` and `loopreport` use `order_parts`'s lab pattern;<br>- a test that an open order's exact shared-file glob never counts for overlap;<br>- the claim module's doc names that exemption. |
| `61862c595` | The harness README's review gate, `loop review` and `loop report` paragraphs now describe the active ORDER of a chain. |
| `c906a9913` | The four repair-chain tests move from `tests/loop_prcheck.rs` to `tests/loop_repair_orders.rs`, which keeps them in scope. |
| `cd8b2d1e9`, `fad00cfab` | `main` merged in, bringing #2929, #2930, #2933 and #2936. No conflicts. |

**Casey's decisions, verbatim, in Claude's chat:**
- 2026-10-21T23:43Z, "Exempt them (Recommended)": the six registration files.
- 2026-10-22T00:05Z, "Add all three (Recommended)": `loopprcheck.rs`, `docs/harness/gates.json` and `docs/roadmap/README.md`.

## Measurements

None. This is tooling, with no model run.

## Checks run at the head

- fmt is clean, and whole-crate clippy with `-D warnings` is clean.
- `loop_prcheck` 28, `loop_repair_orders` 8, `loop_report` 8, `loop_next` 24, `loop_shared_files` 7, `loop_claim` 33, `loop_review` 27, `loop_close` 28 and `loop_queue` 20 passed, as did the lib test `active_order_is_the_highest_of_one_lab_in_one_folder`.
- `loop pr-check --local` at the head is a full PASS (`order`, `scope`, `commit-order`, `report`, `owner-paths`, `rule-sources`, `read-path`), the first on a loop setup PR. `scope` passes because Kimi's scope lines (#2929, #2933, #2936) and this PR's shared-file exemption are both on the branch.
- Negative gates, each re-run by the main session:
  - with the helper refusing more than one ORDER, the chain tests fail;
  - with `review-record` accepting any ORDER of the chain, `a_review_record_for_a_superseded_order_does_not_count` fails;
  - with `find_order` picking the lowest `n`, the `loop report` chain test fails;
  - with the shared-file allowance removed from `scope`, `shared_files_pass_scope_beside_the_items_own_files` fails;
  - with the claim refusal removed, both refusal tests fail;
  - with the cross-chain identity check removed, the three re-point tests (item, lab, setup row) fail;
  - with the shared-file overlap exemption removed, `an_open_orders_exact_shared_file_glob_never_counts_for_overlap` fails;
  - all pass again once restored.
- Full harness suite at `9058ecb73`: 950 passed, 0 failed. After the review fixes and the merge of `main`, at `c906a9913`: 973 passed, 0 failed.
- Claim-wording passes. `xtask links` finds no broken link in the changed files. The copyright check finds 0 matching 8-word runs.

## Skills used

solution-integrity: one resolver instead of five copies. The tools disagreed only because each had its own "exactly one" check.

## Still open

- **No direct `--pr` chain tests:** `loop review` and `loop close` have none; they are covered through `loopnext::added_order` and `loopreport::find_order`.
- **Not refused, outside this task:** `loop suborder` does not refuse a scope covering a shared file, and `pr-check` does not fail an item whose scope at the base lists one.
- **pr12's scope** still needs `looparrive.rs`, its tests and the corrected hook-script path; that has been asked of Kimi, for DeepSeek's #2935.
- **The umbrella and pr08 overlap** on `docs/roadmap/items/*.md`, which is pre-existing.

## Plain summary

When a reviewer asks for changes, the lab writing the fix can now record it as a new numbered order in the same pull request, and the tools follow the newest one. Nine shared files that every change has to touch no longer block labs from working at the same time, as Casey decided.
