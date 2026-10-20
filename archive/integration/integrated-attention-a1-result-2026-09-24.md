# Integrated attention A1: loaded model, failed capability, identified learning boundary

September 24, 2026 · References #973 and #820 · Delivery PR #1386.

## Decision

The owner-adopted [programme](principal-attention-plan-2026-09-24.md) now has an implemented, fitted, exported and reloaded Rust prototype. One artifact joins contextual finite-code formation, bounded occurrence memory, relative energy, geometric state composition, read/write gates and a sparse normalized language head. The **capability result is negative**: generated language is degenerate and the loaded model does not select the annotated source correctly. Milestone A remains open. This candidate does not replace the retained native models or change the application's default model.

Retain this implementation and its negative artifacts as the concrete base for improving learning. Prioritize the query/key representation and the credit from language loss to admission/ranking in this same model. Additional gates, a larger root set, a geometry promotion claim, or a fresh benchmark campaign would not resolve the observed failure.

## What was built

The implementation lives in `crates/uor-r4-core/src/native_geometric/learner/integrated_attention/`, with the executable driver `crates/uor-r4-core/src/bin/integrated-attention.rs`. The [implementation contract](integrated-attention-a1-plan-2026-09-24.md) describes source ownership and exclusions.

- Four finite state lanes; the geometric arm uses the binary icosahedral group 2I, and the ordinary finite arm uses C120. Product and inverse tables are constructed offline and serialized into the artifact.
- Exact bounded raw-token events are retained independently of learned indexing. Owned one-token records have exact occurrence keys. Learned product-code prefixes admit at most 64 records while examining at most 512 posting entries.
- Relative energy has four unary and four pair factors. The reader ranks admitted records and can decline a read. Local frame actions in this first reader are identity; nontrivial learned frame transport remains unimplemented.
- A contextual encoder selects five 120-action rows per lane. The output selects six feature rows plus bias on a binary vocabulary tree, with normalized Generate/Copy/Stop modes. Copy and Generate of the same one-token value enter the same following state, making their surface probability sum exact at this scope.
- Offline Rust sparse SGD uses local losses and truncated state credit. The hard model stays fixed for each episode; updated parameters become visible in the next episode. This is not full sequence differentiation through hard memory routing.
- Artifact binding includes source, tokenizer, data-manifest identity, seed, training contract and parameters. Loaded validation uses integer group tables. The same loaded session is exercised after snapshot serialization/restoration and in a separate process.

Four-bit-valued encoder, output and gate coefficients are initially stored in bytes; energy coefficients are packed. IDs and exact payloads are not four-bit numerical weights. The complete served numerical path still needs a machine-code audit before a D0-b kernel qualification claim. No electrical energy measurement was made.

## Data and provenance

Both arms use the same pinned 4,096-token BPE, seed `71679248566564`, 50 fit episodes and 14 development episodes. Each fit sees **46,584 tokens across two epochs**. Development scores **6,160 tokens**. Natural material consists of separate project documents and Rust source files, capped/chunked prospectively; four authored fit and two authored development correction cases are also included. All material is exposed development data, not a final holdout. The data loader verifies source hashes and exact whole-stream BPE byte alignment.

The 1,421 development source annotations include weak natural recurrences selected using the eventual target token. These are not all uniquely identifiable from the available query and are not semantic truth labels. Admission/selection counts expose the current mechanism, but their failure must not be interpreted as a theorem against learned addressing. Correction answers and source pointers never enter the serving interface.

The combined fit/development manifest BLAKE3 is `e3b2bb4e077484b558ec87ba702c71a364d63732e0ba97c7b307862f93d28601`; the historical artifact field name is `training_data_blake3`. Tokenizer SHA256 is `a7ac75b68aa997fe7cc338d25d843157f2dc9d4265fb2f958ee779bb04828d6f`.

