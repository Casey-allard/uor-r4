> **Outside contributor?** Use this shorter checklist and ignore the lab items below:
> - [ ] I ran `cargo check`, `cargo test` and `uor-r4-claim-wording` locally and pasted the output
> - [ ] No binaries, corpora, generated artifacts or credentials in the diff
> - [ ] `git grep -n -i '<what changed>' -- '*.md'` and stale docs updated
> - [ ] I understand model work may be gated; a maintainer will confirm or redirect
>
> Read [docs/contributing/HARD-RULES.md](docs/contributing/HARD-RULES.md) and
> [docs/contributing/OUTSIDE.md](docs/contributing/OUTSIDE.md) first.

<!-- Delivery cadence: CONTRIBUTING.md § Delivery cadence. Delete lines that do not apply, and say why. -->

## Line and headline (D21, docs/labs/session-goal.md)

<!-- Line: <name> · count n/3 · headline before → after (for example `M2 8/512 → 8/512`). At 3/3 the next action on this line is a pivot card. -->

Line:  · count /3 · headline  → 

## What and why

<!-- One paragraph: the change, the question it answers, the milestone (#2029–#2036). -->

## Result

<!-- Numbers with their artifact, data split and comparison. Negative results count. Decision: KEEP / REJECT / NOT YET PROMOTED. -->

## Checklist

- [ ] `uor-harness check` passes, and `uor-harness lint check --body <this body>` reports no unresolved finding
- [ ] Any model compute here is covered by a linked pre-registration carrying `TEST FITNESS: FIT` (D22)
- [ ] Checks run at this exact head (commands and outcomes posted on this PR)
- [ ] Research record: `docs/labs/<topic>-<YYYY-MM-DD>/README.md` (or `archive/evidence/…`)
- [ ] New entry at the top of `docs/integration/current-state.md`, with **Next:**
- [ ] Every document this change makes stale is updated (`git grep -n -i '<what changed>' -- '*.md'`)
- [ ] STATUS.md / ROADMAP.md / #2028 table updated, or state unchanged
- [ ] README updated in its existing sections, or nothing a reader relies on changed
- [ ] `cargo run -q --locked -p uor-r4-claim-wording -- .` passes
- [ ] After merge: verify on fresh `origin/main`, delete the branch and worktree, `scripts/storage/uor-hygiene --apply`, then comment the result on the milestone issue

References #
