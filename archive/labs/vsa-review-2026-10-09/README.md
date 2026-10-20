# Independent review: DeepSeek's VSA negatives (issue #820, 8 Oct 2026)

> **Correction, 9 October 2026 (code scout):** this review says nothing trains the VSA weight. That is wrong
> for the **scale**, which is trained (gradient and Adam update in `crates/uor-r4-core/.../jepa_trainer.rs`).
> The **codes** are fixed hashes and are not trained. The frozen-artifact critique of the two codebook tests
> stands. The owner also withdrew the "retire below 0.005" rule: a miss leads to a more native VSA design,
> not retirement (see M1 #2029).

Reviewer: Claude (read-only review of origin/main at 4f7eee35b and the four #820 comments). Some points below are from reading code; those not checked in code are marked UNVERIFIED.

## Plain-language verdict

DeepSeek ran four measurements. None of them is a fabricated or careless negative, and each one is honest about its own scope. But only two of them actually tested a VSA mechanism. Both of those swapped the codebook into an already-trained artifact *without retraining it*, and then switched the term off to measure what it contributed. That shows the term is inert **in this frozen artifact**. It does not show that VSA cannot help prediction. The phrase "the VSA term is inert" is accurate only when read with that qualifier. The general question is still open, and one cheap retraining run would settle it.

| Comment | What it really tested | Verdict |
|---|---|---|
| 6050633499 | Selecting the occurrence nearest the query by integer L1 over H4 code indices. **No hypervectors were involved.** | VALID NEGATIVE (scoped): an untrained distance between monolithic hashes cannot select by question. Says nothing about VSA. |
| 6050673419 | An offline token-overlap count: the query and the Source span share 0/32 tokens on the transfer panel and exactly one token in 108/512 rows on the retained panel. | VALID MEASUREMENT. It correctly diagnoses a **task mismatch**: no similarity measure without training (VSA or otherwise) can work here. Not a VSA test. |
| 6051646450 | Load-time codebook swap fixed to root (120 codes) on the Card P7 artifact, then ablating `vsa_scale_q15 = 0`. | INCONCLUSIVE. It is a valid null for "swap the codes in a frozen artifact", but there was no retraining, the scale was not re-fit, and the codes are aliased onto 120 roots. |
| 6052040448 | The same protocol with `learned` codes (root XOR an LSH of `discrete_s2_readout`), giving 4096 distinct codes. | INCONCLUSIVE, leaning valid negative for the frozen additive scorer. It removes the aliasing explanation but keeps the no-retraining flaw. |

## 1. What was tested

**Where the VSA term enters.** It enters the discrete prose scorer as one additive feature per candidate token:

- `jepa_trainer.rs:931` `score_vsa_candidate`: `(vsa_scale_q15 * bipolar_correlation_q15(context_vec, code(candidate))) >> 16`. It returns 0 when the scale is 0.
- The context vector is built by `vsa/context_engine.rs:174` `encode_attended_multiscale_context`. That function bundles an HRR "associative transition" context (bind/unbind over consecutive tokens) with up to 4 "orthogonal VSA attention" head predictions (`vsa/attention.rs`), over a window of up to 64 tokens (`ablate-prose.rs:477`).
- The hypervectors are `Hypervector4096`: D = 4096 bits, binary/bipolar, using XOR bind, majority bundle and a Hamming-derived bipolar correlation (`vsa/hypervector.rs:15-34`).
- In serving, `runtime.rs:918-923` always rebuilds `Codebook::<64>::on_demand(vocab, vsa_seed)`, which is the **fixed** hash codebook. So production serving never uses root or learned codes, even when the artifact declares them. UNVERIFIED: whether `set-vsa-code-mode` is honoured anywhere except `ablate-prose`. Worth checking.

**How codes are formed.**

- Mode 0, `fixed`: `splitmix64(vsa_seed, token_id)`. Random codes with expected Hamming distance D/2, so only identity carries signal.
- Mode 1, `root`: an LSH of the icosian root quaternion that the token is assigned to. At most 120 distinct codes.
- Mode 2, `learned`: the root code XOR an LSH of the token's 5-dim Q1.14 `discrete_s2_readout`, which gives 4096 distinct codes.

**Training signal.** None reaches the VSA part. `vsa_scale_q15` is an `i16` stored in the exported artifact (`jepa_trainer.rs:616`, `binary_model.rs:106`). The trainer config only carries `vsa_seed` (`jepa_trainer.rs:244`). I found no gradient path to the scale, and the codes are never trained. The trainer does build a fixed `Codebook::new(vocab, vsa_seed)` at `jepa_trainer.rs:467`. The scale value in the P7 artifact and how it was chosen are UNVERIFIED.

**Data and gates.**

- Artifact: `native_geometric_prose_model.rgm` (sha256 `7023c450…`).
- Data: two disjoint held-out slices (offset 0 and offset 100M), 8,127 scored positions each.
- Metric: the BPB change when the scale is ablated to 0, with a 95% CI and a band label. Measured deltas were between −0.0010 and −0.0020 BPB, and top-1 moved by at most 0.12 pp.
- Records: `docs/evidence/native_geometric_vsa_code_mode_ablation_2026-10-08.txt` (PR #1873) and `…vsa_stage3b_residual_2026-10-08.txt` (PR #1874, as stated in the comment; I did not re-check that it merged).

**The first two comments** concern the transfer and retained entry panels (32 and 512 rows). Their gates were relation→form mapping accuracy (baseline 0.594, chance 0.5) and within- versus across-form state distance. No VSA was involved.

## 2. Validity threats, checked

**a) Does learning reach the VSA parameters? No.** This is the decisive flaw. Modes 1 and 2 swap the codebook **at load time** into an artifact whose other weights (and its scale) were trained or fitted alongside the fixed codebook, or without the VSA term at all. Ablating a feature that nothing co-adapted with measures how much the frozen model *happens* to lean on it. It does not measure what the feature *could* contribute. A fair test fits at least the scale, and ideally the whole scorer, after the code mode is chosen.

**b) Is the scale too small to matter?** Probably not by itself. The ablation moves BPB by about 0.001–0.002, with a CI that excludes 0 and a consistently negative sign (the term slightly *helps*, because removing it raises BPB a little). So the term is live but tiny. Whether that is because the scale is small or because the feature is uninformative cannot be separated without re-fitting the scale. A sweep over the scale (for example 0, 250, 500, 1000, 2000, 4000) on the frozen artifact would separate them for a few CPU-minutes. DeepSeek did not run it.

**c) Capacity.** Bundling about 64 transition terms plus up to 4 heads into 4096 bits gives a membership SNR of roughly √(D/k) ≈ 8. That is enough for the question "is this candidate in the bundle?". The capacity for recovering *order* or *bindings* through unbinding is much lower, but that matters little here, because only correlation with the candidate is scored. Crosstalk is not the likely cause of the null.

