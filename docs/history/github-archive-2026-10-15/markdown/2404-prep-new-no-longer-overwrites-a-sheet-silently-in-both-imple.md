# #2404 — prep new no longer overwrites a sheet silently, in both implementations (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T00:56:49Z
- **Updated:** 2026-10-14T01:01:06Z
- **Closed:** 2026-10-14T01:00:38Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2404
- **Merged:** 2026-10-14T01:00:38Z
- **Merge commit:** a52fbd9033ac0380ed499e19b1005b742ce9a0eb
- **Base / Head:** main / UOR-Foundation:deepseek/parse-pr-body
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**`prep new` no longer overwrites a sheet silently.** Tooling only.

Decision: **KEEP** (tooling).

## The defect

`prep new` wrote its sheet with **no existence check at all**:

```python
target.parent.mkdir(parents=True, exist_ok=True)
target.write_text(text, encoding="utf-8")
```

**A pre-registration sheet is the D22 gate's own record.** Re-running `prep new` for a milestone that already had one **replaced the bar, the panel and the cost pre-registered *before* the work** — precisely the evidence D22 exists to preserve. **The command that creates the record was the one thing able to destroy it.**

## Fixed in both implementations

A second write is **refused**, naming the file and what it holds, **with the sheet left untouched**; replacing is explicit:

```
second write:  docs/harness/prep/m1-no-mechanism-2026-10-13.md already exists and was NOT overwritten.
               It is the pre-registered bar, panel and cost for this milestone; replacing it
               discards the record that D22 scores the work against.
               Re-run with --force to replace it.
```

**The refusal is asserted to leave the file byte-identical** — a guard that reports refusal and writes anyway would be worse than none.

**Rust had the same defect and got the same guard**, verified by running both against a scratch repo: **refused / accepted with `--force` / sheet unchanged — identical in both.**

## A mistake worth recording

**Adding `--force` to Rust's flag lists broke the build**, and my first repair asserted on the wrong line content:

- I inserted it **twice** — my second anchor matched my first insert;
- `SUBCOMMAND_FLAGS: [&str; 31]` no longer matched its contents, so the length had to change;
- and my fix script asserted **eight** spaces where the line had **four**, found no duplicate, and raised.

**Three wrong assumptions about my own edit in one small change — and `cargo` caught the one that mattered while my own checks caught none of them.** That is the **tenth** silent or wrong-targeted edit this session.

```
cargo test -p uor-r4-harness                    -> 327 passed
cargo fmt --check / clippy -D warnings          -> clean / 0 findings
python3 -m unittest discover -s tests/harness   -> 332 passed (2 new)
parity.py                                       -> exit 0
python3 scripts/check_no_softmax.py             -> OK, no softmax added
```

Record: `docs/labs/prep-new-overwrite-guard-2026-10-14/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried, presence asserted before commit
- [x] Branch deleted after merge

### Stated rather than hidden

- **`--force` is now the only way to re-render a sheet, and the sheet is not versioned** — a deliberate replacement leaves no trace of the previous bar. **Refusing is the improvement; it does not make replacement safe.**
- **The guard is by path.** A sheet written under `--write <other path>` is not guarded, **so the protection is only as good as the default naming.**
- **I did not add `--dry-run`** — the other half of this deferred item, which would let a lab see the sheet before writing it. **Still absent.**
- **`lint`/`prlint` is untouched. This unit does not advance the parser port**, which the objective's own order put next — **worth saying plainly rather than letting the merge imply progress it did not make.**

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

