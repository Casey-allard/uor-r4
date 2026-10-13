# D30 check-port 5 of 10: the `mistakes` rules, and one I refused to ship

**Date:** 2026-10-13 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the D30 order on #2028; `check` module 5 of 10

## The two rules in this unit

| rule | severity | Python | Rust |
|---|---|---|---|
| `mistakes.registry_valid` | error | all pass | 0 findings |
| `mistakes.evidence_exists` | error | all pass | 0 findings |

Coverage reads **18 of 40 rules** (harness 3, mistakes 2, multilab 6, prep 5, status 2).

## The validation layer the Rust parser never had

`mistakes.registry_valid` could not be ported, because **the Rust parser was permissive and never
rejected anything** — so there was nothing for the rule to report. This unit adds
`validation_errors()` and an `errors: Vec<String>` field, mirroring the Python reader: mandatory front
matter keys, id must equal the file stem, kebab-case id, the `severity`/`class`/`status` enumerations,
`YYYY-MM-DD` dates, and a guard that is dotted or `none:`.

**Verified against the real registry: 0 errors** — the validator agrees with Python rather than being
stricter, which is the property that matters. A stricter validator would have reported findings Python
does not, and a rule that cries wolf on a valid registry gets switched off.

**Two failures kept apart:** a file whose front matter is wrong, and **a duplicate id** — which makes the
registry ambiguous, since a lookup by id returns whichever file was read first. Different faults,
different fixes.

## One rule I built, measured, found wrong, and removed

`mistakes.guard_covered` checks that a **block**-severity lesson names a guard rule that exists. I
implemented it against `ruleinventory::all_ids()` — **16 ids, the rules the PORT has implemented.**

**Python checks against the rules the ENGINE registers: 40.** So a lesson guarding on
`multilab.claim_not_merged` — a rule that exists and runs — was reported as *"not a rule this engine
registers"*. **Rust said 4; Python said 0.**

**The two sets are different questions with the same shape, and I reached for the wrong one because it
was the one I had just built.** A guard check against an incomplete port is **not a weaker check, it is a
wrong one**: it would tell authors to delete guards on real rules, and it would stop being wrong only
when the port finished — **exactly when the check stops mattering.**

**I removed the function rather than ship it approximate, and left the reasoning in its place.** A stub
that returns nothing would be the silent-clean failure this line keeps producing; a caller that needs it
must not find one. Porting it needs the full registered-id set — enumerated in Rust or embedded as data
beside the patterns file — which is its own small unit.

## Verification

| check | result |
|---|---|
| Rust vs Python, `registry_valid` + `evidence_exists` | **0 = all pass for both** |
| Rust vs Python, `guard_covered` | **4 vs 0 — the bug, not shipped** |
| real registry validation errors | **0**, agreeing with Python |
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | 0 findings |
| `cargo test -p uor-r4-harness` | **255 passed** |
| `python3 -m unittest discover -s tests/harness -t .` | 319 passed |
| `parity.py` | exit 0, **18 implemented** |

## Limitations, stated

- **Two `mistakes` rules remain unported**: `guard_covered` (above) and `stale_check` (does each
  lesson's glob still match a file). **`stale_check` is `info` severity** and needs the repo file walk;
  it was not attempted.
- **`check` still delegates.** 5 of 10 modules.
- **The validation layer is new code with a new failure mode:** a lesson that Python accepts and Rust
  rejects would now be an **error**, not a warning. Measured clean on the real registry, but **no test
  compares the two validators across a corpus of malformed files** — only the real registry and hand-made
  fixtures.
- **The agreement measurement is a hand-run probe**, read and removed — **and it is what caught the
  `guard_covered` bug.** Nothing in CI compares per-rule counts, so the next such bug will be caught
  only if someone runs the probe again.
