# D30 check-port 6 of 10: the `arena` rules

**Date:** 2026-10-13 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the D30 order on #2028; `check` module 6 of 10

## All four `arena` rules, exact agreement

| rule | severity | Python | Rust |
|---|---|---|---|
| `arena.rows_valid` | error | all pass | 0 findings |
| `arena.parents_exist` | error | all pass | 0 findings |
| `arena.decisive_power` | warn | all pass | 0 findings |
| `arena.knows_what_it_cannot_measure` | warn | all pass | 0 findings |

Coverage reads **22 of 40 rules** (arena 4, harness 3, mistakes 2, multilab 6, prep 5, status 2).

## The two that carry the weight

**`decisive_power` is D22 as a check.** A row claiming `decisive` confidence with fewer than **2 seeds
or 200 fresh rows** is not decisive — it is a provisional reading wearing a decisive label, and **the
label is what a reader trusts.** The rule cannot tell whether the run happened; it can tell whether the
claim is *entitled*, which is the part a machine can decide.

**`knows_what_it_cannot_measure` is the one that ages best.** A row list with no stated limits **reads
as complete** — a reader sees five candidates and concludes the arena covers the space. The `limits`
string is what stops that inference, so its absence is a warning, not a note.

**Forty characters, because a shorter one is a gesture.** `"none"`, `"see docs"` and `"TBD"` all satisfy
"there is a limits field" while stating nothing, and the rule exists to stop the arena being read as
complete — so it has to require a sentence. All four are tested as rejections.

## Three decisions worth naming

**An absent arena is reported, not passed.** `load` returns an empty arena for a missing or unparseable
file, which would make every rule report nothing — **and silence reads as a pass.** `rows_valid` reports
it as its own case: *"missing, unparseable, or has no rows — nothing could be checked."*

**The duplicate-id check runs first, because it invalidates everything after it.** With two rows of one
id, every lookup by id resolves to whichever row was read first, so the rest of the arena's structure
describes a file that does not exist.

**A cycle is reported once per entry point** — the Python's behaviour, kept. Reporting it once would need
a visited set across the whole walk and would hide the second row leading into the cycle, **which is the
row a reader has to fix.** The walk is explicitly tested for termination, since a cycle is the case a
naive parent walk hangs on.

## Verification

| check | result |
|---|---|
| Rust vs Python, all four rules | **identical — all pass in both** |
| the real arena passes every rule | asserted by test |
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | 0 findings |
| `cargo test -p uor-r4-harness` | **267 passed** (12 new) |
| `python3 -m unittest discover -s tests/harness -t .` | 319 passed |
| `parity.py` | exit 0, **22 implemented** |

## Limitations, stated

- **`check` still delegates.** 6 of 10 modules.
- **`rows_valid` reads `candidates` only.** If the arena ever renames that key, the rule sees an empty
  arena and reports "missing or has no rows" — **wrong in the loud direction, but wrong.** Python has the
  same coupling; the port agrees with it.
- **`limits` is measured in characters, not meaning.** A forty-character string of filler passes. **The
  rule can require someone to write something; it cannot require them to mean it.**
- **`decisive_power` trusts the row's own `seeds` and `rows` fields.** A row claiming 200 rows that were
  never run passes. **The rule checks the claim's arithmetic, not its truth** — the same boundary every
  rule in this set has.
- **The agreement measurement is a hand-run probe**, read and removed. **Nothing in CI compares per-rule
  finding counts**, and this probe has now caught one real bug (`guard_covered`, last unit).
