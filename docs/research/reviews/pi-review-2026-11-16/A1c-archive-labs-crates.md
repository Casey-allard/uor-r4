# A1c — archive, docs/labs, crates (PI review slice, 16 Nov 2026)

Read-only, at origin/main `ef1f4a641` (worktree payload-probe-387). Each claim is marked [checked path:line or path] or [hypothesis].
DEV below means `ft/dev`, 14,287 eligible targets, the same denominator in every M1-era lab.

## 0. Headline findings

1. **Count tables beat every trained model on the M1 DEV panel.** On ft/dev (top-1 of 14,287): query-token bigram 6,300 (44%), exact read-code table 3,414, unigram 1,226, trained Q4 writer 149 [checked docs/labs/m1-w2-capacity-capture/README.md:13-16]. The M1-closing saved model scores 455 top-1 and 1,484 top-8 [checked docs/labs/m1-close-saved-model/README.md, bar (b) line]. The best trained arm seen is FBS offline float at 2,520, which then collapses to 1,277 by step 1,024 [checked docs/labs/spiralcore-fbs-offline-float-sheet/README.md:64-67]. So M1 closed below a unigram.
2. **Softmax-era negatives no longer hold.** Several reads were dropped because they lost to softmax or needed a softmax surrogate. Softmax is now banned outright, and a lawful hard read exists: the Pick read, an integer first-pick with least L1 address distance and the latest row winning ties [checked crates/uor-r4-native-vector/src/read.rs:1-11; commit ef1f4a641 #4045].
3. **Table and row engines are the cheap path to 1e8–1e9 parameters.** An n-gram table, product-key and E8 orbit slots, an inverted file and packed 4-bit or icosian weights already exist. See §4.

## (a) archive/CATALOG.md and the archived engines

The catalog holds 1,175 records in 20 research lines (positive 214, negative 193). It indexes records by line rather than listing engines [checked archive/CATALOG.md:11-26, 28-49]. The engines below are rebuilt from archive/catalog.jsonl (jq over `line`, `outcome`, `one_line`).

| Engine / line | What it was | Best measured number | Why retired / status |
|---|---|---|---|
| R4G1 transformerless graph compiler (r4g1-transformerless, Jul–Sep) | Cross-compiles a teacher LM into a packed graph artifact (crates graph-*) | Ordinary lexical served model 6.27 bits/target vs parent 7.03; count reference still wins by 1.56; packed signed4 cuts coefficient bytes 87.35% with output identical [checked catalog.jsonl: integration/principal-continuation-result-2026-09-23, ordinary-lexical-formulation-result, packed-integer-comparison-2026-09-27] | Superseded. Built on a transformer teacher (L-006; the owner banned transformer controls) [hypothesis on the exact retirement cause] |
| Source-free integer n-gram table (m4-serving #989) | Exact integer n-gram table predictor | 22.3% held-out lexical top-1 vs 5.4% unigram; one exact radius tie-break +0.95 pt, byte-identical [checked catalog.jsonl docs/source_free_table_baseline_989.md, ..._geometric_intervention_953.md] | Not retired for cause; labelled "not semantic". **Revival candidate.** |
| Skip-mix table lane (m4-serving #906/#908) | Table-candidate injection lane | +28.45‰ top-1 over base (CI 25.6–31.3); +34.5 in the R4G1Runtime bundle [checked catalog.jsonl docs/skipmix_endtoend_causal_908.md] | Went with the R4G1 stack [hypothesis] |
| Softmax-free rank-weight read (m4-serving, 10 Oct) | Fixed or learned rank-weight table in place of the served softmax read | 0.017–0.26 bpb worse than softmax; "softmax stays in serving" [checked catalog.jsonl docs/labs/softmax-free-read-2026-10-10] | **The reason is gone**: softmax was banned two days later (no-softmax-2026-10-12). Re-score it against the Pick read and count tables. |
| Starling read (m1-float-rung0, 11–13 Oct) | About 7 geometric neighbours, hard selection | 0.78 recall on 5/5 seeds (512 synthetic) after light distillation [checked catalog.jsonl starling-recall-2026-10-12]; from scratch it never learned (no gradient to unchosen keys) [checked starling-read-2026-10-11]; collapsed at 1536 context because each cell keeps only 4 keys [checked starling-context-2026-10-12] | Lost to softmax and needed a softmax surrogate, which D23 banned. **The reason is gone**: hard reads now train natively, and rows-K is lifted to 4,096 (commit 19f1cb2e3 #4044), which addresses the 4-key eviction [hypothesis that it carries over] |
| S1/S2 float stack qualification (13 Oct) | Float geometric stack | — | Stopped when float model compute was banned. D37 (18 Oct) re-allowed offline float [checked catalog.jsonl s1-s2-float-preservation-2026-10-13, d37-offline-float-2026-10-18]. D85 now defaults to native integer, so it applies only partly. |
| Geometric stack (geometric-stack-pretraining, Sep) | Hamilton/H4 quaternion stack with MLP | Parity with a 7M transformer on repo code (1.998 vs 2.011 nats); quaternion transport beats identity by 0.071–0.077 nats; 4-bit QAT export within +0.0106 nats, bit-identical to NEON [checked catalog.jsonl geometric-stack-cycle4-2026-09-27, transport-attribution-d1, d4-geometric-s1-qat-result] | Carries an MLP and matmul (owner direction lock); the transformer control is banned. Generated code "useless". |
| Lorentz/hyperbolic cache read (Sep) | Learned Lorentz cache | −0.0186 nats vs Euclidean/dot; integer form keeps 99.3% of the gain [checked catalog.jsonl geometric-lab-phase2-2026-09-26] | Small gain; parked [hypothesis] |
| Exact memory / read binding (m2-exact-memory, Oct) | Rank-to-rival training, read-binding supervision | 177 → 437/512 complete replies (pooled rank) [checked catalog.jsonl m2-pooled-rank-2026-10-10]; wrong values 49 → 23 of 120 | Pooled rank used softplus (banned). It is re-registered with a hinge in docs/labs/native-geometric-reregister-hinge [checked README, table row 1] |
| Whole-path source (M2, 12 Oct) | Compiler + exact store + E8 index + emitter, shared credit | CPU fixtures only | "Later softmax bans blocked the sampled producer" [checked catalog.jsonl whole-path-recovery-2026-10-12]. **Unblocked** if the producer becomes a hard pick [hypothesis] |
| E8 orbit / product-key memory (11 Oct) | Fixed E8 product-key slots (240×240 = 57,600 per layer, k = 8) in place of FFN layers | Lost to a learned product-key control by 0.0231 BPB; no routing collapse [checked docs/labs/e8-orbit-memory-sigmoid-free-iteration/README.md, lines 1-2 of the result] | Configuration negative. The learned product-key read cut stored-value mistakes 47 → 34/120 at a cost of 37M parameters [checked catalog.jsonl addressed-read-2026-10-11]. **Scale candidate.** |
| VSA hash binding (9 Oct) | Fixed SplitMix64 4,096-bit codes | −0.014 to −0.023 BPB [checked catalog.jsonl vsa-native-test-2026-10-09; docs/labs/r-vsa-hamming-binding] | Small positive, not carried forward [hypothesis] |
| Icosian-mantissa weights (11 Oct) | Two icosian entries + 2-bit mantissa, about 19–20 bits per weight | Matches a dense model of equal size; no multiplier on the weight side [checked catalog.jsonl icosian-mantissa-2026-10-11] | **Revival candidate for the 16 GB laptop**: 1B weights at about 2.5 B each ≈ 2.5 GB [hypothesis arithmetic] |
| Native geometric Sept (native-geometric-sept, 294 records) | Hand-built role/writer rules, bounded scope | Many 100% bounded passes, e.g. 216/216, 492/492 [checked catalog.jsonl native_geometric_*_973] | Bounded scope; flexible prose generation is a recorded negative |
| Helm-D Lorentz manifold (Aug) | Learned manifold R4 construction | NLL 7.71 vs donor 3.67 [checked catalog.jsonl helm_d_learned_manifold] | Negative; parked |

## (b) docs/labs — families and each family's best result

There are 224 entries (ls). The families:

1. **K16 / attention-dose / rows / bvgk / counterfactual / a8 / admission / injective / address-cells (about 60 entries, 15–19 Oct, M1 GPU discrete attention).** Best: rows_v1 is 56× faster than the ordinary proposal kernel with exact parity [checked catalog.jsonl counterfactual-rows-parity-2026-10-18]. Address-cells checkpoint-0 visibility rose from 3.3% to 35.1% [checked address-cells-preflight-2026-10-19]. The learning results stayed small (A1 DEV 65 top-1, 547 top-8 [checked catalog.jsonl a1-completion-docs]).
2. **m1-* undated units (about 25, 20–30 Oct, M1 closure).** Best: M1 closed `PASS_M1_ALL_FIVE_BARS` with 455 top-1 and 1,484 top-8 on DEV [checked m1-close-saved-model/README.md]. The held-out phrasing bar (f) passed with 7,850/7,872 integer top-1 on its own panel [checked m1-heldout-phrasing-transfer/README.md]. r-ranking-with-admission-intact reached 871 DEV top-1, 6.55× its baseline [checked r-ranking-with-admission-intact/README.md table]. W2 capacity is the key negative (§0.1).
3. **r-* research probes (12).** r-hamilton-gate-suppression gives a proof NO: a norm-preserving gate cannot suppress amplitude. r-rank-objective-keeps-cells FAILED (configuration). r-vsa positive (above). r-prime-zeta-vs-nulls is NOT_FIT [checked each README verdict line].
4. **Offline-float sheets (D37/D41): spiralcore-fbs, hamilton-gate, recurrence-gates, float-training-module, d41-*.** Best: FBS 2-atom peaked at 2,520 then collapsed; the transport sign flips between builds [checked spiralcore-fbs-offline-float-sheet/README.md:59-67]. The Hamilton gate beats random and is near unigram at about 14.2k tok/s integer on a 5090 [checked hamilton-gate-offline-float-sheet/README.md]. Recurrence-gates: non-abelian S³ fails two-seed replication [checked].
5. **cpu-training-pilot-* (29 folders, 4–9 Nov, D84, DeepSeek).** Each folder holds only `resource-ledger.json`; the results live in docs/research/notebook/cpu-training.md [checked ls; notebook:20-44]. Best: complete correct-value forcing gave 200–250/256 complete replies vs 0/256 in free generation. So **value selection is the dominant reply blocker** [checked notebook:28]. Retrieval branch selectors completed at most 175/208 [checked notebook:37].
6. **one-model-end-to-end (34 files, 29 Oct–16 Nov, current main line).** Verdict mixed: the two seeds disagree in sign (+548 and −367) [checked one-model-end-to-end/README.md:36-40]. The recent commits are native CUDA = CPU-oracle = serving parity, the Pick read and rows-K to 4,096 (git log).
7. **m2–m7 milestone units.** Best: M2 437/512 retained [checked m2-fresh-heldout-panel/README.md]. The M3 sieve has 0/50 false asserts but compiles 0/220 development phrasings [checked m3-nonrelation-prose-panel/README.md]. The M4 integer libs scan clean (0 float, 0 mul, 0 div) [checked m4-export-uorlut01-cli/README.md].
8. **Infrastructure engines: uor-cuda-inverted-file, object-store-*, spiralcore-wire-*, legal-construction-exact-solve, wasm/studio/tauri, telemetry.** The CUDA inverted file passes 13/13 PTX checks, parity at V up to 4,096 and K up to 16, and 64/64 incremental moves [checked uor-cuda-inverted-file/README.md]. The object store does exact deduplicated byte records [checked catalog.jsonl d19-object-store].
9. **Docs, ops and governance (about 60 entries).** No model numbers.

## (c) crates — one line each

LOC = .rs lines under src. "hits" = a lexical count of softmax|sigmoid|tanh|gelu and of matmul|gemm in src. These counts include ban lists and docs, so they are a [hypothesis] about live use.

| Crate | Implements | Class |
|---|---|---|
| uor-r4-native-vector (2.5k) | Shared Q31 state plus the Pick read (first-pick, L1 address, Hamilton block rotate) | **integer/native, core to the goal** (0 hits) |
| uor-r4-discrete (71k, 2 .cu) | SpiralCore/FBS learning proposals, shared vector writer, CUDA rows-K | **integer/native** (2 softmax-ish, 3 matmul hits) |
| uor-r4-cpu-train (101k) | D84 CPU trainers and oracle; discrete route; Pick read use | **integer/native reference** (0 hits) |
| uor-r4-integer (103k) | Standalone integer serving runtime; `historical-runtime` feature | integer serving; legacy behind a feature (122 softmax-ish hits) |
| uor-r4-tokenizer (2.5k) | Byte-level BPE, stable identity | native utility |
| uor-r4-object-store (11.5k) | Exact ternary byte-object storage (D19 memory), address_read | integer/native, **scales on disk** |
| uor-r4-simd (1k) | AVX2/NEON multiplier-free 4-bit table "GEMV" | integer serving kernel (the name says GEMV but it is a table lookup) |
| uor-r4-lut (1.6k) | LUT artifact metadata and exact grid helpers; engines removed under D29 | integer metadata |
| uor-r4-float-train (81k, 6 .cu) | D37 offline float training of the geometric stack (candle) | float reference (10 softmax-ish, 2 matmul) |
| uor-r4-training (237k, 5 .cu) | Legacy autodiff, CUDA operators, reference eval | float reference plus **banned-op legacy** (561 softmax-ish, 101 matmul) |
| uor-r4-core (348k) | Original R4 engine: zeta embeddings, Hopf coords, transformerless local inference | **legacy, banned-op heavy** (908 softmax-ish) |
| uor-r4-wasm-router (76k) | R4 tangent-space router facade | **legacy banned-op** (2,188 softmax-ish, 18 matmul) |
| uor-r4-router (5k) | Session signatures and tier cascade; old engine behind `legacy-engine` | mostly native |
| uor-r4-model-source (27k) | Teacher: Rust port of llama2.c run.c, `observation-blas-exception` | **banned-op (teacher only)** (98 matmul) |
| uor-r4-graph-compiler / -certify / -cli / -runtime / -format (97k total) | R4G1 cross-compilation of a transformer into a packed graph | float/legacy compiler; format and runtime are integer |
| uor-r4-api (18k) | Typed facade over the graph stack | legacy (11 matmul hits) |
| uor-r4-proof-model (2k) | Executable proof spec for the graph compiler | verification |
| uor-r4-naf, -recon-dag, -source-interface, -vision | Interchange slice, byte-DAG reader, ingest metadata, integer RGBA patches | integer utilities |
| uor-r4-harness, -claim-wording, -pod-provider, -studio-agent, -workbench, repo-model, repo-conformance | Lab harness, wording gate, Vast adapter, Studio contract, Four-fact workbench, conformance registries | tooling |

All are workspace members; the root Cargo.toml lists 30 crates plus xtask and lab-runner [checked Cargo.toml workspace members].

## 4. Flags

**Retired for reasons that no longer apply:** the softmax-free rank-weight read, the Starling hard-neighbour read, the whole-path sampled producer, and the pooled-rank objective (softplus replaced by hinge). S1/S2 float applies only partly, given D37 and then D85.

**Would scale cheaply to 1e8–1e9 parameters:**
- query-token bigram / read-code count tables (6,300 vs 149);
- the source-free n-gram table (22.3%);
- product-key / E8 orbit slots (57,600 per layer, k = 8 sparse reads);
- the CUDA inverted file over table rows;
- rows-K to 4,096 with the Pick read;
- icosian-mantissa or packed signed4 weights (about 2.5 GB at 1B);
- the object store for durable rows.
