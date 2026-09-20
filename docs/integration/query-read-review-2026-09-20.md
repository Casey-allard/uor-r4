# Principal review after PR #1310 — identify what the separable reader learned

Date: 2026-09-20. Reviewed merge `7ea3744ce190c6acd708275e4121a6354b301023` and head `3fd247c94c5f504dadafd9dc7c6ff0aa2ce7012c`, identical tree `09d12e564c7506e412a356c36e61f2b7a81bdc76`. This is a source, retained-artifact, saved-data and literature review. No build, training or model forward ran here. The [original result](query-read-result-2026-09-20.md) and [raw receipt](../evidence/native_geometric_query_read_2026-09-20.txt) remain historical records; read them with these corrections.

**Decision:** retain S as the strongest measured numerical candidate, with E as the established local reference. Do **one frozen S attribution replay before fitting selective writes/reset or expanding history capacity**. Repair tokenizer metadata and the small shared evaluation/serving defects, then measure whether S needs the individual older state or mainly supplies a useful local query/offset correction. The [complete next DeepSeek prompt](deepseek-separable-attribution-step-2026-09-20.md) makes that task concrete. No candidate is promoted to useful language, and no new geometric architecture is rejected as a family.

## The numerical result survives, but its attribution changes

Saved development vectors reproduce the E/Q/S/L micro means and all published paired intervals. Both complete roots have matching listed sets, sizes and BLAKE3 digests, and the three final artifacts are byte-identical between runs. The actual causal indexing, 128-scaled gradient, separate A/B Jacobians, target normalization and 512-batch schedule match the prescribed experiment. All 15 saved scheduled checkpoints have the expected R/W and A/B ages; Q/S warm-up checkpoints differ only in their arm byte.

| Arm | Dev micro bits/target | E minus arm, nominal paired 95% interval |
| --- | ---: | --- |
| E | 7.170815718 | — |
| Q, joint query transport | 7.141505532 | +0.029310 [0.022866, 0.036005] |
| S, separable history/query | 7.032227740 | +0.138588 [0.125831, 0.150844] |
| L, local matched read | 7.101049449 | +0.069766 [0.058944, 0.080294] |

Q fails its registered practical, matched-model and history/query screens. That is a valid negative for the executed mechanism and dose. S improves over L by 0.068821709 [0.057449606, 0.080141572] on this reused panel, but the fitted architectures differ in more than the presence of individual history. That comparison does not identify the source of S's gain.

The decisive omitted datum is already in `query-read-2/result.json`:

**S older-prefix donor penalty = −0.002298261 bits per eligible target, interval [−0.014071554, +0.008581473].** There are 1,177 eligible observations in 36 documents and 564 changed S reads. Matching preserves exact previous/current tokens and older-prefix length. This detects no predictive older-content benefit on its supported subset. The result's sentence that the run showed S benefited from older content is unsupported.

S's query-neutralization penalty, +0.190660114 [0.173466040, 0.207856510] on all targets, shows a useful dependence on the current-token query branch. It does not establish history access. That neutralized-query predictor is actually 0.052072136 bits worse than E, with interval [0.039666408, 0.065231370]. Reversing older order improves S by about 0.005064 bits on the full panel in an additional descriptive reaggregation; it also does not support a strong older-order claim. These observations motivate a frozen decomposition rather than an immediate reset fit.

The reported query intervals used all 17,342 targets; the prompt required the 16,766 positions with an older prefix. The 576 masked positions contribute exactly zero difference. Independent saved-vector reaggregation on that mask gives query penalties Q+0.000332874 [−0.003012317,+0.003472257], S+0.197210288 [0.179469190,0.214992789], L+0.020098130 [0.015052131,0.025232044], using the source query seed. Retain explicit original and corrected masks/seeds. Do not present different populations as directly comparable mechanisms or rewrite the old vectors.

