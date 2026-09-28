# B1: finite-group tracking lanes — Stage A (A5 word problem) and Stage B (inside the stack)

2026-09-27 · Lab 1 (Claude) · [ROADMAP](../../ROADMAP.md) track T1(b) · References #820, #973

**Status** (corrected 2026-09-28 after an independent review of #1442):
- **Stage A:** complete. Its pre-registered gate PASSES.
- **Stage B:** the pre-registered subject was quaternion (2I) lanes, and it **FAILS** its per-seed gates. The pre-registered transformer control is **NOT_RUN**. The reflection-pair arm was added after Stage A; it passes the same gates, which makes it an exploratory result, not a pre-registered pass. §5 and §6 spell out what the kill rule implies.
- Every number below comes from source `09e537a4` and binary SHA-256 `c585f965…` (`tracking-lanes`, release).
- The evidence is in [`docs/evidence/b1-finite-group-lanes-2026-09-27.json`](../evidence/b1-finite-group-lanes-2026-09-27.json): run hashes, report-root manifests and gates.
- Stage A is a synthetic word problem. Stage B mixes synthetic A5 words into text training. **Neither is a language-capability result**: the text gate measures only that tracking costs no text quality.

Labels:
- **Measured:** Rust runs on the owner's M1 at 2 threads.
- **Derived:** arithmetic from measured records.
- **Literature.**

## 0. Findings

1. **The gate passes.** Token-conditioned lanes trained only on words of length ≤32 track the A5 word problem exactly to length 4,096 (128×) once snapped to a group automaton. Non-commutative lanes succeed in 17 of 18 seed×rate runs; commutative and frozen lanes stay at chance in 18 of 18 (*Measured*, §3).
2. **The capability belongs to the finite group, not to quaternions.**
   - The ordinary control is two learned reflections per token (DeltaProduct-style). It learns A5 in 9 of 9 runs; quaternion lanes do so in 8 of 9.
   - Both minimise to the same 60-state automaton, which is A5 itself.
   - By the pre-registered kill rule, the **quaternion-specific serving claim is retired**: 2I lanes do not serve more cheaply than the ordinary control.
3. **Training lands on exact icosahedral geometry either way** (*Measured*/*Derived*, §4).
   - Quaternion lanes converge to 2I, the 600-cell: every transport's real part is within 0.0001–0.016 of an exact 2I value.
   - Reflection pairs converge to the icosahedral rotation group acting on R³ (plus a fixed axis). Their trace signature (1+φ)/4 = 0.6545 is that of a 72° icosahedral rotation.
4. **Snapping, not float precision, makes long lengths exact.**
   - A float reflection-pair model fell to 0.039 accuracy at length 4,096 (lr 0.03, seed 1).
   - Its snapped automaton stayed at 1.000.
   - The served automaton is one byte of state and two table reads per token: no multiplier, no float, no weight map.

5. **Stage B: the pre-registered 2I arm fails; the reflection-pair arm passes as an exploratory result** (*Measured*, §6). The stack is recurrence-primary: `rrar`, width 128, trained on text plus A5 words.
   - With reflection-pair lanes, the stack tracks A5 at 1.000 at every in-context position in 3 of 3 seeds.
   - In each of those seeds, 3–4 of the 8 lanes snap to an exact 60-state automaton that is perfect at length 4,096. The other lanes do not close.
   - Text NLL moves by +0.036, −0.050 and +0.009 nats against the lane-free stack, a mean of −0.002, inside the pre-registered 0.05.
   - Quaternion lanes **fail** on reliability: seed 3 never learned A5 (0.039) and cost +0.121 nats. The other two seeds were exact and within the gate.
   - Phase lanes fail as theory predicts.
   - The lane-free stack reaches only 0.008–0.160 at position 128.

**The pre-registered kill rule applies to 2I lanes.** It is the review §9.3 rule: "keep the architecture with the better ordinary lanes; retire the geometric claim from the serving path; keep geometry as codebook and addressing infrastructure". 2I lanes have no serving-cost advantage over the ordinary control, and they cost more than 0.05 nats in one seed. So:
- **the 2I geometric-state serving claim is retired;**
- **the ordinary reflection-pair lanes are the kept mechanism,** exploratory pending a fresh pre-registered replication;
- geometry stays as codebook and addressing infrastructure (T2).

An earlier revision narrowed the retirement to "quaternion-specific" without flagging it; that narrowing is withdrawn.

What survives is the finite-group state *mechanism*: learned non-abelian lanes compile to an exact
automaton of the icosahedral group, whichever parameterisation learns it. Diagonal (commutative) recurrences provably cannot represent this
state, and constant-depth log-precision transformers are conjectured unable to track it at arbitrary
length (TC⁰ ≠ NC¹; *Literature*, arXiv 2404.08819). Whether such lanes earn a place in a real language
model is Stage B's question.

## 1. Question and pre-registration

From ROADMAP §4.1(b), fixed before any run:
- **Stage A gate:** 2I lanes are exact at 4,096 when snapped in ≥2 of 3 seeds, with commutative lanes at chance.
- **B1 kill rule** ([review §9.3](first-principles-review-2026-09-25.md)): if 2I lanes fail to match the strongest non-diagonal control's tracking *at lower serving cost*, retire the geometric-state claim from the serving path.

## 2. Setup (*Measured* configuration)

**Task.**
- The A5 word problem over three generators of orders 2, 3 and 5 in A5: canonical 2I roots with real parts 0, ½ and φ/2, the first generating triple in canonical order.
- Target at every position: the A5 class (±-pair) of the running product. There are 60 classes, so chance is 0.0167.
- Labels come from the repository's exact 2I Cayley table (`uor_r4_core::…::group_table`), in `compose_context_roots` order.

**Lanes.**
- K = 8 per model, token-conditioned, with no additive input and no decay: `h_t = M[token] h_{t−1}`, starting from `h_{−1} = 1`.
- Free parameterisation: standard-normal raw parameters, not near-identity.
- Arms:
  - **quaternion:** `x ↦ q x`;
  - **reflection_pair:** `x ↦ H(v₂)H(v₁)x = p x r`;
  - **phase:** a unit complex number, which is commutative;
  - **frozen:** the identity.
- Read-out: MLP 32→128→60.

**Training.**
- AdamW, 4,000 updates, batch 64.
- Length curriculum from 4 to 32 over the first 60% of updates; cosine decay to 10%.
- Learning rates 0.003, 0.01 and 0.03; seeds 1–3.
- Every arm with the same seed sees identical words.

**Evaluation.** 256 fresh words at each length from 32 to 4,096, the same words for every arm. The score is accuracy at the final position.

**Serving** (`LaneAutomaton`):
- Close the lane's per-token 4×4 transports into a group by breadth-first search, merging products within Frobenius distance 0.5. Distinct 2I elements are ≥1.236 apart.
- Fit a read-out table by majority over 512 words of length 32.
- Serving is one Cayley-table read per token plus one read-out read.
- Moore partition refinement reports the minimal equivalent automaton.

**Source.** `crates/uor-r4-training/src/stack_tracking.rs` and `examples/tracking-lanes.rs` (`a5` mode). Five focused tests pass:
- the tensor Hamilton product;
- float states equal the served transport products for all four kinds;
- 60 paired A5 classes with a generating triple;
- exact generators close to 120 elements, minimise to 60 and serve exactly at length 1,000;
- the stack split equals the stack forward.

## 3. Results (*Measured*)

The table shows final-position accuracy at length 4,096 for the float model and for the snapped automaton. Runs are seeds 1, 2 and 3 at each rate.

| Arm | lr | Float @4096 | Snapped @4096 | Automaton order |
|---|---|---|---|---|
| quaternion | 0.003 | 0.012, **1.000, 1.000** | 0.023, **1.000, 1.000** | 1 (collapsed to identity), 120, 120 |
| quaternion | 0.01 | **1.000 ×3** | **1.000 ×3** | 120 ×3 |
| quaternion | 0.03 | **1.000 ×3** | **1.000 ×3** | 120 ×3 |
| reflection_pair | 0.003 | **1.000 ×3** | **1.000 ×3** | 60 ×3 |
| reflection_pair | 0.01 | **1.000 ×3** | **1.000 ×3** | 60 ×3 |
| reflection_pair | 0.03 | 0.039, **1.000**, 0.996 | **1.000 ×3** | 60 ×3 |
| phase | all | 0.004–0.023 | 0.016–0.027 | 8–16 (tolerance-quantised, meaningless) |
| frozen | all | 0.016 | — | — |

**Training time**, 4,000 updates at 2 threads:
- quaternion 15.6–21.7 s;
- reflection pair 67.5–83.1 s;
- phase 18.1–29.7 s;
- frozen 13.1–20.5 s.

These are from the committed rerun (`final-a-*`), which ran alongside the storage migration.

The reflection pair is slower here because of its two scans and extra products, which is an implementation cost, not an intrinsic one.

**Minimal automata.**
- The exact-generator test shows that 2I read out by A5 class minimises to 60 states: `s` and `−s` share outputs.
- The final rerun on committed `09e537a4` (`final-a-lr*`) records the minimal order for every run: **60 for every exact run**, quaternion and reflection pair alike.
- It reproduces the pre-commit grid exactly. Every accuracy is identical, including the lr 0.03 seed 1 reflection-pair float drift (0.840 at 512, 0.039 at 4,096).

## 4. Geometry the training found (*Measured*/*Derived*)

**Quaternion lanes.** `max_trace_deviation` is the largest distance from any closure element's real part to the nearest 2I value in {0, ±½, ±φ/2, ±1/(2φ), ±1}. It is:
- 0.0029–0.0158 at lr 0.003;
- 0.0001–0.0020 at lr 0.01;
- 0.0016–0.0049 at lr 0.03.

The learned group is 2I up to conjugation. This is forced: A5 has no faithful representation in SU(2), and every finite subgroup of SU(2) that maps onto A5 is conjugate to 2I.

**Reflection pairs.** Their transports are SO(4) rotations `x ↦ p x r`. The near-constant deviation of 0.1538–0.1545 is |(1+φ)/4 − φ/2|:
- (1+φ)/4 is trace/4 of a 72° rotation of R³ ⊕ R, namely (1 + 2cos 72° + 1)/4;
- the other classes give 0 (180°), ¼ (120°) and (2−φ)/4 (144°).

The ordinary control therefore learned the icosahedral rotation group of R³, with the fourth axis fixed.

## 5. Reading against the pre-registration

- **Gate:** PASS at every rate (2 of 3, 3 of 3, 3 of 3), with phase and frozen lanes at chance.
- **Kill rule:** the reflection-pair control matches tracking (9 of 9 against 8 of 9), and its automaton minimises to the same 60 states. 2I lanes therefore have no serving-cost advantage. Under the pre-registered rule, the geometric (2I) serving claim is retired and the better ordinary lanes are kept (see §0).
- **Survives:** exact finite-group state, learned by any non-commutative lane and served as a table automaton. Quaternion lanes remain the cheaper parameterisation to train here, and the more robust in float at long length: all 8 learned quaternion runs score 1.000 in float at length 4,096, against 7 of 9 reflection-pair runs. That is an engineering preference, not a claim.

## 6. Stage B: inside the stack on text

Pre-registered in ROADMAP §4.1(b): LM gate within 0.05 nats of the lane-free stack at equal tokens over 3 seeds, and A5 accuracy inside the context ≥0.99 with lanes.

**Design** (`tracking-lanes mixed`, `TrackedStack`):
- The geometric stack is recurrence-primary: `rrar`, width 128, 4 heads, MLP 384, context 128, Dot reads, the stack's own rotation. Its vocabulary is 4,096 text tokens plus 3 A5 tokens.
- An optional lane side channel, zero-initialised like any new residual branch, is added to the embeddings. The lanes see every token.
- An auxiliary A5 read-out sits on the final states.
- Each update is one text batch plus one A5 batch, with A5 lengths ramping from 8 to 128.
- Arms: none, quaternion, reflection_pair and phase, 3 seeds each.
- Data: the #1017 reference's token store (`issue-1017/tokens/train.u16`, 120M tokens, ids < 4,096) and its `dev.u16`.

**Results** (*Measured*). 1,500 updates, 16 text and 16 A5 windows per update, lane learning rate 0.03. Development NLL is measured on 512 evenly spaced windows of `dev.u16` (65,536 targets). A5 accuracy is the stack's auxiliary read-out on 256 fresh words of length 128.

| Arm | Seed | Dev text NLL | Δ against none, same seed | A5 at position 8 / 64 / 128 | Snapped lane (order → minimal) @4,096 |
|---|---:|---:|---:|---|---|
| none | 1 | 2.6660 | — | 0.973 / 0.031 / 0.016 | — |
| none | 2 | 2.7290 | — | 1.000 / 0.684 / 0.160 | — |
| none | 3 | 2.6811 | — | 0.992 / 0.004 / 0.008 | — |
| quaternion | 1 | 2.6780 | +0.0120 | 1.000 / 1.000 / 1.000 | 120 → 60, 1.000 |
| quaternion | 2 | 2.6769 | −0.0522 | 1.000 / 1.000 / 1.000 | 120 → 60, 1.000 |
| quaternion | 3 | 2.8022 | **+0.1211** | 1.000 / 0.320 / 0.039 | no lane closed |
| reflection pair | 1 | 2.7017 | +0.0357 | 1.000 / 1.000 / 1.000 | 120 → 60, 1.000 |
| reflection pair | 2 | 2.6788 | −0.0502 | 1.000 / 1.000 / 1.000 | 60 → 60, 1.000 |
| reflection pair | 3 | 2.6904 | +0.0093 | 1.000 / 1.000 / 1.000 | 60 → 60, 1.000 |
| phase | 1 | 2.7548 | +0.0888 | 0.984 / 0.016 / 0.016 | 6 (meaningless), 0.016 |
| phase | 2 | 2.8235 | +0.0944 | 1.000 / 0.844 / 0.348 | 7 (meaningless), 0.012 |
| phase | 3 | 2.7349 | +0.0538 | 0.883 / 0.023 / 0.004 | 6 (meaningless), 0.023 |

**Verdict under the gates as pre-committed.** The evidence assembler encoded them per seed before any Stage B result: every seed's Δ ≤ 0.05 nats, and every seed's A5 accuracy at position 128 ≥ 0.99.
- **Quaternion (2I) lanes, the pre-registered subject: FAIL.** Seed 3's lanes did not learn A5, and that seed cost +0.121 nats. The mean Δ of +0.027 would pass, but the pre-committed rule is per seed.
- **Reflection-pair lanes, added after Stage A: pass on both gates, exploratory.** The worst Δ is +0.036 and the mean −0.002; A5 is 1.000 in 3 of 3 seeds.
- **Transformer control** (pre-registered: "reported at the same lengths"): **NOT_RUN** in this revision.
- **Phase lanes: FAIL,** as predicted for commutative lanes.
- **Lane-free stack:** its body alone tracks A5 only at short prefixes, and inconsistently (0.160 at position 128 at best).

**Observations and scope.**
- **Failed lanes cost text quality; learned lanes do not.** Wherever lanes failed to learn the group (quaternion seed 3, every phase seed), text NLL rose by 0.05–0.12. The lane-free stack trains on the same A5 data without that cost. Lanes that learned the group cost nothing measurable.
- **Seed noise.** Seed-to-seed spread of the lane-free stack itself is 0.063 nats, so single-seed differences below that are noise.
- **Scope.** This shows exact non-abelian state and text modelling coexisting in one small model: a 1,393,604-parameter stack, 1,500 updates, context 128. Side parameters total 7,740 with the A5 read-out alone, 143,004 with quaternion lanes and 274,172 with reflection-pair lanes; the totals include the read-out. Lane tables are read one row per token. It says nothing about state tracking in natural language, which no probe here tests.

**Snapping limits** (found in the independent review):
- `snap` rejects only ambiguous merges. It does not verify group structure, so lanes that are not a group can still close into small automata (orders 1–22 here). The table marks those as meaningless, and they score at chance on fresh words.
- The served Stage A reflection-pair automaton at lr 0.03, seed 1 merged at a Frobenius distance of 0.394, against the 0.5 tolerance. That is safe for its group, whose elements are about 1.66 apart. Every other exact run merged at ≤0.046.
- No report root saves the automaton tables, so T3 has no artifact to port yet.
- Stage C will add a permutation-consistency check and an exhaustive check against `group_table`, save the tables, and record merge distances in the evidence.

**Execution note.** The sequential root `final-b-grid` was stopped by the director, with the owner's approval, after 4 complete runs (none ×3, quaternion seed 1), so that the remaining 8 could run as three parallel streams. Those streams are the sealed roots `final-b-quaternion23`, `final-b-reflection` and `final-b-phase`. `final-b-grid` stays unsealed as an interrupted attempt. Its four run records are complete, and each one's SHA-256 is in the evidence. Seeds and data are identical across roots.

## 7. Resources

- **Build:** about 3 minutes on a cloned warm cache, now the per-lab cache `/Volumes/UOR-Workspace/BuildCaches/claude` (APFS copy-on-write). Incremental rebuilds take 30–40 s; the final clean build of `09e537a4` took 34 s.
- **Tests:** 5 focused tests, about 31 s.
- **Stage A:**
  - pre-commit grid: 36 runs, about 12 minutes at 2 threads;
  - final rerun: 36 runs, 22 minutes at 2 threads (23:47–00:09 UTC), concurrent with the storage migration;
  - peak RSS 1.08 GB.
- **Stage B:**
  - pilots: about 11 minutes;
  - sequential grid: 4 runs, 41 minutes at 2 threads;
  - parallel streams: 8 runs, 50.5 minutes wall at 6 threads (00:50–01:41 UTC);
  - RSS about 0.8 GB per stream.
- **Model slot:** held 23:47–00:50 and 00:50–01:41 UTC.
- **Storage:** all reports are under `/Volumes/UOR-Workspace/uor-r4-lab/claude-b1-20260927`; nothing was written to the internal drive beyond source.