Retained attempt container: `/Users/casey.allard/uor-r4-investigations/integrated-attention-20260924/`. Every fit and replay directory was exclusively claimed, sealed and verified. The [checked-in evidence extract](../evidence/integrated-attention-a1-result-2026-09-24.json) contains identities, complete generated text/token IDs, prompts, counters and the initial/final component digests. The local sealed roots retain full step traces and per-episode metrics. Both source-bound executables are preserved in the container.

| Attempts | Source commit | Model SHA256 |
|---|---|---|
| `c120-dev1`, `c120-replay1` | `de3c2acd9dd0f781492e3a61b836169a1f3d760f` | `0ac00e1864a590b34fde1a2edf25084127c9a85d6d88d390ff55983e229d41fe` |
| `geometry-dev1`, `geometry-replay1` | same initial source | `3a83acc82aab128205af08a051c38898f6c881a05e802177f089ae4604c17333` |
| `c120-dev2`, `c120-replay2` | `0e969e7203090aeefde276c43d7599aae7d70729` | `cf151296baa9be5b95424fd2e918e0e31c06b39516e681f688dbcb6450a4ce03` |
| `geometry-dev2`, `geometry-replay2` | same corrected source | `5721cad2300041d14eccf8e2a4f65d940f20c027fcefcac5ce3e423149732ae2` |

Initial executable SHA256: `88576301a318e6563486c9b8484d13f3a474c87437839c01515a3375c2799910`. Corrected executable SHA256: `5723d013dd759c5f79203b8d55f9c71ec316bd9303a9fa2b17728efcbf562726`.

## First integrated diagnosis and one correction

The first two fitted artifacts each produced **zero indexed writes and zero reads** on all 6,160 development tokens. Read-enabled and NoRead likelihoods were identical: C120 **9.408436** and 2I **9.615514 bits/token**. Generation was degenerate. All five learned component digests changed, so parameter mutation alone would have hidden this failure.

The write objective incorrectly treated every occurrence without a weak future-use annotation as a negative example. In an imbalanced BCE objective the intercept can minimize loss by disabling writes. Missing annotations do not establish that an occurrence is unimportant. The [recorded prospective amendment](../evidence/integrated-attention-a1-write-warmup-amendment-2026-09-24.json) therefore changed only this supervision: useful annotated writes get positive credit; unknown occurrences do not get negative credit. The gate starts enabled, and these episodes fit within the 4,096-record budget. This is **all-write warmup while learning access**, not learned selective writing. Both arms restarted from the same initial configuration and seed, with the same data, epochs, rates and read-gate objective. The first failed pair remains sealed.

## Corrected loaded observations

| Measurement | Ordinary C120 | Geometric 2I |
|---|---:|---:|
| Indexed writes / development tokens | 6,160 / 6,160 | 6,160 / 6,160 |
| Annotated source admitted / 1,421 | 271 | 251 |
| Annotated source selected correctly | 0 | 0 |
| Reads on all 6,160 predictions | 122 | 24 |
| Selected read token equals actual next token | 3 | 1 |
| Read-enabled bits/token | 9.383427 | 9.533292 |
| NoRead bits/token | 9.380686 | 9.554751 |
| Correct complete correction answers / four source variants, reads enabled | 0 / 4 | 0 / 4 |

The geometric arm's small likelihood difference against its own NoRead control does not qualify useful memory or geometry. The ordinary arm has the better absolute likelihood in this initial matched pair. C120 is only an initial finite comparator; the complete programme still requires a competent ordinary sparse learned router, identifiable frame tasks and nontrivial transport controls before geometry promotion. These single-seed, exposed development results do not settle the architectural question.

In each of the two correction episodes, the annotated corrected-value source is admitted on **14/14 answer tokens**, but selected on **0/14**. Aggregate admission alone would misdiagnose these cases. Also, `source_selected` is measured after the gate: zero final selection does not establish that ranking itself never found the source. The bounded diagnostic below separates these decisions on the existing artifacts.

