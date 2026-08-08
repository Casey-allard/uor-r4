# Research: what is measured, what is closed, what is open

R⁴ is a research programme as much as an engine, and the engine's direction is
set by measurements rather than by intent. This document records where that
programme actually stands, so anyone picking up work can see which paths are
closed, which are load-bearing, and which are still open. Every claim here traces
to a merged measurement with a pre-declared exit rule; the issue numbers are the
durable references.

## Measurement discipline

Every substantive claim in this repo arrives with a **pre-declared exit rule, a
null baseline, and a falsifier**. Negative results are recorded and kept, not
discarded — several entries below are negatives that redirected the programme,
and they are more valuable than the positives they replaced.

Long runs additionally follow the run-contract discipline in
[AGENTS.md](../AGENTS.md): compute the reachability ceiling before spending
hours, gate on the cheap instrument first, and pre-declare what each outcome
*causes*. That discipline exists because we lost days to runs whose result could
not have changed the next action.

Two further rules were earned the hard way and are worth stating up front:

- **An all-zero result across every arm is a harness bug until proven
  otherwise.** Seven instruments that could not fail have been found and
  repaired. See [Instruments that could not fail](#instruments-that-could-not-fail).
- **Absence and measured-zero must not share a representation.** A skipped
  measurement serialises as `null`, never as a zeroed row, and a pre-declared
  exit-rule verdict is suppressed rather than printed as NOT MET.

## What works and is load-bearing

The serving stack consults, in order: packed NGRAM context rows (trigram with
bigram backoff), then the graph chain with D4 exact-context precedence, then the
root prior. On natural text the induced-cover store with observed continuation
evidence is the geometry that carries the result; the legacy teacher-hash store
with teacher evidence measured 0.1% off-distribution against it.

Two changes improved results by improving **evidence quality per key**, and both
are shipped:

- **Full-width content-bearing storage** (#434, PR #465). The storage path had
  been discarding fifteen sixteenths of an already-full-width content vector.
  De-banding moved cosine-ranked retrieval MRR from 0.2348 to 0.8948 and router
  anchor accuracy from 9.3% to 11.4%.
- **The two-sided calibration gain** (#446), which is causally legitimate, sits
  at top-1 parity, and grows large at scale in bits.

> **Reading the 0.8948 correctly.** It is a **cosine-ranked** figure. For a long
> time it did not describe `get_top_resonances_native`, whose ordering is word
> overlap — and #486 found the reason: that path compared a *routing* vector
> against the stored *content* vector, so its cosine was at chance. With the
> comparison corrected, the serving path reaches 0.8763 MRR. The number was
> always real; it was unreachable at serving. See
> [geometry_selfmatch_486.md](geometry_selfmatch_486.md).

**A-mode infill serving** is validated and shipped: the FWDA forward-anchor
artifact section plus `score_candidates_infill`, `infill_fill`, and the
`r4 graph infill --skeleton` CLI (#399, PRs #416/#419). Anchors are inputs, so
the mode is immune to the drift that killed the standalone variant.

## What is closed, and why

**Standalone two-pass generation (#399).** Refuted twice. With anchors supplied
externally the channel gives +4.2pp on its live slice; with the engine supplying
its own anchors from a drafted context it goes negative (40.4% vs 41.3%), and a
strict per-step confidence gate does not rescue it (42.0% vs 43.0%). Drift is
diffuse rather than concentrated in low-confidence steps, so no gate over the
draft can filter it. At 2.11M records the inversion reproduces on a
non-degenerate configuration (16.4% vs 26.5%, predicted-anchor accuracy 0.0%), so
it is not a capacity artifact.

**Code-space subdivision as a capacity lever (#460).** Measured negative in its
strongest possible form. Raising STAGES from 4 to 5 bought exactly the
subdivision the hypothesis asked for — occupied full-code keys 47,403 → 90,824,
records per key 36.02 → 18.80, clearing the instrument gate — and Rule 1+2 top-1
came in at 25.6% ± 0.44pp against a 26.5% baseline, *below* it. The store
baseline fell alongside (26.4 → 25.4). Exact-context dominance barely responded
(98.8% → 97.1%).

That fall was originally read as thinner per-key evidence, full stop. The
codebook-fit measurement narrows it: raising the codebook's training set also
lowers records-per-key (5.04 → 4.68) and top-1 *rises* (+0.44pp,
[codebook_fit_460.md](codebook_fit_460.md)). So records-per-key is a **symptom,
not the binding quantity** — thinning is harmful when it comes from added key
*resolution*, which splits evidence that belonged together, and harmless or
better when it comes from improved *fit*, which moves evidence onto the key that
represents it. The subdivision negative stands; only its causal reading narrows.

**Construction-time stratification (#435).** Three routing designs plus an
identity argument, all against pre-declared rules; v3 mass-linear mixing reduces
algebraically to flat. The oracle-stratum edge (36.3 vs 35.2) stands as recorded
unrecovered signal, but no routing design reached it.

One reading from this track has since been revised. The cover's
non-participation — "the absolute entropy floor rejects every split at this
scale", 8 → 22 regions at mass-kept 0.0006 — was recorded as a property of the
geometry. It is a property of the shipped *configuration*: turning on the
already-implemented scaled capacity takes regions 48 → 110 on the 500k fixture
and lifts region-path held-out top-1 by 5.0pp (#460 lever 1,
[cover_scaling_460.md](cover_scaling_460.md)).

**Hopf sector transport as a router-quality lever (#422/#306).** The #306
remediation's occupancy gain (16 → 456 of 512 sectors) does not translate into
retrieval value: sector-filtered MRR 0.0045 against the pre-remediation
projection's 0.0743. Three content-aligned redesign candidates then mapped a
clean spread-versus-retrieval frontier without crossing it.

**Query-projection banding as a retrieval lever (#480).** The query side of
`retrieve_geometric_resonance` was band-only while storage has been full-width
since #465 — a real asymmetry, and the suspicion was that it stranded the adopted
de-banding gain before serving. Measured: making the shapes symmetric is worth
+0.0059 MRR and +0.0080 top-1 while costing 0.0180 of recall@20, against a +0.05
bar. Not adopted; the symmetric shape sits behind `set_full_width_query`, default
off. **#486 later explained this mechanistically**: reshaping a query vector that
was never comparable to the stored vector could not have paid at any shape.

**The lexical ranking weight (#484).** The `shared_count * 100` term was
hypothesised to be suppressing a geometric signal. Swept over five decades:
`W = 1 … 100,000` give bit-identical retrieval, because the geometric term's
dynamic range is ~0.37, so any weight above ~0.4 already yields strict
lexicographic order. NEGATIVE against the +0.05 bar. The deeper finding — that
the cosine was at chance — sent the work to #486.
[lexical_weight_484.md](lexical_weight_484.md)

**The serving path compared the wrong objects (#486).** A category error, not a
tuning or shape problem. `retrieve_geometric_resonance` built its query vector
from the **routing** path; `index_sentence_internal` stores a **content** vector.
Querying with the *exact stored sentence*, its own vector ranked at the 0.4938
percentile of all candidates — chance. Saturated stored vectors and
band-projection loss were both ruled out by measurement. Building the query with
the same `content_state_vector` construction the stored side uses takes retrieval
from 0.7179 to **0.8542** MRR with the weight unchanged, and **0.8763** with the
lexical term dropped; recall rises 0.9720 → 0.9900. Ships default OFF; adoption
is gated as #490. [geometry_selfmatch_486.md](geometry_selfmatch_486.md)

**Cayley–Dickson syntactic morphism (#400), FMM far-field (#290), granularity
(#393), E8 group-keying (#395).** Each measured dead with a scoped record; the CD
term executed 0 times out of 1,998 before its removal from the scoring path. The
surrounding modules and the `cd-compile` / `quantum-eval` commands remain.

**Interaction information at D3 scale (#458).** No measurable synergy over
context variable groups at this corpus size.

## The pattern these results draw

Every lever that added **key resolution** failed — more cover regions, a finer
code space, more stages. The changes that helped improved **evidence quality per
key**. That is the clearest signal the programme has, and it is why the open work
concentrates on evidence and estimation rather than on subdivision.

The distinction to carry forward: *which key evidence lands on* (**fit** — helps)
versus *how many keys there are* (**resolution** — has never helped). Codebook
fit is a fit lever, not a resolution lever, and when measured in isolation it
came out positive at +0.44pp — small, but on the side the pattern predicts, and
the first confirmation of it on something touching the graded code itself.

**One boundary**, from #460 lever 1: added resolution *does* help when a
structure is not merely coarse but barely partitioned at all. The induced cover
sits at 48 regions for 400,006 records, so each region's emission is close to the
global prior, and scaling capacity to 110 regions lifts region-path held-out
top-1 by 5.0pp. Not a counterexample — the resolution levers that failed all
subdivided structures already resolved enough to be predictive. **Resolution pays
up to the point where a structure predicts at all, and not past it.**

## Open, with defined work

| Issue | Question | State |
|---|---|---|
| #490 | Adopt the #486 content-vector query | **Highest-value open item.** +0.1363 MRR measured, gated on `router_reconnect` against a scored R4G1 artifact and the pinned #421 anchor-accuracy rows. Recommends the minimal variant first (query vector only, weight unchanged) so the gate can attribute any movement to one change |
| #487 | Correct #434's Spectral record | #434's Spectral row is the lexical ranking, not spectral geometry — same corpus, same caps, same function as the #484/#486 measurements. Docs-only |
| #488 | Phase timing before Gate C | Gate C is measured *not* to be where the time goes at scale, so the stages around it (cover induction, store build, transitions, emissions) need the same treatment. Needs a real corpus at scale |
| #460 | Cover split criterion and codebook fit | Both levers measured. Codebook fit +0.44pp, saturates near `N/10`, below the exit rule. Cover: the shipped absolute floor does not scale (regions 50→48→46→48 over an 8× data range); scaled capacity fixes it — regions 48→110, region-path top-1 **+5.0pp**. Serving impact capped near 0.15pp because the graph path answers ~1–3% of positions, so this arms the broad-corpus directions rather than paying today |
| #424 | Bott-Fock O(1) context fold | Ceiling measured, A/B not reachable. Long-range signal is worth +1.02pp of top-1 (two thirds order-carried); the shipped decay `>> 2` retains 16%, so the lossless upper bound as shipped is +0.16pp — one standard error. Retuning to `>> 7` would recover the ceiling. [context_horizon_424.md](context_horizon_424.md) |
| #434 | VSA / spectral geometry | Item 1 (zeta-grid at scale) shipped as full-width storage. Item 2 measured: VSA **0.0000** is a **wiring** gap — `index_corpus` populates the corpus index Spectral reads and never the `facet_store` VSA reads, so setting `geometry_type = Vsa` after `index_corpus` silently disables retrieval. **The Spectral arm's score is lexical (see #487)**, so this was never a like-for-like geometry comparison. Disposition open |
| #456–#459 | Reconstructability, block search, IPF reconstruction, estimation ladder | #456 landed (EXCT-disabled reconstruction bits per cover-sweep point, sweep schema 3). #458 landed NEGATIVE (no measurable synergy at D3 scale). #459 landed (estimation ladder). #457 (IPF reconstruction operator) open |
| #320 | Teacher upgrade (SmolLM2) | P1/P2 rehearsal recorded; migration decision open |
| #273 | Template rebase / claim register | On-hold; no implementation |

## Measurement infrastructure

Tooling exists so the results above stay cheap to reproduce and hard to fake.

**Sampled decision runs** (`R4_GATE_C_SAMPLE`) cut Gate C evaluation from 597s to
60s and report the sample size and standard error beside every rate — at 402,802
positions the standard error is 0.07pp while every exit rule we write is ±2pp,
thirty times finer than any decision needs. Selection is a deterministic stride,
not an RNG, so a sampled run is reproducible and diffable.

**The κ-keyed per-record code sidecar** (`R4_CODES_PATH`) cut the instrument from
625s to 39s by caching a deterministic computation every consumer had been
recomputing. It loads only when eight fields and a blake3 digest all agree, and
refuses itself entirely in biased-sampling mode so a partial vector can never
poison a later run.

**The `capacity_scaling` instrument** prints a saturation verdict per structure
and is meant to be run *before* trusting any measurement taken on a given
configuration. It is a hard gate before any run measured in hours.

**Gate C's per-phase wall clock** (#471) exists because "the Gate C phase took
eighty-five minutes" was an unattributable number that two proposals blamed on
two different passes, neither measured. A harness whose cost is invisible gets
optimized by argument. `R4_GATE_C_SKIP_ARMS=right_context` then drops the
whole-corpus pass the profile blamed — 62.9% off the Gate C phase — and the arms
that depend on it are reported **absent** rather than zeroed.

That profile went on to overturn its own issue twice: the table builds it named
were 0.8% of a sampled run, and the 2.11M follow-up found Gate C costs 51.94s
there, not 85 minutes, with per-record cost *falling* as the corpus grows. The
85 minutes was never Gate C — which is why #488 exists.
[gate_c_arm_skip_471.md](gate_c_arm_skip_471.md)

## Instruments that could not fail

An instrument that cannot fail is indistinguishable from one that passes. Seven
found so far; the pattern is common enough to check for first.

- `kappa_reproduction.rs` **silently skips** without the llama2.c checkpoint, so
  κ obligations are discharged against the committed artifact fixture instead.
- `cover_scaling.rs`'s fallback partition was a tail split, so three of four
  sizes skipped and it printed PASS over one row.
- The `scaled-k0` arm passes **degenerately** on any ~500k corpus, because the
  default capacity reference equals the fixture's train count.
- `run_ablation_benchmark` asserted a hard-coded constant against itself, timed
  the wrong function, and reported one quantity twice.
- **#471, found by a machine:** two win/loss cross-tabs stayed flat and published
  all-zero rows on a skipped run — inside a change written specifically to
  prevent that pattern, missed on code read, caught by the equivalence test on
  its first execution.
- **#484:** three separate records (#434, #480, #484) reported the identical
  metric triple 0.6240 / 0.7179 / 0.9720 under three different framings before
  anyone noticed they were the same measurement. **Identical metric triples
  across supposedly different arms are a signal, not a coincidence.**

**Guards that work.** Pin a known reference row and assert against it. Assert the
control arm is non-degenerate *before* comparing. Read a zero row with the full
ranked list, not a truncated recall, and print the random median beside it. When
a matching arm scores near zero, query with the target *itself* — if identity
also fails it is the signal, if identity succeeds it is the probe.

## Running the harnesses

All are `#[ignore]`d and run explicitly with `-- --ignored`. Add `--release` for
anything corpus-scale. The default corpus is the committed fixture unless
`R4_CORPUS_META` / `R4_CORPUS_RECS` say otherwise.

**The cheap gate — run this before any long run:**

```bash
cargo test -p uor-r4-graph-certify --test capacity_scaling -- --ignored   # ~12 min
```

**Certification and capacity** (`crates/uor-r4-graph-certify/tests/`):
`capacity_scaling`, `cover_scaling`, `strata_construction`, `evidence_sparsity`,
`two_sided_context`, `long_range_ceiling`, `interaction_information`,
`estimation_ladder`, `compile_quality_sweep`, `anchor_infill`,
`m1_induced_forward`, `router_reconnect` (needs `R4_SCORED_R4G1`), `r4g1_cd_ab`,
`e8_membership_ab`, `e8_rvq_experiment`, `e8_store_experiment`, `e8_dump_bundles`.

**Router** (`crates/uor-r4-router/tests/`): `hopf_retrieval_quality`,
`hopf_sector_occupancy`, `memory_lift_corpus`, `query_projection`,
`geometry_ablation`, `geometry_selfmatch`, `lexical_weight`,
`zeta_state_retrieval` (needs `R4_ZETA_ARM=1`).

**Core and root**: `kappa_reproduction` (Gate E), `codebook_fit`, `convert_r4g1`,
`smollm2_adapter` (needs `SMOLLM2_SOURCE`), `induction`, `check_graph`,
`status_policy`, `gate_c_arm_skip`.

Example:

```bash
cargo test --release -p uor-r4-router --test geometry_selfmatch -- --ignored --nocapture
```

Each harness's module header states what it measures, its arms, its null and its
pre-declared exit rule. Read it before trusting the output — several encode
scope limits that do not survive being quoted out of context.
