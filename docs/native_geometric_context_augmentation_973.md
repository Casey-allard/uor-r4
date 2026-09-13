# Matched construction-context augmentation — #973

**Actual gate: FAIL_CONTEXT_AUGMENTATION_DEVELOPMENT_GATE. Method gate false; both numeric model gates fail. No promotion; retain 15baec48.** Two matched state-learning arms execute once from 8f8e5c34 with its angular decoder frozen. Varied contexts increase accepted-update support and improve their own training loss, but worsen the recombination probe versus both the parent and repetition. Neither arm generates useful language or Rust.

## Matched intervention

Rust prepares four document-major copies of each original document for the repetition arm and the original plus three distinct length-preserving whole-word variants for the varied arm. Each arm has 48 documents and 2,688 byte/EOS positions; repetition has 12 unique contents and variation 48. The substitutions preserve declared relations, arithmetic and Rust bindings. Twelve disjoint recombination probes contain 672 positions. These are authored construction-family conditional probes, not an independent broad holdout. Their results are now open development evidence.

Both arms start from the same exact 8f8 parent and execute 8,192 proposals with seed 20260913 over the complete TRANSITION/QUERY/KEY/READ domain [1024,2224). Every proposal, including 66/65 no-ops in the repeated/varied arms, evaluates the full 2,688-position causal objective. The identical saved index/root stream accounts for 16,384 evaluations and 44,040,192 proposal positions in total. A change persists only when full mean NLL improves by more than 1e-10. Decoder, embeddings, geometry, phases, caps and null fields remain fixed. Unique-document support is measured on the prior state after selection for descriptive analysis; it is not a selection penalty.

Data SHA256: 6b9f792005776b43c2a5a40e0b7c04b8462e28816359c74ae1c49c0b781aecc4. The sealed Rust dataset records source-corpus identity and explicit variant/probe lineage. The two final states are saved and reloaded as explicit source-bound test witnesses before probe scoring; no changed clone inherits a normal artifact CID. Repeated witness: blake3:0c93455843f4aa6e2730ef593730a2b8c9b7afdf6ef6ff33b538d63cbbb2ab62. Varied witness: blake3:0c43bf3f4fc0361ac7c0b3b05d0b361bf01667155705337e4c59907eb05ba471.

## Actual behavior

| Measurement | Repeated arm | Varied arm |
| --- | ---: | ---: |
| Own training NLL, before → after | 2.839534 → 2.830845 | 3.592353 → 3.491363 |
| Own training correct / 2688, before → after | 664 → 680 | 409 → 439 |
| Accepted updates / final changed entries | 11 / 11 | 64 / 55 |
| Accepted READ / QUERY / KEY / TRANSITION | 7 / 2 / 2 / 0 | 48 / 8 / 8 / 0 |
| Median / maximum unique-document support | 1 / 7 | 5 / 22 |
| Accepted updates touching multiple unique documents | 5 / 11 | 59 / 64 |

Own training losses refer to different texts; their absolute values are not a matched quality comparison. No TRANSITION update was accepted in this finite stream. The preceding exhaustive profile found four improving transition settings, so this does not prove recurrence cannot learn.

| Full panel | Parent correct | Repeated correct | Varied correct | Parent NLL | Repeated NLL | Varied NLL |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Original construction / 672 | 166 | 170 | 149 | 2.839534 | 2.830845 | 3.010464 |
| Opened development / 381 | 52 | 52 | 48 | 3.653604 | 3.671485 | 3.670222 |
| Recombination probes / 672 | 100 | 93 | 93 | 3.731054 | 3.713508 | 3.790710 |

On the recombination probe, ContextDisabled is exactly identical across parent and both arms: 103/672, NLL 3.763306. Varied Full is worse than its ContextDisabled control. Wider access support therefore does not establish useful context learning; the acquired query/key/read choices are harmful on this panel under the frozen decoder. Repetition slightly lowers probe NLL but loses seven correct predictions.

Across the six original construction/opened × Full/ContextDisabled/StateDisabled panels, repeated lost-correct totals against 8f8/1198/0f82/ade9/aaa1 are 18/207/198/183/25, with gains 17/153/413/405/22. Varied losses are 58/240/207/191/59, with gains 28/157/393/384/27. New losses are never cancelled by gains. Original Full repetition gains four without loss; opened Full loses ten and gains ten. Variation loses 25/gains eight original Full rows and loses 19/gains fifteen opened Full rows.

