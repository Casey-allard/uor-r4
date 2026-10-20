# The pure core of `registry_vs_github` — ported, tested, and **deliberately not registered**

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the rule I have deferred longest

## What landed

**`crates/uor-r4-harness/src/githubcards.rs`** — `_newest_card`, `_card_reading` and
`_registry_numbers`: **the part of the rule that decides anything.**

**Coverage does not move: 51 of 54.** See below — that is the point of this unit.

## Why only the core, and why it is not registered

`status.registry_vs_github` is ~135 lines and **three of its parts need a network**: a cache under the
state directory, one `gh issue view` per milestone, and comment pagination.

**I did not port those**, so **the rule stays in the Python set.** A half-ported rule claimed as
coverage would be **the silent downgrade this port refuses** — the same reason `harness.documented_commands`
is unregistered after I declined to approximate it.

**What is ported is where a bug would live.** `_newest_card` → `_card_reading` → `_registry_numbers`
turn comments into a reading, and **a reading is what the rule compares against the registry.** They
are pure, **and they can be compared against Python without a network — which is the only way to
verify them honestly here.**

## The capture corrected me on the strict form

The docstring calls `M<n>: <n>/<n>` the *"primary form"*, and I built a case asserting a body of
`"M1: 37/232 here"` reads as `milestone_number`.

**The capture returned `no_card`.** `_newest_card` requires a **card marker** (`status card`,
`blocked card`, `pivot card`, `headline`) *before* `_card_reading` looks at the body at all.

**So the strict form is a refinement of a card, not a way to find one** — a distinction the docstring
does not draw and that only the capture made visible. **Both readings are now asserted**: no marker →
`no_card`; marker plus strict → `milestone_number`.

## Details ported rather than improved

- **Comments sort by `createdAt` as a STRING**, reversed. An ISO timestamp sorts correctly as text,
  **and a missing one sorts last as the empty string.** Sorting by parsed time would reorder malformed
  timestamps — **and the order decides which card is read.**
- **The 120-character window is taken in CHARACTERS**, not bytes, **because a card quoting non-ASCII
  would otherwise slice mid-character** — the char-boundary panic this port has hit twice.
- **`registry_numbers` reads `secondary` rows too**, and **skips a row missing either half rather than
  reading it as zero** — a missing number is not a claim of no progress. Floats truncate, matching
  Python's `int()`.

## Verification

| check | result |
|---|---|
| eight readings against Python, field for field | asserted |
| the newest card decides | asserted |
| a non-ASCII card cannot slice mid-character | asserted |
| `registry_numbers` covers `secondary`, skips incomplete rows, truncates floats | asserted |
| `cargo test -p uor-r4-harness` | **384 passed** (5 new) |
| `cargo fmt --check`, clippy | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | 332 passed |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |
| `parity.py` | exit 0, **51 implemented — unchanged, deliberately** |

## Limitations, stated

- **`check` still delegates, and this unit does not move the count.** Two rules remain:
  `harness.documented_commands` and `status.registry_vs_github` — plus `prlint.parse`.
- **The `gh` chain is untouched**: cache, one call per milestone, pagination, and the *decision*
  that follows the reading (`no_card`, a qualitative card, a stated-but-unmatched number). **Only the
  reading is ported**, so **the ported code is a helper with no caller** — reachable from tests and
  from nothing else.
- **`newest_card` clones every comment it inspects.** Python sorts in place and returns a reference;
  **the port copies bodies, which is fine at eight comments and wasteful at eight hundred** — and the
  rule runs per milestone, so it is per-milestone waste.
- **A card whose marker appears after 120 characters reads as `headline_text` with no number**, since
  the window starts at the marker. **That is Python's behaviour and it is why the labs' cards put
  `headline` early** — a formatting convention the code depends on and does not document.
