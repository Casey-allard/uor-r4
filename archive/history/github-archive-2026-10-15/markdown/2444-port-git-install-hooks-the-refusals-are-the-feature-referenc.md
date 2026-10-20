# #2444 — Port git install-hooks; the refusals are the feature (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T05:15:07Z
- **Updated:** 2026-10-14T05:18:07Z
- **Closed:** 2026-10-14T05:17:37Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2444
- **Merged:** 2026-10-14T05:17:37Z
- **Merge commit:** 6b863e9c347ed95a91f6af8b0cb93850cf6f371c
- **Base / Head:** main / UOR-Foundation:deepseek/git-hooks-status
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**`git install-hooks` — the refusals are the feature.** Tooling only.

Decision: **KEEP** (tooling).

## What landed

**`uor-harness git install-hooks` is Rust**, and the hook it installs is **byte-identical to Python's** (`md5 5efd0a207567` both sides). **Five `delegate_to_python` sites remain.**

## The hook lives in a data file, deliberately

**1983 bytes of `/bin/sh` with an embedded Python heredoc**, carried as `crates/uor-r4-harness/rules/precommit-hook.sh` and compiled in with `include_str!`:

- **A raw string containing a Python heredoc is exactly the literal a Rust edit mangles** — **this port has corrupted source twice already** writing a backtick-bearing string through a shell.
- **The file is diffable as what it is.** A reviewer reads shell, not an escaped Rust literal.
- **The D23 interlock reads `.rs` files, and a hook is not model code.**

**`include_str!` keeps it compiled in**: no runtime read, **and a missing file is a build error rather than a silently empty hook.**

## Three refusals, each the point of the command

| what is found | what happens |
|---|---|
| **a symlink** | **refused outright** — writing through it would **write to the target**, so the guard is about somebody else's file |
| a file **without the marker** | refused, **showing the marker looked for and the first line found** — what a reader needs to decide merge-vs-delete |
| an **unreadable** file | refused — **not being able to see what is there is not permission to overwrite it** |
| a **harness-written** hook | **refreshed** — the marker separates *"mine, update it"* from *"somebody else's, leave it"* |

## Four tests — the symlink one is why the guard exists

| test | asserted |
|---|---|
| installs, **executable**, re-run says `unchanged` | `mode & 0o111 == 0o111`, bytes equal `HOOK_SCRIPT` |
| **foreign hook refused and left alone** | the original still contains its own content |
| **symlinked hook refused and the TARGET untouched** | the target is byte-identical |
| a previous harness hook is **updated** | old body gone, new present |

**The executable bit is asserted because a hook that is not executable is a file git silently ignores** — reporting *"installed"* for one would be the worst outcome here.

**My first foreign-hook run failed**, asserting the message showed the comment line; **it shows the shebang**. **The code was right and the assertion was wrong.**

## Divergence, stated

**Rust installs into a bare repository with no registry; Python cannot** — on a scratch repo with only `README.md`, Python fails with `FileNotFoundError: docs/harness/milestones.json` **before reaching the hook.** **Installing a hook does not need the milestone registry** — the same doctrine as `delete-merged`'s claim guard needing no network.

```
cargo test -p uor-r4-harness                    -> 402 passed, 4 ignored
cargo fmt --check / clippy -D warnings          -> clean / 0 findings
python3 scripts/check_no_softmax.py             -> OK — 1641, baseline 1641
parity.py                                       -> exit 0
```

Record: `docs/labs/install-hooks-port-2026-10-14/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried, presence asserted before commit
- [x] Branch deleted after merge

### Stated rather than hidden

- **`git status` is the last git action still delegated, and the largest.** A partial port **would report fewer facts than Python's while looking complete**, so it stays delegated until the whole thing lands.
- **The hook itself still calls `python3 scripts/harness/cli.py check --staged`** — and **`check --staged` is not a ported flag**, so **the installed hook runs the Python CLI, which delegates `check` back to Rust.** **The hook is correct and the round trip is wasteful** — stated because a reader would reasonably assume a Rust-installed hook invokes Rust.
- **The port always writes and then reports `unchanged` if the bytes matched**, rather than comparing first. **Python does the same**, and the cost is a write per invocation.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

