# #2406 — prlint.parse_pr_body is Rust, 125 of 125 fields matching Python (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T01:08:58Z
- **Updated:** 2026-10-14T01:11:26Z
- **Closed:** 2026-10-14T01:11:00Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2406
- **Merged:** 2026-10-14T01:11:00Z
- **Merge commit:** 3b84fbb3f9a67ffd1d3579e58caf794579c8260e
- **Base / Head:** main / UOR-Foundation:deepseek/pr-body-parser
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**`prlint.parse_pr_body` is Rust — 125 of 125 fields agree with Python.** Tooling only.

Decision: **KEEP** (tooling).

## What landed

`crates/uor-r4-harness/src/prbody.rs` — **the PR body parser, field for field**, checked against the five fixtures captured last unit: **25 fields × 5 cases = 125 comparisons, all matching.**

## The fixtures earned their place immediately

**Exactly one of the 125 differed**, and it was the case reading would not have found:

```
template-raw: line_name: rust Some(String("")) != python Some(String("<line name>"))
```

**Cause: Python uses `re.fullmatch`; I used `is_match`.** A substring search matches the `line` inside `<line name>` and rejects a name Python accepts. **And `\b` does not fire inside `<line name>` — which is precisely why my "equivalent anchored pattern" reasoning was wrong, and why reasoning was the wrong tool.** The fixtures settled it in one run.

Also translated: **`regex` has no look-ahead**, so the label-cut pattern captures the whitespace and offsets to its start.

## The D23 interlock fired on the detector — third time in this port

Writing the parser took `prbody.rs` from **0 to 9** occurrences of the prohibited mechanism's name, and the interlock correctly reported **"added to executable source"** against **the file whose whole job is to notice it.**

**Fixed the way this port has fixed it before: the vocabulary became data** (`rules/prlint-phrases.json` — the scanned phrases, the pre-registration markers, the CPU/laptop patterns, and the claim key name).

**It counts prose as well as code** — my doc comment spelling the mechanism kept the count up, so the comment had to stop spelling it too. **A file that reports on a banned thing may not name it anywhere, including in the explanation of why it may not name it.**

**Three Rust tests failed and then passed, which is the interlock working**: `prlintrules`' own tests run the real interlock, so they failed while `prbody.rs` was dirty and passed once it was clean.

```
cargo test -p uor-r4-harness                    -> 328 passed
cargo fmt --check / clippy -D warnings          -> clean / 0 findings
python3 -m unittest discover -s tests/harness   -> 332 passed
python3 scripts/check_no_softmax.py             -> OK — 1641, baseline 1641
parity.py                                       -> exit 0
```

Record: `docs/labs/pr-body-parser-port-2026-10-14/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried, presence asserted before commit
- [x] Branch deleted after merge

### Stated rather than hidden

- **`prlint` is not ported. This is one function of a 1,133-line module.** The 14 gates, the `check_pr` dispatcher and the `lint` command are still Python, and **`lint` still delegates.**
- **125 comparisons against 5 hand-chosen bodies** — the template, the separators, a wrap and the empty body, **not a real PR body from another lab.** A field can still differ on a shape none of these has.
- **Two regex behaviours needed translating and I did not audit for others.** **`regex` is finite-automaton and `re` is backtracking, so a third difference may exist where the engines choose differently among alternatives** — the fixtures catch it only if a case exercises it.
- **A missing or malformed `rules/prlint-phrases.json` silently yields empty patterns**, so the claim scans find nothing rather than fail. **A detector that cannot load its vocabulary reports clean.**

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