For each of the two correction prompts, the driver changes only `allowed`/`denied` in the current correction clause, retaining the entity, other raw text and question. The question is generated from raw prefix tokens; its target suffix is excluded. Tokenization may change the number of tokens because the edited words differ. Each variant is run with reads enabled and disabled. Two additional prefixes exercise prose and Rust continuation. Each row emits up to 32 tokens using greedy binary branches, not global vocabulary argmax.

Actual geometric output for the original Orion prompt:

```text
:::encherialencherialencherialencherialencherialencherialencherialen
```

All six geometric read-enabled generated continuations made zero selected reads and matched their corresponding NoRead continuations. Source edits changed some strings through prefix state in both arms; this is not a demonstrated memory-mediated answer. The ordinary model made two selected reads across its read-enabled decoded continuations, but its answers were also malformed. No row used Copy. No generated Rust program or useful prose result is claimed.

All **48 generation rows** across the four fits reproduced after local snapshot restore and byte-for-byte in the corresponding new process. The full evaluation JSON also reproduced byte-for-byte in each pair. This demonstrates deterministic replay of these observations, including their failures.

## Read-only diagnosis: the candidate codes collapse and evidence is not yet useful

The additional `integrated-attention-inspect` driver examined only the existing two correction episodes, **without fitting or changing either artifact**. Evaluator source is `114d9b0acaaaa1f653df2151cf7a2672051d7c46`; executable SHA256 is `c0f7cf612dd6802682fe4cb75c6a80e2e68800c845dac2d645fd7a59c0d8640c`. Sealed roots are `c120-inspect2` and `geometry-inspect2`. [Complete rows and provenance](../evidence/integrated-attention-a1-source-diagnosis-2026-09-24.json).

For **all 28 answer positions in each arm**:

- The annotated source is written and admitted.
- There are 64 admitted candidates but only **one distinct complete four-lane code**. All 64 candidates share the annotated source's code.
- All 64 energy scores tie. The correct source ranks first **0/28 before gating**; the gate opens **0/28**.

This is measured code collapse within these candidate sets, not a claim about every token or every possible state. For example, the geometric Orion source ` allowed`, query and every candidate have code `[2, 30, 3, 1]`; all energies are 15, and the newest tied candidate emits the unrelated token `ate`.

The representational consequence is exact: if `k_i = k_j`, every factor of this reader's `H(q, k_i)` and `H(q, k_j)` receives the same input, so the energies are equal for any learned coefficients. The occurrence-ID tie break supplies order but cannot recover the missing entity/value distinction. More energy training or a scalar read-gate threshold cannot separate those records.

The probe also scores the head with the annotated source token as an **offline one-step intervention**, while retaining the actual causal prefix state and never injecting that source into `Session::read` or subsequent observation. Across 28 answer tokens, forcing the correct source changes NLL by **+5.137353 nats for C120** and **+2.534511 nats for 2I** (positive is worse). The geometric Orion case improves slightly, but the Larch case worsens. This is evidence that the output head has not learned reliable use of the correct value either; it is not an oracle-served success or a full counterfactual rollout. The next change must repair both discriminative codes and source-conditioned language credit.

## Access and machine cost

Each model stores **40,038,150 parameter bytes**; serialized artifacts are approximately 40.09 MB. The corrected read-enabled generation traces inspect **7,817 to 7,817.22 learned coefficients per emitted token** (averaged per 32-token continuation). NoRead inspects 7,298. The encoder accounts for 7,200 of those reads: query, key and state-action code formation are included. These are instrumented logical accesses, not measured DRAM traffic. Every read-enabled decode search reported truncation and returned 64 candidates in these corrected continuation rows; full-history optimality is not established.

The fits plus their development scoring/generation took 2.54–2.86 seconds per process under `/usr/bin/time -l`; maximum observed model-run RSS was **393,330,688 bytes**. These times do not include source implementation or compilation and do not establish general serving throughput, language quality, or an energy saving. Separate process replay took about 1.15–1.19 seconds. All model execution was local CPU work. The cumulative cycle ledger also charges preparation, builds, diagnosis and delivery.

