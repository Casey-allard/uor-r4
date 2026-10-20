# The newest milestone was silently dropped — an off-by-one the fixtures could not catch

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** three more `prlint` gates, and the bug they exposed in the parser

## The bug

Python's milestone set is **`range(2029, 2037)`** — inclusive of **2036**. The ported parser wrote:

```rust
const MILESTONE_HIGH: i64 = 2036;
.filter(|i| (MILESTONE_LOW..MILESTONE_HIGH).contains(i))
```

**A half-open Rust range ends one short.** So **`#2036` — the newest milestone — was never
recognised as a milestone.** It still produced a `milestone_ref` through the fallback, so every
fixture agreed: **the field was populated, just from the wrong branch, and nothing compared the
branch.**

**Found by probing, not by reading.** I was debugging why `current_state_entry` named the topic
instead of the issue, printed `milestone_issue`, and saw `None` where Python saw `2036`.

## Why five fixtures could not catch it

**None of the five captured bodies cites `#2036`.** They cite `#2028` (the tracker, outside the
range), `#2029`, and nothing else.

**The fixtures record what the parser RETURNS; they do not enumerate what it could be asked.** That
is the limit of a behaviour capture, and it is why **a sixth case now exists** — a body citing
`#2036` — and why a test pins **both ends of the range and the value just outside it**:

```
#2029 -> Some(2029)     the first
#2036 -> Some(2036)     the last, inclusive
#2037 -> None           beyond, but still the ref the body names
```

## What else landed

**Three more gates, asserted against Python's own output:**

| gate | severity | condition |
|---|---|---|
| `prlint.prereg_before_compute` | error | model numbers with no pre-registration reference |
| `prlint.no_cpu_model_compute` | warn | CPU or host-fallback compute presented as a result |
| `prlint.current_state_entry` | warn | a claimed current-state entry the file does not support |

Coverage: **43 of 54**, up from 40 — **11 prlint gates in Rust.**

**`prereg_before_compute` is the gate I tripped four times while writing this port**, and the test now
pins why: it triggers on any prose line with a **run-word and a digit**, so a body discussing its own
verification commands trips it. **I did not loosen it** — narrowing a fuzzy gate that does a real job
would be a behaviour change dressed as a port.

**`current_state_entry` has three branches**, and **the issue is named when there is one, even though
it was the topic that failed to match** — I wrote that precedence the other way round first, and my
test caught it.

**`no_cpu_model_compute` inherits the parser's exclusion of defect lines**, which is what stops it
accusing a body that *reports fixing* a host fallback.

## Verification

| check | result |
|---|---|
| `#2036` recognised, `#2029` recognised, `#2037` not | asserted, both ends |
| **6 fixtures × 27 fields = 162 comparisons** | all matching Python |
| `prereg_before_compute` fires, and is silenced by four marker forms | asserted |
| a CPU claim fires; the same claim as a defect does not | asserted |
| the three `current_state_entry` branches | asserted |
| `cargo test -p uor-r4-harness` | **351 passed** |
| `cargo fmt --check`, clippy | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | 332 passed |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |
| `parity.py` | exit 0, **43 implemented** |

## Limitations, stated

- **The off-by-one survived because the fallback masked it.** `milestone_ref` was still populated, so
  **the fixture agreed on a field whose value came from the wrong code path** — a reminder that a
  field being right does not mean the branch that set it was.
- **`check_pr` is still not ported. Three of fourteen gates remain** (`parse`, `checks_at_head`, the
  org rules), plus the dispatcher and the `lint` command. **`lint` still delegates, and none of the
  eleven ported gates is reachable from any command.**
- **This unit took far more edits than it should have.** Repeated string-replace edits against text
  `cargo fmt` had already reformatted failed silently several times, and I mangled the test function
  badly enough to rewrite it. **The line-range approach worked every time; the string-replace
  approach failed most of the time, and I kept reaching for it.**
