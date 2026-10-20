# `team roster` is Rust, and the two recorded findings it corrected

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the first of the six remaining items on the measured scope (#2028)

## What landed

**`uor-harness team roster` runs in Rust, and its output is BYTE-IDENTICAL to Python's.**

| form | bytes | comparison |
|---|---|---|
| `team roster` | 1,936 both sides | `diff` empty |
| `team roster --json` | 6,017 both sides | `diff` empty |

The `--json` form matched only after a fix — see finding 2 below. The comparison is also a test:
`registries::role_tests::roster_matches_python_byte_for_byte` runs the Python harness and diffs both
forms. It is `#[ignore]`d, so CI does not depend on a Python checkout; run it with
`cargo test -p uor-r4-harness -- --ignored roster_matches_python`.

**The previous attempt at this command was reverted, and this one reuses its record rather than its
conclusions.** `docs/labs/roster-port-findings-2026-10-14/README.md` gave the roles shape, the
`(inherit)` route and the column widths. Two of its three findings needed correcting, and both
corrections came from comparing output rather than from reading code.

## Correction 1: the header has ONE newline, not a blank line

The findings record says Python's `f"{n} dispatchable roles (…)\n"` through a writer that adds the
line's own newline "gives one blank line", and the port therefore produced two. **Measured:**

```
$ python3 scripts/harness/cli.py team roster | head -c 80
b'9 dispatchable roles (docs/harness/roles.json)\n  architecture-reviewer    planne'
```

Python's `emit` (`scripts/harness/core.py:411`) writes the text as-is when it already ends in `\n`,
so the string's own `\n` IS the line terminator. **The recorded finding was wrong about Python**, and
a port that trusted it would have shipped a blank line. This is the port's own rule applied to its
own record: the bytes are what a lab diffs.

## Correction 2: `json.dumps` escapes non-ASCII and `serde_json` does not

**Found by the diff, not by reading either implementation.** After the text form matched byte for
byte, the JSON differed in exactly 12 bytes: three role purposes contain an em dash.

```
python: "Is this run admissible at all \u2014 right machine, within the owner caps, …"
rust:   "Is this run admissible at all — right machine, within the owner caps, …"
```

`json.dumps` defaults to `ensure_ascii=True`; `serde_json` always emits UTF-8. Same value, different
bytes, and **it only shows up in the records that happen to carry a non-ASCII character** — which
here is three of nine roles. `ensure_ascii` now escapes the document, including astral characters as
a surrogate pair, exactly as Python writes them. Applying it to the whole document is safe because
JSON's structure is ASCII: every non-ASCII character in the output is inside a string literal.

**This may not be confined to `roster`.** Any ported `--json` path whose data carries non-ASCII has
the same divergence unless its own comparison happened to cover such a value. **Not measured here**,
and stated rather than left to be rediscovered.

## The order, which was the reason the first attempt was reverted

`docs/harness/roles.json` is an object keyed by role id, and Python prints the roles in **file order**
(`architecture-reviewer, experiment-designer, test-methodologist, systems-cost, …`). `serde_json`'s
default `Value` is a `BTreeMap`, so reading the file as a `Value` sorts them — the previous attempt
printed an alphabetical roster, which is a wrong answer that reads like a right one.

The loader visits the map and collects entries in **document order** (`OrderedObject`), and the
`--json` form is serialised from a real struct rather than through `serde_json::json!` for the same
reason: the macro routes values through `to_value`, which is the same `BTreeMap`, and the field order
inside each role would have been sorted too.

`the_real_registry_loads_in_document_order` checks the loaded order against the **file text** — each
id must appear in the document in the order the loader returned it — so the test is a property of the
loader, not a remembered list.

## What the port keeps, deliberately

* **`(inherit)`** for a role with no provider: a blank column would read as a missing value rather
  than as a decision to take the caller's route.
* **`title` falls back to the id, `seat` to `specialist`, `purpose` to empty.** An ABSENT key takes
  the default; an explicit `null` is `str(None)`, which is `None` — a distinction two tests pin.
* **The purpose is cut at 100 CHARACTERS**, with a test whose 100th character is an em dash, because
  a byte slice would panic or split one in half.
* **A missing `roles.json` is an empty roster; a broken one is an ERROR.** `0 dispatchable roles` is
  precisely the wrong answer that looks right, and Python raises rather than printing it.

**Limit, stated:** the ported `str()` covers the scalar JSON types, and would render a list or object
in a text field as JSON text rather than Python's `repr`. `the_real_registry_keeps_its_text_fields_scalar`
asserts on the real file that every text field is a string, so the limit cannot be crossed silently —
but it is an assertion about a file other people edit, so it fails loudly rather than silently if the
schema changes.

## Found while verifying, NOT fixed here: `--repo` makes `check` run ZERO rules

**This is not part of the roster port, it is a defect found while exercising the binary, and it is
live on `main` — the pre-change binary reproduces it exactly.**

```
$ uor-harness check --json            | grep '"checked"'   ->  "checked": 52
$ uor-harness --repo $PWD check --json | grep '"checked"'  ->  "checked": 0
$ uor-harness --repo $PWD check
0 rules, 0 failed (0 error, 0 warn)

$ python3 scripts/harness/cli.py --repo $PWD --json check  ->  checked 59, failed 20

controls:  check --only arena  -> checked 4
           check --only zzz    -> checked 0
```

**`--repo` is forwarded into the subcommand's own argument list** (`main.rs` pushes it into `rest`
deliberately, because `receipt` needs to know where it is writing), and `cmd_check_rules` reads that
same list as its `--only` **filter**. So `--repo <path>` is taken as a filter that matches no rule id,
and the command reports a clean result having checked nothing. The two controls show the mechanism: a
filter matching four ids checks four rules, and a filter matching none checks none.

