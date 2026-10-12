# Invariant-addressed read (route to an address instead of scoring the window)
**The idea (first principles):** the read is this model's context access, it is **on the answer path** (facts removed → memory panel 0/40;
silencing the supervised read → 0/40 with zero wrong values), and it is the **only quadratic component** in the stack — every position scores
every earlier position. Two things the project already owns point at the replacement. **Geometric structure**: icosian/H4 lattice points,
prime/UOR addresses and the fixed zeta-phase tables give a *canonical* identifier per object rather than a learned similarity. **A coarse
invariant that separates classes**: the [murmurations result](https://arxiv.org/abs/2307.06586) (He–Lee–Oliver–Pozdnyakov) shows the average
Frobenius trace over elliptic curves of fixed rank oscillating across the primes with rank-dependent detail — a *family-level* signal keyed by
an arithmetic invariant, not by proximity. Spiralcore's cockpit (owner-supplied) carries the same shape as an engineering discipline:
an **addressing schema** and **symbol registry** kept separate from geometry, **context translation** that re-encodes an address across
demarcs, and **route traces** between symbolic coordinates. The mechanism to test: give every source position an **invariant address**,
**route** the query to a small candidate set by address, and read only those — `O(n)` or `O(log n)` instead of `O(n²)`, with the geometry doing
the work rather than parameterising it.

**What it replaces and why that matters:** the current read scores *all* prior positions by a learned score, so its cost grows with the window
(17.6 % of the model's MACs at context 384, **40.5 % at 4,096**) and its *selection* is a similarity judgement that the failure mode shows is
error-prone (wrong-value failures dominate the memory panel). An address is a decision made by arithmetic structure, not by learned proximity —
it can be computed, translated and compared, and it does not get more expensive as the window grows.

**Where it would live in code:** the read's score path (`geometric_stack.rs`'s `geometric_read*`, the `ReadScore`/`read_lineage` machinery, and
`ReadLineage::PhaseBinding`/`KeyPhase`/`RandomSo4` which already attach fixed structure to keys), the addressing primitives the project has
(`uor-r4-core`'s identity/occurrence types, prime addressing in the router), and — for the served form — the D11 integer engine, whose format
and candidate-join kernels already landed ([#2203](https://github.com/UOR-Foundation/uor-r4/pull/2203), [#2204](https://github.com/UOR-Foundation/uor-r4/pull/2204)).

**What has been tried, honestly:**
- **`read=l2` in the deployed model** — the flat Euclidean control of Lorentz, inherited by every arm this lab has trained; the
  [geometry audit](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6105227220) found geometric operations at **0.05 %** of the
  stack's arithmetic because of it.
- **The geometric read itself**, now measured properly ([record](../labs/geometric-read-2026-10-11/README.md)): `lorentz` against `l2`, three
  matched seeds — **ahead on every seed** (wrong-value 49 → 32 of 120, memory 48 → 59) but **below its pre-registered per-seed margin**. So the
  score is worth having and is not the whole story.
- **Addressing attempts that were aimed at the wrong place**: the identity pointer was keyed on the six tokens around the value
  (hit rate 0.003) and the line was archived; the addressed memory replaced an **MLP**, not the read, and its lookup was 1.65× slower served
  before the kernel question was even settled; `read_lineage` variants (`phase_bind`, `rot`, `carrier`) were measured as **scoring** changes and
  returned a negative — they were **never tested as addressing**, which is the distinction this brief exists to fix.
- **A neighbour-hood read** exists in the sibling line ([`starling-read`](starling-read.md), 600-cell neighbours, no softmax), at chance in rung 1.

**What success looks like:** the routed read matches or beats the full softmax read on the frozen memory panel (**wrong-value failures primary**)
at **lower** per-token cost, with the address's own structure measurably used — and, on the product path, a **higher served tokens/s** at equal
context. **What failure looks like:** the same routing with a **permuted address** (the null control below) performs as well, which would mean the
structure is decorative; or quality falls beyond the panel's noise, which would mean routing is too lossy at this scale.

**A fair test:**
- **Train it in from step 0** — the address function is part of the model's construction, never bolted onto a finished model.
- **≥3 seeds or ≥200 fresh rows**, the frozen M1 memory panel as acceptance, wrong-value failures primary with the ±3-row noise floor.
- **Two controls, both required.** (1) The **full softmax read** at the same budget — the thing being replaced. (2) A **null address**: the same
  routing machinery with the address **permuted** (a fixed random bijection over the address space), so the test separates *routing* from
  *structure*. Without the null control a routing win cannot be attributed to geometry, and that is the trap this line has fallen into before.
- **Report the compute beside the quality**: MACs/token at two contexts and **served tokens/s on the product path** (the owner's metric), never
  one without the other.
- **Bar:** at or below the control's wrong-value count by ≥5 rows on ≥2 of 3 seeds, with the null-address arm **worse** than the real one by more
  than the noise — and no headline loss beyond noise.

**Open design questions:**
- **Which invariant?** Prime/UOR address of the token identity; an icosian/H4 phase; a murmuration-style family key; or a composition (identity
  address + a coarser family key for routing).
- **How to route?** Exact address match, Hamming/phase-nearest, or a small learned scorer *over addresses* (cheap because addresses are few).
- **Granularity?** Per token, per span, or per entity — the project distinguishes occurrence, version and word identity precisely, and the read
  may need more than one of them.
- **Exactness vs learned routes:** phase or lineage tagging makes binding exact rather than learned; the negative on `phase_bind` as a *scorer*
  does not speak to it as an *address*.
