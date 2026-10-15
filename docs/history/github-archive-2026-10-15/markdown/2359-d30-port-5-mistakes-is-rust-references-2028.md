# #2359 — D30 port 5: mistakes is Rust (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T19:19:48Z
- **Updated:** 2026-10-13T19:43:54Z
- **Closed:** 2026-10-13T19:43:36Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2359
- **Merged:** 2026-10-13T19:43:36Z
- **Merge commit:** 01fab6eafaab0a835507f85ce59fcff844b1eaae
- **Base / Head:** main / UOR-Foundation:deepseek/port-mistakes
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**D30 port 5.** Tooling only.

Decision: **KEEP** (tooling).

## What moved

`crates/uor-r4-harness/src/mistakes.rs` — new, **12 tests**. `uor-harness mistakes list|find` is Rust and **out of the delegated set**. `show` and `capture` delegate **by name** — `capture` writes a **lesson file plus a guard**, and it belongs in its own unit.

**Verified against Python on a real file — identical match set:**

```
$ uor-harness mistakes find -f crates/uor-r4-harness/src/status.rs
2 lesson(s) apply:
  [block] host-fallback-is-a-defect - A host-side fallback presented as a cost, not as a defect
      why: path crates/uor-r4-harness/src/status.rs
      guard: prlint.no_cpu_model_compute
  [info] portability-constraint-is-interpreter-discovery - ...
```

## The bug worth naming: `fnmatch` is not a shell glob

**Python's `fnmatch` `*` crosses `/`, and so does `?`.** A shell glob does not. I assumed shell semantics, so **`*team.rs*` did not match `crates/x/src/team.rs`** — a lesson written against a bare filename **silently stopped firing**.

**A guard that stops firing is indistinguishable from a guard that passes.** Pinned by a test quoting Python's answers: `fnmatch('src/*.rs','src/deep/team.rs')` True; `fnmatch('a?c','a/c')` True.

## Two of my own tests were wrong, both the same way

**I checked the rule I had in mind rather than the one written down:**

1. `stale-binary_x` — I asserted three tokens. **`-` and `_` are token characters**, so it is one.
2. `ab cd` — I asserted `cd` survived. The length test is on the **token**; two characters is not more than three.

**The code was right both times.** Worth recording because **a wrong test "fixed" to match wrong code is how a rule quietly changes.**

## Rules kept

- **`status` gates everything** — only `active`/`warn` surface; resurfacing a closed lesson trains readers to skim.
- **Query tokens are longer than three characters** — `the` appears in almost every trigger.
- **`block` before `warn` before `info`** — the reader is about to edit code and needs the stopping condition first.
- **`why` names which rule fired** — a lesson surfaced for the wrong reason teaches the wrong thing.
- Tests load the **real** registry, and one asserts a real harness source file surfaces at least one lesson — **otherwise the command is decorative**.

```
cargo fmt --check                               -> clean
cargo clippy --all-targets -- -D warnings       -> 0 findings
cargo test -p uor-r4-harness                    -> 194 passed (12 new)
python3 -m unittest discover -s tests/harness   -> 319 passed
uor-harness check                               -> 0 errors
mistakes find -f status.rs vs Python            -> identical match set
delegation notice for `mistakes list`           -> gone
```

Record: `docs/labs/port-mistakes-2026-10-13/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried
- [x] Branch deleted after merge

### Stated rather than hidden

- **`capture` is not ported**, and neither is `show`. `capture` writes a lesson **plus a guard**, so porting it means porting the guard convention and the front-matter validation with it.
- **The human output is not identical to Python's.** Python prints the truncated lesson body and the evidence list; Rust prints id, severity, title, `why`, guard. **The match set is identical** — which is what `find` is for — but a script parsing those extra lines breaks.
- **The four `check` rules `mistakes.py` defines are not ported**: registry validity, evidence exists, guard covered, stale. **Which is exactly why `check` stays delegated rather than half-ported** — a Rust `check` skipping them would be a **silent downgrade**.
- **`fnmatch` covers `*` and `?` only.** A character class is treated as a literal, silently.
- **The real-registry match set was compared by hand on one file**, not asserted against Python. A lesson relying on a construct I did not implement would diverge without a test noticing.

## Antigravity's onboarding report — three defects fixed, and all three were mine

Antigravity onboarded as the **fifth lab** and reported **eight** concrete defects from actually running the exercise. The three fixed here were live defects in code I wrote.

### Defect 4 — the committed claims store was being corrupted. This was the serious one.

**`team claim` / `team release` loaded the MERGED view and wrote it back to `docs/harness/claims.json`**, so a lab releasing its **own** branch **committed every other lab's transient live claim** and dirtied the working tree. Codex saw the same on #2029.

Measured before the fix: **live tier 12 claims, committed store 2** — exactly as reported.

**A read view used as a write source.** Fixed by loading **committed only** for the write, while the **conflict check still uses the merged view** — refusing a branch a peer holds only in the live tier would be wrong, since that is what the mirror is for.

**Verified end-to-end against the real live tier:** a claim took the committed store from **2 branches to 3**, no peer claim imported. Both behaviours have regression tests.

### Defect 1 — the roster named four labs while R9 named five

`KNOWN_LABS` was four names, so **`onboard` warned a correctly-identified Antigravity** while `pm-rules.md` R9 listed five. Fixed to five, and the warning now **names R9 as the authority** rather than the constant — a roster in the code and a roster in the rules will always drift.

### Defect 2 — `/opt/homebrew/bin/cargo` is not Rust

**Confirmed on this machine:** it is a symlink to `@cargo-ai/cli`, a **Node.js CLI**; `cargo build` fails with `error: unknown command 'build'`. **The same shape as the stale-binary problem** — it names the symptom and nothing about the cause.

`onboard` now requires `cargo --version` to start with `cargo `. Verified three ways: shadow caught, real cargo passes, `UOR_LAB=antigravity` silent.

## Reported and NOT fixed, stated rather than omitted

- **Defect 3** — the stale global `~/.cargo/bin/uor-harness`. #2354's warning catches it; nothing refreshes it.
- **Defect 5** — `team dispatch` prints a stale unsharded receipt path and dirties the worktree. Python-side, still delegated.
- **Defect 6 — the pre-training gate pulls a D28-banned grader and stale rule citations from the panel registry.** **The most serious of the remainder**: it is a correctness problem inside the gate that **decides whether compute is permitted at all.** It deserves its own unit, and it outranks the port order.
- **Defect 7** — `ONBOARD-PROMPT.md` Step 6 `git checkout main` fails in a linked worktree, which is how labs actually work under R10.
- **Defect 8** — the ten stale live-tier peer claims. The fix stops **new** ones reaching the committed store; it does not clear the mirror, because a live claim may be a lab's only published one.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

