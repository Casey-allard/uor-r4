# Prime-zeta codebook

**Status: ACTIVE.** The constructor is built and qualified, and its table is wired in as the start for new training lineages. No trained artifact has started from it yet: rung 0 and K16 began from a seeded 16-vector table. It is the baseline arm of the 19 October pre-flights and the fallback cell inside the [address-structured table](./address-cells.md). No run has yet trained it against its two matched controls. That comparison is `NOT_RUN` and held ([construction record](../../archive/labs/prime-zeta-construction-2026-10-14/README.md)).

Labels used below: **proof** means derived from definitions or code, **measurement** means a recorded run, **hypothesis** means an untested idea (see [the charter](../../CHARTER.md), section 4.6). Unfamiliar project terms (K16, A8, admission, NOT_RUN) are in the [glossary](../start/glossary.md).

## In plain words

A language model has to start every word somewhere. The usual start is random numbers. In this project the random start gave 4,096 words only 16 different starting positions, so many words were indistinguishable. This mechanism gives every word its own fixed address instead. A word's place in the vocabulary picks a prime (2, 3, 5, and so on), and the prime is turned into a set of angles using the known frequencies of the Riemann zeta zeros. Everything is computed once, offline, and stored as a table of small whole numbers. Whether the zeta angles help beyond simply keeping words distinct has not been tested.

## What it replaces in a transformer

The token embedding table. A transformer starts each token with a random vector and learns it. Here the starting table is constructed, fixed and made of integers. The model can still change it later through its learning moves ([discrete learning](./discrete-learning.md)).

