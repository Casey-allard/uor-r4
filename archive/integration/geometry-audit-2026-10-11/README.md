# The geometry audit: where the arithmetic actually goes, why the project drifted into matrix wrappers, and what a compute-reducing mechanism has to be

Date: 2026-10-11. Author: DeepSeek lab. Trigger: the owner's question of 10 October —
*"the project is mostly wrappers around matrices … we keep converting everything back to matrices … novel
mechanisms that reduce compute via better math, not just reusing the same thing run slightly different."*

This document answers that with the project's own numbers. It is a diagnosis and a decision request, not a
result: no claim here is new evidence about quality, and every number below is either read off an artifact in
`main`'s line or measured in this session by a run whose record is linked.

## 1. The ledger: what the trained, served 29M stack actually computes

Read off the deployed artifact (`chat-29m-B-lr5e-4` lineage; width 576, 8 heads, 10 layers, pattern
`rrarrarrar`, MLP 732, vocab 4096, context 384 — the shape every M1 cycle since #2145 has fine-tuned):

| component | parameters | share | **MACs per token** | share |
|---|---:|---:|---:|---:|
| MLP (SwiGLU, all 10 layers) | 11.4 M | 37.7 % | 12,648,960 | 39.6 % |
| recurrence projections (`rec.in`, `rec.out`, `rec.gate`, `rec.conv`) | 9.9 M | 32.8 % | 9,886,464 | 31.0 % |
| reads (softmax attention q/k/v/o + scores + value mix, 3 layers) | 4.0 M | 13.3 % | 5,308,416 | 16.6 % |
| output head (4096 × 576) | 2.4 M | 7.8 % | 2,359,296 | 7.4 % |
| pointer copy head | 37 k | 0.1 % | 67,584 when its gate fires | 0.2 % |
| **quaternion transport rotations** (the geometry) | **0** | **0 %** | **16,128 multiplies** | **0.05 %** |
| **exact addressed store, prime/UOR addresses, zeta phases, icosian arithmetic** | **0** | **0 %** | **0 lookups** | **0 %** |
| **total** | 30.2 M | | **30,270,720** | |

Three facts a reader should take from that table:

1. **The geometric operation is 0.05 % of the arithmetic.** The quaternion transport is real — seven recurrence
   layers each rotate a carried state by a learned unit quaternion — but it is *parameterised* by
   `rec.in` [1152, 576] and `rec.out` [576, 576] and `rec.gate` [720, 576]: 1.4 M weights per layer producing the
   drive, gate, decay and rotation. The rotation itself, 4×4 per lane × 144 lanes × 7 layers, is **16,128
   multiplies per token** against 30.3 M for the dense projections around it.
2. **Every weight is touched every token.** The stack is dense: 30.2 M MACs and ~30.2 M parameters read per
   token. Nothing is sparse, nothing is addressed, nothing is skipped.
3. **The read's share is context-dependent, and that is where the algorithmic headroom is**: 16.6 % of MACs at
   the trained context of 384, 22.0 % at 1,024, **40.5 % at 4,096** — attention search grows with the context
   while an addressed lookup would not.

## 2. Where the geometry actually is — and what each piece costs

| mechanism | implemented | runs in the trained/served stack | cost |
|---|---|---|---|
| R4/S3 quaternion transport | yes | **yes** | 0.05 % of arithmetic, wrapped in 1.4 M dense weights per layer |
| matched abelian control | yes | used in the ladder | — |
| icosian snap of each rotation | yes (optional) | **off** in the shipped base | — |
| Lorentz read score | yes | **no** — the trained base uses `read=l2`, the flat Euclidean *ablation control* | — |
| fixed zeta-zero phases | yes | **no** — recorded as living in the native learner's score tables, not in the stack | — |
| H4 / icosian codes | yes | in quantiser codecs and the native learner's tables, not in the stack's computation | — |
| prime / UOR addressing, exact addressed store | yes | **no** — `native_geometric` only | 0 in the stack |
| product-key memory (`memory_layers=`) | yes | only since 2026-10-10, as an option, one layer | 173,056 MACs where it replaces an MLP's 1,264,896 (−86 %) |

So the honest description of the artifact the project scales and serves is: **a transformer (softmax attention +
SwiGLU + dense residual stream) with a quaternion-mixed residual and a 0.1 % copy head.** The project's
genuinely novel machinery is in a different family (see B4).

## 3. The connection bugs

These are the answers to *"the whole base problems with how the project is connected together."* Each is
structural, each has an artefact as evidence, and none is a criticism of any individual run.

**B1 — The metric cannot see compute, so compute was never optimised.** Every acceptance criterion in the M1
line is a quality number: BPB, the v5 memory panel, the open reply panel. In the whole D21/D22 cycle contract
there is no requirement to report multiplies per token, weights touched, bytes moved or wall time at fixed
quality. A mechanism that halves the arithmetic and ties on quality is therefore *invisible* to the process and
loses to a mechanism that adds arithmetic and gains a row. The repository already knows better: the
transformerless line wrote the rule — *"a throughput number never travels without its quality number"* — and
publishes `multiplies/token` and `weights+state on disk` in
[COMPARISON.md](../transformerless/COMPARISON.md). The model line never adopted it.

**B2 — The geometry is parameterisation, not computation.** The quaternion transport *is* in the forward pass,
but it does not constrain the computation: it mixes a residual stream that is then processed by dense matmuls
exactly as a transformer's would be. No geometric structure bounds the search, removes a matmul, makes anything
exact, or lets anything be skipped. Geometry enters as an *initialisation and a mixer*, and the arithmetic it
saves is nil.

**B3 — Novel mechanisms entered as optional paths judged against matched controls, and the controls won.** Each
geometric mechanism was added beside a matched ordinary one and kept only if it did not lose on quality. The
Lorentz read lost to `read=l2` — the *control* — and the control became the base. This is the correct
experimental discipline applied to the wrong objective: with a quality-only metric, the safest way to pass is
to be an ordinary model with geometric seasoning.

**B4 — Two families, one critical path.** The novel mechanisms — exact addressed attention, dependent reads,
typed operators, the addressed store — live in `crates/uor-r4-core/src/native_geometric/` (with
`addressed_attention/`, `dependent_read.rs`, `composed_output.rs`, `anchors.rs` and dozens of experiment
binaries). That family is **not** the one the ladder scales, fine-tunes and serves; the stack that is scaled has
0 % of it. And the native family's own headline win does not transfer: **437/512 on its development panel,
0/128 on fresh qualification** (#2164). So the project's novelty and the project's critical path are disjoint —
which is precisely the "how it is connected together" problem.

**B5 — Serving substituted operations without reducing work, and it measured slower.** On the same artifact and
the same windows at identical NLL, the multiplier-free D11 engine ran **68.0 tokens/s** against the float
path's **127.5** ([D11 serving record](../labs/d11-serving-2026-10-10/README.md)). "No multiplies" is a
*contract and energy* property — it is how the frozen runtime is specified — but on this hardware it is not a
compute win. A lookup costs a memory access; a multiply costs a FLOP; the machine is bandwidth-bound long before
it is FLOP-bound. **Replacing operations is not reducing work.**

## 4. What "better math that reduces compute" has to mean here

Given B5, the only three things that can reduce the work per token are:

1. **Touch fewer weights** — sparsity: a top-k gather instead of dense matmuls.
2. **Do less search** — addressing: an exact store lookup instead of attention scoring the context (the term
   that grows to 40 % of the arithmetic at 4 K context).
3. **Carry less state per token** — a smaller residual stream, or a state that is read rather than recomputed.

The repository has an existence proof that a large win is available *in principle*: the transformerless census
runs **zero multiplies, 2.17 MB of weights+state, 77,342 tokens/s** — at **30.3 % teacher agreement**. That is
the shape of the trade the project is actually making, stated honestly: enormous compute reduction at a large
quality cost. The research question is not "can we remove multiplies" (yes), it is **"how much of that
77,000 tok/s survives at acceptable quality"** — and that question cannot be asked while the ledger is missing
(B1).

## 5. The one compute-reducing mechanism already in the tree — and how late it was tried

The product-key memory (`memory_layers=`) *replaces* a layer's MLP: at layer 4 of this stack it costs
**173,056 MACs** (query map 147,456 + sub-key scoring 16,384 + value gather 9,216) against the MLP's
**1,264,896** — **7.3× fewer multiplies at that layer**, with one ninth of the parameters
(2.52 M vs 11.4 M across ten layers if it replaced them all). It is implemented, it trains, and it is the only
mechanism measured in this line whose *first* effect is to remove arithmetic.

Measured in this session, bolted on for 2,000 steps rather than trained in: v5 memory **19/40** (2 seeds) at 64
sub-keys, **22/40** at 256, **20/40** on a read layer, against the adopted base's 20/40, at **+0.026 BPB**. A
matched no-memory control showed that the one better half-steps reading was the shorter fit, not the memory
([addressed-memory record](../labs/addressed-memory-2026-10-10/README.md)).

That is a mechanism that removes 86 % of a layer's multiplies and sits inside the panel's noise — a candidate
that a quality-only metric cannot distinguish from noise, and which the process therefore treated as a failure
five times in one night.

## 6. What to do — three changes

**(1) Put the ledger on the scoreboard.** Every experiment reports, beside its quality numbers: MACs/token,
weights touched/token, store lookups/token, bytes moved/token, wall-time/token at fixed quality, and artifact
size. The format and the rule already exist in
[COMPARISON.md](../transformerless/COMPARISON.md). This is the single change that makes "better math" falsifiable
and stops B1 from repeating. It is a contract change and therefore the owner's to make.

**(2) Make exactly one mechanism load-bearing.** Not an option beside a control: the mechanism *is* the base,
the control is a transformer-shaped model at matched compute. Two candidates, both using what is already built:

- **C1 — address-and-copy output path.** `out = gate · copy(exact store) + (1 − gate) · head`. On a copy step the
  4096 × 576 head matmul and the softmax sampling disappear, replaced by one store read. This is the D19 priority
  ("grounded conversation and durable memory first") expressed as *arithmetic*, and it is exact and auditable by
  construction. First experiment: the forced-copy oracle below, then a small model whose stored facts live only
  in the store.
- **C2 — exact icosian weights.** Express the recurrence's rotations (already quaternions) as exact icosian
  `Z[phi]` coefficients: multiplication by a unit icosian is shifts and adds with no multiplies and no
  quantiser, so 32.8 % of the parameters become multiplier-free *by construction* rather than by a LUT that
  costs a memory access per weight. First experiment: train one rung with icosian-snapped rotation weights and
  compare BPB and MACs/token against the float rotation, at matched steps.

**(3) Decide which family is the project.** Either the native machinery (`addressed_attention`,
`dependent_read`, the exact store) is promoted onto the scaled, served path — which means the serving contract
and the ladder change with it — or the stack gets that machinery. Keeping both, with the novelty in one and the
scale in the other, is B4 and it has produced a year of small numbers.

## 7. The cheap decisive test, before any of the above

**The forced-copy oracle**: a serving-time hook that supplies the pointer the true value span, then score the
frozen panel. No training, ~a day of work. It answers the question the whole address thesis rests on: *when the
copy source is exact, does the value come out?*

The evidence that this is the right first question comes from this session's order-3 cycle
([pointer-address record](../labs/ptr-addr-2026-10-10/README.md)): an exact one-atom content route raised the
pointer's hit rate **0.377 → 0.54** in three seeds **at zero likelihood cost** (BPB 1.17387 / 1.16947 / 1.17109
against 1.17425), and the memory panel stayed flat (19/21/23 of 40, wrong-value 17/15/9). **The address works;
the emit does not.** If a perfect address still refuses to yield the value, then the project's addressing
mechanisms are being tested through a decoder that cannot express their result, and no amount of addressing work
will show up until the output path is rebuilt (C1).

## 8. What to stop

- **Flag sweeps on the transformer-shaped base.** The last three cycles of this lab were configuration
  variations of one 29M artifact. They produced real mechanism readings — a 39× faster memory fit, an address
  key that raises hit rate at zero cost, a control that exposed a false win — and none of them could touch B1–B5.
  That is the honest accounting of my own recent work.
- **"Geometry as an option beside a matched control"** (B3) as the *default* form of a mechanism test. A control
  is still required; the change is that the *novel* path is the base and the ordinary path is the control.
- **Reporting a cycle whose only number is a panel row.** Under the ledger rule a row of panel movements with no
  compute column is an incomplete result.

## 9. Decision requested from the owner

1. **Adopt the compute ledger** (MACs/token, weights touched, lookups, bytes, wall-time at fixed quality,
   artifact size) as a required field of every experiment and result PR, per §6(1).
2. **Choose the first load-bearing mechanism**: C1 (address-and-copy output path — my recommendation, because
   the session's own evidence implicates the emit path and it is the D19 priority) or C2 (exact icosian weights).
3. **State which family is the project**: promote the native machinery onto the served path, or port it into the
   stack (§6(3)).

Until those are answered, this lab's next piece is the forced-copy oracle of §7 — the cheapest test that can
change the direction, and one that needs no owner decision to run.
