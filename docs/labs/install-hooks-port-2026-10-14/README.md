# `git install-hooks` — the refusals are the feature

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the delegation tail, continuing from `switch` and `delete-merged`

## What landed

**`uor-harness git install-hooks` is Rust**, and the hook it installs is **byte-identical to
Python's** (`md5 5efd0a207567` both sides).

**Five `delegate_to_python` sites remain.**

## The hook lives in a data file, and that is a deliberate choice

**1983 bytes of `/bin/sh` with an embedded Python heredoc**, carried as
`crates/uor-r4-harness/rules/precommit-hook.sh` and compiled in with `include_str!`. Three reasons:

- **A raw string containing a Python heredoc is exactly the literal a Rust edit mangles** — and this
  port has corrupted source twice already by writing a backtick-bearing string through a shell.
- **The file is diffable as what it is.** A reviewer reads shell, not an escaped Rust literal.
- **The D23 interlock reads `.rs` files, and a hook is not model code.** Keeping it out of the Rust
  sources keeps that scan about what it is for.

**`include_str!` means it is still compiled in**: no runtime file read, and **a missing file is a build
error rather than a silently empty hook.**

## Three refusals, and each is the point of the command

> *"Install/refresh the marked pre-commit hook. **Never clobbers a foreign hook.**"*

| what is found | what happens |
|---|---|
| **a symlink** | **refused outright** — `write_text` through it would **write to the target**, so the guard is about somebody else's file, not this one |
| a file **without the marker** | refused, **showing the marker it looked for and the first line it found** — those two lines are what a reader needs to decide whether to merge by hand or delete it |
| an **unreadable** file | refused, because **not being able to see what is there is not permission to overwrite it** |
| a **harness-written** hook | **refreshed**, not refused — the marker is what separates *"mine, update it"* from *"somebody else's, leave it"* |

**Every refusal returns 2 and names the command to re-run**, so the reader is not left to guess.

## Four tests, and the symlink one is why the guard exists

| test | asserted |
|---|---|
| it installs, **executable**, and a re-run says `unchanged` | `mode & 0o111 == 0o111`, bytes equal `HOOK_SCRIPT` |
| **a foreign hook is refused and left alone** | the original still contains its own content |
| **a symlinked hook is refused and the TARGET is untouched** | the target file is byte-identical |
| a previous harness hook is **updated** | the old body is gone, the new one is present |

**The executable bit is asserted because a hook that is not executable is a file git silently
ignores** — and reporting *"installed"* for one would be the worst outcome this command can produce.

**My first run of the foreign-hook test failed**, asserting the message showed the comment line; **it
shows the shebang**, because that is the file's first line and what a reader needs to recognise it.
The code was right and the assertion was wrong.

## Divergence, stated

**Rust installs into a bare repository with no registry; Python cannot.** On a scratch repo with only
`README.md`, Python fails with `FileNotFoundError: docs/harness/milestones.json` **before reaching the
hook at all**. **Installing a hook does not need the milestone registry**, and the port does not ask
for it — **the same doctrine as `delete-merged`'s claim guard needing no network.** Recorded as a
divergence rather than quietly enjoyed.

## Verification

| check | result |
|---|---|
| the installed hook vs Python's `HOOK_SCRIPT` | **byte-identical**, and installed end to end |
| four guard tests | pass |
| `cargo test -p uor-r4-harness` | **402 passed, 4 ignored** |
| `cargo fmt --check`, clippy | clean, 0 findings |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |
| `parity.py` | exit 0 |

## Limitations, stated

- **`git status` is the last git action still delegated, and it is the largest.** It assembles worktree
  facts, PR facts and stale-branch advice — **a partial port would report fewer facts than the Python
  one while looking complete**, so it stays delegated until the whole thing lands.
- **The hook itself still calls `python3 scripts/harness/cli.py check --staged`.** The hook is a shell
  script that invokes the harness, **and `check --staged` is not a ported flag** — the Rust `check`
  does not implement `--staged`. **So the installed hook runs the Python CLI**, which then delegates
  `check` back to Rust. **The hook is correct and the round trip is wasteful**; it is stated because a
  reader would reasonably assume a Rust-installed hook invokes Rust.
- **The port does not compare the installed hook against what is already there byte-for-byte before
  writing** — it always writes and then reports `unchanged` if the bytes matched. **Python does the
  same**, and the cost is a write per invocation on a file that did not change.
