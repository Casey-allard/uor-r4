# REPORT: infra-loop-harness, claude-49 (a faithful archive copy's links are reported, not failed)

Lab: claude. Base: `origin/main` after #3006 (`ee71c1dcf`). Order: [ORDER.md](ORDER.md), committed first (`8e6db50a2`). The cancel probe found that at the base the gate fails a newly added byte-identical archive copy whose link resolves from the original, because `MOVED` needs an R100 rename. So the task stands.

## What did not work, first

- **The gap was invisible until PR 8.** The 2026-10-20 cutover moved 8,710 files as pure renames with no stubs. §4.7 keeps a stub at the old path, so git records a modification plus an addition, never a rename pair, and `MOVED` never matched (Kimi's card, 6351470799).
- **Kimi's option 1 was too broad as written.** Reporting *every* NEW broken link under `archive/` would also hide a new archive record with genuinely broken links. `COPIED` therefore requires four things:
  - the source is newly added;
  - it is byte-identical to a file present at REV;
  - its link resolves from that original's directory;
  - the target exists now.
- **Clippy `-D warnings` on `xtask` fails, but not on this change.** Five errors are in files this PR does not touch (`figures.rs:1149`, `archive_verify.rs:394` and `:396`, `book_stage.rs:342`, `docs_relink.rs:249`). `origin/main` gives the same errors under the pinned toolchain. `docs_links.rs` itself is clean; the builder fixed two `octal_escapes` in its own new test.
- **The new count line shows only in baseline mode.** `copied (archived copy)` appears only in `--baseline --renames-from` mode, which PR 8's gate uses. A plain `links` run prints the same totals as before.

## What changed

| Commit | What |
|---|---|
| `8e6db50a2` | This ORDER, alone. |
| `0175854d0` | See the list below. |
| this REPORT's commit | This REPORT. |

What `0175854d0` changes in `xtask/src/docs_links.rs` (written by a builder subagent):
- **The decision:** a pure `split_copied(new, originals, existed_at_rev, exists_now)`. It runs after `MOVED`, on what `MOVED` leaves, and is off under `--simulate`, as `MOVED` is.
- **The git calls:** one `git ls-tree -r -z REV`, parsed into the paths at REV and a map from blob id to the first path, plus one `hash-object --stdin-paths` batch for the new `archive/` sources. Both run only when a NEW link has an `archive/` source.
- **Output:** a `COPIED` line for each case and a count line.
- **Refactor:** the existing hashing in `replaced_stub_map` moves into a shared helper, with unchanged behaviour.
- **Docs:** the module doc gains a paragraph.

## Measurements

None.

## Checks run at the head

- `cargo test -p xtask`: 200 passed, 6 of them new.
  - Tests (a)–(e): a faithful copy is `COPIED`; a copy whose link fails from the original fails; an archive file matching no REV blob fails; a source that existed at REV is not copied; a non-archive source is never copied.
  - One more for the ls-tree parser.
- **Planted fault:** with a missing original falling back to a live path, so byte equality is ignored, test (c) fails (24 passed, 1 failed). Restored, all pass.
- `cargo run -p xtask -- links`: same totals as before (943 broken).
- **Clippy:** `docs_links.rs` clean. Five pre-existing errors elsewhere, as above.
- `loop report --check` and `loop pr-check --local`: see the PR body.

## Skills used

scope-guard: one file, `xtask/src/docs_links.rs`, added to scope by #3006.

## Still open

- **PR 8 (#2982) merges `main`** after this lands. Its 19 archive-copy links should then print as `COPIED`.
- **The five pre-existing xtask clippy errors** are a separate cleanup.

## Plain summary

Kimi's PR 8 archives exact copies of some pages, and their relative links only break because the copies sit in a different folder. The link checker now recognises an exact copy whose links work from the original and lets it through, while still failing any genuinely broken new link.