**d) Collisions from LSH of 120 roots.** This one is real. Mode 1 caps the codebook at 120 codes for 4096 tokens. DeepSeek identified it themselves and removed it in mode 2. It is not the explanation, because mode 2 is also null.

**e) The shared-vocabulary explanation.** It is correct for comments 1 and 2. A fixed random or hash code space carries only identity. With 0/32 token overlap between query and record, no similarity-based selection without training can work. Getting there needs a *learned* map from query to frame (or a learned VSA role/filler binding trained on varied phrasings). DeepSeek's conclusion, that the mechanism needs a learned query typer and these panels cannot train or test one, is sound. It is a statement about the panel, not about VSA.

**f) Panels that cannot show a benefit.** Partly. With fixed codes, the VSA feature amounts to a "the candidate appeared or followed in recent context" cache/induction signal. This stack already has an **exact** prime-route Copy head and Dot/Lorentz context reads that supply the same evidence without approximation. A redundant approximate copy feature should be close to 0 on BPB *even if it works perfectly*. Neither sweep tested this with the copy head ablated.

**g) Seeds and variance.** There is one artifact, one training seed and one VSA seed. The bootstrap CIs cover position sampling but not seed variance. Two slices agreeing is good. It is still n=1 model.

**h) Bugs.**

- XOR-binding the root code with the readout code is a legitimate way to make residual codes. Distinct roots stay about D/2 apart, and within a root the distance follows the readout LSH. I see no sign or axis error in the parts I read.
- Two real issues: serving ignores the code mode (`runtime.rs:920`), and `ablate-prose` prints a stale label, "MIS-WIRED: heads bind a fixed random codebook…" (`ablate-prose.rs:407-408`), as DeepSeek noted.
- The `learned` mode uses a 5-dim proxy for the embedding (`discrete_s2_readout`), not the training-time continuous embedding. DeepSeek scoped this correctly.

