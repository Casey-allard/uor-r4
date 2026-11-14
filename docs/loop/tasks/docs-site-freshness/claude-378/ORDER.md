# ORDER: docs-site-freshness, claude-378 (page records, `book-fresh`, the staged banner, mdBook 0.5)

Lab: claude. Item: [docs-site-freshness](../../../../roadmap/items/docs-site-freshness.md) (#3972). Base: `c5978c5d5` (`c5978c5d5eee4c08f0ec605c833e7820af1f509a`, main after claude-377's #3971 registered the item).

## Goal

Do the item: one freshness record per book page, `cargo xtask book-fresh` to report on them, a one-line banner in the staged copy of each page, and the mdBook toolchain moved from 0.4.40 / mdbook-mermaid 0.13.0 to 0.5.4 / 0.17.1.

## Owner approval

Casey, 14 Nov 2026, in claude's chat: "yes extend the current mechanism and modernize it". Asked how much to take on, Casey chose, verbatim, "All of it (Recommended)": freshness tracking, a per-page banner and the toolchain upgrade. It reached this session in the parent session's brief at 2026-11-14T18:59:16Z (when it was recorded here; the answer came earlier). It approves the one owner-path edit: `.github/workflows/docs-site.yml` (owner path `.github/**`), the two `cargo install` versions and the tools cache key only.

## How this unit was claimed

By hand, as claude-377 was: the app's edit guard lets this session write only in its own worktree, `.claude/worktrees/docs-site-freshness-1d411d`, so `loop claim` is not used. The branch `claude/docs-site-freshness` is reset onto `origin/main` at the base. By hand:

- the item is on `origin/main` (#3971) with a `scope:` that covers every file below;
- of the open PRs (#3941, #3937, #3889), only #3889 edits a page this unit records, `docs/research/notebook/cpu-training.md`, in a hunk starting at its line 10; this unit adds only that page's first line;
- the five pages #3969 rewrote (`docs/mechanisms/README.md`, `exact-memory.md`, `flock-read.md`, `route-read.md` and `docs/model/pipeline-map.md`) are left unrecorded, as are `CHARTER.md` and `docs/rules/changes.md` (owner paths);
- there is no bench inbox on this machine (`local/bench/INBOX.md` in the main checkout is absent).

## Cancel probe (run first)

- **Does mdBook 0.5.4 read the book at all?** Stage the book, run `mdbook-mermaid install .` and `mdbook build` with both new binaries from `local/mdbook-0.5/bin`. If it cannot be made to build with no ERROR, the toolchain part stops and is reported; the records and `book-fresh` go ahead.
- **Classed:** instrument (documentation tooling). Nothing here touches the model.

## Files in scope

- `xtask/src/book_fresh.rs` (new), `xtask/src/book_stage.rs`, `xtask/src/main.rs`
- `book.toml`, `.github/workflows/docs-site.yml` (versions and cache key only)
- `docs/harness/README.md` (the record format, one new section; a shared file)
- the first line of each book page that gets a record (the item's page globs)
- `docs/loop/tasks/docs-site-freshness/claude-378/**`

## Done when

1. **Every book page with real sources has a record**, its sources each matching a tracked file. `checked=` names only a commit a page was read against in this unit; every other page is left unchecked and reported so.
2. **`cargo xtask book-fresh`** counts, per page, the commits since `checked` that touch its sources (`git log`, every `GIT_*` variable removed), lists stale, unchecked, unrecorded and broken pages, writes `--json OUT`, and exits 0 (advisory, D76). Its tests can fail: each pins a behaviour a deliberate break of the code breaks.
3. **`book-stage`** strips the record and writes the banner line in the staged copy only.
4. **At the head, with mdBook 0.5.4 and mdbook-mermaid 0.17.1:** `book-stage`, `mdbook-mermaid install .`, `mdbook build` (0 ERROR) and `xtask book-check` (0 broken links) pass; the xtask tests pass.
5. **`loop report --check` and `loop pr-check --local`** pass, and a fresh exact-head review is recorded with `loop review --context fresh`.

## Stop and decide (never wait) if

- a 0.5 change alters how a page renders against GitHub's view: set the `book.toml` option that keeps GitHub's reading, and name it in the REPORT;
- a page's record cannot be made honest (no source matches): leave the page unrecorded and list it.
