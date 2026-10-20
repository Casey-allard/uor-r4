# DeepSeek execution prompt — one learned older-prefix group-state channel

> Historical executed specification. PR #1304 delivered a scoped numerical negative with incomplete artifact/reload and generation qualification. Read the [review after #1304](prefix-pilot-review-2026-09-20.md) and [current execution prompt](deepseek-readout-diagnostic-step-2026-09-20.md) before acting; do not repeat these fits to repair reporting.

Work on **UOR-R4 Geometric Language Model**, `UOR-Foundation/uor-r4`. Read this prompt and `docs/integration/ordered-prefix-review-2026-09-20.md` completely. Refresh origin/main beyond audited base `3095c1d48e213deb234d31d21c94da7adcca7e6e` (PR #1302), including the protected delivery of these instructions.

**Implement, fit and evaluate one hard learned group-state/read channel above the unchanged step-512 prior.** Compare learned older-prefix actions against fixed actions and an equally sized fitted local-tail channel. The question is predictive use of older history, not whether extra parameters can improve a weak local predictor. Do not rerun the unchanged prior's training or the complete frozen replay. Complete the few provenance/diagnostic corrections below in the same task. Execute the pilot, preserve negatives and make one evidence-supported next recommendation.

## 1. Recovery, authority and complete projection

Read AGENTS.md, DECISIONS.md D0-b/D1/D2, current policy/state and canonical plan. Rust data, fitting, artifacts and inference. Offline float/gradients/matmul are allowed; served learned weights/action codes are <=4 bits, with integer add/subtract/shift/table kernels. Geometry remains primary; no hidden provider, transformer backbone or Python model. Frozen R4G1 is separately scoped. Use a clean isolated full `codex/…` worktree, preserve all original checkouts/artifacts, refresh #973/#820/#963/#964 and assign active work appropriately.

Retained owner root is `/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/`:

- `attempt-1/prior_realtext.cpl2`, SHA256 `cd5a3aa1b804ccc3582a4a25090e25c862489e0bc4e5bb86c1d1f363a7338a00`, 454,788 bytes; unchanged frozen base.
- `attempt-1/prior_realtext.ckpt`, SHA256 `1c3dfedb2248ec76c21f320b7ecec585c7a522a2d47e35ed5dc9eaca2454bff2`; exposure metadata only, not a training continuation.
- Source/derived tokenizer JSON; derived SHA256 `a7ac75b68aa997fe7cc338d25d843157f2dc9d4265fb2f958ee779bb04828d6f`.
- `eval-replay-2`, sealed 14 members; result SHA256 `6db96a1c7f9cccefe13b9c07a610bbb02559ece6e3762a7c36f35ba9dcc6f661`; `parity-before`/`parity-after`; `inputs/docs` pinned at corpus commit `e9c04e80`.

Refresh the authoritative owner `.uor-models/native-joint-learning-2026-09-04/model-time.json`. Principal review reconciled #1302's previously recorded 1,800,000 ms charge once: expected balance **154038565 /154400000 ms**, remaining **361435 ms**. Do not charge it again or use the stale 152238565 value. Earlier +1,000,000 ms headroom was released, not active. Preserve the historical charge's approximate timing provenance.

Record a complete new projection and storage inventory before builds/preparation/fits. Proposed **5,400,000 ms**: 300 s derived-provenance repairs and inputs; 1,800 s implementation/build/independent checks; 300 s real-path timing/cache preparation; 1,800 s three matched fits/evaluation/controls/generation; 1,200 s checkpoint/receipts/delivery/stop reserve. Under standing authorization, record **+5,400,000 ms** local allowance before use, making the proposed limit **159800000 ms**, available **5761435 ms** after the prior charge. Refresh actual values and adjust before overruns; do not ask for the already-authorized class of local extension. Charge actual elapsed intervals, including failed attempts/tests, once. Synchronize both JSON and readable ledger.

One worker, <=4 Cargo jobs, <=8 GiB RSS, <=512 MiB new retained/temporary data plus <=1 GiB incremental reused build output; preserve 128 MiB storage margin. Read inventory using existing tooling; overlapping paths are not additive. No paid/external compute, deletion, corpus download or broad historical suite. A session limit is not depletion of local allowance; leave an exact resumable checkpoint when necessary.

## 2. Preserve the valid replay; fix its derived metadata

Do not withdraw the corrected legacy/spread loss gates: source and saved-statistic audits support them. Make focused corrections to `prior-frozen-evaluate.rs` and report interpretation:

- Fit-window document offset is `window_index * 64`, not global window ID times 64, in BOTH JSON and binary index writers. 38,152 old offsets are wrong; document IDs, indices, lengths and byte offsets are valid. Claim a new derived report, write corrected indices with old hashes, and verify a boundary example across two documents against pinned tokenization. Never modify sealed `eval-replay-2`.
- A pair fixed point requires `prev == cur && argmax == cur`. Preserve the observed `(32,32)` fixed point and correct `(284,198)` to an immediate repetition followed by `(198,198)->504`. One-step reference agreement does not establish the cause of entire generation collapse. Historical reference generation remains NOT_RUN unless actually added below.
- Correct tune pool versus sample:38 pool docs, 32 actually used by 64 windows. Preserve their old smoothing result as historical; no retuning needed to repair its label.
- Record literal permutation seed `0xA5A51234` and effective `seed|1 = 0xA5A51235`; retain the saved donor map. Label the old eligible subset as changed-context records. New conditional controls below use a data-defined permutable subset.
- Report that the real-text artifact was frozen while one named parity fixture executed 60 toy updates; do not repeat that fixture as a claim of update-free testing.

Use focused fixtures for these changes and small saved-data derivation only. Reuse the verified old results; no complete 52-second replay is required just for metadata.

## 3. Exact bounded forward and scope

Implement an experimental component in the existing `native_geometric/learner` family, with one thin runner and reusable evaluator helpers. Do not fork another independent training stack or change the old CPL2 meaning. Freeze every parameter and byte of the parent prior. V=4096, new width d=16, total context H=64, one group state. The predictor at position i uses the local pair `(x[i-1],x[i])` and target `x[i+1]`. The new state is the chronological product over `x[max(0,i-63)..i-1]` in Rust half-open notation: it EXCLUDES the last two tokens. For i<2 that slice is empty; disable the new contribution. Assert this indexing with an explicit four-token trace.

Use an offline-precompiled, artifact-bound exact binary-icosahedral 2I product table, identity and root order. Inspect `learner/group_table.rs` for orientation/padded addressing and `native_geometric/training.rs::geometry` / `validate_h4_binary_icosahedral_closure` for exact canonical construction. Do not mix the two root orders without an explicit verified mapping. Preserve q versus -q. Store table/identity/palette/digests in the new artifact; no lazy floating nearest-root construction in its declared served computation. Reuse existing exact group checks; no unrelated proof campaign.

Choose a deterministic data-independent palette of **K=8 distinct exact group elements**: identity, generators a/b and their inverses, plus distinct products as needed. Pick the first canonical generator pair satisfying a*b != b*a and full 120-element closure; verify closure of the complete palette. Do not use targets to pick generators. The learned per-token palette ID lies in 0..8 and is nibble-packed; state IDs remain 0..120. Fixed palette choice is not learned language knowledge.

At every transition, `q_next=product[q, palette[argmax A[token,:]]]`. A has 8 offline floating logits per token and deterministic lowest-ID ties. Export only hard codes. The output is

`Z_total[v] = Z_parent[v] + ((sum_j Wcode[v,j] * Rcode[q,j]) << 8)`

where both codes are ternary and the product is implemented by selecting addition/subtraction/zero. Equivalently read `Rcode << 5` and apply ternary W with output shift 3. Parent F=10 remains the sole score-to-logit exponent; each residual code-sum unit is 0.25 logit. These fixed scales are intentionally simple and must be recorded, bounded and tested. Reader is 120×16; output is 4096×16. No new bias, ReLU, normalizer, sampler or gate. Max absolute residual score is 4096; validate the combined parent+residual envelope before execution. Do not silently allow wrapping/saturation.

Compile fixed-scale packed weights directly with validated codes/shifts (reuse `TernaryLinear::from_packed` where appropriate); its adaptive `quantize` must not silently replace the specified fixed scales. Ternarize masters with thresholds ±0.5, ties explicitly defined, and clamp masters to[-1,1] after updates. Keep integer forward identical across training, export/reload and generation. Caller-owned buffers should avoid introducing new steady-state allocation in the numerical kernel; scope allocation and instruction claims to what is actually measured.

This state is a bounded coalescing summary, not exact memory, a semantic metric or an independent golden/Galois companion. No zeta feature is added without a role. Pure group transitions cannot selectively erase; H=64 supplies explicit bounded forgetting. Generation may recompute the <=62 older actions from the bounded ring initially, rather than implementing a new eviction subsystem.

## 4. Explicit offline credit; verify before fitting

Use repaired Adam with betas(.9,.999), epsilon1e-8, weight_decay 0. Reader/output learning rate 0.03; action-logit rate 0.003; categorical temperature 1. These are one fixed pilot configuration, not a development sweep. Reader masters initialize from seeded ±1, output masters exactly zero. Action rows initialize with one seeded palette choice at 0.1 and the other logits 0; save the actual initial map. Step 0 must reproduce the parent's integer scores exactly. Zeroing both reader and output would deadlock learning; do not do it.

Use the real hard state and quantized integer logits in every training forward. Offline loss is stable uncapped cross entropy in bits. First apply `dZ=(p-onehot)*2^-F/ln2`. In code units the identity-STE equations are `gW[v,j]=256*dZ[v]*Rcode[q,j]`, `g[j]=256*sum_v dZ[v]*Wcode[v,j]`, `gR[q,j]=g[j]`, and local state adjoint `u[s]=sum_j g[j]*Rcode[s,j]`. Apply no additional dyadic factor to that state/action credit: using `Rcode<<5` with this already-scaled g would introduce an erroneous factor 32. Test all three adjoints independently. Divide by the actual scored-target count once per update. The quantizer uses an explicitly declared straight-through surrogate; do not use a second floating prediction model.

For the action credit, the multilinear extension is `c[h]=sum_{s,k:s*gamma[k]=h} p[s] pi[k]`. At actual hard state q and hard action k0, if the state adjoint is u:

`d_previous[s]=u[product(s,gamma[k0])]`

`d_palette[k]=u[product(q,gamma[k])]`

`d_A[k]=softmax(A)[k]*(d_palette[k]-sum_l softmax(A)[l]*d_palette[l])`.

Use the local state adjoints in unshifted code units as defined above. Add future state credit and backpropagate through the complete bounded window, with explicit reset and no future-target leakage. Accumulate repeated-token action gradients. No dense 120² group convolution or 120×V logit cache is needed. These are derivatives of a continuous extension plus a **biased hardmax surrogate**, not true derivatives of the discontinuous served action.

Independently check reader/output scaling with literal small nonzero integer fixtures and the palette/state adjoints against finite differences of the continuous multilinear extension and softmax Jacobian. Do NOT finite-difference hard argmax and call a mismatch a derivative bug. Verify learned action credit reaches an earlier token while local tail and frozen parent remain unchanged. Use fixed noncommuting AB/BA and equal-tail fixtures to check causality, not as the headline language fit or a substitute for the real-text pilot. Verify code bounds, combined accumulator envelope, zero/disabled parity, exact reload and complete mid-pass resume for the NEW optimizer/config/seed/permutation/base/data/group identities.

## 5. One fixed matched pilot

Use the corrected existing tokenized fit windows. Select the **first 512 entries of the previously recovered 4,096 consumed-window permutation**, without sorting or selecting on losses. Each 64-token window retains the existing n−1 target semantics and starts with empty history. Save every actual occurrence/target/window length and discarded-tail policy. The prior was trained on the larger 4,096-window pool and its bias on all fit data; this is shared identically by all arms. Do not describe the new 512-window dose as the parent's training exposure.

Three arms, seed 13, identical reader initialization, parameterization, fit order and dose:

1. **Learned older-prefix:** eight-way actions and reader/output trained.
2. **Fixed-action older-prefix:** same initial hard action map frozen; reader/output trained.
3. **Learned local-tail:** same palette/action/reader/output sizes, but its state is the product of the two local token actions only. Apply the same nonempty-older-history availability mask as the primary, so target coverage is identical. It has no older-prefix information. Parameter budgets match, but its reachable state set may be smaller (at most K² distinct pair products); report occupancy and do not call the hypothesis classes identical.

For each arm run 256 batch-8 updates: four passes over the 512-window set, with a saved Fisher–Yates order per pass. The first 64 updates train reader/output only; then train actions in arms1/3 for 192 updates. The fixed-action arm still gets 256 reader updates. Keep separate Adam update counts: action bias-correction age starts at 1 on its first actual update after warm-up; reader/output age starts at the first reader update. Persist those counts in checkpoints. The two older-prefix arms must have identical learned codes, reader/output masters and optimizer moments, and integer scores through step 64; check these fields before action learning diverges (arm labels in checkpoint metadata may differ). Record actual n−1 targets, all updates and unequal measured compute. Report steps 0/64/128/256 hard-export diagnostics; **step 256 is the predeclared primary artifact**, not the best dev checkpoint. A zero residual after warm-up is an observed optimization failure, not permission to substitute soft-forward scores or hidden oracle actions.

Before the full fits, measure a small actual batch/probe including backward, group credit and export. Any probe optimizer state is either the preserved beginning of a named arm or explicitly discarded and charged. Reuse parent integer scores in bounded RAM when useful (512 windows need roughly0.5 GiB at full length); bind and verify cache keys to parent hash/config and exact `(prev,cur)` (or pinned window hash plus prediction offset), never just target-token identity, and keep memory within 8 GiB. Do not write a huge 120-state vocabulary cache. Checkpoint each arm and stop/extend accounting before an overrun; no unannounced new seeds, palette/width sweeps or extra epochs.

Keep exact/quantized unigram and old full-fit/4,096-consumed count references visible. Add a 512-window conditional-count reference backed by the same full-fit unigram; tune its small fixed smoothing grid only on declared tune data. All 38 tune docs may be used in a newly named first/last-window selection (<=76 windows); record the exact population, distinct from the old 32-doc tuning sample. These are comparisons, not deployment fallbacks or Bayes ceilings.

## 6. Evaluation that distinguishes history from added local parameters

Freeze input IDs, metrics, thresholds and conditional donor maps BEFORE fitting or new scores. Use a primary open development panel spanning all 36 original dev-pool documents: up to eight distinct evenly spaced 64-token windows per document (<=288 windows), with explicit short-document policy. Save overlap with legacy/spread panels. Score all arms, the frozen parent and every reference on exactly these occurrences. This is new open development data from the same corpus, not final held-out or an independent replication.

Reuse true document aggregation and paired 2,000-draw bootstrap; save per-document loss sums/counts for every comparison. Fix support at >=20 documents and >=1,024 targets. Report bits/target, token-micro/document-macro, decision flips, fit loss, hard action changes/occupancy, state occupancy, reader/output nonzero codes and gradient norms. Do not treat floating action-logit movement as learned hard transitions.

Controls on the primary artifact:

- State-disabled/zero residual must reproduce parent integer logits exactly.
- Permute older-prefix state donors **within `(exact prev, exact cur, exact older-prefix length)`**. Donors may cross development documents; persist both source and recipient document IDs. The declared estimand includes topic/document information carried by the older prefix, not only within-document effects. Exact length is essential: coarse buckets let a pure position counter `q=g^L` pass without using older token content. Keep targets/local inputs fixed. Save a seeded bijection from occurrence IDs before fitting; its map is independent of learned states. No-history records are excluded from this intervention. The eligible population is every record in a stratum of size>=2, including unchanged donors; report changed-state counts separately. Do not condition support/CI on whether the trained model happened to change a state. A within-document exact-tail/length intervention may be reported separately if supported; do not substitute its population for the declared primary.
- Report full-population and eligible-population intervention loss. Require >=20 eligible documents and >=1,024 eligible targets for an inferential claim. Determine coverage before fitting. If insufficient, keep the effect descriptive/INCONCLUSIVE; do not loosen conditioning or add favorable documents after scoring.
- Reverse only the older-prefix order while preserving its multiset and local tail; report state/decision changes and loss. This is sensitivity under an altered distribution, not a proof of useful order or 2I superiority.
- Compare primary to both independently fitted controls on aligned records. A gain over the parent alone may reflect local calibration/capacity.

Predeclare the screen: >=0.10 bits/target gain over the frozen parent with paired interval excluding zero, positive paired gains over both fitted controls with intervals excluding zero, and positive conditional-permutation penalty with interval excluding zero and adequate support. Report the numerical magnitude and all component outcomes even if the combined screen fails. This is a pilot criterion; do not relabel it as D2's 0.01 BPB equivalence margin. Keep D2 mechanism-class interpretation: small/inconclusive effects do not retire geometry.

For a positive learned-transition claim, show that hard action IDs actually change and that primary beats the fixed-action fitted control. A fixed-action win is learned reading of a fixed carrier. Tail-only matching/beating primary indicates no selected older-prefix advantage under this test. Superiority of 2I over another algebra remains NOT_TESTED; that would require a separately matched C120 or other control. Renaming all group IDs with a consistently transformed table changes no mechanism.

Generate 64 greedy tokens from the same six retained prompts for parent and each final reloaded arm, with unchanged decoding. Add the previously missing count-reference trajectories if cheap using the same pinned reference and tie rule; keep them clearly diagnostic. Save all IDs/bytes/repetition, including failures. Pair repetition alone is NOT a cycle certificate for the new model: the sufficient state includes the bounded context ring and its length/reset state. Only report a new-model cycle if that complete state repeats; otherwise report observed repetition without claiming a proven cycle. Never add penalties, sampling, blacklists or overrides to improve this pilot's outcome.

## 7. Delivery and next decision

Bind each final artifact/checkpoint/report to the unchanged parent, tokenizer, exact group/palette, actual fit/tune/dev records, initialization/optimizer and source/binary/compiler identities. Save complete new optimizer state for genuine continuation; do not resume the legacy CPCK or invent its missing historical training identity. Claim each attempt exclusively after argument validation and before loading/preparation, check required writes, seal and verify the complete expected file set. Preserve failed attempts and the original sealed roots.

Compile the touched Rust path, run focused causal/arithmetic/surrogate/serialization/resume tests, fmt/offline checks, actual training/exported evaluation and raw generation. Test new integer kernel instruction/allocation boundaries where practical; distinguish arithmetic-count projection from measured latency/RAM and physical energy. A queue acknowledgement is not a test. Update claim wording and run its script. Do not expand into a blanket historical benchmark suite.

Deliver named changes through a protected PR and verify actual merge/tree equality. Update README, current-state, canonical dependency status, continuation, evidence corrections, resource/storage receipts and #973/#820/#963/#964; update project items only if they exist. Preserve broad open acceptance and all raw historical receipts. Report the exact JSON balance after charges/extensions.

The final answer must state: valid baseline preserved; precise metadata/interpretation corrections; three hard-artifact outcomes and conditional-control support; whether actions were actually learned and useful; count-reference comparisons; all generated outputs; timing/RAM/storage and actual cumulative charge; hashes and verified delivery; and ONE next mechanism justified by the observed failure or success. Do not announce general language, noncommutative advantage or solved repetition from this bounded pilot. Do not stop after writing a new module or passing a synthetic witness: execute the real-text experiment within the projected allowance.
