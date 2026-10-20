# Research takeover and architecture assessment — 2026-09-19

References #820, #973, #963 and #964. Source baseline: `127c0eafc8c33cfb8476e3b9514d67fd9b550066`.
This is a source, history and literature investigation, not a new model result.
No training, inference, benchmark, fresh acceptance evaluation or Rust build was run for this review.

## Decision

Continue the geometric addressed-memory research. Before a long real-text fit, qualify the causal state, the declared training gradient and the evaluation instrument; measure real-text route coverage, then measure complete training-step cost. The [DeepSeek execution prompt](deepseek-next-step-2026-09-19.md) makes this one bounded implementation task with a conditional small pilot. The proposed 2,000-step run is not yet justified by either a valid capacity bound or measured throughput.

The objective remains a useful local geometric language model for conversation/memory and Rust coding/reasoning, ultimately frontier capability with less wasted compute on consumer hardware. Owner clarification in this takeover ratifies the recorded decisions: offline Rust training may use matrix products; D0-b permits bounded low-bit linear maps implemented through additions, subtraction, shifts and lookup. Geometric routing remains the preferred research direction. This is an opcode/precision contract, not a claim that additive linear maps cease to be mathematical matrix products. Served computation must use no floating-point arithmetic or libm transcendentals; the declared numerical kernel must execute no multiplier instruction. No transformer backbone or provider-authored serving is adopted.

## Coverage and recovery

The review reconciled the live repository, open programme issues, PR #1290, model/time ledger, storage inventory, selected mechanism source, raw evidence, prior Codex recovery records, relevant Antigravity plans/walkthroughs and the three recent Zed/DeepSeek tasks. The latter were read directly from the local read-only task database: “UOR R4 Research Project Report,” “LowBitCore backward pass STE Adam,” and “UOR-R4 real-text path kickoff”; their metadata identifies the provider/model as `deepseek` / `deepseek-flash`. This does not independently identify a provider's marketing version.

Prior Codex tasks were not available in the app's current/archived task listings. Retained recovery summaries, committed reports and observed activity supplied that history. The Antigravity records were checked against committed code and Git history rather than treated as qualification. Private conversation text and unrelated activity are not reproduced in the repository.

Research coverage reuses the [September architecture audit](architecture-2026-09/README.md), [September 13 synthesis](geometric-attention-research-2026-09/README.md), [September 16 review](review-2026-09-16/00-project-brain-index.md), and [research archive](../../research/README.md). Decision-relevant primary literature was refreshed. This is not a claim to have read every archived file, paper, proof or upstream repository. Existing upstream pins remain dated unless explicitly refreshed; an index entry is discovery, not verified implementation. Live default-branch head metadata was also refreshed for NEMESIS, W33, Prism, UOR-Framework, uor-addr, HELM, prime-router and NavierStokesAndEuler in the [public source-refresh receipt](../evidence/takeover-upstream-heads-2026-09-19.json). NEMESIS and HELM match their older audited pins; W33 has intervening work. Head metadata does not mean those newer source trees were re-audited or adopted.

At recovery, local main and origin/main were clean at `127c0eaf`; PR #1290 was merged and there were no open PRs. #820 and the broad capability responsibilities remained open. GitHub's tracker body still described an older artifact and next action; its project-items list was empty. This review uses an isolated full worktree and preserves the owner's checkout, all artifacts, sealed populations and research.

## What happened, and what survives

| Phase | Implemented work and result | Interpretation to retain |
| --- | --- | --- |
| Earlier native work, through September 12 | Exact values/versions, selected copying and arithmetic, learned source choice, shared emission, historical memory repair; positive and rejected artifacts preserved | Useful bounded capabilities, with important failed preservation controls; not a general language model |
| September 13–14, #1254–#1279 | Learned typed reads, dependent query updates, Read/Emit/Stop, completion distinction, multiword spans, ordered occurrence witnesses and role learning | Real causal composition within authored curricula. The independent neighbor result was 2,016/2,304, with 288 interior-name `will` failures |
| September 17, #1281–#1282 | Role repair reported 2,304/2,304 and retained prior traces | The successor adds authored syntax and predicate membership, including exposed `trust`; this is rule-assisted open-development repair, not untouched independent learned transfer |
| September 18–19, #1283 | TinyStories training, JEPA/discrete tables, hierarchical lattice, VSA, skip-engrams, binary artifact and chat entry point | Significant engineering and natural-text training effort; a separate model path from the retained dependent-language artifact |
| September 19, #1284 | Ablations, codebook/attention repairs, stripped artifact, group table, D0-b and low-bit primitives; instruction attempt failed | Useful corrections and honest negatives. Continuous training score and served model score diverged; no chat capability |
| September 19, #1286–#1290 | Ordered bucket memory, optional group filters/composition, synthetic experiments, operation counts and static real-text baselines | Current research core; no trained real-text or general-generation result for this core yet |

