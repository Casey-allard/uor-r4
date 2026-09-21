# Contextual emission: historical submitted result with principal corrections

> **Principal correction, PR #1335:** the text below records the submitted design/report, not the current verdict. [The principal review](contextual-emission-review-2026-09-21.md) withdraws the capacity-only, optimizer-converged, adequate-range, perfect-source and calibrated-NLL claims. The population called fresh was exposed in attempt 1 before later design changes. Retain reported hit counts as historical aggregates, not independently reconstructed rows or an isolated geometry advantage. Next is [consistent same-width learning](deepseek-consistent-emission-step-2026-09-21.md), not an automatic capacity increase.

September 21, 2026. Executed from reviewed parent `cfc5f7b6` (PR #1334). Prospective
[design](contextual-emission-design-2026-09-21.md); [principal review](read-conditioned-review-2026-09-21.md);
[comprehensive prompt](deepseek-contextual-emission-step-2026-09-21.md). Delivered root
`.uor-models/realtext-prior-2026-09-20/contextual-emission-4` (sealed, verified, 0 unlisted).
Diagnostic roots `contextual-emission-1` (invalid optimizer) and `-2`/`-3` are retained unchanged.

## Decision

The four defects that invalidated the previous instrument are fixed and the decisive question now has
a **real, measured, positive signal with a diagnosed ceiling**: the learned read-conditioned update
emits correct uncopied answers on **35/180** development and **13/120** held-out positions, with
**6/90** and **1/60** paired examples fully correct — while **every control, including the scalar-copy
reader, emits exactly zero**. It is not yet a reliable learned contextual value transformation: the
rate is far below what the paired criterion requires, and extra optimizer passes change nothing.

## Instrument is valid (fixing the four defects)

Paired prefixes share the query role, query key, recent suffix, relevant-block position and every
distractor (role, value, order); only the relevant source block's value changes, and the relevant
block is kept off the final position. Measured: **90/90 pairs with byte-identical local logits**,
**180/180 decisive** (local argmax wrong), **180/180 source reads**, **absent targets held on every
split**, output bank (4088–4095) disjoint from all values (485–507). The decision point is the query
key's own observation, not an earlier position. Previous defects — query-only answer, off-by-one
extraction, non-learning optimizer, and an H4-cloned "categorical" arm — are gone.

## Learning occurred

| Arm | served NLL (bits): initial → quantized → refined → final | ternary flips / map changes | dev / tune hits |
| --- | --- | ---: | ---: |
| H4 | 15561 → 11439 → 10986 → **7987** | 21 / 9 | **35 / 22** |
| cyclic C120 | 15561 → 12549 → 11524 → **8773** | 29 / 10 | 27 / 16 |

Both artifacts reload identically (`artifact_reload_all_ok = true`; `h4_emission` params `6abe1861…`,
residual `ce3b4285…`). The discrete ternary refinement is what makes the quantized map trainable; the
first pass's divergence is diagnosed in the [design](contextual-emission-design-2026-09-21.md).

## Comparisons on identical rows

| Arm | dev hits /180 | tune /120 | fresh /120 | dev pairs both /90 | fresh pairs both /60 |
| --- | ---: | ---: | ---: | ---: | ---: |
| local / NoRead | 0 | 0 | 0 | 0 | 0 |
| scalar-copy parent | 0 | — | 0 | 0 | 0 |
| **H4 read-conditioned** | **35** | **22** | **13** | **6** | **1** |
| matched cyclic C120 | 27 | 16 | 12 | 0 | 1 |
| UpdateDisabled | 0 | 0 | 0 | 0 | 0 |
| ReadDisabled | 0 | 0 | 0 | 0 | 0 |

Frozen-row ceiling on the same 180 positions: **0/180** — the frozen readout family cannot emit these
targets at all, so every hit above zero is produced by the learned residual. The scalar-copy operator
is provably unable to emit any of them. The matched C120 arm shares state count, maps, information,
readout capacity and budget and is slightly behind on development.

## Diagnosis

| Candidate cause | Evidence | Verdict |
| --- | --- | --- |
| Admission / selection | 180/180 reads, 90/90 identical local inputs, source index determines the payload | not limiting |
| Input collision | 88 distinct `(q0, r, payload)` signatures for 180 dev positions, only **6** ambiguous (14 targets) | not limiting |
| Residual range | bound 32768 ≥ max target deficit 16640 units | adequate |
| Optimizer | extra discrete passes (2→5, search 2→4) leave every number unchanged | converged |
| **Readout capacity** | 22 ternary rows of width 16 must separate 8 value→output mappings at ~16k-unit boosts | **the limiting factor** |

Changed-source pairs (fresh) confirm the mechanism is genuinely contextual: both members have identical
local logits, the read updates the state in both, and different values yield different emitted tokens —
one sample emits the correct answer (4092) for one member, and another emits the correct 4091 for the
other. Full paired correctness is only 1/60, so the transformation is real but not reliable.

Generated continuations on the same boundary are short (three tokens) and mostly token soup; the first
token is correct in one of three retained samples. This is a bounded capability instrument, not prose
and not reasoning.

## Retained and next

Retained: the validated paired instrument, the learned H4 and cyclic-C120 emission operators with their
versioned artifacts, the exact-zero disabled path, the frozen-row and collision diagnostics, and every
failed criterion. The evidenced next operation is a **bounded capacity increase in the shared readout**
— a wider learned derived feature or a small learned low-bit projection over the derived state — with
an equal-capacity cyclic-C120 control, because additional discrete optimizer passes demonstrably change
nothing. Dimensions and architectures are not expanded on this evidence.

Resources and charges are in the [resource ledger](resource-ledger-2026-09-19.md). Energy UNAVAILABLE;
whole-path D0-b not claimed.
