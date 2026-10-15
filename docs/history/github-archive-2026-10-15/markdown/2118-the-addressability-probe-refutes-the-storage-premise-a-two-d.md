# #2118 — The addressability probe REFUTES the storage premise: a two-digit value already has an ordered home (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T01:00:47Z
- **Updated:** 2026-10-10T03:23:25Z
- **Closed:** 2026-10-10T01:02:50Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2118
- **Merged:** 2026-10-10T01:02:50Z
- **Merge commit:** 197c918ce00f62238a1c7e2083898216ea025ed2
- **Base / Head:** main / UOR-Foundation:numeric-addressability-probe
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## The probe ran, and it kills the premise

**CPU only, no pod, no GPU, no training, no model run, $0.** One focused test over the numeral codec the memory path already reads through — `crates/uor-r4-core/src/native_geometric/numeral.rs` — with **all 5 numeral tests passing, including the probe added here**.

| question | answer |
|---|---|
| **(a)** does `84` come back as the tokens `8` then `4`? | **Yes — and the codec recovers it as ONE value `84`**, with the run recorded as its inclusive interval `start 0, end 1` |
| **(b)** is ORDER preserved? | **Yes.** `8`,`4` → **84**; `4`,`8` → **48**. The identity is carried by order, not by a bag of digits. |
| **(c)** is `74` distinguishable from `84`? | **Yes** |
| **(d)** leading zeros / repeated digits? | `0`,`7` → **7**; `7`,`7` → **77**; the same value twice is **separated by interval** |

**VERDICT: REFUTED — "a two-token value has no single address to live in" is not true of this representation.** The codec accumulates digits left-to-right across token boundaries and retains the inclusive token interval, so order is what carries the value's identity; repeated values at different positions are already distinguished by interval, which is the project's own rule that equal tokens at different positions must not share identity.

## What it does NOT settle — the honest limit

The probe is **untrained** and tests the **representation, not the learned path**. It cannot say whether the model's read/emit path *uses* this codec to deliver a stored value. **The diagnosis's measured emission failure stands unchanged** — 10 of 12 failing numeric rows emit no digit, 2 emit a digit never stored, the one passing row emits the stored value exactly — and is now **localized to the learned path rather than the representation**. Those are different failures and the probe separates them.

## Consequence for the plan

- **Option A (number-aware tokenizer) LOSES ITS STATED RATIONALE and goes back on the shelf.** It was the option that invalidates every artifact trained on `d36d3e87…` and needs the whole ladder rebuilt — and it would have been spent on a refuted premise. **That is the probe's value: it stopped a full retrain for nothing.**
- **Option B is not refuted but its premise is weakened.** "The value has no slot" is wrong at the storage layer; what survives is the **emission** question.
- **The plan's own falsification condition 1 has fired**, and by the plan's terms the addressability fix is the wrong lever.

## The ledger — four readings, three dead

| reading | status | what killed it |
|---|---|---|
| Token count explains the cluster | **REFUTED** | numbers are **2 tokens against 3.74** for words |
| Minimal pairs / digit order | **REFUTED** | `mem-040`'s distractor `74` is a **transposition** of its `84` and it passes; **0 of 12** numeric failures name the distractor |
| Value addressability | **REFUTED by this probe** | the codec recovers a two-token run as one ordered value with an interval |
| **The learned read/emit path does not deliver a correctly stored value** | **OPEN — the only survivor** | 10 of 12 numeric failures emit no digit; the passing row emits the stored value exactly; `expected_value` 40 of 40 |

## Delivery

Record [`docs/labs/numeric-addressability-probe-2026-10-09/README.md`](docs/labs/numeric-addressability-probe-2026-10-09/README.md); the focused test; newest-first [`current-state.md`](docs/integration/current-state.md) entry ending in **Next:**; bundle on cloud-store: `numeric-addressability-probe-2026-10-09.tar`, 6,656 bytes, md5 `8de73aa3804d250fa423c43280c92e69`. `check_claim_wording.py` passes; `scripts/__pycache__/` not committed; one live branch.

**Criterion 1 remains NOT MET on both halves; 43/232 unchanged; v5 was not re-run. References #2029.**


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