Movement of 1,549–2,241 A codes and 1,544–1,581 B codes rules out a completely unmoving hard-map explanation. It does **not** prove adequate optimization or establish an arrangement-versus-optimizer diagnosis. The curves cover only the first windows of eight tune documents, not full fit/tune populations, and S is still improving between saved late observations. No convergence claim follows.

## Artifact, continuation and cost corrections

**Tokenizer binding regressed.** Every retained CPX3 has zero bytes at offsets 73–104. `QueryTrainer::hard_core` passes `[0;32]`; the runner separately hashes the hexadecimal derived-digest string, reports `adce1fa0…`, and never assigns it to the artifact. The required raw derived digest is `a7ac75b68aa997fe7cc338d25d843157f2dc9d4265fb2f958ee779bb04828d6f`. Parent binding is correct. Re-export metadata-only descendants of Q/S/L and verify all numerical fields, logits and generated IDs unchanged. The same externally verified tokenizer produced the saved scores, so this defect does not by itself invalidate those scores.

**Source provenance is incomplete.** `source_rev` names preimplementation base `92240a86`, while `evaluator.git_rev` is `unset`; no binary digest is bound in the result. The actual merged source is inspectable and consistent with the reported mechanism, but the old fields do not identify the executable that ran. New reports must bind actual source-file and executable hashes and distinguish base revision from running source. Do not invent missing historical provenance.

**CPQK is incomplete continuation.** It saves masters/moments/ages and some configuration, but not the dose permutation/cursor, full source/configuration/optimizer/quantizer/table/tokenizer identities. Every saved data identity is zero; `resume_from` adopts it rather than checking an expected identity. The runner has no production resume mode. The mini check writes a file, then resumes the same in-memory byte vector and supplies subsequent batches from the surrounding loop. It does not read the checkpoint back or start four processes. Keep its narrower parameter-state/next-update parity evidence. Full continuation must be repaired before another fit, but is not a prerequisite to this frozen-artifact task; no new continuation framework or optimizer test campaign is selected now.

**Serving currently performs training-oriented work.** `read_path` folds older history, then `read_path_from_older` rebuilds an allocated chain. Q/S fold history twice; L folds history unnecessarily before constructing its local state. Separate the small inference row-selection helper from training-chain construction, preserving exact numerical behavior and donor semantics. This is a concrete cost correction, not a new model.

**Timing and generation claims need tighter scope.** The fold microbenchmark discards outputs without `black_box` and uses i=62 on a 64-token window, which folds 61 older tokens, not 62. Its 36–51× ratio is not a robust primitive-cost qualification. Report observed end-to-end timings at their actual scope and remeasure the corrected path; include E in the same protocol. CPX3's 53,555 bytes exclude its separately loaded 454,788-byte E parent. Training masters plus moments are not serving parameter storage. The cache's recorded peak is sampled after a reset, so it can miss within-batch growth.

All generated outputs are visibly repetitive. Q/S have full bounded-ring cycle certificates only for the all-32 prompt (entry 48, period 1); their earlier repeated local pairs are insufficient to certify future behavior. The receipt's count of five period-1 pair cycles is also inaccurate. Preserve token outputs and distinguish observed repetition from a sufficient-state cycle certificate. Energy remains UNAVAILABLE.

## The next attribution test

Let u(s)=128WR[s]. The frozen separable reader is

`Z_S(q,b) = Z_E + u(q) + u(b)`.

An identity-anchored decomposition is

`C=2u(e), H(q)=u(q)−u(e), K(b)=u(b)−u(e)`, so `Z_S=Z_E+C+H+K`.

Measure the four exact corners `(q,b), (e,b), (q,e), (e,e)`, E, and direct zero-row history-only/query-only conditions. Preserve the original absence mask everywhere. Verify the exact integer identity `Z11+Z00=Z10+Z01`. The raw zero-row and identity substitutions answer different questions: replacing q by e retains a learned constant, while dropping u(q) does not. Cross-entropy does not obey the same additive identity because log-sum-exp is nonlinear.

