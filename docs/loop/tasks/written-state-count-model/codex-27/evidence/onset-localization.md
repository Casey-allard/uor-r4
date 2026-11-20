# [codex] D27 frozen onset readback

2026-11-19 · Independent result-side audit by codex/d27fit.

This is a host-side inspection of recorded integer states, original prompt IDs
and the already fitted D21 seed-1 ordered bank. No threshold, exemplar, source
selection or model parameter was changed. Oracle head injection is a diagnostic
intervention only. The ordinary path retained the original prompt and roles.

## Original row 2703

The unannotated question asks what Owen collects, with “Owen collects copper
lamps” followed by “Owen catalogs ceramic bowls.” Original BOS-inclusive
coordinates are fact person `[17,20)`, predicate `[20,22)`, original value
`[22,26)`, and query person `[53,56)`. The causal source is position 22,
immediately after the asserted predicate and before the period at position 26.
The gold value starts with ID 581 and spans IDs `[581,2195,2527,85]`.

The ordinary staged exact and geometric cue selections both returned original
head `[17,20)`. The normal client captures for ordered exact, ordered geometry,
and recurrence exact each recorded one supporting fact, zero completed values,
no released source and the same emitted fallback “Got it, I'll remember that.”
The recurrence head-oracle witness returned `nonpositive` with no source on this
row. That is a separate scoring path from the ordered bank. No recurrence
geometry client capture for 2703 was present when inspected.

In the ordered head-oracle witness, the eligibility mask is `0b111` and the
word-start mask is `0b010`. Their intersection admits only source 22, at
offset 1 in the zero-based three-candidate packet. Its four-token distance
to the fence passes the strict less-than-eight bound. The frozen bank then
reports status 2 and supplies no source. In this case a missing candidate,
downstream duplicate disagreement, and the output writer cannot explain the
first failure.

The first value word ends at position 24. The candidate's bank shape is 514
(depth 2 and word length 2) and its ordered class signature is 153, encoding
`[START, CONT, START, CONT]`. The 200-record bank has nine records with that
shape and two with the exact shape/signature pair, both positive. The nearest
positive distance is 120 and the frozen radius is 124, so positive admission
passes. There is no matching negative exemplar. The missing negative distance
causes the bank to return status 2 before computing a signed score.

## Ordered head-oracle census

The D24 witness adapter was replayed from scratch with the original D27
prompt IDs and one independently isolated original fact head per case. The
adapter rebuilt the original candidate packet and required its final status
and source to match the production binder witness. The census covers 92
assertion visits across 84 distinct held prompts with at least one oracle
assertion. Duplicate and contradictory assertions contribute separate visits.
The remaining eight prompts have no oracle assertion to score.

| First ordered bank failure | Assertion visits | Evidence |
| --- | ---: | --- |
| Correct value start outside the three candidate offsets | 16 | Status 1. No active ranked candidate. |
| Correct admitted value start lacks matching negative support | 52 | Status 2. Includes row 2703. |
| Correct admitted value start lacks all matching shape/signature records | 8 | Status 2. Neither label has a matching record. |
| A different admitted candidate exceeds its positive radius | 16 | Status 2. The correct candidate itself has signed margin 16/3 in all 16. The other candidate has distance 112 or 120 against radius 88. The ordered policy marks that candidate unseen and withholds the entire assertion. |

For the separate recurrence diagnostic, 42 assertion visits returned
`nonpositive`, two tied, and 48 picked a source. Of those 48 picks, 28 matched
the original value start and 20 picked a different address. These are
assertion-level, host-only observations. They do not establish a normal-client
panel score, cold parity, GPU writer execution, or improvement after changing
the bank.

## Bindings

| Input | SHA-256 |
| --- | --- |
| D21 seed-1 ordered carrier | `02b922d37308593de956bf9c7c8ee0dbb259d2152528d6993eb9c89be84f65e3` |
| `panel-02/held.blind.jsonl` | `351ea27424f67972552f7243192d0dfe40be7b8b64173a43f864fbcbd62d27c3` |
| `ordered-value-witness.json` | `22dad67fa50d9428b497e14c0390c80c0adfeb5eb7c5ff87430dd7dd8142dd8d` |
| `recurrence-value-witness.json` | `15032935a5f12bf850b785a69860560b2dfce9d277cc90374cc406189583062c` |
| Local D24 adapter replay JSON | `422bf04aa096855b37ff2004b0fcf52bdea4ec3dd738e11b822735dd7ed78642` |

The subsequently completed recurrence geometry client capture for 2703 also
accepted exactly one supporting fact, completed zero values, withheld the
source, and emitted ID 1155 at position zero. Its full record passed the
independent writer comparison.
