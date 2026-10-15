# #475 — test(certify,core): #424 — long-range context ceiling and the Bott-Fock decay constant

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-07T07:01:38Z
- **Updated:** 2026-08-07T07:34:06Z
- **Closed:** 2026-08-07T07:24:31Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/475
- **Merged:** 2026-08-07T07:24:31Z
- **Merge commit:** 8403d496950603d06df5bae9244f260620adc69c
- **Base / Head:** main / UOR-Foundation:measure/424-context-horizon
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

#424 asked for a flagged Gate C A/B of `BottFockContextStore` against the hard 8-token window. Under the `AGENTS.md` reachability gate that A/B should not be launched. This PR ships the arithmetic that says so, as two reproducible instruments rather than a claim in a comment.

## Run contract

```
metric to move:       Gate C top-1 via a long-range context carrier
reachability ceiling: +1.02pp (measured here) for ANY carrier on this corpus;
                      +0.16pp for the fold at its shipped decay constant
instrument + verdict: long_range_ceiling.rs -> "MECHANISM SOUND, CONSTANT WRONG"
exit rule:            ceiling >= 0.5pp AND shipped decay retains >= half -> A/B
if positive:          activate behind the flag, run the Gate C A/B
if negative:          retune the decay first, or park the gap question
cost estimate:        41s for the ceiling; the A/B it replaced is hours
```

## Ceiling A — how much long-range signal exists (carrier-independent)

Pinned bundle `blake3:194db0ee…`, 3,000 stories / 361,693 positions, document-level split.

| Arm | best λ | top-1 | GAIN vs base | null gain |
|---|---:|---:|---:|---:|
| BASE (window only) | — | 21.62% ± 0.16pp (n=66,536) | — | — |
| CACHE (order-free bag) | 0.05 | 21.99% | +0.36pp | +0.01pp |
| INDUCTION (order-sensitive, unbounded) | 1.0 | 22.64% | **+1.02pp** | −1.29pp |

The gain does not survive a cross-document null, so it is this document's history and not a prior on frequent continuations. Two thirds of it is order-carried — an order-free cache cannot collect it, which is a point in the fold's favour.

## Ceiling B — how far the fold can see (carrier-specific)

| lag k | 0 | 8 | 16 | 32 | 48 | **64** |
|---|---:|---:|---:|---:|---:|---:|
| L1 influence | 1,363,342 | 134,203 | 14,514 | 141 | 2 | **0** |

`cell <- cell - (cell >> 2)` is a 3/4 geometric decay. A token 64 positions back leaves a **bit-identical** state, and ~90% of the mass sits inside the eight most recent tokens — the window the runtime already has. A control test confirms this is decay, not state collapse.

## The two together

Replaying the fold's inductive bias **losslessly** upper-bounds the fold itself. Sweeping the decay separates "wrong mechanism" from "wrong constant":

| decay | horizon | GAIN | retained |
|---:|---:|---:|---:|
| **0.75 (shipped)** | 24 | **+0.16pp** | **16%** |
| 0.90 | 66 | +0.32pp | 31% |
| 0.95 | 135 | +0.61pp | 60% |
| 0.99 | 687 | +1.06pp | 104% |

Monotone in the horizon, saturating near 0.99. The mechanism is sound; the constant is not. `>> 2` was chosen for the P-4 shift discipline, and `>> 7` (127/128) satisfies it equally while landing on the ceiling.

## A corrected metric, recorded on purpose

An earlier pass reported the ceiling as `observed − null` and got **+3.13pp**. That flatters by degrading: a wrong donor damages the baseline, and a deployed system's alternative is *no* distant evidence, not *wrong* distant evidence. The harness now keys on `observed − base` and uses the null only to admit the gain. The discarded figure is in the record because the failure mode generalises.

## Scope limits

Documents in this bundle cap at 128 tokens, so this measures long-range dependency *within 128 tokens*. The baseline is a backoff n-gram at 21.62%, weaker than the full stack — so +1.02pp is generous. Two independent implementations (Python prototype, shipped Rust harness) agree to 2dp on every arm.

## Gates

`cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `check_claim_wording.py`, `cargo check --target wasm32-unknown-unknown -p uor-r4-wasm-router --lib`, and the touched suites all green. The new always-on test runs in 0.18s; the harness is `--ignored`.

References #424

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