Three model paths must stay distinct: (1) historical exact-memory/dependent-language models, (2) the TinyStories `.rgm` prose path, and (3) the new `GeometricAttention` research core. Running old retention tests does not qualify a new artifact. In particular, `scripts/verify_qualification.sh` loads historical dependent-language evidence, not the prose `.rgm` or the new core.

Preserve the exact-memory infrastructure, learned Read/Emit/Stop and continuation controls as later integration assets. Preserve the group's exact operations and representation-aligned VSA routing as mechanism donors. Reuse the Rust tokenizer/corpus tooling and artifact discipline. A common language implementation still has to connect these assets and earn broad behavior.

## The current core, stated mathematically

For the default order-2 path, let `e(x)=x mod 120` and `a_i=120 e(x_(i-2))+e(x_(i-1))`. Before predicting token `x_t`, the state contains

```text
S_t[a] = sum of v(x_i) for earlier observed transitions with a_i = a, 2 <= i < t
logits_t = W_o N(S_t[a_t])
```

`v` and `W_o` are learned power-of-two-scaled ternary values/readout; `N` is the implemented power-of-two normalization, optionally followed by ReLU. `observe`, `logits` and `ingest_all` in [geometric_attention.rs](../../crates/uor-r4-core/src/native_geometric/learner/geometric_attention.rs) implement this behavior. The address identifies a bucket; the contents depend on the prefix.

**The last run's “entire context is the residue pair” and “ceiling” conclusions are incorrect.** Histories `[0,1,2,0,1]` and `[0,1,3,0,1]` have identical query tails but write different successor values to the selected bucket. Suitable value/readout weights distinguish them. This is a source-derived counterexample, not a new executed test.

There is a second reason: a fitted Jelinek–Mercer estimator's held-out cross-entropy is an achieved score, not the unknown Bayes conditional entropy. For a fixed information variable A,
`CE(P,q) = H_P(Y|A) + E_A KL(P(.|A) || q(.|A))`.
The estimation gap need not be zero. A higher-order finite estimator can score worse than a lower-order estimator without contradicting conditional-entropy monotonicity.

Keep the existing recorded values, with corrected meaning:

| Internal development corpus | Unigram | Residue order 1 | Residue order 2 | Token order 1 | Token order 2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| Repository source | 8.7520 | 6.5785 | 5.0019 | 5.1736 | 4.1573 |
| Repository Markdown | 9.0957 | 7.3446 | 5.9546 | 5.7979 | 4.8955 |

These are reported bits/token from [the preserved receipt](../evidence/native_geometric_realtext_ceiling_2026-09-19.txt). The 81.6%/74.8% figures describe relative gains of these fitted baselines, not fractions of all available predictive information or geometric advantage. The shuffled refit is useful; lambda=0 selecting unigram explains the zero reported CE gain. These internal code/prose samples are neither an instruction corpus nor a final independent benchmark. Do not compare their BPB directly with TinyStories BPB.

**A genuine structural obstruction is the empty route.** The trainer resets state between sequences. An unwritten order-2 bucket has zero vector, and the default served readout has no bias. Its logits are zero, so its loss is `log2(4096)=12` bits at every such position. If `p_empty` is the measured fraction of scored empty routes, default mean loss is at least `12 p_empty` bits/token. Value cancellation can create additional zero reads. Corpus-trained count baselines have global priors that this reset state does not contain. Measure coverage under the actual window/reset policy before training for thousands of steps.

