# Pointer-copy head with an exact identity key
**The idea (first principles):** A memory answer ("Ana's locker is 47") is *copied* from context, not generated. The pointer-copy head (A1) mixes a copy distribution into the output: `p = (1-g)·softmax(z) + g·Σ_j a_j[x_j = v]` (module docs in `crates/uor-r4-training/src/geometric_stack.rs`). The project-specific part is how the pointer picks the source `j`. Under UOR addressing every token has a registered prime, and a key is an exact prime product. A source is admitted by `gcd > 1` or by ordered n-let equality (ADR-0003, `docs/adr/0003-fixed-zeta-prime-route-attention.md`). Identity is exact: no near-miss confusion, and rare atoms weigh more (`ln gcd`). **The key should be the slot/entity address (Ana + locker), so the pointer lands on the value stored there. The surrounding wording should not be the key.**
**What it replaces and why that matters:** It replaces learned dot-product attention, which must discover which token answers the question. Measured: it latches onto the sentence frame. An exact address needs no search, has no float score to serve and is auditable.
**Where it lives in code:** `geometric_stack.rs`:
- `PointerConfig` (`score`, `identity`, `select`, `route`) and `PointerSelect::TopK`
- `PrimeRoute` with `RouteAdmission::{SharedAtom, Ngram}` and `ranked`, scored by `ROUTE_SHARPNESS = 4.0`
- `PointerIdentity` (`weight_bp`, `POINTER_IDENTITY_WINDOW = 6`)
- `gate_supervised_loss`, and `set_pointer_key_fold` (canonical ids from `copy_identity.rs`)

Export serves only an unrouted, unselected pointer.
**What has been tried, honestly:**
- #2123, #2128 and #2133 were diagnosis only. The pointer attends the frame (a bare space holds 0.92 of the attention) and never selects the value slot or the full two-digit run.
- #2145 (gate supervision): memory 10→21/40, but the matched control reached 22/40. The fine-tune stream did it, not the mechanism.
- #2161 (token-identity key = the six tokens before the source vs before the query): wrong-value failures 17→12, memory 19/40, dev pointer hit 0.003 at weight 1.0.

**No test has used an entity-address key.** #2161 keyed on the context *preceding a value*, not on the slot's name. The exact route had frozen earlier (`docs/evidence/routed_pointer_frozen_2026-10-09.txt`). Every run was a bolted-on fine-tune of one 29M artifact, one seed, on a 40-row panel.
**What success looks like:** Own metric: pointer hit and copy mass on the bound value's positions, and fewer wrong-value failures. Headline: M1 #2029 v5 memory `check_pass` (/40) and the open reply panel (/232).
**What failure looks like:** An oracle-supplied correct slot address still fails to raise hit rate or memory over a matched control. That would mean exact-address copying does not help this model.
**A fair test:**
- First, a serving-time oracle ablation: supply the true slot address. It bounds the gain at no training cost.
- Then a parsed or learned address key, trained in, with matched controls, ≥3 seeds and ≥120 memory rows (±3/40 is noise).
- Pre-register a bar of at least +5/40-equivalent over the control.

**Open design questions:**
- Should the key be entity prime × attribute prime, written when the fact is stated?
- Should a value span be admitted (the whole run), rather than one token?
- Exact admission plus learned ranking (`ranked: true`) over addresses?
- Integer serving: route by gcd with no float score?
