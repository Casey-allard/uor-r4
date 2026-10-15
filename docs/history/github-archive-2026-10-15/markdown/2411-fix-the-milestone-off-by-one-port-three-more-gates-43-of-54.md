# #2411 — Fix the milestone off-by-one; port three more gates (43 of 54) (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T01:49:08Z
- **Updated:** 2026-10-14T01:51:55Z
- **Closed:** 2026-10-14T01:51:31Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2411
- **Merged:** 2026-10-14T01:51:31Z
- **Merge commit:** 3e38881a2ae423097f261c6cddccea2e9349c479
- **Base / Head:** main / UOR-Foundation:deepseek/state-cpu-gates
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**The newest milestone was silently dropped — an off-by-one the fixtures could not catch.** Tooling only.

Decision: **KEEP** (tooling).

## The bug

Python's milestone set is **`range(2029, 2037)`** — inclusive of **2036**. The ported parser wrote:

```rust
const MILESTONE_HIGH: i64 = 2036;
.filter(|i| (MILESTONE_LOW..MILESTONE_HIGH).contains(i))
```

**A half-open Rust range ends one short.** So **`#2036` — the newest milestone — was never recognised as a milestone.**

**It still produced a `milestone_ref` through the fallback, so every fixture agreed**: the field was populated, **just from the wrong branch, and nothing compared the branch.**

**Found by probing, not by reading** — I was debugging why `current_state_entry` named the topic instead of the issue, printed `milestone_issue`, and saw `None` where Python saw `2036`.

## Why five fixtures could not catch it

**None of the five captured bodies cites `#2036`** — they cite `#2028` (the tracker, outside the range) and `#2029`.

**The fixtures record what the parser *returns*; they do not enumerate what it could be *asked*.** That is the limit of a behaviour capture — and why **a sixth case now exists**, with a test pinning **both ends and the value just outside**:

```
#2029 -> Some(2029)     the first
#2036 -> Some(2036)     the last, inclusive
#2037 -> None           beyond, but still the ref the body names
```

## Three more gates

| gate | severity | condition |
|---|---|---|
| `prlint.prereg_before_compute` | error | model numbers with no pre-registration reference |
| `prlint.no_cpu_model_compute` | warn | CPU or host-fallback compute presented as a result |
| `prlint.current_state_entry` | warn | a claimed current-state entry the file does not support |

**Coverage 43 of 54** — **11 prlint gates in Rust.**

**`prereg_before_compute` is the gate I tripped four times writing this port**, and the test pins why: it fires on any prose line with a **run-word and a digit**. **I did not loosen it** — narrowing a fuzzy gate doing a real job would be a behaviour change dressed as a port.

**`current_state_entry` has three branches, and the issue is named when there is one** even though the *topic* failed to match — I wrote that precedence backwards first and my test caught it.

```
cargo test -p uor-r4-harness                    -> 351 passed
cargo fmt --check / clippy -D warnings          -> clean / 0 findings
python3 -m unittest discover -s tests/harness   -> 332 passed
python3 scripts/check_no_softmax.py             -> OK — 1641, baseline 1641
parity.py                                       -> exit 0, 43 implemented
```

Record: `docs/labs/milestone-off-by-one-2026-10-14/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried, presence asserted before commit
- [x] Branch deleted after merge

### Stated rather than hidden

- **The off-by-one survived because the fallback masked it.** `milestone_ref` was still populated, so **the fixture agreed on a field whose value came from the wrong code path — a field being right does not mean the branch that set it was.**
- **`check_pr` is still not ported. Three of fourteen gates remain** (`parse`, `checks_at_head`, the org rules), plus the dispatcher and the `lint` command. **`lint` still delegates, and none of the eleven ported gates is reachable from any command.**
- **This unit took far more edits than it should have.** Repeated string-replace edits against text `cargo fmt` had already reformatted failed silently several times, and I mangled the test function badly enough to rewrite it. **The line-range approach worked every time; the string-replace approach failed most of the time, and I kept reaching for it.**
- **`prereg_before_compute` has a real false-positive rate** — I hit it four times on legitimate bodies. **Ported as-is, so the port carries that rate rather than hiding it.**

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