**The present order-2 operation is not a test of distinctive group geometry.** It concatenates two residue labels and reads one bucket; it bypasses the group convolution. Bijectively renaming the 120 labels preserves that addressing behavior. Composition and relative-element filters exist on separate paths, but zeta phases, chirality and group multiplication do not explain this order-2 baseline. This is a reason to design a geometry-sensitive control later, not to discard geometry now. An injective map from 4,096 tokens to 120 single roots is impossible; learned placement can improve collision structure but needs additional coordinates or exact side identity to preserve arbitrary tokens.

## Correctness findings to resolve before a substantial fit

All locations below refer to the audited source baseline, and remain unexecuted findings in this review.

| Finding | Source at audit baseline | Required verification |
| --- | --- | --- |
| Short-prefix unsigned subtraction precedes fallback padding | `geometric_attention.rs` lines 393, 566, 744 | Explicit checked padding; empty/one-token/order-2 tests with overflow checks. Wrapping release arithmetic must not define behavior |
| Value and kernel gradients apparently divide by sequence length twice | Lines 826, 976, 1025, 1041 | Independent analytic check of the declared mean-loss STE, including two lengths and parameter groups; raw finite differences of a quantized step function are not that test |
| Count tool says full-training refit but evaluates fit-only counts | `geometric-realtext-ceiling.rs` lines 501, 567 | Implement a declared refit protocol or correct the claim; record exactly which data fit counts and interpolation |
| Top-1 uses first observed context argmax, ignoring fitted interpolation | Count tool lines 428–437 | Derive CE and argmax from the same normalized distribution, with a lambda=0 fixture |
| Scoring starts after two tokens but BPB denominator covers the whole document | Count tool lines 419, 701 | Declare/align the scored byte population; tokenize/byte-account without hidden boundary optimism |
| Scope of kernel compliance is overstated | Named-symbol zero-check versus callers/callees and other modes | Report exactly inspected binary, symbols and reachable callees; no complete-path conclusion from one symbol |

Extra gradient scaling is a mismatch to the declared objective, but its effect on prior results is unmeasured: Adam can partly cancel a fixed factor. Likewise, failed filter optimization does not prove filter capacity is irrelevant or exclusively identify a bottleneck upstream. Diagnose these possibilities before making stronger causal claims.

The timing recommendation survives, with corrections. The order-2 trainer clears touched buckets after initial allocation; it does not clear the entire 1.84-million-value buffer every sequence. It does re-quantize both full parameter tables in each sequence's `accumulate`. Readout is dense in `V*dv`, and its operation count is useful, but neither 1,800 seconds nor “3–10 times slower” is a measurement. Time complete steps including allocation, quantization, forward, backward, optimizer and evaluation.

## Earlier metrics and research claims that need narrower scope

- The reported 555M training-time **1.2372 BPB** is not the deployed integer scorer. The September 19 receipt measured **1.8055** for a discrete full-vocabulary evaluation and **1.7989** for the stripped artifact; shortlist slices have other scores. Keep scorer, artifact and slice attached to each number.
- The “matched Kneser–Ney” baseline used the same evaluation tokens but was fit on at most one million tokens, while the native learner used the much larger remainder (`train-native-prose.rs`, lines 630–657). Its lower-order implementation uses raw counts rather than a verified standard modified-Kneser–Ney implementation. Call it the repository discounted n-gram comparator pending verification. The split is a contiguous token split, not demonstrated document separation. Card P3's recorded decision is historical; it does not settle geometric language-model viability.
- The 555M run duration was conservatively charged from a report without an independently recovered raw training receipt. Artifact existence/hash was verified here; the full training number was not rerun or independently qualified.
- September 17's repaired roles include authored auxiliary/predicate rules in `dependent_language/contextual_role.rs`; preserve the practical repair but do not relabel it learned held-out transfer. Future independent transfer must be fresh after selection.
- `617 tok/s` native versus about `36.1 tok/s` CPU Qwen was measured at unequal quality and different timing boundaries. It is not an efficiency win at equal usefulness. Energy/token remains unavailable in the recorded evidence; no new energy measurement was attempted.
- A count of static multiply instructions is not an executed count. A zero check excludes multiply instructions only within its inspected ranges and needs transitive call coverage for a whole-kernel claim. New-core allocations and complete serving compliance have not been qualified by the old allocation tests.

## Literature synthesis and mathematical corrections

