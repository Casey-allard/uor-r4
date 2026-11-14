# REPORT: docs-site-freshness, claude-378 (page records, `book-fresh`, the staged banner, mdBook 0.5)

Lab: claude. Order: [ORDER.md](ORDER.md). PR #3975. References #3972.

## What did not work, first

- **Only 2 of 97 recorded pages are checked.** A page gets `checked=` only when I read it whole against its sources. I did that for `projects/riemann-lean/README.md` and `projects/research-archives/README.md`, and every file count, byte count, path count and toolchain pin they state matched. The other 95 have no `checked=`: 93 have sources and `book-fresh` reports them as unchecked, and 2 are `sources=none`. Three read-only subagents were briefed to check pages in batches. Each one ran out of context after 0 to 2 pages, so none of their output is used as a check. The sources themselves were derived from what each page links to and names, and then filtered so that every entry matches a tracked file.
- **Six pages I read are wrong today, so they stay unchecked:**
  - `docs/start/repository-layout.md` says it lists every top-level folder and the files at the top, but it leaves out `bake-off/`, `rules/`, `src-tauri/`, `PLAN.md`, `STATE.md` and `HANDOFF.md`.
  - `projects/README.md` says there are seven side projects. `projects/` has eight folders: `uor-cuda-inverted-file/` is not listed.
  - `projects/msa/README.md` says the folder holds "the README and a hash-and-pointer file". It actually holds `README.md` and `code/README.md`, the original README.
  - `docs/model/gpu-and-cuda.md:33` says `discrete.cu` "has 718 lines". It has 749.
  - The export list in `docs/products/web-client.md` omits `memoryInfo`, `markReplyStart`, `clearReplyStart`, `loadDiscrete`, `predictDiscrete`, `discreteInfo` and `exactArithmetic`, all of which are in `crates/uor-r4-integer/src/wasm_bindgen_exports.rs`.
  - `docs/minimal_client.md` describes `./r4-app.sh`, which exists only at `archive/legacy-cli/r4-app.sh`.

  None of these pages is fixed here. That is a page-content unit, not this tooling unit.
- **One version warning is left.** `mdbook build` prints "The mdbook-mermaid preprocessor was built against version 0.5.0 of mdbook, but we're being called from version 0.5.4". mdbook-mermaid 0.17.1 compares the two version strings exactly (`src/bin/mdbook-mermaid.rs:59`), and its own `Cargo.lock` pins `mdbook-preprocessor` 0.5.0. Both versions speak the same 0.5 protocol. The only way to remove the warning is to install mdbook-mermaid without `--locked`. That would let roughly a hundred of its dependencies float in a job that holds a Pages write token. **Decision (my own review, reversible):** keep `--locked` and accept the warning. The workflow fails only on `ERROR`, and there are none.
- **The bench inbox** (`local/bench/INBOX.md` in the main checkout) does not exist on this machine, so I read no messages and marked none.
- **The ORDER was edited after its first commit** to add the `## Owner approval` section, because `loop pr-check` requires one for a change to `.github/**`.

## What changed

- **`xtask/src/book_fresh.rs`** (new) and **`cargo xtask book-fresh [--json OUT]`**:
  - It reads the first-line record of every page in `docs/SUMMARY.md`: `<!-- freshness: sources=a,b checked=<40-hex> -->`, or `sources=none`.
  - It sorts each page into one of seven states: fresh, stale, unchecked, no-sources, unrecorded, broken or malformed. A source is a `:(glob)` pathspec, and the page's own file is excluded.
  - It runs `git` with every `GIT_*` variable removed.
  - `--json` writes the schema `uor-r4.book-fresh/1`. It is advisory (D76): it exits 0 whatever freshness it finds and lists a page it cannot read or assess as `broken`; only a bad argument, an unreadable SUMMARY, no git history or an unwritable `--json` file exits 1. `git log` runs with `--no-show-signature`.