Aligned root/source identity Hamming versus 8f8 is 717/231 for repetition and 1547/377 for variation; prediction Hamming is 198/425. These are categorical disagreements, not semantic distances or mathematical evidence of dynamical instability. Per-panel and per-prior deltas are retained in the evidence.

Each arm's four original Full prompts and four new Full prompts produce incoherent 96-byte continuations without EOS: all 16 arm outputs fail useful generation. The four parent continuations on new prompts also fail. The Rust prompts produce no usable program; generated-code execution is not claimed. Numeric gates and this manual coherence assessment both prevent promotion.

## Validation, preservation and resources

The release build and two focused tests pass (dataset structure/arithmetic and driver stream/acceptance/lineage); two manual experiment tests are ignored in that focused invocation. The separate Rust preparation and actual paired experiment execute successfully. Execution success is distinct from the failed method/model gates. Independent static review verifies every saved proposal tuple, full exposure, acceptance decision, loss continuity and actual outputs without more model calls.

All 15,795 saved prior rows and 60 prior outputs replay exactly. Seventeen older sealed roots / 668 files, the 22 parked dirty paths and retained model hash verify unchanged. The new dataset and paired attempt are separately claimed/sealed. Production runtime/training/artifact source and exact geometry remain unchanged. Both 512-step runtime censuses observe at most 23 prediction angular comparisons, within bound 27; allocation measurement is NOT_RUN. No full suite, fresh broad holdout, general-language/coding qualification or complete-path energy claim is made. GitHub compatibility acknowledgements are not tests.

[Tracked evidence](evidence/native_geometric_context_augmentation_973.json) binds 139 source files, compiler, pinned binary, reports, independent reviews, deltas and resources. Active worktree: /Users/casey.allard/uor-r4-worktrees/shared-geometric-core on codex/geometric-context-augmentation. Original evidence and navigation remain under /Users/casey.allard/uor-r4-worktrees/initial-previous-intent/.uor-handoff/2026-09-12-codex-v7/shared-core-first-step/context-augmentation-1/. The dirty memory repair and V3–V7 remain parked.

Model/build/preparation charge: 135,526/420,000 ms (102,083 build/focused tests; 187 preparation; 33,256 actual paired command, 33,178 internal experiment). Parent cycle: 2,674,644/3,000,000 ms; shared ledger: 122,425,217/132,950,000 ms. A necessary 200,000 ms local-cycle extension was recorded before use under standing authorization; the shared ceiling is unchanged. Added storage at execution close: 53,252,096 bytes within 128 MiB; parent growth 1,658,023,936 bytes within 2 GiB, retaining the 128 MiB stop margin. Peak sampled RSS: 2,452,766,720 bytes. No paid compute or cleanup. Final engineering and delivery receipts append locally.

## Research and next decision

The local knowledge map and prior credit-assignment/source records informed the matched complete-trajectory intervention. Existing SpiralCore/FBS research continues to distinguish intrinsic routing/state changes from useful emitted behavior; no Bell/IP semantic weights or donor runtime are imported. [JEPA research](native_geometric_jepa_research_973.md) remains available as a candidate auxiliary objective; no JEPA model or loss is implemented here.

This result rejects this dose of context augmentation as sufficient for transfer under the frozen decoder. It does not reject diverse data generally, establish a geometric capacity limit or show that JEPA is necessary. The decoder was fitted to the original construction data and starts with substantially worse loss on the varied texts. Decoder adaptation is therefore an unresolved interaction. The earlier coupled-head profile tested two particular coordinates on original data; it did not test these new frozen multi-parameter states and varied contexts.

**Next action (NOT_RUN):** One bounded four-cell conditional decoder-refit comparison on frozen state snapshots: parent+repeated data, learned-repeat+repeated data, parent+varied data, and learned-varied+varied data. Use the saved 48-document datasets, exact geometry/caps, one common depth-three/minimum-leaf64 rule (matching minimum-leaf16 on originals repeated four times), and no further state proposals. First require parent+repeated fitted branch payload to reproduce the original 8f8 decoder; metadata appropriately differs. Stop and diagnose a failed reproduction before interpreting other fits. Use explicit source-bound conditional-head witnesses with complete state/head/data provenance, not inherited normal artifact CIDs. Freeze all four fit calls and complete resource/acceptance limits before execution; select each head on its own construction data only, then compare now-open recombination probes, all prior rows/controls and actual generation. Separate decoder-only adaptation from the additional effect of learned state. No tuning against probes, new regularization family, JEPA loss, automatic fit campaign or promotion is included. JEPA remains a candidate auxiliary objective after this unresolved decoder interaction is measured.
