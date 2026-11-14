# ORDER: docs-site-freshness, claude-379 (read book pages against their sources; fix the six wrong ones)

Lab: claude. Item: [docs-site-freshness](../../../../roadmap/items/docs-site-freshness.md) (#3972). Base: `a70d6189d` (`a70d6189df3368575e745d351ed692cb000e9388`, main after claude-378's #3975 merged).

## Goal

Do the item's open page work that claude-378's REPORT names as the next unit ("Still open" 1): fix the six pages claude-378 found wrong against their current sources, then read as many of the 93 unchecked pages against their sources as fit in this unit, correct what is wrong, and set each page's `checked=` to the commit it was actually read against.

## Why under this item

claude-378's REPORT, "Still open" 1: "Each needs a whole-page read against its sources. Those that pass get `checked=`, and the six above need fixes first. This is the next unit, page by page." The item's own Sources line says it "adds only the one record line at the top of each page and never edits a page's text"; that sentence described claude-378's tooling unit. This unit amends it in the item file so the item covers correcting a claim its sources contradict, which its "Honest records" rule already requires before a page can be checked. No bar, status or other scope changes, and the item stays `ready`: its done-when 5 ("no version warning") is still open.

## How this unit was claimed

By hand, as claude-377 and claude-378 were: the app's edit guard lets this session write only in its own worktree, `.claude/worktrees/docs-site-freshness-1d411d`, so `loop claim` is not used. The branch `claude/docs-site-freshness-pages` starts at the base. Checked by hand:

- the item is on `origin/main`, and its `scope:` covers every page below;
- the open PRs are #3941 (kimi), #3937 and #3889 (deepseek). Only #3889 edits a book page, `docs/research/notebook/cpu-training.md`, from its line 10; this unit edits no notebook text;
- the bench inbox (`local/bench/INBOX.md` in the main checkout, and the `.worktrees/bench-main` copy it links to) does not exist on this machine, so there are no messages to read or mark.

## Cancel probe (run first)

- `cargo run -q --locked -p xtask -- book-fresh --json <scratch>` on the base. If it does not report the 93 unchecked pages claude-378 left, re-scope from what it does report.
- **Classed:** documentation only. Nothing here touches the model, a bar or a run.

## Files in scope

- the book pages in the item's `scope:` globs: page text where a claim is wrong, and each page's first-line record (`checked=`, and `sources=` where a source is missing or wrong)
- `docs/roadmap/items/docs-site-freshness.md` (the one Sources sentence above)
- `docs/loop/tasks/docs-site-freshness/claude-379/**`

## Not in scope (stop list)

- **Dated records are history and are never rewritten:** notebook pages (`docs/research/notebook/*.md`) keep their text; REPORTs, `docs/labs/**` and `archive/` are not touched. A notebook page whose text is wrong today stays unchecked and is listed.
- The seven unrecorded pages (owner paths `CHARTER.md` and `docs/rules/changes.md`; the five #3969 pages) stay unrecorded.
- `xtask/**`, `book.toml` and `.github/**`: no tool or toolchain change.
- No `checked=` for a page I did not read whole against its sources. A subagent's reading is never a check.

## Done when

1. `book-fresh` at the head shows **0 stale** pages and **fewer unchecked** pages than the base's 93, with the before and after counts in the REPORT.
2. The six pages claude-378 named wrong are corrected against their current sources and checked, or left unchecked with the reason.
3. Every page left unchecked is listed in the REPORT.
4. `book-stage`, `mdbook build` with 0 `ERROR`, and `xtask book-check` with 0 broken links pass at the head.
5. `loop report --check` and `loop pr-check --local` pass, and a fresh exact-head review is recorded with `loop review --context fresh`.

## Stop and decide (never wait) if

- a page's fix needs a fact only a run or a record can give: leave the page unchecked and list the claim;
- a wrong claim is in a dated record: leave it, list it.