- **`book-stage`** removes the record from the staged copy and puts a banner line first. The banner gives the checked commit with a link, its date and the count of later changes, or "Not yet checked …". Only the copy under `target/book-src` changes.
- **Records on 97 pages.**
  - Left unrecorded: `CHARTER.md` and `docs/rules/changes.md` (owner paths), plus `docs/mechanisms/README.md`, `exact-memory.md`, `flock-read.md`, `route-read.md` and `docs/model/pipeline-map.md` (edited by #3969).
  - Two pages have `sources=none`: the harness-governance notebook and `transformer-vs-geometry.md`.
- **Toolchain.** `.github/workflows/docs-site.yml` moves to mdbook 0.5.4 and mdbook-mermaid 0.17.1 under the cache key `mdbook-0.5.4-mermaid-0.17.1-…`. Only the two install versions and the cache key change, under the owner approval quoted in the ORDER.
- **`book.toml`** sets `smart-punctuation = false` and `definition-lists = false`. mdBook 0.5 turns both on by default, and GitHub renders neither, so these settings keep the site's text the same as GitHub's. Admonitions and hash-files stay on, and `mermaid-install` leaves `book.toml` unchanged. I checked the 0.5 changelog: no field this file uses was removed.
- **`docs/harness/README.md`** has a new section, "Book page freshness", that documents the format and how to re-check a page.

## Measurements

`cargo xtask book-fresh` at the head: **104 pages: 2 fresh, 0 stale, 93 unchecked, 2 no-sources, 7 unrecorded, 0 broken, 0 malformed.** The 280 distinct sources each match at least one tracked file.

## Checks run at the head

- With `local/mdbook-0.5/bin` first on `PATH` (mdbook v0.5.4, mdbook-mermaid 0.17.1):
  - `book-stage` staged 105 pages and 70 figure files.
  - `mdbook-mermaid install .` passed.
  - `mdbook build` exited 0 with 0 `ERROR`; its one warning is described above.
  - `xtask book-check` reported 107 pages and 0 broken links.
  - The staged `projects/riemann-lean/index.md` opens with the fresh banner, and no staged page still holds a record.
- `cargo test -p xtask`: 241 passed, 0 failed. `clippy -p xtask --all-targets` gave no warning in the new lines; its two warnings are pre-existing, at `book_stage.rs:343` and `docs_relink.rs:250`. `rustfmt --check` passes on `book_fresh.rs`.
- **Negative gates:**
  - Without `:(glob)`, `git_cli_counts_commits_touching_sources` fails.
  - Without the self-exclusion, the same test fails.
  - When `stage_page` passes the text without `strip_record`, `staged_pages_carry_a_banner_and_lose_the_record` fails.
  - When the `env_remove` call is disabled, `git_command_removes_every_git_variable` fails.
  - Each test passes again once its code is restored.
- **Two fresh reviews of the code at `51b69f051`** (read-only, spec and diff only), both APPROVE WITH CHANGES, no blocker. What each found, and what was done:
  - SHOULD (both): the command could still exit 1, because a page it could not read or a failing git call passed its error up through `?`. Fixed: `assess_or_broken` lists such a page as `broken` with the reason. The docs now say what does exit 1: a bad argument, an unreadable SUMMARY, no HEAD, or an unwritable `--json` file. A third review of the fix commit (`8347cd6b7`) found `git show` also needed `--no-show-signature` (SHOULD, fixed) and that a broken page dropped its parsed record from the JSON (NIT, fixed).
  - SHOULD: the CLI layer and the `GIT_*` removal were not pinned. Fixed with two new tests. `json_has_schema_every_count_and_null_sources_when_unrecorded` checks the schema, the seven counts and `sources: null`. `git_command_removes_every_git_variable` passes a fixed environment to the new `GitCli::command`; its negative gate is listed above. A third test, `unreadable_pages_and_git_errors_are_broken`, pins the first fix.
  - NIT: a user's `log.showSignature` setting would add lines to the `%H` count. Fixed with `--no-show-signature`.
  - NIT: the README's banner list was missing four texts, and this REPORT said 95 pages had sources when 2 of them are `sources=none`. Both fixed.
  - NIT, not done: a page that names only itself as its source stays fresh forever. No page does this, because every page's sources are other files.
  - The reviews confirmed: the workflow diff is exactly the three approved lines; checkout uses `fetch-depth: 0`, so `checked=` commits resolve in CI; each of the 97 page edits adds one line, above an H1; and neither review found a forbidden file touched.
- `uor-harness loop report --check` and `loop pr-check --local` both pass. Two advisories remain, `read-path` (`AGENTS.md` at 83 lines) and `filemap` (five missing rows, all outside this diff).

## Skills used

None applied.

## Still open

1. **95 unchecked pages.** Each needs a whole-page read against its sources. Those that pass get `checked=`, and the six above need fixes first. This is the next unit, page by page.
2. **The mermaid version warning** will clear when mdbook-mermaid publishes a release locked to the mdbook version the workflow uses.

## Plain summary

The project's documentation website now records, for each page, which code the page describes and when someone last compared the two. A new command lists the pages whose code has changed since. The website shows a small line at the top of each page saying whether the page has been checked and how many changes it has missed. Only two pages are marked as checked so far, because a page is marked only when someone has really read it. Six of the pages I read were found to be out of date and are listed above. The site's build tools moved to the current versions, and the site's links all still work.
