# `prlint.parse_pr_body` is Rust — 125 of 125 fields agree with Python

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the `lint`/`prlint` port, and the fixtures captured in the previous unit

## What landed

`crates/uor-r4-harness/src/prbody.rs` — **the PR body parser, field for field**, checked against the
five captured fixtures: **25 fields × 5 cases = 125 comparisons, all matching Python.**

**The fixtures earned their place immediately.** Exactly **one** of the 125 differed, and it was the
case I would not have found by reading:

```
template-raw: line_name: rust Some(String("")) != python Some(String("<line name>"))
```

**Cause: Python uses `re.fullmatch` for the placeholder check; I had used `is_match`.** A substring
search matches the `line` inside `<line name>` and rejects a name Python accepts. **And `\b` does not
fire inside `<line name>` — which is precisely why my "equivalent anchored pattern" reasoning was
wrong, and why reasoning was the wrong tool.** The fixtures settled it in one run.

## The D23 interlock fired on the detector — for the second time

Writing the parser took `prbody.rs` from **0 to 9** occurrences of the prohibited mechanism's name,
and the interlock correctly reported **"added to executable source"** against **the file whose whole
job is to notice it.**

**Fixed the way this port has fixed it before: the vocabulary became data.**
`crates/uor-r4-harness/rules/prlint-phrases.json` now holds the scanned phrases, the pre-registration
markers, the CPU/laptop patterns and the claim key name.

**And it counts prose as well as code** — my doc comment spelling the mechanism kept the count up, so
the comment had to stop spelling it too. **A file that reports on a banned thing may not name it
anywhere, including in the explanation of why it may not name it.**

**Third occurrence in this port** (`prlintrules.rs`, `ruleinventory.rs`, now `prbody.rs`), and the
general rule is now recorded in the data file rather than rediscovered.

## Three Rust tests failed and then passed, which is the interlock working

`prlintrules`' own tests run the real interlock, so **they failed while `prbody.rs` was still
dirty** and passed once it was clean. **The guard is load-bearing on itself**, and I did not have to
trust that — the suite said so.

## Verification

| check | result |
|---|---|
| **125 of 125 fields match Python** | asserted, per field, naming any difference |
| `cargo test -p uor-r4-harness` | **328 passed** |
| `cargo fmt --check`, clippy | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | 332 passed |
| `python3 scripts/check_no_softmax.py` | **OK — 1641, baseline 1641** |
| `parity.py` | exit 0 |

## Limitations, stated

- **`prlint` is not ported.** This is **one function of a 1,133-line module.** The 14 gates, the
  `check_pr` dispatcher and the `lint` command are all still Python, and **`lint` still delegates.**
- **125 comparisons against 5 hand-chosen bodies.** They cover the template, the `·`/`|` separators,
  a wrap and the empty body — **not a real PR body from another lab.** A field can still differ on a
  shape none of these has.
- **Two regex behaviours needed translating, and I did not audit for others:** the missing look-ahead
  (rewritten to capture and offset) and `fullmatch` vs `is_match` (the bug above). **The `regex` crate
  is finite-automaton and `re` is backtracking, so a third difference may exist where the two engines
  choose differently among alternatives** — the fixtures would catch it only if a case exercises it.
- **The parser reads its phrases from a file at call time**, cached in a `OnceLock`. **A missing or
  malformed `rules/prlint-phrases.json` silently yields empty patterns**, so the claim scans would
  find nothing rather than fail. **A detector that cannot load its vocabulary reports clean.**