Add one prespecified **offline** calibration-preserving comparator: replace the individual history reader vector by its fit-only occurrence-weighted mean conditional on older-prefix length, retaining the actual query branch. This keeps the average history offset and positional exposure while removing the specific older content. Save exact integer numerator/counts; this mean of logits is not a probability mixture, a fitted new head or a serving candidate. Its result is another fitted-model dependence diagnostic, not a causal sufficiency theorem. The exact-tail donor test remains necessary to distinguish content under its supported conditioning.

Replay the established panel and one deterministically chosen, nonoverlapping set of windows from the existing open-dev documents. Freeze the selection from document/window identities before scoring. New positions in the same documents are a replication check, not independent documents or final held-out qualification. No new corpus, training, width, precision or decoder changes are needed.

## Mathematical roadmap and relevant research

The current-token B map selects only eight reader rows; useful local class/calibration learning can therefore explain a gain without demonstrating older-memory use. That is a hypothesis to measure, not a defect to hide. Retain any such emission improvement with its exact scope while continuing toward geometric state and addressed memory.

For distinct q1,q2 and a common future group product g, `q1*g != q2*g`. Pure right translations cannot merge existing states. Identity actions skip inputs, and the 64-token window eventually drops them, but neither is a learned many-to-one reset. A later minimal hypothesis can select `Continue(gamma_k):q->q*gamma_k` or `ResetTo(gamma_k):q->gamma_k` using a four-bit action. The update transformations generate a monoid of translations and constant maps; the state remains an exact 2I element. This adds selective global reset semantics, not register capacity or keyed/versioned memory. It remains a future hypothesis pending the present diagnosis.

- [Hooker, Mentch and Zhou](https://arxiv.org/abs/1905.03151) explain why unrestricted permutation with correlated inputs can force extrapolation. Our interpretation therefore distinguishes anchor ablations, calibration-preserving replacement and exact-tail conditional donors. The resulting bootstrap intervals are descriptive fitted-model evidence, not a formal conditional-independence test.
- [PD-SSM](https://arxiv.org/abs/2509.22284) admits noninvertible column-one-hot transitions. That supports considering state-merging operations; its arbitrary-FSA guarantee requires a richer family than translations plus reset and is not inherited here.
- [Gated DeltaNet](https://arxiv.org/abs/2412.06464) distinguishes rapid memory erasure from targeted associative updates. The distinction informs the roadmap; its floating matrix state and kernels are not adopted.
- [Lag Operator SSMs, AISTATS 2026](https://proceedings.mlr.press/v300/tomonaga26a.html) supplies a recent basis/lag-operator construction of discrete state recurrences, recovering HiPPO in a special case. It is useful future design material, not evidence that replacing the current finite-state implementation would solve this observed attribution question.
- [Repeat After Me](https://arxiv.org/abs/2402.01032) and [Zoology](https://arxiv.org/abs/2312.04927) motivate eventual exact copying and multi-binding memory evaluation. They do not prove that this run's CE result is caused by the 120-state capacity.

The coherent programme remains local emission -> useful state access and maintenance -> exact occurrence/version memory and shared composition -> useful conversation and executable Rust on the same artifact -> measured full consumer-machine cost. Reuse the existing native-memory research rather than building a separate memory product. Prime/zeta/R4/S3/H4/icosian roles remain primary design material, with implemented mechanisms and evidence kept distinct.

## Resources and delivery

Live JSON is `176138565 /178100000 ms`, leaving `1961435 ms` (32.69 minutes). Preserve the latest 5900000-ms debit as mixed measured/estimated; the claimed phase accounting does not establish that entire amount as measured nonoverlapping wall time. The next prompt proposes 3600000 ms complete work and a standing-authorized +2400000-ms limit extension to 180500000 ms, to refresh and record before use. This review changes neither balance nor limit. Preserve all roots, owner checkout and the 128 MiB storage stop margin; no paid compute or deletion. #973/#820/#963/#964 stay open.
