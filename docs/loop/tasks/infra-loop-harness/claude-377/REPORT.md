# REPORT: infra-loop-harness, claude-377 (register docs-site-freshness)

Lab: claude. Order: [ORDER.md](ORDER.md). PR #3971. References #2904.

## What did not work, first

- **One unit was not enough.** The brief asked for the item and the work in one unit, but `loop pr-check` refuses an order whose item does not exist at the merge base (`crates/uor-r4-harness/src/loopprcheck.rs`, "item {id} does not exist at the merge base"). So the item is registered here, as claude-280 registered `float-kernel-fusion`, and the work is claude-378 under the new item.
- **The first title was too long.** `roadmap check` refused 98 characters against its 80 limit; the title is now "Book page freshness records, a staged banner, and mdBook 0.5".
- **No bench inbox.** `local/bench/INBOX.md` does not exist in the main checkout, and its README's new home, `.worktrees/bench-main/local/bench`, is absent too. No waiting message was read or marked.
- **The sync did more than this item.** Run from fresh `main` plus this item, `roadmap sync --apply` also created the issues of two merged items that had none: `native-space-state-contract` (#3973) and `real-text-corpus` (#3974, closed at once as `done`). It re-rendered 14 generated bodies from their merged item files. Each carries the "Edit that file in a PR, not this issue" footer, so no hand-written text was replaced. Two warnings are left as they were: `m6-code-task-rustc-verifier` (#3432) and `m7-lexlean-studio-tools` (#3704) are closed issues whose items say `ready`, and sync does not reopen them.

## What changed

- **New: `docs/roadmap/items/docs-site-freshness.md`** (#3972), `status: ready`, `needs_gpu: false`, `done_class: host`. It asks for one HTML-comment record line per book page (sources and the commit it was checked at), `cargo xtask book-fresh` (advisory, D76), a banner in the staged copy only, and mdBook 0.5.4 with mdbook-mermaid 0.17.1. Its scope leaves out `CHARTER.md` and `docs/rules/changes.md` (owner paths) and names #3969's five pages as left unrecorded until that PR merges.
- **`docs/roadmap/issues.json`** maps the three new ids, and **`docs/roadmap/README.md`** is re-rendered.
- Size: one item file (57 lines), this folder, 3 lines of `issues.json` and the generated README.

## Measurements

None. This unit makes no run.

## Checks run at the head

- `cargo xtask roadmap check`: 146 items OK.
- `cargo xtask roadmap render --check`: current.
- `uor-harness loop report --check`: PASS. `loop pr-check --local`: verdict PASS (two D76 advisories, `read-path` and `filemap`, both about files outside this diff).
- `gh issue view 3972`: the body is the item's text, the owner's "All of it (Recommended)" and the m7 note included.
- **A fresh exact-head review** at `ef801e7a5`: APPROVE WITH CHANGES. It confirmed the 104 SUMMARY links, the workflow's pins (lines 72 and 77–78), the merge-base refusal at `loopprcheck.rs:356`, the three owner paths, the verbatim quotes and that `issues.json` and the README hold only what sync wrote. Its findings, and what was done:
  - SHOULD: add `docs/harness/README.md` to the scope. Not done: it is one of the nine shared files every PR may edit (`loopclaim::SHARED_FILES`), and `loop claim` refuses an item whose scope covers one.
  - NIT: `xtask/Cargo.toml` is not in scope. Not needed: `book-fresh` uses std and the `serde_json` xtask already has.
  - NIT: `docs/mechanisms/*.md` and `docs/model/*.md` take in #3969's five pages. Accepted knowingly: the item names them and its unit leaves them unedited; `loop claim`, which compares scopes, is not used.
  - NIT: this section pointed at the review record instead of the results. Fixed here.

## Skills used

None applied.

## Still open

1. **The work unit, claude-378** under #3972, once this merges.
2. The two closed-but-ready issues the sync warned of are another unit's to settle.

## Plain summary

The project's documentation website is built from the same pages people read on GitHub. Nothing says when a page was last compared with the code it describes, so pages quietly go out of date. Casey asked for that to be tracked and for the site's build tools to be updated. The project's rules say every piece of work needs a roadmap item first, and the checker reads that item from `main`, so this small change only adds the item and its GitHub issue (#3972). The tool, the page records and the upgrade come in the next change.
