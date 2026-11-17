---
id: docs-site-freshness
title: Book page freshness records, a staged banner, and mdBook 0.5
milestone: research
area: tooling
status: ready
open_to: lab
needs_gpu: false
depends_on: []
done_class: host
scope: [xtask/src/book_stage.rs, xtask/src/book_fresh.rs, xtask/src/main.rs, book.toml, .github/workflows/docs-site.yml, docs/README.md, docs/start/*.md, docs/products/*.md, docs/model/*.md, docs/geometry/*.md, docs/mechanisms/*.md, docs/research/*.md, docs/research/lines/*.md, docs/research/notebook/*.md, docs/rules/README.md, docs/rules/decision-map.md, docs/reference/*.md, docs/formal_vocabulary.md, docs/CONFIGURATION.md, docs/reproducibility.md, docs/minimal_client.md, docs/RELEASE_PIPELINE.md, projects/README.md, projects/*/README.md, docs/roadmap/items/docs-site-freshness.md, docs/loop/tasks/docs-site-freshness/**]
---

## Why it matters

The owner, Casey, 14 Nov 2026, in claude's chat: "yes extend the current mechanism and modernize it". That answered claude's finding that a separate LLM-written docs wiki cannot live in `docs/`: `docs/` is the source of the mdBook site, and CHARTER.md is the only rule book. Asked how much to take on, Casey chose "All of it (Recommended)": freshness tracking, a per-page banner and the toolchain upgrade. That answer is the owner approval for the `.github/workflows/docs-site.yml` edit, since `.github/**` is an owner path (`docs/harness/owner-paths.json`).

The book has 104 pages (`docs/SUMMARY.md`). Nothing records when a page was last read against the code it describes, so a page goes stale and nobody sees it. `docs-status-refresh` (#3380) refreshes the pages' content by hand; it does not cover the staging tool, `book.toml` or the workflow, and it leaves no record a later reader or tool can check.

## What is missing

**November 17, 2026.** The records, book-fresh command, staged banner and mdBook upgrade landed. Claude-379 corrected and checked 23 more pages. Its report leaves the mermaid version warning open under criterion 5. Pages with a record can still be unchecked. [PR #3975](https://github.com/UOR-Foundation/uor-r4/pull/3975) · [PR #3976](https://github.com/UOR-Foundation/uor-r4/pull/3976)

- **No freshness record.** A page names no source paths and no commit it was checked against. `xtask/src/book_stage.rs` copies each page into `target/book-src` unchanged apart from links and maths.
- **No report.** Nothing counts the commits that touched a page's sources since it was last checked, or lists the pages never checked or naming paths that no longer exist.
- **No banner.** A reader of the site cannot tell a page checked last week from one untouched since September.
- **An old toolchain.** `.github/workflows/docs-site.yml` pins mdBook 0.4.40 and mdbook-mermaid 0.13.0; the build warns that mermaid was built against mdBook 0.4.36. The current releases are mdBook 0.5.4 and mdbook-mermaid 0.17.1. mdBook 0.5 makes unknown `book.toml` fields an error and turns on smart punctuation, hashed asset names, definition lists and admonitions by default.

## Done when

1. **One record format,** documented in `docs/harness/README.md` (a description of the tool, not a new rule: CHARTER is the only rule book): one invisible HTML-comment line per page naming the repo-root paths or globs of what it describes and, when it was actually read against them, the full commit hash it was checked at.
2. **Every SUMMARY page carries it,** except pages another open order holds, which are listed. A page nobody read against its sources has no `checked` value and is reported as unchecked, never given a guessed one.
3. **`cargo xtask book-fresh [--json OUT]`** lists the stale, unchecked and broken-source pages: for each page, the commits since `checked` that touch its sources, from `git log` run with every `GIT_*` variable removed. It is advisory (D76): it reports and exits 0. Finite tests fail without the rules they pin.
4. **`book-stage` adds a one-line banner** to each staged page: the short commit and its date and the number of later changes to its sources, or "not yet checked". Only the staged copy changes: no rendered file is committed, and GitHub's view of `docs/` is the source unchanged.
5. **The site builds on mdBook 0.5.4 and mdbook-mermaid 0.17.1** in the workflow and its cache key, with `book.toml` adjusted for 0.5: `book-stage`, `mdbook-mermaid install .`, `mdbook build` (no ERROR and no version warning) and `book-check` (broken links no more than on `main`, each named) pass.
6. **One REPORT** gives `book-fresh`'s summary counts on the tree, and one fresh exact-head review covers the whole diff.

## How to start

- Read the mdBook 0.5 migration guide (the 0.5.0 entry of mdBook's `CHANGELOG.md`) before touching `book.toml`.
- Install the new tools into a build folder inside the worktree (`cargo install --root <worktree>/local/mdbook-0.5`), never over the shared binaries in `~/.cargo/bin`.
- Write the record format and `book-fresh` first, then read the pages against their sources in batches, then the banner, then the toolchain.

## Rules that apply

- **Owner paths.** The workflow edit is approved by Casey's answer quoted above, and only for the toolchain versions and cache key. `CHARTER.md` and `docs/rules/changes.md` are owner paths: they get no record line, and `book-fresh` lists them as unrecorded.
- **Other open orders.** PR #3969 (claude-375) edits `docs/mechanisms/README.md`, `exact-memory.md`, `flock-read.md`, `route-read.md` and `docs/model/pipeline-map.md`; this item leaves those five pages unrecorded until it merges.
- **Advisory, never a gate (D76).** `book-fresh` exits 0 whatever it finds; the banner reports, it does not block a build.
- **Honest records.** `checked` names a commit the page was read against; a page with a claim its sources contradict is left unchecked and the claim listed.
- **Rust only** (§3.11): the tool is in `xtask`; no scripts.

## Sources

- Casey's words of 14 Nov 2026, quoted above.
- `docs/SUMMARY.md`, `xtask/src/book_stage.rs`, `xtask/src/book_check.rs`, `book.toml`, `.github/workflows/docs-site.yml`.
- `docs/roadmap/items/docs-status-refresh.md` (#3380). Its scope shares most of the book pages, for their content; this item adds the one record line at the top of each page, and edits a page's text only to correct a claim its sources contradict before the page is checked (claude-379).
- `docs/roadmap/items/m7-studio-live-model-sources.md` names a separate, still unapplied proposal to automate the Studio publish in `docs-site.yml`. This item does not take it on: its workflow edit is the toolchain versions and cache key only.
- mdBook's `CHANGELOG.md`, the 0.5 migration guide.
