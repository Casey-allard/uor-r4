# #1786 — addressed_attention: unforced emission measurement — the read, the gate and the carry-forward are three separate blockers

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-06T04:32:50Z
- **Updated:** 2026-10-09T13:22:55Z
- **Closed:** 2026-10-06T04:33:51Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1786
- **Merged:** 2026-10-06T04:33:51Z
- **Merge commit:** 6cb86a705c1f6fe5d37801b5f9b6632ab6d0cad9
- **Base / Head:** main / UOR-Foundation:deepseek/unforced-emit
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Measurement-only. **Production code unchanged** (`git diff --stat` empty; the only change is the new bin). `source_digest` and `parameter_digest` are byte-identical to sealed predecessor `aa-arith-oracle-20261005-204407`, and the natural arm reproduces **all ten** of its published numbers exactly (`mismatches: []`).

## The pre-registered comparison is VOID — and saying so is the honest result

No natural position has read **and** operator both correct, so the intended-problem intersection is **0/80**. The mechanism **did** execute — `Work` counters over the whole arm: predictions 232, observations 232, `circuit_calls` 1554, `candidate_scores` 1976, `scalar_decodes` 226, **`selected_operations` 9** — so the comparison is void on the *antecedent*, not on the firing count.

| natural run (80 scored positions, no forcing) | |
|---|---|
| read A / B exact intended span | 6/80 · 2/80 |
| **both intended spans** | **0/80** (payload metric also 0/80) |
| **intended operator** | **2/80** (any AddAB/SubAB 6/80) |
| read **and** operator both natural | **0/80** |

## The emission readout is separately shown unable to carry a produced byte

Direct capture of the packed `Phase::Emit` input at bits **792..800** (wire self-check: on all **45/45** rows carrying an active lease the captured bits equal `offer.active().byte()`):

- The operator produced a value on 6/80 positions. On those 6, the produced cursor byte is **present in the input 6/6** and the emitted symbol equals it **0/6**. Concretely, as (example, input byte -> emitted, context): ex3 49 -> 147 (ctx 125); ex5 48 -> 243 (ctx 293); ex9 49 -> 66 (ctx 413); ex21 48 -> 9 (ctx 45); ex22 50 -> 108 (ctx 487); ex25 50 -> 186 (ctx 63).
- Whole run: `emitted == its own input byte` **0/80**.
- Multi-digit: 16 later-digit positions, target byte present **0/16** — a rejected emission drops the provisional lease (`engine.rs:466-475`, `objects.rs:629-648`), so the cursor never advances.

**Frozen cascade reach** (0..255 sweep of the byte field, all else fixed): distinct contexts **min 1 / mean 2.45 / max 8** out of 256 byte values; on **48/80** positions the target byte shares its context with another byte. Counterfactual from sealed rows: substituting the target byte at 792..800, the frozen head's argmax equals the target on **0/48** digit positions, and on **21/48** the substitution does not even move the context.

## Forced-Emission-only arm

Overrides fired 232/232, but read pair **0/80**, emitted == produced **0/1**. It separates cleanly: **arming the readout cannot carry the byte**, because the natural read/operator never produces a target-relevant byte — the binding constraint is **upstream of the readout**. It also exposes an acceptance coupling: forcing the proposal changes what `observe` accepts, so the trajectory changes (digests `9449a345…` vs `238b14c0…`; provisional 6 -> 1).

**Diagnosed mode:** `b_value_produced_but_context_not_distinguishing`, **gated by** a read failure. **Not (a) for the operator** — it computed the exactly correct value for every operand pair it was given (6/6).

## Three named missing components (D20: a gap yields a component, never a verdict)

**(a) Emission readout** — a context builder injective on the 8-bit active-byte lane at 792..800, plus head rows that select the produced byte. **At frozen init it is the GATE parameters (`policy.rs:10,78-83`) that collapse the byte field**, so head rows *alone* cannot separate bytes sharing a context.

**(b) Carry-forward coupling** (`engine.rs:466-475` / `objects.rs:629-648`) — the emitted proposal gates whether the provisional lease survives, so without a correct first emission no later digit can ever be presented. Measured 0/16.

**(c) Read** — the query/null root heads do not identify the authored decimal occurrences (0/80 both-span; truncated spans where one operand is right).

**What would change the answer:** a **trained** gate/emission parameter set (this is unfitted `Parameters::seeded(7341)` only); a correct read (*necessary but insufficient* — see the 0/48 counterfactual); or an acceptance-coupling change so a produced value survives a rejected proposal.

**Author's honest caveat:** the 0/6 and 0/48 figures are conditional on the *natural* rest-of-input, so they show the frozen head cannot carry the byte in the contexts the unforced run actually reaches — they do not prove no upstream-correct trajectory could ever be carried.

**Disk discipline:** the build took free space 46.49 -> 39.30 GiB, below the 40.12 GiB reserve; the author removed **only its own** 2.1 GiB cache and restored free to 41.38 GiB. Cost: build 185 s / 2.40 GiB RSS; run 24 s / 23 MiB RSS; CPU only, no external cost. Attempt 1 is preserved and documented as superseded. References #820, #1747, #973, #1552, #1746.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

