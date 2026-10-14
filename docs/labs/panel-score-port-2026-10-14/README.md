# `panel score` is Rust — and the comparison caught two things I had wrong

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the last delegation site (`arena`/`budget`/`panel`), first of its seven subcommands

## What landed

**`uor-harness panel score` runs in Rust.** It is the D22 acceptance-panel check: one measured value
against one frozen bar, answering `PASS` / `FAIL` / `AT NOISE`.

**The unported-command count went 7 → 6; the delegation-SITE count went 1 → 2.** Both numbers are
true and only one of them is progress, so say it plainly: the general fallback used to carry all seven
subcommands and now carries five (`arena` x3, `budget` x2), while `panel show` is delegated by its own
named line. **A site count is not a progress measure here.**

## The two things the comparison caught

**1. I had read only half of `classify`.** The Python function continues past the PASS/FAIL branches:

```python
if band is None and improvement != 0 and needed != 0:
    out["reason"] += " - noise is unstated for this bar, so this cannot be called a real difference (D22 section 3)"
```

**D22 section 3: with no stated noise band there is nothing to compare a movement against**, so the
reason has to say the difference cannot be called real. My first port produced the shorter sentence on
every unstated-noise panel — which is most of them. Reading the function to its end is not optional.

**2. Python quotes the headline bar id and not the alternatives.** `no --bar given: scoring the
headline bar 'complete-correct-512' (also available: fresh-transfer)` — the headline id goes through
`!r`, the list is `', '.join(str(...))`. The first version quoted both.

**3. An ordering bug the comparison also caught:** the `--bar` note was printed *before* `--value` was
validated, so a command that then failed still printed a note about the headline bar. argparse refuses
before any of the logic runs, so the required flags are now checked first.

## Formats that had to be written out

| piece | why it is hand-written |
|---|---|
| `%g` / `%+g` | `150` must not print as `150.0`, and `1e+06` is Python's exponent form |
| `--json` | Python's **dict order**, and `improvement`/`needed` exist only on the numeric path — a renderer that always emitted them would invent two fields |
| `noise_band` | an unreadable band is `None`, never `0`: **a band of zero turns every movement into a real difference** |

## Verified: both implementations, twenty case groups

Run against the repo's real panels (`m2-reply512`), text and `--json`, covering: at target (PASS), at
current, midway, a decimal value, zero, an explicit `--bar`, `--label`, an unknown panel, an unknown
bar, a prefix panel match, and a panel with no bars.

| check | result |
|---|---|
| stdout / stderr / exit on 18 cases | **all MATCH, except two argparse usage blocks** |
| `cargo test -p uor-r4-harness` | **461 passed, 0 failed, 6 ignored** (was 450: 11 new tests) |
| fmt, clippy `-D warnings` | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | exit 0, 0 failure lines |
| `python3 scripts/harness/parity.py` | **exit 0** |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |

## Stated rather than hidden

* **A missing or non-numeric `--value` prints this binary's message, not argparse's usage block**
  (`uor-harness: argument --value is required` vs three lines of usage). The exit code is 2 in both.
  This is the same divergence already recorded for `check` and `git switch`.
* **`panel show` still delegates, on purpose.** `show --json` prints each bar as the panel file's own
  object in the file's key order (`id, label, current, target, of, unit, direction, note`); `serde_json`
  is a `BTreeMap` and there is **no `note` field on `registries::Bar` at all**. Matching those bytes
  needs the order-preserving parse this crate so far has only for `roles.json`. **A `show` that answered
  with the keys re-ordered would look right and diff wrong.**
* **`--value nan` / `inf`**: Python's `json.dumps` writes bare `NaN`/`Infinity`; this renderer does the
  same, and `%g` writes `nan`/`inf` in the text form as Python does.

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** `panel show` (needs the ordered-JSON parse), then `arena` (3 subcommands, `compare` is 91
lines), then `budget` (2 subcommands, 908-line module). The D30 condition is met only when all six are
done and the fallback can be deleted.
