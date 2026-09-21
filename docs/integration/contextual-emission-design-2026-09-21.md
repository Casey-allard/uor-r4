# Contextual transformation with a learned shared low-bit emission residual — prospective design

> **Principal correction, PR #1335:** the text below records the submitted design/report, not the current verdict. [The principal review](contextual-emission-review-2026-09-21.md) withdraws the capacity-only, optimizer-converged, adequate-range, perfect-source and calibrated-NLL claims. The population called fresh was exposed in attempt 1 before later design changes. Retain reported hit counts as historical aggregates, not independently reconstructed rows or an isolated geometry advantage. Next is [consistent same-width learning](deepseek-consistent-emission-step-2026-09-21.md), not an automatic capacity increase.

September 21, 2026. Executes the [principal read-conditioned review](read-conditioned-review-2026-09-21.md)
and the [comprehensive prompt](deepseek-contextual-emission-step-2026-09-21.md) on reviewed parent
`cfc5f7b6`. Recorded **before** extraction, training and any final outcome. The
[resource ledger](resource-ledger-2026-09-19.md) holds the projection and the increment taken first.

## The decisive question

Can the **same query** use **different relevant older evidence** to produce **different uncopied
answers**? The previous run's instrument failed this: its answer was a function of the query role
alone, its extraction stopped before the final query key, its optimizer could not move at a zero
ceiling, and its "categorical" arm was the same H4 algebra.

## Instrument (declared before fitting)

Paired prefixes. Both members share the query role, the query key, the complete recent suffix, the
position of the relevant block, and every distractor's role, value and order. **Only the relevant
source block's value changes** (placed off the final position, so both members have byte-identical
local logits). The required next token is that value's output class from a bank chosen disjoint from
every prefix token and every admitted payload, so it is absent from every prefix and every payload and
no single-payload boost can emit it. Several same-key distractor sources are present, so the read must
select the right occurrence.

Decision point: the observation whose ring holds the full prefix up to and including the query key
(`tokens.len() - 2`). An empty pool there is retained as NoRead; no earlier position is substituted.

## Operator

```
q0 = bounded causal query state (frozen)      r = directed relation to the selected occurrence
v  = V[selected payload]                      q1 = (q0 * T[r]) * V[v]     (exact signed-H4)
z  = z_local + ( W . (R[q1] - R[q0]) ) << shift
NoRead / UpdateDisabled: q1 = q0, residual exactly zero
```

`R` is a seeded nonconstant ternary `120 x 16` state embedding; `W` is the **learned** ternary
`vocab x 16` output map (initialized to zero, so the disabled path is exact and a learning signal
exists once the state changes). Coefficients are two-bit; the only scale is the declared integer shift,
chosen from the development target deficit by one rule: the smallest `s` with `16 * 2 * 2^s` covering
the maximum deficit, capped at 12.

## Learning and selection

`W` is trained by full-batch NLL gradient descent on development; the transport/value maps are then
improved by a bounded discrete search on the **mean margin** objective (NLL/margin, not integer
accuracy, so partial progress exists before the answer becomes the argmax). The run reports initial and
final parameter hashes/nonzero counts, NLL before and after, accepted map changes and the residual's
range bound. One candidate per declared comparison; tune is a development check; the fresh instrument
is one held-out draw.

## Controls and causal test

Frozen-row diagnostic (`argmax(z_local - u(q0) + u(q))`, 120 rows) as a baseline; local/NoRead; the
scalar-copy parent; the learned H4 arm; the **matched cyclic C120** arm `q1 = (q0 + T[r] + V[payload])
mod 120` with the same state count, maps, information, readout capacity and budget; UpdateDisabled and
ReadDisabled; and the **real changed-source test** — the paired prefix whose only difference is the
older payload — reporting both members' emitted answer and whether both are correct. Short generated
continuations are scored at the first emitted token with equal horizons.

## Scope, artifacts and resources

A bounded authored capability instrument, explicitly not broad language or reasoning. Parameters bind
through versioned `RLRC`/`RLCE` artifacts reloaded and compared before use. Energy UNAVAILABLE;
whole-path D0-b not claimed. Projection: one targeted pass (no legacy panel rerun) within the recorded
ledger increment.

## Diagnosed defect and fix (recorded before the final evaluation)

The first pass (`contextual-emission-1`) produced a **valid** instrument — 90/90 pairs with identical
local logits, 180/180 decisive, absent targets held, 180/180 reads — but its saved traces show an
**optimizer defect**: the float fit reduced NLL only marginally and a second fit diverged (final NLL
`4.1e5`/`7.3e5` bits), so the served ternary map was effectively untrained (H4 12/180 dev, 0/90 paired).

Fix applied before any final evaluation: the gradient step is scaled by the mean feature magnitude
(the declared shift makes unscaled steps diverge), the output-row budget is raised to 64 ternary rows,
and a **bounded discrete ternary refinement** (single-coefficient flips under the mean-margin objective)
follows quantization. The diverging second float fit is removed. The instrument, populations, split,
controls and selection rule are unchanged; the delivered root is `contextual-emission-2` and
`contextual-emission-1` is retained unchanged as the diagnostic artifact.
