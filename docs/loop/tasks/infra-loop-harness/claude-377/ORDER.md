# ORDER: infra-loop-harness, claude-377 (register the docs-site-freshness item)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904), the umbrella that edits roadmap items. Base: `e2175455e` (`e2175455e79d377f724658ed2486e7f22a389f48`, main after codex-2's #3967).

Setup row: 2 (item-file maintenance: a new gap item).

## Goal

Add `docs/roadmap/items/docs-site-freshness.md`, render the roadmap, and open its issue with `cargo xtask roadmap sync --apply`, run from this worktree. The work itself is the next unit, claude-378, under the new item: `loop pr-check` reads an order's item at the merge base, so the item must be on `main` before that unit's PR.

**Why.** The owner, Casey, 14 Nov 2026, in claude's chat: "yes extend the current mechanism and modernize it". Asked how much to take on, Casey chose, verbatim, "All of it (Recommended)": freshness tracking, a per-page banner, and the toolchain upgrade. That answer is recorded here verbatim as the owner approval for the `.github/workflows/docs-site.yml` edit the item's unit makes (`.github/**` is an owner path in `docs/harness/owner-paths.json`). This unit edits no owner path.

## How this unit was claimed

By hand, as claude-280 was: the app's edit guard lets this session write only in its own worktree, `.claude/worktrees/docs-site-freshness-1d411d`, so `loop claim` (which cuts `.worktrees/<lab>-<topic>`) is not used. The branch `claude/docs-site-freshness-item` is cut there, off `origin/main` at the base. Every other check `loop claim` makes holds by hand:

- `infra-loop-harness` is on `origin/main` with a `scope:` that covers `docs/roadmap/items/*.md`;
- no open order names this unit's files (the open PRs are #3970, #3969, #3968, #3941, #3937 and #3889; #3968 edits `docs/roadmap/README.md`, the generated page, and `infra-loop-harness.md`, which this unit does not edit);
- the bench inbox, `local/bench/INBOX.md` in the main checkout, does not exist on this machine (its README says it moved to `.worktrees/bench-main/local/bench`, which is also absent), so there is no waiting message to act on.

## Cancel probe (run first)

- **Does an item already cover this?** Search `docs/roadmap/items/` on `origin/main` for `book_stage`, `book.toml`, `docs-site.yml`, `mdbook` and freshness.
- **Are the scope's paths real?** Check each named file with `git cat-file -e origin/main:<path>`; `xtask/src/book_fresh.rs` and the task folder are files the item's unit creates.
- **Classed:** instrument. Nothing cancels the unit unless an item already covers it.

## Files in scope

- `docs/roadmap/items/docs-site-freshness.md`
- `docs/roadmap/issues.json`
- `docs/roadmap/README.md`
- `docs/loop/tasks/infra-loop-harness/claude-377/**`

## Done when

1. **The item** is on the branch with the six required sections, `status: ready`, `needs_gpu: false` and a `scope:` naming only the files its unit edits.
2. **`cargo xtask roadmap check`** passes, and `roadmap render --check` reports the render current.
3. **Its issue exists**, opened by `roadmap sync --apply` from this worktree, and `issues.json` maps the id to it.
4. **A fresh exact-head review passes,** checking the item's facts against `origin/main` and its scope against the work its done-when needs.
5. **`loop report --check` and `loop pr-check --local`** pass.

## Stop and decide (never wait) if

- the review finds a cited fact wrong: fix the item before the issue is opened, or re-sync after;
- another open order holds a file in the scope: narrow the scope, or order the units.