The priority is a falsifiable geometric learner, with external mechanisms supplying alternatives when an observed failure requires them.

| Primary source | Useful transfer | Boundary |
| --- | --- | --- |
| [Linear Transformers](https://arxiv.org/abs/2006.16236), [Fast Weight Programmers](https://arxiv.org/abs/2102.11174) | Distinguish learned slow parameters, prefix memory, capacity and read/write semantics | A fast-weight analogy does not establish semantic geometry or exact retention |
| [DeltaNet](https://arxiv.org/abs/2406.06484), [Gated DeltaNet](https://arxiv.org/abs/2412.06464) | Corrective writes/forgetting when conflicting successors are the measured failure; chunkwise offline training | Their standard floating gates/normalization are not a drop-in D0-b kernel |
| [MatMul-free LM](https://arxiv.org/abs/2406.02528), [BitNet b1.58](https://arxiv.org/abs/2402.17764), [T-MAC code](https://github.com/microsoft/T-MAC) | Quantization-aware learning, additive/LUT readout, memory traffic measurement | Their titles do not establish zero float or zero variable multiplication throughout UOR-R4 serving, or a geometric contribution |
| [HadamRNN](https://arxiv.org/abs/2502.00047) | Structured binary/ternary recurrent alternatives if recurrence is revisited | A useful counterexample to family-wide impossibility, not a proven UOR language solution |
| [Poincaré embeddings](https://arxiv.org/abs/1705.08039) | Optional offline hierarchy learning, if a hierarchy improves routing/representation at matched cost | Hyperbolic coordinates alone do not guarantee cheaper computation or accurate candidate selection |

Several statements in the dated September 19 design reviews are withdrawn as universal conclusions:

1. **Ternary recurrence is not intrinsically incompatible with stable memory.** Signed permutations are orthogonal ternary matrices. A Sylvester Hadamard matrix of dimension `4^k`, scaled by `2^-k`, is orthogonal with power-of-two scaling. Finite-precision rounding and nonlinearities require their own checks. The measured random-matrix failures remain valid at their measured scope.
2. **Sequential serving does not imply necessarily sequential training.** DeltaNet supplies a chunkwise training construction. Whether a suitable Rust/M1 algorithm is worthwhile requires implementation-specific timing.
3. **The review does not establish a universal local-training impossibility or a billion-parameter minimum for coherent conversation.** Large-model scenarios are cost scenarios, not lower bounds. Frontier capability remains uncertain; training scale, data quality, sample efficiency and task scope require evidence. No paid/external compute is authorized by this review.
4. **Relative-element kernels are not synonymous with class functions.** `k(q^-1 g)` is invariant to simultaneous left translation. Conjugacy-class dependence imposes additional symmetry. Nine classes span the central/class-function subspace, not all functions on 2I; a class-indexed table is not automatically a band-limited spectral representation. See [group convolution notes](https://symm4ml.mit.edu/symm4ml_s26/notes/group-convolution) and [representation theory](https://klein.mit.edu/~etingof/reprbook.pdf).
5. **Equal cardinality is not an isomorphism.** The 14,400 ordered root-pair buckets do not acquire the Coxeter H4 group law because H4 also has order 14,400. Distinguish the 120-element binary icosahedral group, its root coordinates and Coxeter symmetry group.

These corrections reopen alternatives without launching another architecture survey or replacing the current core. The assertion that every learned representation requires a linear map is also not needed to justify D0-b; the owner-selected engineering allowance stands independently of that historical rationale.

## How the wider project mathematics can earn its place

| Family / source map | Concrete role to retain or investigate | Evidence still needed |
| --- | --- | --- |
| Prime router, angular routing, UOR-ADDR/Prism/Framework | Exact identities, ordered n-lets, bounded candidate/page access, canonical persistence | Routing recall and bytes/work at matched useful outputs; prime/hash identity is not semantic distance |
| H4/2I, exact Z[phi], SpiralCore | Finite composition, signed transport, orientation-preserving shared operators | Learned task representation for which geometry-sensitive controls affect predictions |
| W33 | Ordered finite operators and immutable shared pages/DAGs | Explicit mapping into the learner; old failed mod-9 mapping does not reject the whole archive |
| NEMESIS | Typed state-transition specifications and audit/replay interfaces | Concrete executable bridge and reviewed source/license; mathematics alone does not compile arbitrary models |
| HELM, Hopf/fiber and paired icosian work | Offline frame/gauge references, retained fiber/orientation, explicit encode/transport/decode domains | Matched geometry controls; S3 to S2 observation loses fiber without extra information; coupled companion is not independent learned state |
| Zeta phases, spectral/group kernels | Structured channels, clocks or learned relative transport | A stated prediction task and controls against random/learned alternatives; fixed phases require no classical RH claim |
| Hyperbolic and Navier–Stokes/Euler inspirations | Optional source of hierarchy or dynamics hypotheses | A concrete operator, bounded arithmetic, causality and quality/cost experiment; mathematical analogy alone is insufficient |

The source inventories above, [imports audit](architecture-2026-09/imports.md), [mathematics audit](architecture-2026-09/mathematics.md), and the retained SpiralCore v68 extraction keep these options accessible. No whole-model import is recommended. New upstream code adoption requires reading the actual source and refreshing its pin/license.

## Architecture and systems direction

Keep learned corpus knowledge distinct from mutable session memory. The current cold bucket exposes that missing connection. A candidate can combine a learned geometric prior/state with selected causal memory; it must demonstrate that both channels matter through controls. The smallest next change, if empty routes dominate, should supply a learned cold-context prediction and train it through the same discrete path, with an explicit fallback marker and matched comparisons. Do not simply inject the global count baseline and call it learned geometric progress.

Then address value interference, useful composition and candidate pruning in the order actual measurements demand. A shortlist must report target recall, probability/mass treatment and quality loss at equal data/artifact scope; a projected 480-fold operation reduction omits routing-score cost and is not measured speed or energy. Geometric placement should preserve exact token/occurrence identity separately from coarse addresses.

For durable memory and eventual distributed use, bind artifacts to tokenizer, data, source, geometry and operator versions; bind mutable state to an artifact and identity scope. Use explicit causal writes, conflict/version/eviction semantics and deterministic replay. Immutable UOR-addressed records and copy-on-write pages are useful when context scale warrants them. Do not introduce distributed coordination or remote serving to solve the immediate local learning failure. The frozen no_std R4G1 contract remains separate from the new native core.

The roadmap sequence is: correctness and causal coverage → measured small real-text learning → useful cold-context prediction and contextual gain → measured geometric contribution and bounded candidate routing → sustained prose/instructions → exact-memory/operator integration → executed coding and reasoning → quality-matched laptop cost → API/WASM/Studio qualification. These are evidence gates, not guarantees that the mechanism will succeed.

## Resources and preservation

The live JSON at recovery recorded `146438565 / 154400000 ms`, leaving `7961435 ms` (132.69 minutes). This is the operational recorded balance, not a newly certified sum of every historical charge. [The ledger reconciliation](resource-ledger-2026-09-19.md) distinguishes ordering repair from unresolved arithmetic/receipt discrepancies. No balance was silently increased or decreased in this review.

The established storage inventory measured 51,761,754,112 free bytes at 2026-09-19 21:21 UTC. Model storage was only a lower bound, 20,435,673,088 allocated bytes, because three sealed paths were unreadable; they were not opened or chmodded. Rows overlap and must not be summed. The 128 MiB stop margin remains required. The review adds text and an isolated checkout only; no model output, paid compute or deletion.

Fresh SHA256 identities: original `.rgm` `7023c4507039acc01653b2dd6a41cf4329635767c66811a17d76591a4f356d96`; stripped `.rgm` `a25a87c4c5cdd5f7e6af1479273e29cc996d3cc78a1984315af8cf9b70a10a1b`; source tokenizer `9ca9acddb6525a194ec8ac7a87f24fbba7232a9a15ffa1af0c1224fcd888e47c`. The new core has no promoted real-text artifact. Original receipts and negative candidates remain intact.

## Next owner/agent handoff

Use [the self-contained DeepSeek prompt](deepseek-next-step-2026-09-19.md). DeepSeek is well suited to bounded implementation, controlled experiments and receipt production. Architectural promotion should review the actual diff, gradients, distributions, generated outputs and retained controls. Every run ends with one current-state update and the owning GitHub issue outcome; long historical reports remain evidence, not competing live instructions.