The [cycle closeout](../evidence/integrated-attention-a1-closeout-2026-09-24.json) charges **4,227,466 ms**, including an explicit five-minute delivery reserve, to the shared cumulative ledger: **469,466,786 / 477,000,000 ms**. All four fits total **186,336 training tokens**, below the prospectively amended 200,000-token cycle cap. Ten sealed attempt directories and the preserved executables occupy **166,756,352 allocated bytes**. At closeout the filesystem has **30,330,789,888 bytes free**, above the preserved reserve and stop margin. The earlier cache cleanup recovered 4,839,772,160 bytes; subsequent builds and retained evidence consumed part of that headroom. No unique artifact was deleted and no paid compute was used.

`FitMetrics.access` omits output training reads and is not a D5 serving trace. Teacher-forced evaluation counts vocabulary/Copy likelihood scoring separately. The generated-step counters above come from actual `Session::read`, `Session::output` and `Session::observe` calls.

## Validation and remaining integration

- Initial source passed the offline library and driver checks; **25 focused integrated-module tests passed**, with 1,129 unrelated tests filtered out. They cover group arithmetic/invariance, signed coefficient constraints, causal memory/eviction, allocation-limit validation, normalized token/mode support, training signs and loaded snapshot prediction.
- Both source commits built the release driver. The final write-supervision correction was exercised in the two complete fresh fits and their separate-process replays. The unchanged arithmetic tests are not relabeled as a newly rerun full suite.
- Named Rust files passed `rustfmt --check`; capability wording and whitespace checks are recorded at delivery. The PR's queue compatibility statuses are not model validation.
- The raw event ring has a bounded retention window. This prototype is not unbounded exact memory, an authenticated action-history proof, learned entity/relation/version binding, cross-source memory, multi-token Copy, prime/zeta integration, a physical Hamiltonian flow, or a demonstrated LLM replacement.

## Next development decision inside Milestone A

The next model change is **joint candidate-to-answer credit with discriminative contextual addressing**, continuing in this artifact/session path. It should join a causally available coarse key with a task-trained finer key, energy, read permission and output head; retain exact value/occurrence identity separately. The present index requires lane-0 equality, while keys receive observed value evidence and queries do not. Pulling a query toward a moving hard positive key, without a balanced contrastive code objective, permits both admission mismatch and overloaded pages. The targeted loaded diagnosis now demonstrates identical candidate codes in both correction answer sets, plus an output head that does not reliably benefit from the correct source. Tuning only the final gate or only the page index cannot be justified as the complete repair.

Implement the next training interface around query-identifiable sources and natural-text distractors, using the existing observed-span/role and scoped-memory mechanisms as contracts to integrate, not gold fields supplied to serving. Use contrastive source alternatives and page-capacity pressure together with candidate-conditioned language loss over admitted records and NoRead. The output head must first learn how supplied evidence changes an uncopied consequence; training-only source scaffolding may expose that connection, but exported serving must receive raw text alone. A pure lowest-current-loss selector can otherwise lock in NoRead before evidence is useful. Connect downstream language credit to query/key and state choices rather than relying exclusively on the present one-step surrogate. Preserve all-write warmup until selective-write utility is measurable; a missing annotation remains unknown. The conditional next representation choice is a value-blind coarse admission code followed by discriminative context/value factors; inspect its occupancy and hard-source routing during normal model learning before committing to larger geometry.

The complete next deliverable remains one loaded model that uses a source beyond its local token prior to produce an appropriate **uncopied** consequence and useful prose/code continuations, compared with its ordinary arm and NoRead. The four authored fit cases are insufficient language training. Reuse the preserved natural-text corpus with a declared larger fit window after this addressing/credit repair, within a new prospective cost projection. Do not turn this finding into a new sequence of isolated pass/fail fixture gates. Keep final independent evaluation separate until a useful design is selected.