![Left: a ladder of primes feeding eight clock dials, one per zeta zero, each hand turned by that zero's frequency times the log of the prime; an arrow drops the word onto a 16 by 16 board of cells. Right: before and after piles of 4,096 dots, 16 piles of 256 for the random start against 256 cells of 16 for this table.](../figures/mechanisms/prime-zeta-codebook.svg)

*A word's prime sets eight dial angles, and the angles pick a cell on a 16 by 16 board. The random start put 256 words on each of 16 patterns. This table puts 16 words in each of 256 cells, all distinct.*

## How it works

**Prime address (Definition).** Token $v$ with vocabulary id $\mathrm{id}(v)\in\{0,\dots,V-1\}$ gets the prime $p_{\mathrm{id}(v)+1}$. The vocabulary maps one-to-one onto the first $V$ primes, and $p_{4096}=38873$.

**Zeta phases (Definition).** Take the first 512 imaginary parts $\gamma_k$ of the zeta zeros from a pinned table of published values. The phase and phasor of channel $k$ are
$$\theta_k(p)=\gamma_k\ln p \pmod{2\pi},\qquad z_k(p)=\cos\theta_k+i\sin\theta_k .$$
Using a finite table of these constants does not require solving the Riemann hypothesis.

**Quantize and place (Definition).** Four consecutive channels per 8-dimensional block form a vector in $\mathbb{C}^4$. It is split into two $\mathbb{C}^2\to\mathbb{R}^4$ spinor pairs and stored as integers (`PhaseQ29`, `UnitS3Q30`). The pairs are embedded in the four orthogonal 2D root planes of a 16-root orbit of the E8 lattice. Each block takes the orbit root with the largest exact integer inner product.

**Cells (Definition).** A token's *writer cell* is the pair (root of block 0, root of the last block). That makes a $16\times16=256$ cell grid. Two passes then run:

1. A **quota pass** puts exactly $V/256=16$ tokens in each cell.
2. A **uniqueness pass** gives interior blocks distinct roots, so all 4,096 full vectors differ.

**Where floating point lives.** The logarithm, sine and cosine are confined to this one-time host step, under the October 14 zeta-zero exception. The output is an integer table loaded as the model's starting table (`pinned_table_v1`). The served model sees only integers.

**Two matched controls share the same pipeline.** *TangentNull* shifts every $\gamma_k$ by 1.0. *NormalNull* permutes the primes and moves the phases off the critical line.

**An honest caveat (proof, by reading the code).** The quota and uniqueness passes also run on both controls. So the zero collisions come from those passes, not from prime phases. The seed does not change the PrimeZeta table: seeds 0, 1 and 2 give identical tables.

**Rule check.** No softmax or squashing (D23). No matrix multiplication; each token is scored by a few 8-element integer inner products, once, on the host. This is data preparation: no model quantity is computed there (D24, D30). The compiled integer module contains 0 floating-point instructions (D11, D32). Float is allowed here only by the October 14 exception.

## What has been tried

Almost all of the evidence concerns construction and admission. No row measures language quality. "Admitted" means a target word sits in one of the few cells the reader opens ([admission](./admission.md)).

| Date | What was run | Result | Verdict | Record |
|---|---|---|---|---|
| Jul 2026 | First prime router with hypervector algebra (UOR-Foundation/uor-r4#1, #6, #9) | Structural work only; no model quality recorded | inconclusive | [#9 issue record](../../archive/history/github-archive-2026-10-15/markdown/0009-feat-implement-proof-carrying-semantic-addresses-vsa-algebra.md) |
| 27 Jul–3 Aug | Phase-clock digits as an address channel in the old graph-compiler store (#392) | All rows below the shipped row: 12.5–16.7% top-1 against 34.7% | negative, for that store | [design](../../archive/phase_clock_address_design.md) |
| 3–6 Aug | Re-check of the zeta-grid retrieval headline (#434, #440) | The 0.89 MRR headline was re-attributed to vector fullness, not zeta geometry: full-width content vectors scored 0.8948, above zeta's 0.8932 (#440). The stored path kept 1/16 of each vector and cost about 0.66 MRR (0.2348 to 0.8948) | negative | [#440 control run](../../archive/history/github-archive-2026-10-15/markdown/0440-test-router-434-band-matched-controls-zeta-claim-re-attribut.md), [de-banding audit](../../archive/debanding_audit_434.md) |
| 26 Aug | Source-free prime-route attention (#958) | Mechanics pass (stages 1–3). Product probe and teacher comparison `NOT_RUN` | inconclusive | [record](../../archive/prime_route_attention_qualification_958.md) |
| 5 Sep | Occurrence selection with prime postings and zeta phase paths | First token 15/32 to 20/32; Rust exact still 0/32; phase term not isolated | positive, unattributed | [record](../../archive/native_geometric_occurrence_selection_973.md) |
| 28 Sep | Review of prime and zeta features against plain tabulation | No predictive gain recorded; kept as exact identity | negative | [toolbox](../../archive/integration/geometric-toolbox-2026-09-28.md) |
| 14 Oct | Diagnosis of the random start | Symbolic analysis: exactly 16 full-vector patterns, each shared by 256 tokens, for every seed at this shape | proof | [diagnosis](../../archive/labs/a2-pilot-diagnosis-2026-10-14/README.md) |
| 14 Oct | Construction qualification, bounded GPU controls, zero training | Bit-exact on two CPU architectures; 4,096/4,096 unique vectors, 256/256 cells; 4,090/4,096 tokens differ from TangentNull and 4,096/4,096 from NormalNull; 17 base plus 6 PrimeZeta controls pass with 0 applied updates; 0 float instructions; pod cost about $0.13 | positive | [record](../../archive/labs/prime-zeta-construction-2026-10-14/README.md) |
| 18 Oct | Wired in as the start for new lineages; host census on the 14,287-row development panel | Distinct vectors 16 to 4,096; reachable rows (alias ceiling) 1,986 to 14,287; tie mass 12,301 rows to 0. Host check only | positive | [wiring](../labs/injective-codebook-init-2026-10-18/README.md), [census](../labs/coverage-census-2026-10-18/README.md) |
| 19 Oct | Zero-update pre-flight at K=8 | Aliasing gone, but words spread over all 256 cells, so only about 8 of 256 target cells are opened. Admitted 127 of 3,847 against a gate of 500. Run not started | diagnostic stop | [pre-flight](../labs/injective-preflight-and-c2-qualify-2026-10-19/README.md) |

**Reading these honestly.** The last row closes the configuration "spread words evenly at K=8". The record calls it an instrument-reachability problem, not a failure of prime-zeta geometry. The older retrieval results ran on a previous store, and one of them was a banded-storage artifact. None tests zeta phases inside a token codebook on the current stack, and the #958 and #973 results never isolated the zeta term. D22 applies: these close configurations, not the mechanism.

## What has never been tested

- **Trained attribution.** PrimeZeta against TangentNull, NormalNull and a quota-only injective control, at matched budget, with at least 2 seeds or 200 fresh rows.
- **Meaning in the phases.** Primes are assigned by token id, which is arbitrary, so the phases encode identity and not meaning. No test checks whether phase-near words behave alike (hypothesis).
- **Any language effect.** Every result is construction, census or zero-update admission.
- **Seed variance.** The table is a function of token id alone. Seed diversity needs a declared extra axis, such as a channel offset or batch order.
- **Special tokens** (start, end, pad) with reserved small primes or fixed points.
- **Zeta phases as a position encoding** on the discrete stack.

## What comes next

The mechanism is ACTIVE, so this is a plan, not a reopening. A trained-in comparison needs its own pre-registered sheet that passes `TEST FITNESS: FIT`, plus the owner's training release. The sheet should report admission, top-1 and top-8, separated hinge losses and fresh transfer. Since the address-structured table now places words by their predecessors, a second question is whether this table still earns its place as the fallback, or whether a random injective table would do the same job.

## Also known as

Prime-rank indexing plus zeta-phase routing; prime-zeta complex routing; `CodebookMode::PrimeZeta`; the "P8" baseline arm of the 19 October pre-flights; `pinned_table_v1` (the loader that carries it); the complex-plane zeta-zero exception (October 14); ancestors: the prime router, prime-route attention, the 16-window zeta projection and the phase-clock design.

## Code

- [`crates/uor-r4-discrete/src/prime_zeta.rs`](../../crates/uor-r4-discrete/src/prime_zeta.rs): the constructor, the quota and uniqueness passes, both controls and the zero-float PTX check.
- [`prime-zeta-qualify.rs`](../../crates/uor-r4-discrete/src/bin/prime-zeta-qualify.rs) and [`prime-zeta-codebook.rs`](../../crates/uor-r4-discrete/src/bin/prime-zeta-codebook.rs): qualification and table export.
- [`training.rs`](../../crates/uor-r4-discrete/src/training.rs): the `pinned_table_v1` loader and the checkpoint-0 check.
- [`coverage_census.rs`](../../crates/uor-r4-discrete/src/coverage_census.rs): distinct vectors, alias groups and tie mass.
- [`crates/uor-r4-core/src/zeta_zeros.rs`](../../crates/uor-r4-core/src/zeta_zeros.rs): the 512 pinned zero ordinates.
- Pre-registration: [`m1-prime-zeta-complex-routing-2026-10-14.md`](../../archive/harness/prep/m1-prime-zeta-complex-routing-2026-10-14.md).

## Related mechanisms

[address-cells](./address-cells.md): builds on this table and uses it as the fallback for words with no placed predecessor. [discrete-learning](./discrete-learning.md): loads this table as its starting codebook. [admission](./admission.md): its gates exist because spreading over 256 cells starves the reader. [spiralcore-fbs](./spiralcore-fbs.md): supplies the E8 orbit roots. [route-read](./route-read.md), [flock-read](./flock-read.md) and [relation-read](./relation-read.md): the reads that consume these addresses. [exact-memory](./exact-memory.md): also uses primes, as exact keys rather than phase sources. [hopf-observation](./hopf-observation.md): shares the unit-quaternion helpers. [tokenizer](./tokenizer.md): fixes the vocabulary size $V$.
