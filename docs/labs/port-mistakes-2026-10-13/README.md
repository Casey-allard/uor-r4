# D30 port 5: `mistakes` is Rust

**Date:** 2026-10-13 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the D30 order on #2028; port 5 of the series

## What moved

`crates/uor-r4-harness/src/mistakes.rs` — new, **12 tests**. `uor-harness mistakes list|find` is Rust
and **out of the delegated set**. `show` and `capture` still delegate **by name** — `capture` writes a
**lesson file plus a guard**, a bigger surface than reading one, and it belongs in its own unit.

**Verified against Python on a real file** — identical match set:

```
$ uor-harness mistakes find -f crates/uor-r4-harness/src/status.rs
2 lesson(s) apply:
  [block] host-fallback-is-a-defect - A host-side fallback presented as a cost, not as a defect
      why: path crates/uor-r4-harness/src/status.rs
      guard: prlint.no_cpu_model_compute
  [info] portability-constraint-is-interpreter-discovery - ...
```

Same two lessons, same severities, same guards, same `why`. Python truncates the lesson body and
prints it; Rust omits it (see limitations).

## The bug worth naming: `fnmatch` is not a shell glob

**Python's `fnmatch` `*` crosses `/`, and so does `?`.** A shell glob does not. I assumed shell
semantics, so **`*team.rs*` did not match `crates/x/src/team.rs`** — meaning a lesson written against
a bare filename **silently stopped firing**.

**A guard that stops firing is indistinguishable from a guard that passes.** That is the same failure
class as this whole line, and it is now pinned by a test with Python's answers quoted in it:
`fnmatch('src/*.rs', 'src/deep/team.rs')` is True; `fnmatch('a?c', 'a/c')` is True.

## Two tests of mine that were wrong, not the code

Both in the same direction — **I checked the rule I had in mind rather than the one written down**:

1. `query_tokens("stale-binary_x")` — I asserted it split into three. **`-` and `_` are token
   characters**, so it is one token.
2. `query_tokens("ab cd")` — I asserted `cd` survived. The length test is on the **token**, and two
   characters is not more than three, so neither survives.

The code was right both times. Worth recording because a wrong test that gets "fixed" to match wrong
code is how a rule quietly changes.

## Rules the port keeps

- **`status` gates everything** — only `active` and `warn` lessons surface. A closed lesson is history,
  and resurfacing it trains readers to skim.
- **Query tokens are longer than three characters.** `the` appears in almost every trigger, so
  admitting it would match everything and mean nothing.
- **Results sort `block` before `warn` before `info`** — the reader is about to edit code and needs the
  stopping condition first.
- **`why` names which rule fired.** A lesson surfaced for the wrong reason teaches the wrong thing.
- **The tests load the real `docs/harness/mistakes/`**, and one asserts that a real harness source file
  surfaces at least one lesson — **otherwise the command is decorative**.

## Verification

| check | result |
|---|---|
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | 0 findings |
| `cargo test -p uor-r4-harness` | **194 passed** (12 new) |
| `python3 -m unittest discover -s tests/harness -t .` | 319 passed |
| `uor-harness check` | 0 errors |
| `mistakes find -f status.rs` vs Python | **identical match set** |
| delegation notice for `mistakes list` | gone |

## Limitations, stated

- **`capture` is not ported** — it writes a lesson **plus a guard**, and porting it means porting the
  guard-convention and the front-matter validation with it. `show` still delegates too.
- **The human output is not identical to Python's.** Python prints the truncated lesson body and the
  evidence list; Rust prints id, severity, title, `why` and guard. **The match set is identical, which
  is what `find` is for**, but a script parsing the extra lines breaks.
- **The four `check` rules `mistakes.py` defines** (registry validity, evidence exists, guard covered,
  stale) are **not ported**. They run today through the Python `check`, so a Rust `check` that skipped
  them would be a silent downgrade — which is why `check` stays delegated rather than half-ported.
- **`fnmatch` is implemented for `*` and `?` only.** Character classes (`[abc]`) are treated as
  literals, silently.
- **The tests are not compared against Python for the real registry** — the match set was compared by
  hand on one file. A lesson whose pattern relies on a construct I did not implement would diverge
  without a test noticing.

## Antigravity's onboarding defects — three fixed here

Antigravity onboarded as the fifth lab and reported **eight** concrete defects from actually running
the exercise. Three are fixed in this PR, and they are the three that were **mine and live**:

### Defect 4 — the committed claims store was being corrupted. This was the serious one.

**`team claim` / `team release` loaded the MERGED view and wrote it back to `docs/harness/claims.json`**,
so a lab releasing its own branch **committed every other lab's transient live claim** and dirtied the
working tree on a command that should not have touched it. Codex saw the same on #2029.

Measured before the fix: **live tier 12 claims, committed store 2.** The defect report is exactly right,
and the cause is one line of mine — the `mistakes` port has no part in it.

**A read view used as a write source.** Fixed by loading **committed only** for the write, while the
**conflict check still uses the merged view** — refusing a branch a peer holds only in the live tier
would be wrong, since that is what the mirror is for.

Verified end-to-end against the real live tier: a claim took the committed store from **2 branches to
3**, with no peer claim imported. Both behaviours have regression tests, so the fix cannot silently
revert to the version that corrupted the store.

### Defect 1 — the roster named four labs while R9 named five

`KNOWN_LABS` was `["kimi", "claude", "codex", "deepseek"]`, so **`onboard` warned a correctly-identified
Antigravity that it was "not one of" the four** — while `docs/harness/pm-rules.md` R9 listed five.

Fixed to five, and the warning now **names R9 as the authority** rather than the constant, because a
roster in the code and a roster in the rules will always drift. A lab not in the list is still
allowed — the list only decides whether a note is printed.

### Defect 2 — `/opt/homebrew/bin/cargo` is not Rust

**Confirmed on this machine:** `/opt/homebrew/bin/cargo` is a symlink to `@cargo-ai/cli`, a **Node.js
CLI**, and `cargo build` against it fails with `error: unknown command \'build\'`. The error names a
command the tool does not have — **the same shape as the stale-binary problem**: it points at the
symptom and nowhere near the cause.

`onboard` now runs `cargo --version` and requires the output to start with `cargo `, which is the only
test that distinguishes rustup's cargo from a same-named tool. Verified all three ways: the shadow is
caught, real cargo passes, and `UOR_LAB=antigravity` no longer warns.

## Defects reported and NOT yet fixed

Stated so the report is complete rather than flattering:

- **3 (stale global `~/.cargo/bin/uor-harness`)** — the staleness warning added in #2354 catches this,
  but the installed copy is still old until someone rebuilds. Not fixed here.
- **5 (`team dispatch` prints a stale unsharded receipt path and dirties the worktree)** — not fixed.
  `dispatch` still delegates, so this is Python-side.
- **6 (`prep new -m m1` embeds a D28-banned `qwen2.5:7b` grader and pre-D25 rule citations)** — **not
  fixed, and it is the most serious of the remainder**: the D22 pre-training gate is pulling a banned
  grader and stale rule text from the panel registry. That is a correctness problem in the gate that
  decides whether compute may run, and it deserves its own unit rather than a line here.
- **7 (`ONBOARD-PROMPT.md` Step 6 `git checkout main` fails in a linked worktree)** — not fixed. A doc
  fix, but it makes the documented onboarding path fail for the way labs actually work under R10.
- **8 (the ten stale live-tier peer claims)** — they are still in the live tier. The fix stops NEW
  ones being committed; clearing the existing mirror is a separate decision, since a live claim may be
  a lab's only published claim.

