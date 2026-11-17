# REPORT for three permanent branches

[codex] Infrastructure and documentation delivery. [ORDER](ORDER.md). References #2904 and the owner's November 17 branch decision.

## What did not work

The former publishing workflow recreated a branch the owner asked to retire. Three independent consumers also encoded the old list: cleanup, delivery diagnostics and the Studio publisher. Changing only GitHub’s branch names would have made the next documentation change recreate the preview. The local bare-remote regression also reproduced a stale book push restoring a Studio update after a deliberate rollback: Git accepted the proposed publication as a fast-forward from the older commit. A second regression showed that a Git ignore rule could omit a checked image during staging while publication reported success.

## What changed

The owner designated main, gh-pages and archive. I applied that decision to the charter clause and recorded the change in the charter log. Main carries research, gh-pages carries the public book and Studio, and archive preserves original commits. Short-lived delivery branches follow the existing pull-request lifecycle.

A push to main builds and checks the book with a read-only token. A manual dispatch from main selecting gh-pages passes the checked book to a separate publisher. The book and Studio publishers each pin the forward push to the exact gh-pages head they staged and check that the new commit has that head as its only parent. The publisher verifies that the Studio tree is intact. It transports the checked book as an archive to preserve hidden assets. Before packaging and again before publication it rejects Git control files throughout the book. Before committing it compares each checked file’s raw bytes with the Git index by blob ID, including files affected by runner-local ignore or attribute settings. After a successful Studio update, its printed rollback command pins the lease to the exact commit that the helper published. Branch cleanup protects all three permanent names, and the Studio helper accepts only the live destination.

## Measurements

All 217 files in a clean build of the rebased book matched across the source, extracted archive and published Git tree in a disposable remote seeded from Pages commit `023ab1e3d1d594040d063ee58b80267cc002fa39`. The Studio subtree retained its original Git tree ID. Ten Rust publishing regressions and eight pre-existing typed runtime checks passed. The full harness library suite passed 689 tests with one ignored. The focused harness protection group passed all seven checks after the forward-lease repair.

## Checks run at the head

The local workflow and Studio destination tests exercised a real bare Git remote, verified the live book updates, protected the Studio tree and refused conflicting book paths. The signed-off local exercises include a linked hidden file, colliding paths, the exact post-publication rollback lease, rollback between staging and pushing for both publishers, and rejection of a two-parent candidate even when its first parent matches the remote. New cases exercise nested Git control files, a linked hidden image, repository-local exclusion and line-ending conversion. Reproduce these ten cases with `cargo test --locked -p uor-r4-harness --test pages_branch_policy`. The pinned book build checked 107 pages with zero broken links. Rust formatting and shell syntax passed.

## Skills used

Git object inspection, the repository harness and independent exact-head review.

## Still open

After protected merge, dispatch the production workflow and verify the live site. Preserve the preview’s final commit in the remote archive before retiring its branch name. Check the remote branches again after the prior publisher has finished.

## Plain summary

Researchers can publish the book beside Studio without keeping a separate preview branch. Original experiment histories remain available through the archive branch.