**This is the failure this port has spent four units on, one level up from `cmd_check`.** The earlier
correction — *"`check` runs the format contract, not the rule set"* — was fixed by making `check` call
the rule set; **this one makes the rule set disappear on an invocation a lab would reasonably type**, and
the output is indistinguishable from a repository with nothing wrong in it. `parity.py` does not catch it
because it does not invoke `check` that way.

**Not fixed in this PR**, because the fix is in `check`'s argument handling rather than in `roster`, and
this port's units are single-purpose with their own evidence. **It is the first thing the next unit
should do**, before more porting: every later unit's verification runs through `check`, and a `check` that
can report zero rules is not a gate.
## Verification

| check | result |
|---|---|
| `team roster` vs Python | **byte-identical, 1,936 = 1,936** |
| `team roster --json` vs Python | **byte-identical, 6,017 = 6,017** |
| live diff test (`--ignored`) | **passes** |
| `cargo test -p uor-r4-harness` | **425 passed, 0 failed, 5 ignored** (was 411/4: 14 new tests, 1 ignored) |
| `cargo fmt --check`, clippy `-D warnings` | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | exit 0 |
| `python3 scripts/harness/parity.py` | **exit 0** |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |
| `delegate_to_python` sites | **5 — unchanged in count; line 1582 is now `dispatch` alone** |

**The site count does not move because `team dispatch` still delegates, and it is not wired
deliberately:** `dispatch` also WRITES a receipt through the ledger shard writer, so porting it means
porting `receipts.record` with its redaction, its lab attribution and its unattributed warning.
Wiring the read half while the write half stays in Python is the half-ported command this port keeps
refusing to ship. **The objective's remaining scope is five items: `team dispatch`, `mistakes
show|capture`, `multilab sync-live|migrate-ledger`, and `arena`/`budget`/`panel` behind the general
fallback.**

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** **the `--repo` / `check` defect above first** — `uor-harness --repo DIR check` must run the
rule set Python runs, not zero rules, because every later unit's verification passes through it. Then
`team dispatch` — port `receipts.record` (the shard path, redaction, lab attribution and the unattributed
warning) and compare the written receipt AND the shard file against Python's; then `mistakes
show|capture`, `multilab sync-live|migrate-ledger`, and `arena`/`budget`/`panel`.