**i) Does a null on two slices justify "inert" in general? No.** It justifies "inert in the frozen P7 additive scorer at any code resolution available from the artifact". That is useful, and DeepSeek's own scope paragraph says nearly this. The headline sentence overreaches.

## 3. Is VSA still potentially useful for prediction?

**The case for it.**

- VSA gives D11-safe operations (XOR, popcount, majority) that bind structure (role⊗filler, position⊗token). The additive log-linear scorer cannot express such feature interactions. DeepSeek's Stage 4 point is that VSA is one of the few multiplier-free ways to give that scorer a conjunction feature.
- Similarity-preserving codes derived from learned geometry are also the natural keys for approximate associative memory. Grounded recall needs exactly that once the query is typed.
- The literature supports VSA as a **memory and binding substrate**, not as a stand-alone language model: Plate's HRR, Kanerva's hyperdimensional computing, random indexing for distributional semantics, and the Kleyko et al. surveys. I am not aware of any VSA-only model that is competitive at next-token prediction (moderately confident).

**The case against it.**

- In this stack, the jobs VSA could do are already covered by stronger exact mechanisms. The quaternion recurrence carries state, the exact prime-route Copy head does induction and copying, and the Dot/Lorentz reads do content addressing.
- With random codes, VSA similarity is identity only. With codes from 120 roots or 5 dims, it carries little semantics.
- As an approximate copy signal, it is dominated by the exact copy head. Its realistic niche is a learned binding feature or a recall key. Both require training it, which has not happened.

**Overall: not conclusively dead. It is unproven, with a narrow plausible niche.**

## 4. One decisive experiment

**What to train.** Retrain the P7 prose scorer with the VSA term **inside the training objective**, in four arms with identical data, seed, steps and budget:

- A: VSA off.
- B: fixed codes, with a learnable scale (or per-head scales).
- C: `learned` codes chosen before training, with a learnable scale.
- D: C with the exact Copy head disabled. This is the redundancy control, compared against A with Copy disabled.

Use 2 training seeds, and evaluate on the same two disjoint held-out slices (8,127 positions each).

**Pre-registered rule.**

- VSA is useful if B or C improves held-out BPB over A by **≥ 0.01** (about 5× the measured ablation effect), with the 95% CI excluding 0 on both slices and both seeds.
- It is redundant (keep it only as a recall key) if D beats its Copy-off control by ≥ 0.01 but B and C do not beat A.
- Retire the VSA term from the scorer (keeping the scoped evidence) if no arm reaches 0.005.

**Cheap pre-check (CPU-minutes).** Sweep the scale on the frozen artifact. If no scale reaches a 0.005 gain, that lowers the prior, but it does not replace the retraining.

**Compute.** UNVERIFIED (I did not find the P7 training time): I estimate 4 arms × 2 seeds at well under one RTX 5090-day if P7 trains in hours, run through `uor-pod` under the caps.
