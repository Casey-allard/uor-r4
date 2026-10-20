# D30 check-port 7 of 10: the `budget` rules

**Date:** 2026-10-13 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the D30 order on #2028; `check` module 7 of 10

## The rules in this unit

| rule | severity | Python | Rust |
|---|---|---|---|
| `budget.receipts_parse` | error | all pass | 0 findings |
| `budget.disk_floor` | warn | all pass | 0 findings |
| `budget.no_new_spending_class` | info | — | **not ported, see below** |

Coverage reads **24 of 40 rules** (arena 4, budget 2, harness 3, mistakes 2, multilab 6, prep 5,
status 2).

## Why `no_new_spending_class` is not ported

**It has no check function.** Its registration is `info` severity and its entire content is that **the
owner caps are printed** so a session cannot silently adopt a new spending class. There is nothing to
decide and nothing to fail — **the rule is a display.**

Porting it here would mean a function that always returns no findings: **a rule that cannot fail,
dressed as a rule.** That is the shape this line keeps flagging — *a guard that stops firing is
indistinguishable from a guard that passes* — and it belongs with the `budget` command's own
rendering, where the caps are actually shown.

## `receipts_parse` protects every other rule's evidence

The ledger is where the record of what happened lives. **A line that will not parse is not a missing
datapoint but a corrupt one**, and a reader who silently skips it draws conclusions from what remains.
So a bad line is an **error**, not a warning.

**Three failures, kept apart because the fixes differ:**

- **not JSON** — the line is corrupt; repair or drop that line;
- **JSON but not an object** — a bare array or scalar, not a receipt at all;
- **an object missing `ts` or `kind`** — parseable and meaningless, **and the dangerous one**, because
  it looks like a record while carrying none of the fields a reader filters on.

**A null key counts as missing.** `{"ts": null}` is not a timestamp, and testing only for absence would
have accepted it.

**The line number is in every message and every fix.** "The ledger has a bad line" is not actionable;
"line 47 of `kimi-2026-10.jsonl`" is.

## `disk_floor` gives a stop condition a machine

**Below 30 GB is a stop condition in the owner's rules.** A stop condition that lives only in prose is
one nobody notices until the disk fills, so this is the check that makes it a machine.

**The measurement is real:** `df -Pk`, POSIX-specified output, reading the fourth column. The
alternative was `libc::statvfs` behind an FFI bind, and **the harness has no `libc` dependency** —
adding one for a read-only disk query is more moving parts than shelling out to a tool on every POSIX
system.

**An unmeasurable disk is a warning, not a pass.** An unmeasurable disk could be any size, and
reporting nothing would read as "fine".

Measured on this machine during the port: **38.9 GB — inside the band.**

## Verification

| check | result |
|---|---|
| Rust vs Python, both rules | **identical — all pass in both** |
| `disk_free_gb` on the real repo | **38.89 GB**, a real measurement |
| `df` unavailable for a bogus path | returns `None`, reported as a warning |
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | 0 findings |
| `cargo test -p uor-r4-harness` | **276 passed** (9 new) |
| `python3 -m unittest discover -s tests/harness -t .` | 319 passed |
| `parity.py` | exit 0, **24 implemented** |

## Limitations, stated

- **`check` still delegates.** 7 of 10 modules.
- **`receipts_parse` reads the committed ledger directory only**, not the live tier. Python does the
  same; the port agrees. A corrupt line in a live-tier shard would not be seen by either.
- **`disk_floor` measures the repo's filesystem, not the machine's free space overall.** On a machine
  where the repo is on a small volume and `/` has room, this reports the small volume — Python uses
  `shutil.disk_usage(paths.repo)` and has the same behaviour. **The port agrees with Python rather than
  with what a reader might assume "free disk" means.**
- **The `df -Pk` column index is positional.** A `df` whose output format differs (a busybox variant,
  a locale that changes the header) would misread. **`-P` is specified to prevent exactly that**, but
  the parse has no fallback if a system ignores it.
- **The agreement measurement is a hand-run probe**, read and removed. **Nothing in CI compares
  per-rule finding counts.**
