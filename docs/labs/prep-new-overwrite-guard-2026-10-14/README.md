# `prep new` no longer overwrites a sheet silently

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the deferred gap in the `prep` port, closed instead of deferred again

## The defect

`prep new` wrote its sheet with **no existence check at all**:

```python
target.parent.mkdir(parents=True, exist_ok=True)
target.write_text(text, encoding="utf-8")
```

**A pre-registration sheet is the D22 gate's own record.** Re-running `prep new` for a milestone that
already had one **replaced the bar, the panel and the cost that were pre-registered *before* the
work** — which is precisely the evidence D22 exists to preserve. **The command that creates the
record was the one thing able to destroy it.**

## Fixed in `fields`-level `new_sheet`

**A second write is refused**, naming the file and saying what it holds, and **the sheet is left
untouched**. Replacing is available and explicit:

```
first write:   written
second write:  docs/harness/prep/m1-no-mechanism-2026-10-13.md already exists and was NOT overwritten.
               It is the pre-registered bar, panel and cost for this milestone; replacing it
               discards the record that D22 scores the work against.
               Re-run with --force to replace it.
with --force:  written
```

**The refusal is asserted to leave the file byte-identical**, not merely to return 1 — a guard that
reports refusal and writes anyway would be worse than none.

## Both implementations, because parity requires it

**Rust had the same defect and got the same guard** (`crates/uor-r4-harness/src/main.rs`), plus
`--force` declared in its option lists. Verified by running both against a scratch repo:

```
              refused   accepted with --force   sheet unchanged by the refusal
python           yes              yes                      yes
rust             yes              yes                      yes
```

## A mistake worth recording

**Adding `--force` to Rust's flag lists broke the build**, and my first repair attempt asserted on
the wrong line content:

- I inserted it **twice** — once in each list, because my second anchor matched the first insert;
- `SUBCOMMAND_FLAGS: [&str; 31]` then didn't match its contents, so **the array length had to change**;
- and my fix script asserted `== '        "--force",\n'` with **eight** spaces where the line had
  **four**, so it found no duplicate and raised.

**Three separate wrong assumptions about my own edit, in one small change** — and `cargo` caught the
one that mattered while my own checks caught none of them. **This is the tenth silent or
wrong-targeted edit in this session**, which is the strongest argument yet for the rule I have been
applying: **read the bytes, do not reason about the edit.**

## Verification

| check | result |
|---|---|
| python refuses a second write, file unchanged | asserted |
| python `--force` replaces it | asserted |
| rust refuses, and `--force` accepts | verified by running both |
| `cargo test -p uor-r4-harness` | 327 passed |
| `cargo fmt --check`, clippy | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | **332 passed** (2 new) |
| `parity.py` | exit 0 |
| `python3 scripts/check_no_softmax.py` | OK, no softmax added |

## Limitations, stated

- **`--force` is now the only way to re-render a sheet**, and **the sheet is not versioned** — a
  deliberate replacement leaves no trace of the previous bar. **Refusing is the improvement; it does
  not make replacement safe.**
- **The guard is by path.** A sheet for the same milestone written under `--write <other path>` is a
  different path and is not guarded, **so the protection is only as good as the default naming.**
- **I did not add `--dry-run`**, which was the other half of this deferred item and would let a lab
  see the sheet before writing it. **Still absent.**
- **`lint`/`prlint` is untouched.** The parser port remains the next real unit, and **this unit does
  not advance it** — it closes a deferred safety gap instead, which is worth saying plainly because
  the objective's own order put `prlint` next.
