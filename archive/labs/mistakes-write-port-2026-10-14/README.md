# `mistakes show|capture` are Rust — and `capture` was refusing its own output

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the third of the six remaining items on the measured scope (#2028)

## What landed

**`uor-harness mistakes show` and `uor-harness mistakes capture` run in Rust, and the `mistakes`
delegation site is GONE: live `delegate_to_python` call sites 3 → 2** (the general fallback and
`multilab sync-live|migrate-ledger`).

`show` reads one lesson body; `capture` writes one. **`capture` is the writer:** it validates the
draft against the same rules `mistakes.registry_valid` applies, refuses to overwrite without
`--force`, and reads the file back — removing it if it somehow fails — so a capture can never leave
behind a lesson that turns the lab's own `check` red.

## Three defects the comparison found, all in the port, none visible by reading

### 1. `capture` refused every lesson with an auto-generated guard

```
refusing to write …/demo-lesson.md: the draft is not a valid lesson
  demo-lesson.md: guard "\"none: captured by hand on 2026-10-14; no rule decides it yet\"" must be
  a dotted rule id or 'none: <reason>'
```

**Rust's front-matter reader did not strip quotes.** `build_lesson_text` writes the default guard
quoted (it contains a colon), Python's YAML subset strips the quotes, and this reader kept them — so
the guard began with a `"` instead of `none`, failed the guard rule, and **the command could not
write the lesson it had just generated.** Fixed by porting `core._coerce`'s one relevant case.

### 2. The success message was three lines short

Python prints the write line **and** the dispatch guidance:

```
wrote …/demo-lesson.md
  severity warn, class process, guard none: captured by hand on 2026-10-14; no rule decides it yet
  read it back: python3 scripts/harness/cli.py mistakes show demo-lesson
  check: python3 scripts/harness/cli.py check --only mistakes
```

**The port printed only the first line**, because `capture` had been read as far as the write. The
stdout comparison is what showed it — reading the function would not have.

### 3. Every front-matter value goes through the YAML renderer, not only the awkward ones

Python writes `f"{key}: {_yaml(fm[key])}"` for all twelve keys. Wrapping only the ones with
punctuation in them meant a **title with a space** was written `title: A demo lesson` where Python
writes `title: "A demo lesson"` — a difference in every captured lesson, found by diffing the
written file.

## Two defects in the rule the draft is validated against

**`mistakes.registry_valid` was missing three checks Python has**, and used Rust's debug formatting
where Python uses `!r` and tuple reprs:

* `'trigger' needs at least two phrases, or the lesson is not retrievable`
* `'paths' is empty: name a glob so the lesson is recalled`
* `'<key>' must be an inline or block list`
* `severity must be one of ('block', 'warn', 'info'), got 'nope'` — Python's tuple, not Rust's
  `["block", "warn", "info"]`; and `id 'x'` rather than `id "x"` throughout.

**These were invisible because Python's registry loader refuses a lesson with a bad id or severity
before the rule runs**, so the only way to compare was a lesson that loads and is still wrong. With
one — valid id and severity, everything else wrong — **both implementations now report the same eight
findings, in the same order, with the same messages.**

## Verified: both implementations, scratch checkouts

Twelve case groups, each comparing **stdout, stderr, exit code AND every file in the mistakes
directory**: `show` exact / prefix / unknown / missing id; `capture` minimal (writes), full flags
(writes), then-exists, then-`--force`, single trigger, bad id, missing evidence, missing title.

**All match except one message**: with a required flag absent, Python's argparse prints a usage block
and this port prints `mistakes capture needs --id and --title`. Both refuse with exit 2.

| check | result |
|---|---|
| `cargo test -p uor-r4-harness` | **445 passed, 0 failed, 6 ignored** (was 438: 7 new tests) |
| `cargo fmt --check`, clippy `-D warnings` | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | exit 0, 0 failure lines |
| `python3 scripts/harness/parity.py` | **exit 0** |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |
| `check` rule-id set on the real tree | **41 = 41**, no new findings from the added checks |
| live `delegate_to_python` call sites | **2 — the `mistakes` site is gone** |

## Found while comparing, NOT fixed here

**`check`'s TEXT rendering is not Python's.** Same verdicts, different layout: Python prints
`harness check: 8 rules, 8 failed (8 error, 0 warn)` then `FAIL <rule> [<path>]: <message>` with a
trailing hint line; this port prints `8 rules, 8 failed (8 error, 0 warn)` then `  error <rule>
[<path>]` with the message on its own line and no hint. **It surfaced only because comparing the
rule messages required reading both renderings side by side**, and it is the same class as the ledger
line: identical verdicts, different bytes, invisible to every JSON comparison. It is a `check` unit of
its own, not a `mistakes` one.

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** `multilab sync-live|migrate-ledger` then `arena`/`budget`/`panel` (which removes the general
fallback), plus the recorded follow-ups: `check`'s text rendering, `status.no_orphan_branches`
reporting `ok` where Python warns, and the receipt `ts` rendering.
