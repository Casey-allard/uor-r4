# Barrier assessment, 5 October 2026

This is a whole-project assessment of UOR-R4, run read-only at `71b54adf`. It covers the history, the active and retired mechanisms, the roadmap, the recent evidence, external literature and the mathematical programme. It ends with an adversarially vetted plan for the in-context retrieval and geometric attention barrier the project has been stuck on since about July.

**Method.** A workflow of 30 agents:
- seven parallel surveys;
- one synthesis;
- five expert proposals (geometer, number theorist, recall/scaling architect, inference systems, first-principles outsider);
- three adversarial reviewers per proposal (generalization; D11 serving and cost; process and evidence);
- a judge, then a completeness critic.

The labels are those of [formal_vocabulary.md](../../formal_vocabulary.md): **P** proof or exact by construction, **M** measured at a stated scope, **H** hypothesis.

## Files

| File | What it holds |
|---|---|
| [final-assessment.md](final-assessment.md) | The plan: executive summary, integrated mechanism, sequenced experiments with decision and kill rules, stop list, roadmap and process changes |
| [completeness-critique.md](completeness-critique.md) | What the plan misses or overstates. **Read it together with the plan.** |
| [synthesis.md](synthesis.md) | The knowledge map: eras, mechanism and evidence ledgers, barrier diagnosis, constraints |
| [proposals-and-reviews.md](proposals-and-reviews.md) | The five proposals and all 15 adversarial verdicts |
| [surveys/](surveys/) | The seven source surveys |

## Headline findings

1. **No proposal survived review.**
   - The reviewers refuted every one: Clocked Icosian Lineage, Lineage-Exact Admission, the quaternion n-let lineage read, the Icosian Lineage Cache, and Offset n-let lineage.
   - The integrated plan is built only from the parts the reviewers marked salvageable.
2. **"Canonical token lineage" (F2, #1701/#1704) is real but not yet shown to be geometric.**
   - P: a pure-imaginary predecessor code is self-incoherent, because `<g·x, x> = Re(g)|x|²`.
   - M: on a synthetic bench, 1.36M parameters, two seeds, it makes recall robust.
   - It matches known prior art: RWKV token shift, H3/Based/Mamba short convolutions, and the Canon layers.
   - Before calling `j` canonical, it must beat an identity shift, a random fixed SO(4) and a learned shift or convolution. If those tie, keep it only as the cheapest exact D11 form.
3. **Predecessor identity has been rediscovered three times without being carried forward:** ADR-0003 in August, Codex #1580 on 1 October, and #1701 on 4 October. That is a process failure.
4. **Measure before building.** Step 0 is training-free:
   - a failure taxonomy of the open panel: does recall even limit it?
   - a tokenizer and format audit of key tuples;
   - a teacher-forced rehearsal probe;
   - a D11 read-share profile.
5. **The decisive chat test is the F2 pod A/B already in flight, plus a recall-dose control.** The fine-tune mix is about 0.8% MQAR, and the 6-layer `rararr` reaches 1.000 on the bench without the shift. So "dose, not mechanism" is a live alternative.
6. **Stop:** score-geometry swaps while key content is unchanged, KV-codec and LSH work at context 384, single-seed verdicts, and benches whose layout makes passing a tautology.

## Scope notes added by the Claude lab after review

- **"Copy capped at 0/33"** is the DeepSeek disjoint panel's copy cell (#1552). The tokenizer boundary is `m` (79) vs `Ġm` (283) under `add_prefix_space:false`. The D19 grounded-session Copy category scores 29–32/33 at 29M and 96M. Step 4 is therefore scoped to the disjoint panel, not the D19 sessions.
- **The critique's point 2 is a design input for Step 2.** The D19 query ends in "{k} is", so the predicting position holds "is", not the key. Step 2 must include an arm with lineage on both queries and keys, `q_t + j⁻¹q_{t−1}` with `k_t + j·k_{t−1}`.
- **Evidence still pending at the time of writing:** the F2 pod A/B (base + Arm C, key shift off vs add, D19 with the sieve off).

References #820.
