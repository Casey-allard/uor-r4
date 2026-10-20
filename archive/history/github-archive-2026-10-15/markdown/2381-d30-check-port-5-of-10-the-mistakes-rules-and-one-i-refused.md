# #2381 — D30 check-port 5 of 10: the mistakes rules, and one I refused to ship (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T21:07:34Z
- **Updated:** 2026-10-13T21:31:07Z
- **Closed:** 2026-10-13T21:30:49Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2381
- **Merged:** 2026-10-13T21:30:49Z
- **Merge commit:** 757a634acb7bc7c8259d5ff3d18bcdb7109c95c7
- **Base / Head:** main / UOR-Foundation:deepseek/mistakes-rules
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**D30 check-port 5 of 10.** Tooling only.

Decision: **KEEP** (tooling).

## The two rules in this unit

| rule | severity | Python | Rust |
|---|---|---|---|
| `mistakes.registry_valid` | error | all pass | 0 findings |
| `mistakes.evidence_exists` | error | all pass | 0 findings |

Coverage reads **18 of 40 rules** (harness 3, mistakes 2, multilab 6, prep 5, status 2).

## The validation layer the Rust parser never had

`mistakes.registry_valid` **could not be ported**, because **the Rust parser was permissive and never rejected anything** — there was nothing for the rule to report. This adds `validation_errors()` and an `errors` field mirroring Python: mandatory front matter keys, id must equal the file stem, kebab-case id, the `severity`/`class`/`status` enumerations, `YYYY-MM-DD` dates, a dotted or `none:` guard.

**Verified against the real registry: 0 errors** — the validator **agrees with Python rather than being stricter**, which is the property that matters, since **a rule that cries wolf on a valid registry gets switched off.**

**Two failures kept apart:** bad front matter, and **a duplicate id** — which makes the registry ambiguous, because a lookup by id returns whichever file was read first.

## One rule I built, measured, found wrong, and removed

`mistakes.guard_covered` checks a **block**-severity lesson names a guard rule that exists. I implemented it against **`ruleinventory::all_ids()` — 16 ids, what the *port* has implemented.** **Python checks what the *engine* registers: 40.**

A lesson guarding on `multilab.claim_not_merged` — a rule that exists and is enforced — was reported as *"not a rule this engine registers"*. **Rust said 4; Python said 0.**

**The two sets are different questions with the same shape, and I reached for the wrong one because it was the one I had just built.** A guard check against an incomplete port is **not a weaker check, it is a wrong one** — it would tell authors to delete guards on real rules, and it would stop being wrong only when the port finished, **exactly when the check stops mattering.**

**Removed rather than shipped approximate, with the reasoning left in its place.** A stub returning nothing would be **the silent-clean failure this line keeps producing.**

```
cargo fmt --check                               -> clean
cargo clippy --all-targets -- -D warnings       -> 0 findings
cargo test -p uor-r4-harness                    -> 255 passed
python3 -m unittest discover -s tests/harness   -> 319 passed
parity.py                                       -> exit 0, rule inventory: 18 implemented
```

Record: `docs/labs/port-check-mistakes-2026-10-13/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried
- [x] Branch deleted after merge

### Stated rather than hidden

- **Two `mistakes` rules remain unported:** `guard_covered` (above) and `stale_check` (`info` severity; needs the repo file walk — **not attempted**).
- **`check` still delegates.** 5 of 10 modules.
- **The validation layer is new code with a new failure mode:** a lesson Python accepts and Rust rejects is now an **error**, not a warning. Measured clean on the real registry, but **no test compares the two validators across malformed files** — only the real registry and hand-made fixtures.
- **The agreement measurement is a hand-run probe** — **and it is what caught the `guard_covered` bug.** Nothing in CI compares per-rule counts, so the next such bug is caught only if someone runs the probe again.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

