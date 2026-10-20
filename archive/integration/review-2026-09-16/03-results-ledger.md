> **UOR-R4 Project Knowledge Base — source report A2 (results ledger).** Produced 2026-09-16 by a Claude research session at Casey's request, from the GitHub clone (main @ a5655e93, 2026-09-14) and live sources. Read-only audit; nothing in the repo was modified. Treat "[Inference]"/"[ASSESS]" as reviewer judgement and everything else as quoted/derived from the cited files. Index: `00-project-brain-index.md` (this folder) / `claude/00-project-brain-index.md` (Claude Project).

# A2 — UOR-R4: Complete Ledger of Experimental Results and Negative Findings

**Reviewer role:** skeptical ML research reviewer.
**Repository:** partial clone of `github.com/UOR-Foundation/uor-r4`, `main @ a5655e93` (2026-09-14), at `/home/claude/work/uor-r4`.
**Report date:** 2026-09-16.
**Sources read in full:** `README.md`; `docs/README.md`; `docs/RESEARCH.md` (2,977 lines, all); `docs/geometric_intelligence_programme.md` (2,427 lines, all result-bearing sections); `docs/geometric_intelligence_evaluation.md` (policy + result sections); `docs/integration/current-state.md` (3,109 lines; every verdict header plus the Sep 12–14 shared-core chain, Sep 8 recovery, initial recovery and historical handoff); `docs/integration/recovery-2026-09-08.md`; `docs/integration/model-direction-2026-09.md`; `docs/integration/project-track.md`; `docs/integration/release-qualification-alpha.md`; `docs/project_baseline_audit_2026_08_18.md` §13–§14; `docs/transformerless/{BASELINE,COMPARISON,PROOF,EXTERNAL_REFEREE_300}.md`; `docs/inference_contract.md`; `docs/hologram_r4_formal_monograph.md`; `CONFORMANCE.md`; ~45 per-issue result records (`docs/*_NNN.md`) and ~12 `docs/*_result.json` / `docs/evidence/*.json` files. All 119 `docs/evidence/*.json` were listed; structure and numbers were opened for the neighbor-transfer, relational-attention, shared-core, Sep 8 recovery and free-running JSONs and spot-checked against the prose records (they agree).

Conventions: every number below is copied verbatim from the cited repository file. Text marked **[Inference]** is my reading, not the project's statement. The project's own claim vocabulary (`PASS`/`FAIL`/`NOT_RUN`/`UNAVAILABLE`, "bounded", "authored", "construction"/"open"/"fresh", "retained", "preserved") is kept where it is load-bearing.

---

## 0. Orientation: the eleven tracks and what "the model" means at each date

The repository is not one experiment; it is a sequence of at least eleven mechanically distinct systems, each with its own metrics, and the documentation deliberately forbids transferring results between them. A reviewer must keep them apart:

| # | Track (dates) | What the system actually is | Serving arithmetic |
|---|---|---|---|
| T1 | Transformerless / TLA / R4G1 table-native kernel (Jul–Aug 2026; #11–#516, #655–#946) | Teacher-distilled n-gram/exact-context lookup store with graded codes, Hamming region cover, integer scoring; teacher = llama2.c `stories15M`, later SmolLM2-135M/360M | Genuinely multiply-free, allocation-free integer kernel (`docs/inference_contract.md`) |
| T2 | Geometric router (Hopf/S³/VSA/spectral retrieval; #306–#502) | Sentence-retrieval router with content vectors, cosine ranking, `f64` | Floating point, outside P-4 kernel |
| T3 | S0–S4 staged programme on T1 (#822–#946, Aug 18–25) | Prompt-conditioning arms, calibrators, free-running evaluation, bounded planner on the T1 bundle | T1 kernel |
| T4 | Geometric decoder/mixer spikes (#950/#951/#958, Aug 26) | SmolLM2-135M with layer-29 attention replaced by a learned bounded mixer | `uor-matmul` exact GEMM (float) |
| T5 | Geometric Intelligence programme, synthetic route attention (#952–#989, #953, #973 Aug 27–28) | Prime-route / ordered-S³ / H4 exact-table selectors on tiny authored fixtures (2–12 decisions) | Integer/table (research harness) |
| T6 | R4-softmax family (#973 Aug 28–Sep 1; #1011–#1019; #1039/#1041) | Ordinary transformers (RMSNorm/RoPE/SwiGLU/Q·K softmax) whose heads are split into R4 blocks with H4 frame transport; HELM-D-inspired Lorentz variants; retained-attention variants | Float, matmul, softmax (Python/MPS; Rust f32 parity) |
| T7 | Grounded correctness on T6 (#954 C1-SB0..SB5) | SFT/LoRA adapters on the #1017 checkpoint | Float |
| T8 | Zoology MQAR chain (#1043–#1079) + clause adapters (#1082–#1102) | Stock HazyResearch Zoology attention cell on synthetic key–value recall and a 4-fact controlled English world | Float |
| T9 | Native geometric model, "retained normal artifact" (#973 Sep 4–7; #1136–#1140) | Rust count tables + learned signed-H4 selectors + exact copy/add operators on authored template corpora (~0.5 MB) | Integer/table with an inherited additive predictor; float in startup geometry |
| T10 | Sep 8 recovery / alpha withdrawal | Audit of T9's release claims | — |
| T11 | Shared geometric core (#973 Sep 12–14) | Isolated H4 finite-table recurrence + Hamming/H4 reads + byte/EOS decision trees, trained by offline hard-forward coordinate search on authored template tasks | Integer/table |
| T12 | GPT-2 dense offline executor (#704) | Offline teacher forward pass via certified native lanes | Float, offline only |

The README (2026-09-14) names the "retained normal artifact" `15baec48` (T9) and the "latest evaluated experimental artifact" `60d19679` (T11), and states plainly: *"Frontier capability, sustained general prose, general reasoning/coding and complete-task energy savings have not been established."* (`README.md:9`).

---

## 1. RESULTS LEDGER

Columns: Date / issue · Experiment · What was measured · Dataset / panel · Metrics & numbers · Baselines / controls · Result · Project's own stated limitation. Rows are grouped by track and are chronological within a track.

### 1.1 Track T1 — Transformerless / TLA / R4G1 table-native kernel

| Date / issue | Experiment | Measured | Dataset / panel | Metrics & numbers | Baselines / controls | Result | Stated limitation |
|---|---|---|---|---|---|---|---|
| pre-2026-07-18 (migrated), PROOF.md P2 | A-binary shipped runtime vs A-f32 vs B bit-prefix | top-1 vs actual, teacher-argmax agreement, Witten–Bell bits/token | stories15M teacher, 150,000 teacher-labeled tokens, 757 stories, 30,192 held-out (teacher-sampled text) | A-f32 31.5% / 34.7% / 6.3214 / 86,574 keys; **A-binary 28.9% / 31.7% / 6.5427 / 89,200**; B 26.0% / 28.6% / 7.6969 / 323,937; teacher floor 1.5960 bits, ceiling 70.4% | teacher floor; own f32 ablation | Positive at stated residual (`docs/transformerless/PROOF.md:77-102`) | "price of full multiplication freedom … 3.0 agreement points and 0.22 bits/token" |
| 2026-08-01, #327/#335 (500k re-pin) | Same, 500k-token corpus, TLA7 | as above | 500,000 tokens, 2,507 stories, 100,306 held-out | **34.7% / 39.0% / 8.0249 WB / 179,068 keys**; f32 ablation 35.7%/40.3%/8.1063; cpy8 35.3%/39.6%/8.5247/195,650; mantissa-fold 34.6%/38.9%/8.3097/162,119 (not adopted); write-time fan-out 20.3%/22.7%/8.1473/817,683 | teacher floor 1.4260, ceiling 70.5% | Positive (`docs/transformerless/BASELINE.md:64-69`, `PROOF.md:153-191`) | none beyond distribution scope |
| pre-migration, COMPARISON.md | Throughput vs classical runtimes | tok/s, single thread, Linux container | same stories15M artifact | **mul-free 77,342 tok/s** (31,956 with op census); llama.cpp q8_0 344.28±7.32; llama.cpp f32 157.43±0.86; L2E 103; in-crate teacher 62; run.c 48 → "225× … 491×"; artifact 2.17 MB vs 25.4–93.8 MB; ≈1.8×10⁵ int ops vs ≈15M mul-adds | llama.cpp at 100% teacher agreement | Positive throughput **at 31.7% agreement** (`docs/transformerless/COMPARISON.md:25-46`) | "This is a comparison of runtime architectures at stated quality, not a claim of quality parity" |
| same | Scenario suite (14 prompts) | agreement w/ teacher trajectory; top-1 vs real text | 4 classes; positions 385/291/108/253/209 | in-domain prompt 30.9%; OOD prompt 21.3%; real in-domain text **17.6% top-1 vs teacher 47.2%**; Shakespeare 0.8% vs 1.2%; stress 36.8%; tless 66–79k tok/s vs teacher 92–101 | teacher | Mixed; "the honest gap is real human-written text" (`COMPARISON.md:87-132`) | sample→human shift |
| 2026-07-22 (#64/#65) | Gate C fixture: graph Σ-cloud vs Rule 1+2 | top-1 agreement, bits/token | fixture corpus, 30,036 held-out | Σ-cloud **0.3% / 70.47** (refuted); Rule 1+2 **31.7% / 9.86**; TLA3 store 31.7% / 11.88 | TLA3 store | Σ-cloud negative; Rule 1+2 argmax-identical to store, better bits (`BASELINE.md:71-73`) | Rule 1+2 rows "entirely EXCT-driven" |
| 2026-07-23, #75 | D3 first pass, SmolLM2-135M | as above | natural: 400 Simple-Wiki articles, 17,457 held-out; continuity 40,342 held-out | natural: Σ-cloud 0.03%/60.41; Rule1 0.19%/56.69; **1+2 15.04%/13.30**; TLA3 15.04%/18.76. continuity: 0.005%/50.32; 3.23%/46.97; **14.76%/14.62**; TLA3 14.76%/20.63. Full n=3000: **28.1%/11.3565 vs 28.1%/11.2481** | TLA3 | M.V.G. target 1 stretch FAIL / floor PASS; target 2 PASS continuity, **FAIL natural at n=3000** (`BASELINE.md:141-228, 335-347`) | "ExactContext 100%, Graph 0, Novel 0 … the current judge measures exact-context memory" |
| 2026-07-21 | Op census & allocation census | ops/token, allocations | SmolLM2 path, 32 greedy tokens | **144,496 ops/token** (adds 48,530, xors 36,864, shifts 11,666, compares 1,324, table reads 46,112); 0 allocations steady state; 5 allocs/496 B warm-up; store parse 57,498 allocs/5.40 MB for a 494 KB container; per-token latency **pending**; bytes read **deferred** | — | Structural positive (`BASELINE.md:242-257`) | "allocation-free hot path is amortized, not unconditional" |
| 2026-08 P2, #320 | SmolLM2-135M / 360M rehearsal | top-1, agreement, bits | 20k-token teacher-generated corpora; 4,419 / 4,098 held-out | 135M: table 7.6% top-1, 13.0% agree, 20.6962 bits vs 4.9482 floor; graph 5.6%/48.1532. 360M: floor 3.7908; table 5.54%/8.61%/21.1834 (**+17.39 bits over floor**); graph 5.0%/14.78 | teacher floor | Negative for migration ("graph does not improve on the same-corpus TLA baseline") (`docs/smollm2_teacher_baseline_320.md:90-110, 155-162`) | "compiler-bound … not teacher-bound" |
| 2026-08, #509 (P3) | Teacher-breadth swap on broad text | held-out top-1, bits | Simple-Wiki 3,000 articles → 21,235 records; 4,358 held-out D3 positions; EXCT-miss 62.5% | legacy teacher-hash 0.07%/72.46; Rule 1 0.32%/13.51; **Rule 1+2 10.19%/12.72 ±0.46pp**; normalized 13.47%/12.08; TLA3 25.70%/14.70; **latent-mix 29.0%/11.71**; ORACLE-RIGHT 15.9%/12.39 (not causal); SHUF-CLASS 21.5%/12.12; table artifact 17.3%/25.4%/14.6954 (+11.09 over 3.6015 floor) | unigram null, shuffled-class, teacher floor | **POSITIVE** ("~100–290× movement") (`smollm2_teacher_baseline_320.md:223-277`) | "not a byte-identical A/B … the narrow-teacher figure is the recorded off-distribution serving number, not a re-run" |
| 2026-08-09, #516 | Full-scale broad baseline (PINNED) | as above | 360,924 records / 2,994 stories; 72,864 held-out; EXCT-miss 25.7% | **Rule 1+2 24.30% / 11.94**; best live arm 31.48% / 10.43; TLA-3 28.21% / 13.62; Rule 1 1.78% / 15.68; anchor-hat 17.69%; SE 0.0016; 64/0 witness replays | teacher floor 3.6015 | Positive, pinned (`smollm2_teacher_baseline_320.md:317-338`) | "graph ceiling … remains well above the floor" |
| 2026-08-?? , #300 | External referee of teacher floor | bits/token | 596 held-out articles, 71,714 tokens | external HF SmolLM2-135M **3.8425 bits**; r4 story-contiguous floor **11.17** | — | Diagnostic: r4 pipeline floor is 7.33 bits worse than the same teacher scored externally (`docs/transformerless/EXTERNAL_REFEREE_300.md`) | "residual floor is pipeline-internal" |
| 2026-08-18 audit | Gate C re-run at `aea30bae` | top-1, bits | 500k fixture, 100,306 held-out | rule12 **36.55% / 8.32**; TLA3 39.17% / 8.50; fused live arm 44.84% / 6.69 (n=63,543); **`rule12_generalization` (EXCT-miss slice, n=14,943): 1.81% / 16.89**; EXCT resolves 85,363/100,306 | — | In-distribution positive; off-exact-context **~1.8% top-1** (`docs/project_baseline_audit_2026_08_18.md:455-475`) | "memorization/generalization split in one number pair" |
| 2026-08-18 audit | Live `r4 ask` generation probes | verbatim output | 2 prompts × 3 bundles | 135m: `ounds Callounds Callounds Call…` 235 s, **byte-identical for both prompts**; `--sample 42` byte-identical; 360m: `<|im_start|>cescescesutionces…` 93 s, prompt-invariant; 1.7B: `cut cut cut cut…` 5 s | — | **Negative**: "mechanically working, semantically degenerate" (`project_baseline_audit_2026_08_18.md:477-525`) | — |
| 2026-08-16/17, #655/#745/#755 | Root-cause of word salad | qualitative | `smollm2-135m-instruct` bundle | corpus 99.93% story-scrambled; post-fix recompile: ~199 s hang → ~0.2 s answer, "real, grammatical (if topically wandering and prompt-insensitive) English"; two prompts returned same output | — | Bug fixed; text still prompt-insensitive (`RESEARCH.md:2237-2284`) | "does not by itself establish the architecture produces good text" |
| 2026-08-17, #758/#759/#762 | Prompt-insensitivity quantified | distinct outputs / 15 prompts | 15 prompts | 135m post-fix **9/15 distinct** (largest group 5 identical); 360m-broad **11/15**; `--sample` 13/15→15/15 | — | Negative (greedy attractor-basin collapse); mitigation opt-in only, unreachable on R4G1 path (`RESEARCH.md:2874-2875`) | "codebook-collision root cause itself remains unfixed" |
| 2026-08-19, #655 F-p2 canary | Validity vs distinctness | valid completions | 20 prompts × 2 passes | greedy **0/15 valid** (digit attractor); seeded sampling 15/15 valid; distinctness **5/15** with 8 identical | — | Decode fix shipped; distinctness unchanged (`RESEARCH.md:2372-2393`) | "convergence sits upstream of decode" |
| 2026-08-20, #833 | Attested #755-native rebuild | top-1 | 72,130 held-out | Rule 1+2 24.39% (vs 24.30%); best live 31.11% (vs 31.48%); TLA-3 28.12% (vs 28.21%); EXCT-miss 27.36% | #516 pin | RATIFY/RETAIN (`docs/attested_broad_baseline_833.md:221-236`); later scope-corrected: loader was quality-bypass, normative serving **NOT ESTABLISHED** (`:264-277`) | "teacher floor RETAINED by invariance … re-measurement UNAVAILABLE (~24 h at ~1 position/s)" |
| 2026-08-25, #933 | Normative `R4G1Runtime` census | greedy top-1, paired ‰ | 72,130 held-out | **21,293/72,130 = 29.5203%** vs TLA 20,284 = 28.1214%: **+13.988‰ [11.057, 16.919]**; sections-absent 18,806 = 26.0723% → RF-31 lane +34.479‰ [31.681, 37.277]; label-shuffled −229.003‰; witness 64/64 | TLA; sections-absent; shuffled | **RATIFY RF-31** for exact bundle (`RESEARCH.md:1822-1841, 2348-2370`) | "does not establish live-teacher parity, instruction following, reasoning, or free-running coherence" |
| 2026-08-2x, #908/#910 | Skip-mix lane (`R4Engine`, off-serving) | top-1, paired ‰ | same | **29.702%**, +28.45‰ | — | Positive but reference/off-serving only (`RESEARCH.md:2334-2346`) | "explicitly outside the ADR-0001 normative served-token selector" |
| 2026-08-19, #804/#605 | Dormant route-attention kernel vs traced SmolLM2-360M teacher | support overlap | 62,875 broad records, 6.6M eligible steps | fitted 0.396 vs permuted 0.192; **anti-vacuity null (shift-by-one) 0.292**; 119/120 heads vacuous | permuted-code null; N2 shifted null | **FAIL — instrument vacuous** (`RESEARCH.md:2720-2742`) | "teacher attention supports are temporally smooth" |

### 1.2 Track T2 — Geometric router (retrieval)

| Date / issue | Experiment | Measured | Panel | Numbers | Controls | Result | Limitation |
|---|---|---|---|---|---|---|---|
| #434 (PR #442/#465) | De-banding full-width storage | cosine-ranked MRR, anchor accuracy | router fixture (2000 stories/200 probes/46,342 windows) | MRR **0.2348 → 0.8948**; top-1 0.208→0.832; anchor accuracy 9.3%→11.4% (free top-1 8.17% vs 7.64%) | arm-1 content-free 0.1674; shuffled 0.0002 | Positive, shipped (`RESEARCH.md:1886-1891`; `docs/debanding_audit_434.md:118-124`) | "It is a cosine-ranked figure … unreachable at serving" until #486 |
| #486/#490/#502 | Serving path compared routing vector to content vector | MRR, recall | same | self-query at **0.4938 percentile** (chance); content-query 0.7179→**0.8542**; W=0 → **0.8763**; recall 0.9720→0.9900 | — | Category error found & fixed (`RESEARCH.md:2006-2016`) | #480/#484 verdicts superseded |
| #480 | Query-projection banding | MRR, top-1, recall@20 | same | +0.0059 MRR, +0.0080 top-1, −0.0180 recall vs +0.05 bar | — | NEGATIVE (later superseded) | — |
| #484 | Lexical weight sweep W=1…100,000 | retrieval | same | bit-identical retrieval | — | "inert" NEGATIVE (conditional on dead cosine) | — |
| #422/#306 | Hopf sector transport | sector occupancy, MRR | same | occupancy 16→456 of 512; sector-filtered MRR **0.0045 vs 0.0743** | — | NEGATIVE (`RESEARCH.md:1959-1963`) | — |
| #496/#507 | VSA encoder + facet index vs Spectral | top-1/MRR/recall | 2000/200/46,342 | VSA 0.73/0.81/0.98 vs Spectral 0.85/0.89/0.98; gap 0.43→0.08 MRR | deranged null dead in both arms | VSA kept, Spectral default (`RESEARCH.md:2864`) | — |
| #400/#290/#393/#395 | Cayley–Dickson morphism, FMM, granularity, E8 group keying | — | — | CD term executed **0/1,998** | — | "each measured dead" (`RESEARCH.md:2018-2021`) | — |

### 1.3 Track T1 capacity/structure levers (graph compiler)

| Issue | Experiment | Numbers | Result |
|---|---|---|---|
| #399 | Standalone two-pass generation | external anchors +4.2pp; self-anchors **40.4% vs 41.3%**; with gate 42.0% vs 43.0%; at 2.11M records 16.4% vs 26.5%, predicted-anchor accuracy 0.0% | Refuted twice (`RESEARCH.md:1920-1927`) |
| #460 | Code-space subdivision (STAGES 4→5) | keys 47,403→90,824; records/key 36.02→18.80; **25.6% ±0.44pp vs 26.5% baseline**; store 26.4→25.4; exact-context dominance 98.8%→97.1% | NEGATIVE "in its strongest possible form" |
| #460 | Codebook fit | records/key 5.04→4.68; top-1 **+0.44pp** | small positive |
| #460 lever 1 | Cover scaled capacity | regions 48→110; region-path top-1 **+5.0pp**; serving impact capped near 0.15pp (graph path answers ~1–3% of positions) | positive but non-load-bearing |
| #435 | Construction stratification (3 designs) | oracle-stratum edge 36.3 vs 35.2 unrecovered | NEGATIVE |
| #424 | Bott–Fock O(1) context fold | long-range signal +1.02pp ceiling; shipped decay retains 16% → +0.16pp (one SE) | ceiling only; A/B not reachable |
| #456 | Reconstructability | EXCT-disabled recon **16.3 bits / 1.5%** vs unigram 8.7 / 6.4%; deranged tables degrade ~3 bits | NEGATIVE (sub-unigram) |
| #457 | IPF-consistent reconstruction (Arm B) | naive 23.92 bits / 0.18%; IPF inconsistency 5.40→0.096 bits; consistent joint **8.60 bits / 0.062 = unigram floor** | NEGATIVE |
| #458/#459 | Interaction information; estimation ladder | "No measurable synergy"; k≥3 counting noise | NEGATIVE |

### 1.4 Track T3 — S0–S4 stage programme on the T1 bundle (Aug 18–25)

| Date / issue | Experiment | Measured | Panel | Numbers | Controls | Result | Limitation |
|---|---|---|---|---|---|---|---|
| 2026-08-20, #834 §6.1 | Teacher-grounded prompt-conditioning bake-off | top-1 agreement with teacher; causal-influence-delta | 24,044 held-out Simple-Wiki positions; 1,460 natural minimal pairs | full-context **26.9% vs 2-token suffix 27.1%**; delta **−1.6‰ [−2.4, −0.8]**; k1..k8 flat ≈27%; follows **0/1,460** pairs | prompt-swap, suffix-only, shuffled-state | **`NO PROMPT-CONDITIONING ARM ESTABLISHED`** — "the deployed model is suffix-local" (`RESEARCH.md:2422-2436`) | Ψ arms UNAVAILABLE in deployed artifact |
| 2026-08-20, #834 §6.2 | Ψ segment-lane reference arm | ‰ agreement | 288,794 train, 72,130 held-out | **264.1‰ vs 246.6‰, +17.5‰ [15.9, 19.0]**; follows 10/4,722 pairs | suffix baseline | modest positive, off-serving (`RESEARCH.md:2436-2445`) | "+1.75pp overall, few hard same-suffix pairs resolved" |
| 2026-08-21, #834 §6.3 | Conditional residuals arm | ‰ | same | 262.8‰, **+16.2‰ [+14.6, +17.9]** — below 20‰ floor; vs Ψ −1.3‰ [−2.4, −0.1] | residual-shuffle null | **REVISE**; S1 kill/redesign criterion met | — |
| 2026-08-21, #822 D1/D2 | Joint conditional tables | ‰ | same | strict joint +6.8‰ [4.8, 8.9]; **backed-off mix 277.2‰, +30.6‰ [+28.6, +32.5]** (first floor-clearing arm, off-serving); prompt-swap −33.2‰; key-shuffle −116.3‰; minimal pairs 10→78–83 of 4,722; d4skip +49.6‰; D2 region-conditional +19.5‰ [17.0, 21.9]; depth-2 +23.3‰; code-shuffle −162.5‰ | planted nulls | D1 SELECT; D2 REVISE (`RESEARCH.md:2468-2502`) | off-serving reference arms |
| 2026-08-21, #837 | Artifact-only calibrator | false-answer UCB95 at coverage | document-disjoint partitions | no arm meets 10‰/50‰ gates; bucket table 11‰ error at 10‰ coverage; raw margin most-confident slice **520‰ wrong**; 2,454 content-answerable novel positions discarded | — | **`NO CALIBRATOR ESTABLISHED`** (`RESEARCH.md:2528-2550`) | — |
| 2026-08-22, #931 | Content-evidence calibrator re-entry | same | CAL 24,232 | best slice 243 served / 8 wrong (UCB95 46‰); at 20‰ floor UCB95 79‰ vs 10‰; at 50‰ 180‰ vs 50‰ | label/feature/key shuffles | **`NO CALIBRATOR ESTABLISHED`**; S2 **LIMIT** | TEST never opened |
| 2026-08-21, #841 | Free-running trajectory gap | first-divergence step, cycles | 100 held-out story prompts, H=8/32, greedy | **median first-divergence 0**; 590‰ diverge at step 0; 0 survive H=8; TF ceiling 304‰; **99/100 rollouts identical to 2-token suffix-only**; **710‰ collapse into ≤4-period cycles** (110‰ teacher-prefix); 0 abstentions | shuffled-prompt (980‰ at step 0), teacher-prefix | "the gap is total" (`docs/free_running_eval_841.md`; `docs/free_running_841_result.json`) | sampled mode & judge UNAVAILABLE |
| 2026-08-22, #840 | Reachability of corrective observation | same, two engines | same | skip lane raises TF 304→348‰ but step-0 divergence 590→620‰, cycles **710→1000‰**; correctable footprint ~1‰ of steps vs 100‰ bar | base vs skip engine | **GENERATION-NOT-ESTABLISHED**; corrective run not launched | "the limiting factor is representation" |
| 2026-08-22, #842 | State-starvation diagnostic | failure classes | 100 rollouts | **0/100 StateStarvation**; SingleStepAt0 59, CandidateGap 16, RankLimit 25 | — | **NOT TRIGGERED; GENERATION-NOT-ESTABLISHED**; S3 LIMIT | "teacher-forced continuation/retrieval system, not a generative engine at this scope" |
| 2026-08-25, #944/#946 | Prefix-signature memory; trajectory regions | changed candidates/tokens | 512 cases | #944: 492/512 explicit rows; secondary probe attempted 0×; **0 changed**. #946: 1,549 admissions in preflight; **0/512 candidate lists, 0/512 tokens changed** | — | **INERT** ×2 | — |
| 2026-08-22, #843 | Bounded planner RF-33 | correct-outcome rate | 20 development cells ×512 | 1.0000 all cells; clears floor in **12/20** (other 8 solved by direct continuation) | strongest non-oracle null | **LIMITED**; off-serving | no deployed caller |
| 2026-08-22, #845 | W(3,3) geometry vs Hamming/binary/VSA/spectral orderings | budget & correctness | n=512 per cell | 0 of 12 reduction cells; fails all 9 tightened cells; "at the level of its own randomized and scrambled controls" while goal-distance & Hamming hold 1.0000 | randomized/relabelled/phase | **NO GEOMETRIC ADVANTAGE** (`docs/w33_geometry_qualification_845.md:19-33`) | — |
| 2026-08-22, #846 | Final-partition reasoning certification | certifiable samples | — | **0 × 512 = 0** (partition cells 0–7 all opened by fitting/prior eval) | — | **REASONING NOT ESTABLISHED**; S4 LIMIT | — |

### 1.5 Track T12 — GPT-2 dense offline executor (#704, 2026-08-15)

| Measured | Numbers | Result |
|---|---|---|
| Paired candidate/conventional forward time, 3 GPT-2 stories, 11 tokens, Apple M1 | earlier prototypes ~220×, 1357.803945×, 45.6–57×, 16.903285271× slower; layer-0 gate 1.838843890× (upper 1.847055652 < 4.0); whole-model **median 0.694987183×**, bootstrap upper **0.703876225× ≤ 3.0**; 27,845,417 ns/token vs 39,944,807 ns/token; 0 allocations; bit-exact parity | PASS (`docs/gpt2_dense_704.md`). **Offline teacher path only**; not a serving result. |

### 1.6 Track T4 — Geometric decoder / mixer spikes (Aug 26)

| Issue | Experiment | Panel | Numbers | Controls | Result |
|---|---|---|---|---|---|
| #950 G0 | Layer-29 attention replaced by bounded mixer in SmolLM2-135M | 5 prompts × 32 tokens | source control 4/5 human rubric; layer-0 replacement → period-2 EOS cycle (FAIL, preserved); layer-29 treatment 32 cycle-free tokens; coordinate permutation changed 48,896 logits, L∞ 0.000025749207 | disabled-mixer replay bit-for-bit | PROMOTE_TO_G1 — "structural reachability, not evidence of a quality advantage" |
| #951 G1 | Fit the mixer (18 train / 9 held-out positions) | 9 held-out | total loss 1.0046638350→0.9274201393; **operator and token terms flat at ≈1.0**; support term 1.0233→0.6371; real vs coordinate-permuted **4.4416278% < 5% required** (teacher 4.4031181%, student 4.5190205%); memory prob 0.1034 vs permuted 0.0491 | coordinate-permuted, memory-permuted | **REDESIGN_REPRESENTATION** (`docs/geometric_mixer_qualification_951.md`) |
| #958 | Prime-route attention qualification | source-free fixtures | Stages 1–3 PASS (mechanics); stage 4 product probe **NOT_RUN**; stage 5 teacher comparison **NOT_RUN**; 1.484× four-worker compile speedup (661,192,500 vs 445,318,333 ns) | — | **RETAIN_STORAGE_RECALL_ONLY** |

### 1.7 Track T5 — Geometric Intelligence synthetic route-attention (Aug 27–28)

| Issue | Mechanism | Panel size | Numbers | Result |
|---|---|---|---|---|
| #952 A1.0 | Ordered route summaries | 3 matched contrasts | **21/21 pair-level cells and 966/966 non-digest comparisons equal** | REDESIGN_ORDERED_ROUTE_SUMMARY (negative) |
| #967 A1R | Associative noncommutative fold | 6 queries | distinct ll/rr states 6/6; same-candidate change 5/6; **Cayley distance tied 6/6 at energy 2** | RETAIN_STATE_ONLY |
| #970 A1P | Paired-H4 R4-heatmap identifiability | 14,400 pairs, 45 classes; 36 decisions | construction pure 12/12; validation coverage 10/12; **strict transfer 0/6**; 8 incompatible classes | RETAIN_H4_STATE_ONLY (negative) |
| #969 | Causal S³ least-cost path | 2 prompts | `aa bb dd qq`→`rr ll`; `bb aa dd qq`→`ll rr`; last-only abstains; state-disabled `rr` both | PROCEED (mechanism only) |
| #953 | Decoded loop | 2 prompts | `slowly carefully` / `carefully slowly` — audit: exact lexical relabel of #969; agreement contrast: full path chose `still run` **for both orders**; placement overlay **0/2** vs permuted 2/2, shuffled 1/2 | REVISE_I1_GENERATOR_IN_PLACE (negative) |
| #983 | Construction causal return | 6 held-out decisions | coverage **0/6**; ceiling 0/6; 21 R_min/24 R_full classes | UNAVAILABLE_ZERO_CONSTRUCTION_TRANSFER |
| #986 | Corpus signed transport | — | codec/pair commitment absent | UNAVAILABLE_FRAME_OR_POPULATION (nothing ran) |
| #989 B0 | Table-native lexical baseline | 446,342 held-out known-target positions | **99,362 (22.261404%) vs unigram 24,163 (5.413561%)**, +16.847843pp; selections trigram 319,336 / bigram 108,738 / unigram 18,268; 35,655,288-byte artifact | ESTABLISHED (non-geometric reference) |
| #953 B1 | `MultiscaleCountRadiusR4V1` (tie-breaking only) | same | **103,604 (23.211797%)**, +4,242, **+0.950392pp**; changed 56,280 choices (geometry right 6,753, baseline right 2,511) | PROCEED — the only accepted geometric increment; "changed only maximum-count ties" |
| #973 Gate 0 | Prior-sentence count radius | 2 prompts | real 2/2; scope-disabled 1/2; candidate-permuted 0/2 | RETAIN |
| #973 paragraph / conversation | Entity spin-path selectors | 2 prompts each | real 2/2; disabled 1/2; permuted 0/2; reversed 2/2 | RETAIN; "construction-bound exact descriptor reuse" |
| #973 global V1 | Exact-spin left fold | 2 | premise falsified: `Pavel` and `helix` map to same root, `prism` to identity; both orders end at −1, fiber 110444176, torsion −10509096 | negative |
| #973 global V2 | Noncommuting exact spin | 2 | real 2/2; identity-disabled 1/2; permuted 0/2; EOS 6/6 | RETAIN (synthetic witness) |
| PR #997 | Corpus-induced document placement | 35,028 targets (36,533 anti-recall positions) | real **2,931 (8.367592%)** vs #953 fallback **4,281 (12.221651%)**; order-shuffled 2,934; operator-permuted 2,966; −1,350 routes (−3.854060pp) | **RETAIN_BOUNDED_GLOBAL_ONLY_REDESIGN** (negative; below shuffle) |
| #973 | Gated-delta retention core | 28 next-token / 112 associations (synthetic) | geometric **16/28, 55/112** vs plain delta **23/28, 98/112**; no-delta 15/28, 58/112; transport-permuted 15/28, 62/112; order-shuffled 17/28, 54/112 | NO_ADVANTAGE_ON_THIS_FIXTURE |

### 1.8 Track T6 — R4-softmax / HELM-D / Lorentz / gauge family (Aug 28 – Sep 1)

| Date / issue | Experiment | Panel | Numbers | Controls | Result | Limitation |
|---|---|---|---|---|---|---|
| 2026-08-28 | `DirectCausalGeometricAttentionR4V1` V2 | 8 | 8/8 rejected on review: plain arms had 2 DoF vs 3 | — | NON_PROMOTABLE_BUDGET_MISMATCH | — |
| 2026-08-28 | V3 equal-manifold budget | 12 prefixes | **full H4 3/12; plain 12/12**; seed-disabled 7/12; current-only 6/12; alt-connection swap 10/12; key-isometry 7/12; order-shuffled 5/12; value-permuted 8/12 | plain, permutations | Negative | alt swap not separately trained |
| 2026-08-29 | `ConnectionGaugeCovarianceV4` Phase I | 16 construction cases | H4/alt/plain each 16/16; current-only 8/16; 120 frames / 14,400 pairs covariant | — | positive (construction only) | — |
| 2026-08-29 | V4 Phase III sealed reveal | 24 held-out | **H4 13/24, alternative 13/24, plain 13/24**; current-only 12/24; order-shuffled 13/24; value-permuted 12/24; gauge-mismatch 11/24 | destructive controls | **FAIL … STOP_BEFORE_PAIRED_H4_E8** ("insufficient destructive-control drop") | — |
| 2026-08-30 | `HELM-D-R4` full-decoder parity | **3 teacher-forced positions + 2-token greedy continuation** on SmolLM2-135M, one D3 document | 196,608 logits; max Δ 0.00001049041748046875; mean Δ 0.0000022742100540540378; top-1 3/3; decode `, and` = `, and`; frame-permuted control Δ 23.08442449569702 | donor; permuted frames | PASS (parity) | "not a perplexity or coherence benchmark … not an R4 predictive advantage" **[Inference: orthogonal change-of-basis; parity is expected by construction]** |
| 2026-08-30 | `IntrinsicLorentzR4AttentionV1` attempt 01 / 02 | construction | 01: checkpoint JSON round-trip defect; 02: barycenter covariance **9.121400701417315e-08 vs 1e-08 ceiling**; diagnostic NLL worse than donor by 1.2531 and than flat R4 by 0.20893 nats/token | — | UNAVAILABLE (stopped before D3) | — |
| 2026-08-30 | `HelmDLearnedManifoldR4ConstructionV2` attempt 02 | construction-validation | learned-Lorentz NLL **7.71061809923296**; donor 3.667626465210025; learned-Euclidean 4.483153905078387 | 3 destructive controls (passed) | **FAIL … REVISE_PROJECTION_SCORE_CENTROID_OR_TRAINING** | "controls establish sensitivity only" |
| 2026-08-30 | Score-centroid localization | 2-document preflight | tangent readout raised normalized audit MSE, pooled ratio **1.0643688804269025** | — | REJECT_TANGENT_READOUT | — |
| 2026-08-3x | `R4SoftmaxReferenceGeneratorV1` generation (SmolLM2-135M weights, R4 frames) | 5 prompts × 32 tokens | 4/5 frozen rubric both passes; 5/5 exact replay; 30 layers audited; donor reproduced P1 through EOS, P2–P5 all 32 tokens | source donor | PASS | "transformer-compatible and f32/multiply/alloc/source-weight backed" |
| 2026-08-3x | HTTP bridge canary | 8-token prompt | identical tokens/CIDs to CLI | — | PASS; hosted Pages static/offline, no chat backend | — |
| 2026-08-3x | `R4SoftmaxTraceStudentV1` (source-free Q16 suffix) | construction traces | "bounded distillation effect against count and document-permuted controls, but its autonomous continuation loops" | count/permuted | mixed/negative | — |
| #1011 | `R4SoftmaxTraceStateStudentV1` | 9 positions, 422,875 Q16 mass | geometric CE 2.660705367; suffix 2.660721032; plain 2.660770919; transport-permuted 2.660729215; **margin 0.000023848 vs 0.10 required**; all arms 3/9 teacher / 2/9 actual top-1; same period-two `, Scotland` loop | transport-permuted | **FAIL_PROMOTION** | — |
| #1012 | Leave-one-document-out observability | construction | coverage aggregate 0.6202622204224402, **min fold 0.3469116829611222 < 50%**; full Q/K/V CE 2.215410922655504 vs suffix 2.215064603216862 (0.0003463 worse), direction 0/4; label control 1.3807454322642605 nats 4/4 | fixed-label control | **INSUFFICIENT_SUPPORT_COVERAGE** | — |
| #1014 | End-to-end 7,155,360-param R4/Spin causal-softmax LM, random init | **TinyStories** (pinned snapshot; 30,000,000 train tokens; 249,856 sealed test positions) | enabled sealed NLL **2.127407277216677 nats/token**; attention-off **4.804799838144271** (Δ 2.6773925609275944 ≥ 0.10); dev NLL 2.131356526593693; MPS overfit 80.821061%; Rust parity Δ 0.00000762939453125; retention **3/5**; replay 5/5 | attention-off intervention | **ATTENTION_ESTABLISHED / FULL_QUALITY_DOD_FAILED** (NLL > 1.50; 3/5 < 4/5) | "does not establish a geometry advantage … transformerlessness, multiplication-free … execution" |
| #1017 | Continuation of #1014 to 149,995,520 tokens | fresh 250,000-token sealed tranche | dev NLL 1.580241072373312; **sealed NLL 1.5727521962806827 (fails < 1.50)**; 26.071880399960784% improvement; retention **5/5**; parity Δ 0.0000057220458984375 | — | **FULL_QUALITY_DOD_FAILED_NLL_ONLY** | same |
| #1019 | 12-layer 13,130,784-param capacity rung | — | population PASS; overfit 81.9752%; Rust parity Δ 0.0000443459; **MPS projection 20.66 h > 8 h** (memory 21.03%); fused AdamW 4.485223 s/step slower than 3.491307 | — | **UNAVAILABLE_HARDWARE_BUDGET**; full run NOT_RUN | not a model result |
| 2026-08-31, #1017 addendum | M1 local inference, 4 tokens | 1 prompt | exact `uor-matmul` **3.060506042 s** generation / 3.41 s wall; Apple Accelerate **0.116236875 s / 0.52 s** (26.33× / 6.56×) | — | measurement only | float BLAS path |
| 2026-09-01, #1039 | Reference surface, 24 tokens | 1 prompt | wall 0.22 s and 0.17 s; byte-identical JSON | — | bounded prototype | — |
| 2026-09-01, #1041 | Normal-use probes | 3 narrative + 2 history | narrative **2/3** (N2 `Sora → boat`); history **0/2** | no-history controls | **KEEP_RAW_CONTINUATION_ONLY** | — |
| #973 | Sparse geometric candidate softmax KV; quaternion-cube residual (no fit) | 2 prompts × 16 tokens | 12 and 3 common tokens with recurrent comparator; materialized scores −15.27%; 1,272 R4 blocks, block-norm error 7.152557373046875e-07; "visibly degraded text" | fixed recurrent arm | mechanism only | RoPE limits to 120 positions |
| #973 | Quaternion-cube full-context fit | 120-token graph | update-1 loss 10.436132, grad norm 6.284435; 78.177→25.757 s to update 1; 8 of 128 updates | — | RESOURCE_UNAVAILABLE | — |
| #973 | Group-addressed retention decoder (3.17M params) | 4,096 training decisions | "memorized only 4,096 training decisions and failed aggregate validation loss"; state ablation positive; scramble CE better | — | RETAINED-DECODER FAIL | — |
| #973 | `R4RetainedLanguagePathV1` (252,160 params each arm) | TinyStories slice: 5,241,600 train decisions; **247,920 validation decisions** (fresh story range) | retained NLL 8.326806644→**3.899861931**, top-1 18→**73,726**; ordinary 8.328084567→**3.903394426**, top-1 **73,909**; retained state-off **4.234849487**, 57,066 (−16,660) | equal-parameter ordinary causal softmax (positive control) | **RETAINED_LANGUAGE_PATH_PASS** (retained 0.003532495 nats better; 0.073814pp behind top-1) | **"H4 specificity NOT_EVALUATED"**; 115,200 vs 58,080 score pairs (not equal work); retained arm 1,427 s vs ordinary 136 s |
| #973 | Retained-only 5×64-token generation smoke | 5 prompts | 43/50/48/41/47 unique tokens; no EOS/cycle; "All five outputs drift from their prompt subjects or scenes" | — | AUTONOMOUS_GENERATION_SMOKE_COMPLETE; coherence NOT ESTABLISHED | — |
| #973 | Paired-H4 addressing successor | 512 prompt directions | structural repeats **−97.5477%**; prompt contrast worse; slightly better fresh-language | V1 | **PAIRED_H4_PROMPT_CAPACITY_FAIL** | — |
| #973 | Direct retained readout | 512 directions | gain 0.0076304198→**0.0215897894**; 343/512; fresh NLL −0.1636410364; state-off −1.1234286047 nats; **49.8% of absolute floor** | V1 | PARTIAL | — |
| #973 | Layerwise-normalized readout `E@[N(h)+(g/√2)(N(a1)+N(a2))]` | 512 directions / 8,192 target tokens; fresh 247,920 | gain **0.02869802096506591** vs V1 0.007331623694789724; 339/512; own NLL 3.479876528760464 vs 3.6930405921095097; fresh 3.712641167679153 / 31.661826% vs 3.8850003882891597 / 29.728138%; state-off −1.3495375636624845, −20,595; **floors 0.04332169878499658 (absolute) and 0.025341569256760274 (incremental) missed** | g=0 control | PARTIAL; "ended the parameter-free readout ladder" | — |
| #973 | `R4LearnedCandidateLeafAssociativeReadoutV1` | 512 sealed directions; fresh 247,920 | geometric gain 0.00637679, 299/512, NLL 3.710383; V1 0.00642365, 308/512, 3.712799; **pooled (address-blind) 0.01026323, 324/512, 3.682891** — strongest; fresh V1 3.903636/29.6285%, geometric 3.901412/29.6342%, pooled 3.873756/30.0428%; geometry−pooled **−0.00388645** (209/512); geometry−deranged **−0.00028887** (251/512); required 308/512 | pooled; fixed-leaf deranged | **NO_CAPACITY; GEOMETRY_ATTRIBUTION_FAIL** | pooled preserved "only as a non-geometric control" |
| #973 | `R4PredictiveBlockDeltaBindingV1` (V5) | 512 directions; fresh 247,920 | geometric gain **0.03896945868086732**, 375/512, NLL 3.5419674206289073 vs floor 0.04332169878499658; V1 0.005190052751459007; pooled 0.009168421948743344; geometric−plain 0.023929811749894725 < 0.025341569256760274 and plain NLL better (3.518444197495228); geometric−transport-permuted +0.03181032686529761 (310/512, passed); geometric−additive **−0.006512463228773413** (234/512); fresh geometric 3.84055165318221 / 30.979348% vs pooled 3.85444653890486 / 30.14924169% | plain delta; additive; transport-permuted | **PREDICTIVE_BINDING_NO_TERMINAL_CAPACITY → STOP_WITHOUT_GENERATION** | first scoring attempt: tail-batch harness bug (NOT_RUN, preserved) |

### 1.9 Track T7 — Grounded correctness on the #1017 checkpoint (#954)

| Sub-campaign | Panel | Numbers | Result |
|---|---|---|---|
| C1-SB0 | 3 frozen probes (supported/unsupported/conflict) | 384 MPS steps in 883.7735486670863 s; product population **1/3 — all three decoded `ABSTAIN`** | FAIL_GROUNDING_PRODUCT_TRANSFER_ABSTENTION_COLLAPSE |
| C1-SB0 baseline observation | 2 prompts | `Context: The sky is blue. Question: What color is the sky? Answer:` did not produce the answer; cloze `the sky was` → `dark` | negative |
| C1-SB1 `R4SourceSpanPointerV1` | 128-item dev gates ≥95% | answer 89/128 (69.53125%), abstain 114/128 (89.0625%), conflict 117/128 (91.40625%), pointer 121/128 (94.53125%) | FAIL_DEVELOPMENT_GATE_STOP |
| C1-SB2 `R4SourceRelativeRelationHeadV1` | fit 12/20/6; sealed 12/20/6 | fit **12/12, 20/20, 6/6**; sealed **5/12, 14/20, 0/6**; only candidate-order control passed | FAIL_MATCHED_TRANSFER_PREFLIGHT_STOP |
| C1-SB3 rank-8 LoRA on Q/K/V/O | fit 126; sealed 63 | sealed positive recall **0/76 → 73/76**; negative specificity 234/239; fit **124/126**; sealed **56/63** (19/21, 19/21, 18/21); copies 19/21 | FAIL exact gate (bounded mechanistic transfer) |
| C1-SB4 record-level margin | same | exact records **70/126 fit, 35/63 sealed**; negatives 394/478, 197/239; "a question-ignoring ` is inside ` rule reproduces every aggregate count exactly" | FAIL_JOINT_CANDIDATE_MARGIN_PREFLIGHT |
| C1-SB5 paired-query matrix | 56 fit / 28 sealed | fit **56/56**; sealed **14/28**; pair-mean-query and attention-off 0/28 | FAIL_PAIRED_QUERY_BINDING_PREFLIGHT; #955 blocked |

### 1.10 Track T8 — Zoology MQAR and controlled-English chain (#1043–#1102)

| Issue | Experiment | Numbers | Result |
|---|---|---|---|
| #1043 | Position-preserving causal K/V via coherent R4/H4 frames | logit Δ **2.193450927734375e-05 vs 2e-5 bound**; 257,136 identical top-1; construction diagnostic **30/87,360 MQAR, 578/8,190 English, 2,730/2,730 no-history abstentions** | INVALID_POSITION_KV_BINDING (2-ULP numerical miss; "learned the easy classifier") |
| #1045 | Role-tagged associative curriculum | construction 65,500/65,536 (99.945068%); dev **7,137/8,192 (87.121582%)** vs 99% req.; best epoch 53 7,162 (87.426758%); NLL 1.1712778; 966.7488 s | OPEN_MQAR_NOT_LEARNED |
| #1049 | Reduced-calibration stock cell | construction 32,758/32,768 (99.969482%); dev best **999/4,096 (24.389648%)**, final 980 (23.925781%) | SCALED_SOURCE_CALIBRATION_MISS |
| #1050 | Released-config Zoology reproduction (CPU, 4 threads) | **11,900/12,000 = 99.1666667%** at epoch 20; NLL 0.05124610455830892; 577.834602 s | SOURCE_REPRODUCTION_POSITIVE |
| #1053 | Positive cell on exact #1045 bytes | dev **984/8,192 = 12.01171875%** | STOCK_CELL_TRANSFER_MISS |
| #1055 | Optimizer-clock correction | dev 12.01→**68.2861328125%** | CLOCK_MATCHED_TRANSFER_MISS |
| #1057 | Fixed-checkpoint continuation | dev **98.52294921875%** (+30.23681640625) vs >99% | miss (near target) |
| #1059/#1061 | Coherent-R4 inference adapter | 12,000 / 8,192 decisions preserved through R4 transport | R4_INTEGRATION_PRESERVED |
| #1063 | English supplied-context binding | construction **2,396/8,192 = 29.2480%** (req. 8,111); dev 218/1,024 = 21.2891%; complete groups **0/256**; question change altered prediction 12/512 = 2.34375% | ENGLISH_BINDING_CONSTRUCTION_MISS |
| #1067 | Query-object readout | construction 3,735/8,192 (45.5933%); object pairs 447/2,048; owner pairs 47/2,048 | partial |
| #1069 | Owner+object encoding | 50.2686% aggregate; slot 4 99.6582% vs 29.4922–36.0352% elsewhere | preservation fail |
| #1071 | Cyclic fact order | 20.7764–21.0083% vs reference 40.8569–45.5933% | preservation fail |
| #1073 | Learned compound binding (explicit roles from fixed grammar) | 100% supported/absent construction and held-out-combination development in 4 orders | COMPOUND_BINDING_FRESH_PASSED |
| #1075 | Compound preservation through R4 | 46,080 predictions preserved | pass |
| #1077 | Learned clause-role interface (141,571-param reader over frozen 286,976-param core) | held-out passed | LANGUAGE_INTERFACE_HELDOUT_PASSED |
| #1079 | Two-stage R4 preservation | **156/156** criteria; 25,600 answers; 358,400 role decisions; 384,000 vectors; max logit diff 8.82149e-6; fact-frame corruption drop 84.1797–93.2617pp **6/6**; token-frame drop 28.7231–52.6367pp **3/6** (50-pt floor missed); 57.0422 s, 2.2171 GiB | LANGUAGE_R4_PRESERVED_CONTROL_WEAK |
| #1082 | Token exposure diagnostic | 286,720 measurements; fact locations 0.0035% / objects 0.0039% changed-frame attention | descriptive only |
| #1094 | Clause adapter comparison | 1,600 valid rows (320 authoring + 1,280 withheld), 80 refusal, 16 boundary; all matched; 6,400 forwards, 15.821630625 s | CLAUSE_ADAPTER_PRESERVED |
| #1102 | Native Rust bridge | 320/320 answers, 4,480/4,480 roles, 16/16 refusals; max tensor error 4.768372e-6; 1,280 forwards, 8.809784 s | NATIVE_REFERENCE_PRESERVED |

### 1.11 Track T9 — Native geometric model (Sep 4–7, #973 recovery → #1136–#1140)

Corpus for the entire track: **524,288 bytes** = 127,231-byte `prime_route_attention.rs` + first 397,057 bytes of `TinyStoriesV2-GPT4-train.txt`, chunked into 4,096-byte documents: 78 count-construction / 26 readout / 25 open-development; later enlarged to 1,385,121 bytes / 391,725 target positions (`docs/native_geometric_recovery_973.md:39-47, 169-174`). All later "answers" panels are authored templates (e.g. "suri has 13 coins. orin has 4 coins … sum").

| Date / record | Experiment | Panel | Numbers | Controls | Result |
|---|---|---|---|---|---|
| Sep 4 recovery | Fixed-formula geometric readout | 32,764 eval positions | full **0.2289% / 0.1862% / 0.1312%** (windows 32/128/512) vs geometry-disabled **21.5206%** | geometry-disabled | Negative ("repeated evidence overwhelming the lexical signal") |
| same | Learned readout | same | full **26.9259% / 28.6717% / 24.7833%** vs geometry-disabled **27.5089% / 28.7267% / 26.1415%**; "All three global geometric gates become zero" | geometry-disabled | Geometry slightly hurts; "not evidence of geometric advantage" |
| same | Larger corpus | 146,668 dev positions | full 36.2485%; geometry-disabled 36.1429%; zeta-disabled **36.2513%**; H4-disabled 36.1429%; paired-disabled 36.1565%; learned memory reader **36.2485→33.2663→32.7638%** | ablations | ≤0.1pp deltas; memory reader regresses |
| same | Supplied-fact controlled read | 96 prose queries; 48 pairs | exact-cue 512: **86/96 vs 24/96 memory-disabled**; 41/48 pairs; geometry-disabled 66/96; zeta-disabled **91/96**; H4-disabled 82/96; joint 82/96 prose / 30/96 Rust vs 43/96 & 39/96 | ablations | bounded positive; zeta hurts |
| Sep 5 `/4` `/5` | Occurrence selection; response state | 32 prose / 32 Rust | prose 5/32→6/32; Rust **0/32**; `/5` 5/32; Rust 0/32 | — | negatives |
| Sep 5 typed value | Literal store + integer add | 12/12 prose, 8/12 Rust numerals; complete 2/16 & 0/16; **all 20 generated Rust sources fail compilation**; `/2` fix: 24/24 numerals, 4/4 swaps; whole-word disabled 4/24 | H4/zeta controls make same selections | bounded |
| Sep 5 completion / entry / word copy | continuation heads | 12/16 & 12/16 vs 2/16 & 0/16; entry 16/16 & 16/16 vs 12/16; word copy 12/12 transfers, copy-disabled 0/12; two city transfers `Unknown` | within-artifact disables | bounded; "not a matched geometry-free refit" |
| Sep 5 #1136 | Zero-match entry | 16/16 wording; open 16/32 (8/24 supported); 62 preserved | — | bounded |
| Sep 5 #1137 | Role-aware source/entry | 62/62; 8/8; **24/24 vs 13/24 parent**; 4 Rust fns / 28 assertions; first fit 61/62 kept as negative | parent | bounded positive |
| Sep 5 #1138 | Exact relation memory; role paths | 56/56 OPEN; 26/28 answers vs 12/28; **21/28 writes**; role path 84/84; 28/28 vs 24/28 & 14/28 | parent | transfer negative then positive |
| Sep 5 #1139 | First learned H4 routing block | **735 authored positions**: parent 45; angular 182; **exact-code 223**; fixed 177; 0/8 target continuations; retention 38/62; relation answers angular 16/28 vs exact-code 28/28 | exact-code selector | Negative for geometry |
| Sep 6 #1139 | Dependent reads | 735: parent 45; angular 120; exact-code 144; 0/8; preservation 6/62 vs 0/62; both Rust continuations fail compile | exact-code | Negative |
| Sep 6 #1139 | Retained-source routing | 320/320 fit; **62/62, 24/24, 24/24 vs exact-code 36/62, 10/24, 10/24**; codes-disabled 4/24 | exact-code | angular > exact-code (first such) |
| Sep 6 #1139 | Dependent source read | **40/48 vs exact-code 32/48 vs parent 20/48** | exact-code | angular > exact-code |
| Sep 6 #1139 | Writer binding; NoWrite cache | 48/48; row comparisons 580,944→**226,101,330**, then cache 21,799/21,907 skips → 539,448 | — | cost regression then repair |
| Sep 6 #1139 | Typed roles; operand provenance | **12/12 vs exact-code 2/12**; provenance 12/16 vs parent 3/16 (**exact-code also 12/16**) | exact-code | mixed |
| Sep 6 #1139 | Literal selection/admission/NoRead/source | 8/16→16/16; 7/16→12/16 (regressions, not promoted); 12/16→16/16 (exact-code matches); 12/16→14/16; source 20/20 vs 14/20 parent & 10/20 exact-code; 551/603; "13 instead of Paris" | exact-code | mixed |
| Sep 7 #1139 | Joint admission; literal binding; source context | 558/615→563/615; open 4/6, fresh 8/12 (parent, angular, equality all equal); binding 571/631→630/631, fresh **16/16 vs equality 8/16**; context 654/663→663/663, fresh **32/32 vs context-disabled 24/32** | equality; context-disabled | bounded positives |
| Sep 7 #1139 | Source span; span context | open 1/6→6/6, fresh 3/10→9/10 but construction **663→647/663** (rejected); span context 5/14→14/14, 2/6→6/6, 3/9→9/9; pair removal 11/14, 4/6, 6/9 | pair removal | one rejected, one accepted |
| Sep 7 #1140 | Retained/reverse spans; relation starts | 6/6, 6/6; 26/28→28/28 runtime fix; reverse 3/6→6/6, diagnostics 0/2; relation start 2/8→8/8 construction, open 2/8→6/8, **selection failed** | — | one negative |
| Sep 7 #1139 | Contextual phrase starts; writer refinement `79710468` | starts 12/50→50/50, 3/12→12/12, 12 fresh; removal 38/50, 7/12; writer 232/232, 32, 12/12, 76/76, 12 evicted, 4 boundary; **663 retained answers; 41 preexisting reader failures among 200 anchors** | root removal | bounded positives (the "retained" artifact at Sep 8) |

### 1.12 Track T10 — Sep 8 recovery and withdrawal

| Finding | Source |
|---|---|
| "Twelve-axis 100% alpha qualification" passed prose for ≥1 token, coding/reasoning for nonempty text, Studio/efficiency for metadata flags; security audit returned literal `true`; `m1_profiler.rs` supplied **3500 mW** and nominal thermals as constants with M1/16 GiB hardcoded; prediction time split 40%/60% into "lookup/operator"; benchmark table numbers **>260× latency, >100× energy, 0% throttling "cannot be retained as measured results"** | `docs/integration/recovery-2026-09-08.md:14-27` |
| Empty artifact loading silently trained a tiny fixture model | same, table row 8 |
| Experimental `b2da9f9a`: 183/183 construction, 12/12 open (parent 8/12), 0/12 intermediate-removal, 12/12 fresh (same names/wording); but repeated `13+4` after `14+4` → **`35.`** where parent → `Unknown.`; retained chain diagnostic **3/6 FAIL**; baseline browser prompt about astronomy produced **"Where is selra."** | `recovery-2026-09-08.md:60-73, 86`; `docs/evidence/native_recovery_2026_09_08.json` (`negatives.general_prose`) |
| Cumulative model compute 6,960.652/7,650 s at that date | same |

### 1.13 Track T11 — Shared geometric core (Sep 12–14)

Population for the first phase: 12 authored construction documents → 672 construction positions; 249/381-position authored "open" draws; later phases: authored four-record/two-clause template tasks (names like `bruno`, `izpkgzr`, verbs `trust`, `call`, `today … will … tomorrow`).

| Date / record | Experiment | Numbers | Controls | Result |
|---|---|---|---|---|
| Sep 12 shared core | Hard-forward coordinate fit (100,000 proposals, 440 accepted) | train NLL 5.820628→5.223800; correct 4→25/672; held-out NLL 5.792988→**5.710435 (−1.425% < 2% req.)**; acc 2→**0**/249; context removal +0.0686%, prior-root +0.1226% (< 1%) | context/state/transport/zeta-disabled all ≈5.71 | **FAIL_CONTEXTUAL_TRANSFER_SMOKE** |
| diagnostic | branch analysis | first branch 206/672 errors (best landmark 189); ASCII 292/660 (≥168 forced); full-leaf mode 25→33/672, 0→2/249 | — | "restricted emission interface is a demonstrated bottleneck" |
| calibrated emission | schema-2 two-component operator | NLL 5.612466→3.473156; 4→65/672; root 12/672 ok; **ASCII 220/660 vs <216 req.**; audit min 10/672, 202/660 | — | FAIL_CALIBRATION_DOSE_NO_JOINT_FIT |
| block calibration | exhaustive 327,571,200 settings, 94 nodes, 15.198 s | 3.473156→3.422394; 65→80/672; 211/660 | — | PASS_BLOCK_CALIBRATION |
| joint fit `ade9a1cf` | seed 443, 100,000 proposals/149 accepted, 31.444 s | 3.422394→3.375391; **80→79/672**; open 381: legacy 5.747049 / 2 correct; prior 3.685167 / 29; block 3.707224 / 29; joint **3.663040 / 28**; 1.19185% < 2%; context removal 1.10791%; prior-root 0.27630% | — | FAIL_CONTEXTUAL_TRANSFER_SMOKE; all 4 continuations incoherent, 96 bytes, no EOS |
| credit assignment | 8 hot params, 6,171 replays | 0/8 improved; root 26 worsens 0.959586 vs root 76 improves 3.075335; oracle-gain **−0.312027 (<0.5)** | — | FAIL |
| tied categorical | 4 seeds | hard exports beat coordinate 4/4, fixed-joint **0/4**; oracle gain 0.788002; complete 3.375391→3.467544, 79→67/672 | — | FAIL |
| full objective | 14,400-pair exhaustive table | zero improving settings; [119,119] unique minimum (next 3.433977) | — | FAIL |
| final emission | 327,571,200 settings | 3.375391→3.354320; 79→81/672; open 28→33/381; **35 preservation losses** | — | FAIL (preservation) |
| angular tree | 66 trees / 368 nodes | construction 3.354320→**2.444899**, 81→206/672; open 33→57/381 but NLL 3.659923→3.908623; preservation 188/181 losses | — | FAIL |
| decoder validation | 3 folds × 4 configs | depth-3/leaf-16 `8f8e5c34`: 166/672 (2.839534); open 52/381 (3.653604); losses 195/180/199 | — | FAIL |
| tree-state pair; coupled profile; active sensitivity | 14,400 / 239 / 136,851 settings | no improvement; next-best [119,48] 2.843190/108; READ 1937 → 168/672 but open NLL 3.653604→3.662621 | — | FAIL ×3 |
| context augmentation | 48 varied vs 4×12 docs | parent 100/672 3.731054; repeated 93 (3.713508); varied 93 (**3.790710**); varied **worse than its ContextDisabled control 103/672 3.763306** | ContextDisabled | FAIL |
| decoder refit 4-cell | | 100/672 3.731054; 99/3.694897; 81/3.418240; 79/3.461138 | — | FAIL |
| addressed attention: primitives / forward / pilot | 4 particles × 64 positions | 18/18 tests; forward/backward 309,230 µs; pilot dev CE 5.5530255→5.5516114, correctness 1/154; 0/8, 0/16 exact | Read/Payload/Transport-disabled | **FAIL_ADDRESSED_ATTENTION_LEARNING_PILOT** |
| causal-credit estimator; matched pilot | 32 batches | covariance ratio 0.525886 (−47.4%); pilot CE **5.5530255→5.5711165** (worse); 0/306 train | — | FAIL |
| Hamming refinement; Hamming policy | 5 authored cases; 24 examples | 856,800 comparisons: 727,320 strict / 129,480 ties / 0 reversals; initialized **0/24; 1/460 symbols; CE 10.9981724105; 1536/1536 max ties** | — | primitives PASS; no learning |
| Sep 13 relational attention | typed one-read, 4 records | 64/64; 32/32 changed-source; uniform routing 16/64; ReadDisabled/QueryReversed 0/64; attempts 14→16→42→64 | — | PASS (typed) |
| Sep 13 dependent / adaptive / text / recurrent | typed curricula | 128/128 (untrained 32/128); 192/192 (AlwaysEmit 96/192, FixedTwo 56/192); 64/64 spans incl. 48/48 longer than training; recurrent 64/64, depths 32/32,12/12,12/12,8/8 | disabled → 0 | PASS ×4 |
| Sep 13 ordered state; language relation | 64 dev; 128 dev (4 raw sentences + question, 4 words each) | 64/64, controls 0/64, FinalRootOnly 56/64; relation 128/128, **ExactIdentity/FinalRootOnly/FeedbackDisabled also 128/128** | ExactIdentity | PASS; "not a geometric metric advantage" |
| Sep 13 relative language → span → phrase update | 768/768 (first attempt 384/768; EndpointDisabled 384/768); scheduling 4,096/4,096 & 1,536/1,536; depth 1,280/1,280; completion 384+512 (predecessor emitted prematurely 1,024/1,024); span 1,728/1,728 vs one-word parent 576/1,728 (first attempt: 976 inseparable rows); phrase update 864/864 | ExactIdentity = Full in all | PASS ×7 |
| Sep 14 occurrence correspondence | 384 dev | unchanged 144/384; injective witnesses 296/384; rule fit 384/384 but training **1,904/2,816** and retention **3,992/6,304**; no-fit ceiling 1,904/2,816 | — | **FAIL** |
| Sep 14 ordered correspondence; phrase order | 384/384; 2,816/2,816; 80/80 (StructureDisabled 16/80) | ExactIdentity = Full | PASS |
| Sep 14 lexical role reuse | 24+24 | `bruno` **24/24**; person-name `will` **0/24** (global context flag = payload barrier) | — | **FAIL** |
| Sep 14 occurrence role | 504/504; 312/312; 72/72; 48/48; 80/80; 6,688/6,688 retained | — | PASS |
| Sep 14 role transfer (unchanged artifact) | 204 | **120/204**: coverage 12/60, styled 60/96, predicates 48/48 | ExactIdentity = Full 204/204 | **FAIL** |
| Sep 14 query participation; styled roles | 12/60→60/60; 96/96; styled 60/96→96/96; QueryRefinementDisabled 2,904/2,976 | — | PASS ×2 |
| Sep 14 neighbor transfer (unchanged `0175ad86`) | 492 | **468/492**: valid 84/96, conflicting 84/96; key (13,64,15) collision | ExactIdentity = Full | **FAIL** |
| Sep 14 unknown neighbors `60d19679` | 492 | **492/492**; AnchorsOnly / NoProjection 468/492 | — | PASS |
| Sep 14 **independent** neighbor panel (frozen acceptance, OS-random seed, 3,776 novel 7-letter tokens) | 2,304 | **2,016/2,304**: 7/8 shapes 96/96 ×3; `New word + will + new word` **0/96 ×3**; ExactIdentity 2,304/2,304 = Full; ReadDisabled/UpdateDisabled 0 correct; retained 492/300/200/6,688 exact | ExactIdentity, reload, Read/Update-disabled | **FAIL_INDEPENDENT_NEIGHBOR_TRANSFER** (`docs/native_geometric_independent_neighbor_973.md`) |

---

## 2. WHAT HAS EVER BEEN SHOWN vs NOT

### 2.1 Has any variant ever produced free-running natural prose (perplexity or human-judged)?

**Yes — but only the transformer-shaped float models of Track T6, never a no-matmul / geometric-native model.**

- The #1014/#1017 models are ordinary 7.15M-parameter TinyStories transformers (RMSNorm, RoPE, SwiGLU, learned Q/K/V/O, full-prefix scaled dot-product, stable causal softmax) whose heads are re-expressed in R4 blocks. #1017 produced five 128-token TinyStories-style continuations passing a 5/5 "subject-or-scene" rubric (`docs/r4_softmax_quality_capacity_continuation_1017.md:83-94`); #1014's exact outputs are printed and are readable but semantically loose (`docs/r4_softmax_end_to_end_attention_1014.md:141-247`). The project itself says these "do not establish … transformerlessness, multiplication-free or table-native execution" (`:258-264`).
- `R4SoftmaxReferenceGeneratorV1` is SmolLM2-135M with R4 frame transport: 4/5 rubric, fluent by construction because it is SmolLM2 (`docs/helm_d_r4_softmax_decoder_973.md`, `docs/geometric_decoder_spike_950.md`).
- `R4RetainedLanguagePathV1`'s retained-only 5×64-token smoke "drift[s] from their prompt subjects or scenes"; coherence "NOT ESTABLISHED" (`docs/r4_retained_language_path_v1_973.md:263-289`).
- **Every table-native / integer / geometric-native track is negative on free-running prose**: T1 produced `ounds Callounds…`, `cut cut cut…`, digit attractors, 99/100 rollouts identical to suffix-only, 710‰ cycles (`#841`, `#840`, audit §13.2); post-#755 text was "grammatical-if-wandering … prompt-insensitive" and never installed as a canonical bundle. T9's browser probe about astronomy returned **"Where is selra."** T11's every "Full continuation" is described as "incoherent, 96 bytes/no EOS" across ~15 records. No human-judged or perplexity-based prose evaluation exists for any geometric-native artifact.

### 2.2 Any measured perplexity / bits-per-byte on a real corpus?

**Yes, for T1 (teacher-forced bits/token) and T6 (NLL nats/token); none for T9/T11.**

- T1 on Simple English Wikipedia (real text): Rule 1+2 **11.94 bits/token** (24.30% top-1) on 72,864 held-out positions, teacher floor 3.6015; best live arm 10.43; TLA-3 13.62 (`#516`). Earlier stories15M fixture: 8.0249 WB bits/token (teacher floor 1.4260). On the EXCT-miss slice the graph generalizes at **1.81% top-1 / 16.89 bits** (audit §13.1). Note these are *bits/token* under the project's canonical cross-entropy definition, comparable only within scorer+distribution (`BASELINE.md:128-134`); no bits-per-byte is reported anywhere.
- T6: TinyStories sealed NLL **2.127407** (#1014, 30M tokens) → **1.572752** nats/token (#1017, 150M tokens); `R4RetainedLanguagePathV1` **3.899862** nats on 247,920 fresh TinyStories decisions with a 4,096-token BPE at 252,160 params. These are float transformer results.
- T9/T11 report only "next-piece accuracy" on a 0.5–1.4 MB authored/TinyStories fragment (26.9–36.2%) and NLL on 672/249/381 authored positions (e.g. 3.375391 / 3.663040). Nothing on a real corpus.
- **[Inference]** No number in the repository allows a like-for-like perplexity comparison between a geometric-native artifact and GPT-2 124M or SmolLM2 on any public benchmark. The README's own list of comparators never appears with a measured geometric-native counterpart.

### 2.3 Any latency / energy measurement on M1 for a complete request?

**No complete-request latency or any energy measurement exists. All published energy/thermal/comparative-speed claims were withdrawn on 2026-09-08.**

- The recovery record states `m1_profiler.rs` "supplied 3500 mW and nominal thermals as constants, with M1/16 GiB identity hardcoded" and that ">260× latency, >100× energy and 0% throttling cannot be retained as measured results" (`recovery-2026-09-08.md:19, 27`). `release-qualification-alpha.md:9`: "Power, energy, sustained thermals and dense-model speed/memory comparisons are UNAVAILABLE."
- What does exist: (a) T1 single-thread throughput 77,342 tok/s on a Linux container at 31.7% teacher agreement (not M1, not a complete request, and at a quality far below the teacher); (b) T12 GPT-2 offline executor 0.695× conventional on M1 (teacher path, not serving); (c) T6 #1017 4-token generation 3.06 s exact / 0.116 s Accelerate and 64-token in 0.22 s (float BLAS on M1, matmul); (d) T9/T11 "small warm-step measurements exclude loading and ingestion" and "allocation NOT_RUN" in most records; a 512-step zero-allocation census passes for the T11 kernel. The project repeatedly states "Tiny warm-kernel timings are not complete-model submillisecond claims" (`model-direction-2026-09.md:145`).

### 2.4 Any independent (held-out, non-template) evaluation?

**Partially, and only for T1/T6/T8. None for the geometric-native language core.**

- T1: held-out Simple-Wiki articles under a content-hash split (real natural text, non-template) — but the deployed scorer resolves 85% of positions by exact-context lookup and is "suffix-local" (#834: full context 26.9% vs 2-token suffix 27.1%).
- T6: TinyStories sealed splits by `BLAKE3(story) mod 100`, tokenizer trained on train only; test opened once (#1014/#1017). Retained-language-path validation is "nonsealed" fresh stories.
- T8: synthetic MQAR; the English worlds are "supplied clauses, known lexicon/query form and already-observed semantic worlds" (`#1079`).
- T9/T11: every panel is authored from templates by the developers or by a seeded Rust generator; the one panel the project calls "independent" (Sep 14, 2,304 cases) draws 3,776 random 7-letter pseudo-words into a fixed two-clause grammar ("today izpkgzr will call zlrkawr will ikbjwdx tomorrow.") and is described by the project as "independent finite authored two-clause/four-record transfer … no general prose qualification" (`evidence/native_geometric_independent_neighbor_973.json` → `summary.scope`). The project acknowledges "Open feedback informed construction design. Previously opened preservation panels are not new held-out evidence" (`recovery-2026-09-08.md:71`) and that many "development" panels were fully seen in training ("All development feature rows were seen during training", `current-state.md:169`). **[Inference]** No third-party benchmark, no natural-text held-out set, and no blind human evaluation has ever been run on a geometric-native artifact.

### 2.5 Any result where geometry beat a matched non-geometric control?

**A handful of small, bounded positives; the preponderance is null or negative, and the project itself never claims a geometric predictive advantage.**

Positives (all bounded, all on authored or tie-break-only populations):
1. #953 `MultiscaleCountRadiusR4V1`: +0.950392pp top-1 over the #989 count table on 446,342 held-out positions — by changing **only max-count ties** (56,280 of them; geometry right 6,753 vs baseline 2,511). Real corpus, byte-replayed. The project calls it "the only accepted #953 claim" and "not semantic geometry, attention, correctness".
2. T9 #1139: angular (signed-H4) selector 62/62, 24/24, 24/24 vs exact-code 36/62, 10/24, 10/24 (source routing); 40/48 vs 32/48 (dependent source); 12/12 vs 2/12 (typed roles); literal binding fresh 16/16 vs equality 8/16; source context 32/32 vs context-disabled 24/32. All on ≤100 authored template items.
3. `R4RetainedLanguagePathV1`: retained NLL 3.899862 vs ordinary softmax 3.903394 (Δ 0.0035 nats) at equal parameters — but *not equal work* (115,200 vs 58,080 score pairs; 10.5× wall time) and "H4 specificity NOT_EVALUATED".
4. T3 #822 backed-off mix +30.6‰ — not geometric (joint count tables).

Nulls / negatives where a matched non-geometric control was equal or better: #1139 first routing (exact-code 223 vs angular 182 of 735), dependent reads (144 vs 120), operand provenance (equal), joint admission (equal), literal admission (equal); T11 every ExactIdentity control equals Full on every PASS record (Sep 13–14), FinalRootOnly 56/64; recovery readouts geometry-disabled ≥ full at all three windows and zeta removal *improves* accuracy (36.2513% vs 36.2485%; 91/96 vs 86/96); #973 learned associative (pooled > geometric; geometry−deranged −0.00028887), V5 (plain delta NLL better; additive beats delta), gated-delta (16/28 vs 23/28), direct attention V3 (3/12 vs 12/12), V4 (13/24 all arms), document placement (below order-shuffled), W(3,3) planner (at level of its own randomized controls), #845 "NO GEOMETRIC ADVANTAGE"; #1079 token-frame corruption drop only 3/6 views. The README's own snapshot: "Does this show a geometric predictive advantage? Not on this panel: ExactIdentity has the same outcomes" (`README.md:192`).

---

## 3. NEGATIVE RESULTS — every failed gate / withdrawn claim, with cause

Grouped by cause class. Verdict strings are the project's.

**A. Claims withdrawn as unsupported or fabricated by construction (Sep 8, 2026)**
1. Twelve-axis alpha qualification — gates tested nonempty output / metadata flags (`recovery-2026-09-08.md` table).
2. Security audit PASS — literal `true` fields.
3. Verified manifest / byte-exact install — nonempty CID used as equality.
4. Sustained M1 energy/thermal measurements — 3500 mW and thermals hardcoded.
5. Six serving stages measured — 40/60 split of one timer.
6. Quality-matched coding benchmark — test wrote and compiled a literal program.
7. Formal serving/geometry proof — caller-recorded counts, fixed flags, arbitrary quaternion pair.
8. Delivered artifact behavior — empty artifact silently trained a tiny fixture model.
9. Public benchmark table (>260× latency, >100× energy, 0% throttling) — no comparator, power trace or task criterion.
10. `v0.1.0-alpha` publication — "does not qualify alpha" (`release-qualification-alpha.md`).

**B. Generation / coherence failures**
11. T1 free-running: median divergence 0; 99/100 suffix-identical; 710‰ cycles (#841) — cause: representation is suffix-local exact-context lookup (#834/#842).
12. T1 live probes: prompt-invariant token cycling on all three bundles (audit §13.2) — causes: #755 corpus story-scrambling (fixed in code, bundles not recompiled) and greedy argmax over quantization-collided codes (#759).
13. Skip lane worsens free-running (cycles 710→1000‰) (#840).
14. #944/#946 prefix memory and trajectory regions INERT (0/512 changes).
15. T11 every Full continuation incoherent at 96 bytes/no EOS (≥12 records, Sep 12–13).
16. Retained language path 5/5 outputs drift from prompt; #1041 history binding 0/2.
17. T9 astronomy prompt → "Where is selra."; T9 Rust generations fail compilation (20/20; both #1139 continuations); city facts → `Unknown`.

**C. Frozen quality thresholds missed (float T6)**
18. #1014 sealed NLL 2.127407 > 1.50; retention 3/5.
19. #1017 sealed NLL 1.572752 > 1.50.
20. #1019 UNAVAILABLE_HARDWARE_BUDGET (20.66 h > 8 h).
21. Paired-H4 prompt capacity FAIL (−97.5477% structural repeats; prompt contrast worse).
22. Direct readout PARTIAL (49.8% of floor); layerwise PARTIAL (both floors missed).
23. Learned associative readout NO_CAPACITY + GEOMETRY_ATTRIBUTION_FAIL (pooled non-geometric control strongest).
24. V5 predictive block delta NO_TERMINAL_CAPACITY (0.03897 < 0.04332; plain NLL better; additive beats delta) → STOP_WITHOUT_GENERATION.
25. Group-addressed retention decoder memorized 4,096 training decisions, failed validation.

**D. Geometric mechanism ≤ matched control**
26. Gated-delta 16/28 vs plain 23/28.
27. Direct attention V3 H4 3/12 vs plain 12/12.
28. V4 held-out 13/24 all arms; destructive controls ≈ same.
29. Document placement 8.37% < #953 fallback 12.22% and < order-shuffled 8.38%.
30. Intrinsic Lorentz: covariance 9.12e-8 vs 1e-8; NLL worse than donor by 1.2531 nats.
31. Learned-manifold Lorentz NLL 7.7106 vs donor 3.6676 / Euclidean 4.4832.
32. Tangent readout raised MSE (ratio 1.0644).
33. Trace state student margin 0.000024 vs 0.10.
34. #1012 insufficient support coverage (min fold 34.7% < 50%).
35. #951 mixer 4.44% < 5%; operator/token loss flat at 1.0.
36. W(3,3) planner: no advantage; at level of randomized controls (#845).
37. #970 heatmap transfer 0/6; #983 0/6; #952 21/21 cells identical; #967 6/6 ties; #953 same choice for both orders, placement 0/2 vs permuted 2/2.
38. T9 recovery: geometry-disabled ≥ full at all windows; global geometric gates → 0; zeta removal improves; memory reader regresses 36.25→32.76%.
39. #1139 first routing / dependent reads: exact-code 223/144 > angular 182/120; target continuations 0/8.
40. #1079 token-frame control 3/6 views (< 50-pt floor).
41. Route-attention kernel vs teacher: instrument vacuous (N2 null 0.292 ≥ signal); 119/120 heads vacuous (#804).
42. Hopf sector transport MRR 0.0045 vs 0.0743; CD/FMM/granularity/E8 keying dead; #400 CD term executed 0/1,998.
43. Code-space subdivision 25.6% < 26.5%; every "resolution" lever failed (#460/#435).
44. Reconstruction sub-unigram (16.3 bits vs 8.7); IPF joint at unigram floor (#456/#457).

**E. Learning-method failures on the shared core (T11, Sep 12–13)** — 45–56: FAIL_CONTEXTUAL_TRANSFER_SMOKE (×2), FAIL_CALIBRATION_DOSE, FAIL_OCCURRENCE_CREDIT_DIRECTION_PROBE (oracle gain −0.31), FAIL_TIED_CATEGORICAL (0/4 vs fixed joint), FAIL_FULL_OBJECTIVE (14,400-pair table has no improving setting), FAIL_FINAL_EMISSION (35 preservation losses), FAIL_ANGULAR_TREE (188/181 losses; open NLL worse), FAIL_DECODER_VALIDATION, FAIL_TREE_STATE_PAIR, FAIL_COUPLED_PROFILE, FAIL_ACTIVE_SENSITIVITY, FAIL_CONTEXT_AUGMENTATION (varied worse than its own ContextDisabled control), FAIL_CONTEXT_DECODER_REFIT, FAIL_ADDRESSED_ATTENTION_LEARNING_PILOT (×2; causal-credit pilot made CE worse), Hamming policy 0/24 with 1536/1536 ties. Cause per project: restricted emission interface, LUT4 dependency cone (≤64 inputs per output bit), finite proposal families, credit-assignment variance.

**F. Transfer failures on authored panels (T9/T11, Sep 6–14)** — 57–66: #1139 literal selection regressions (not promoted); source span construction 663→647 (rejected); relation start selection FAIL (longer candidate wins tie); `990ccbce` coefficient-only negative (8 span regressions); `b2da9f9a` 13+4→`35.` regression; chain diagnostic 3/6; FAIL_OCCURRENCE_CORRESPONDENCE (rule fit loses 912 training rows, 2,312 retained traces); FAIL_LEXICAL_ROLE_REUSE (`will` as name 0/24); FAIL_UNCHANGED_ROLE_TRANSFER 120/204; FAIL_UNCHANGED_NEIGHBOR_TRANSFER 468/492; FAIL_INDEPENDENT_NEIGHBOR_TRANSFER 2,016/2,304 (interior `will` role alias, key (13,64,64)).

**G. Grounded correctness (T7)** — 67–72: SB0 abstention collapse 1/3; SB1 development gates 69.5–94.5% < 95%; SB2 sealed 5/12, 14/20, 0/6; SB3 exact gate 124/126, 56/63; SB4 70/126, 35/63 (a question-ignoring rule reproduces aggregates); SB5 14/28 sealed. #955 reasoning blocked; #846 REASONING NOT ESTABLISHED (0 certifiable samples).

**H. Calibration / abstention** — 73–75: #837 NO CALIBRATOR (margin slice 520‰ wrong); #931 NO CALIBRATOR (UCB95 79‰ vs 10‰); #811 all five semantic-OOD probes stayed SERVABLE.

**I. Zoology (T8)** — 76–82: #1043 INVALID (2-ULP; 30/87,360 MQAR learned); #1045 87.12% < 99%; #1049 24.39%; #1053 12.01%; #1055 68.29%; #1057 98.52% < 99%; #1063 English binding 29.25% construction / 0/256 groups; #1069/#1071 preservation fails.

**J. Resource/`UNAVAILABLE` stops (not model evidence but recorded)** — 83–88: #986 frame/population unavailable; quaternion-cube fit 8/128 updates; #1019 MPS; intrinsic Lorentz attempt 01 JSON defect; V5 first scoring tail-batch bug; #1094 interpreter denied; independent-neighbor evaluation interrupted by storage guard (resumed).

**K. Instruments that could not fail (self-reported)** — seven harness bugs: `kappa_reproduction.rs` silently skipping, `cover_scaling.rs` tail split, degenerate `scaled-k0` arm, `run_ablation_benchmark` self-assert, #471 all-zero cross-tabs, #484 identical metric triple 0.6240/0.7179/0.9720 across three "different" records, VSA 0.0000 category error (`RESEARCH.md:2910-2936`).

---

## 4. HISTORICAL TRACKS — what each was, what it found, dormancy

| Track | What it was | What it found | Status |
|---|---|---|---|
| **TLA / R4G1 kernel (T1)** | Teacher-distilled table-native store: graded codes, exact-context (EXCT) precedence, region cover, integer Q8.8 scores; `docs/inference_contract.md` (no float/mul/div/alloc in hot path); `docs/hologram_r4_formal_monograph.md` (Jul 24 spec: 9-edge algebra, information-bottleneck objective, bounded planner, Boolean lowering). | Genuinely multiply-free; 144,496 int ops/token; 77k tok/s single-thread; ~35–39% teacher agreement on teacher-sampled stories; 24–31% on real Simple-Wiki with 11.9 bits/token vs teacher 3.6; **85% of positions answered by exact-context lookup; ~1.8% top-1 off exact context**; suffix-local; free-running degenerate. RF-31 ratified at +13.988‰ over TLA. | **Dormant/"preserved runtime reference"** since ~Sep 4. README: "infrastructure donors, not a ready geometric language learner". |
| **Transformerless "score"/COMPARISON** | Throughput and equivalence-class comparison vs llama.cpp/run.c on stories15M. | 225×/491× throughput at 31.7% agreement; 2.17 MB artifact; human-text top-1 17.6% vs teacher 47.2%. | Historical (pre-migration, Linux container); `r4 compare-report` prints the recorded certificate, not a re-measurement (audit §13.2). |
| **Geometric router (T2)** | Hopf/S³ sentence retrieval; content vectors; VSA/spectral. | Retrieval MRR 0.88+ after fixing a routing-vs-content category error; Hopf sector transport negative; its "word-Markov generator is not a language model" (`RESEARCH.md:2714-2719`). Ancestor prime-router "used Ollama for language and is not generation evidence". | Component retained; generator abandoned. |
| **S0–S4 programme (T3)** | Staged gates on the T1 bundle: prompt conditioning (S1), calibration (S2), free-running (S3), planning/reasoning (S4). | S1 REVISE then one floor-clearing off-serving arm (+30.6‰); S2 LIMIT; S3 LIMIT (GENERATION-NOT-ESTABLISHED); S4 LIMIT (REASONING NOT ESTABLISHED; W(3,3) no advantage). | Closed Aug 25. |
| **GNAF / formal (#653, #623)** | Vendored Lean4 proof that a reference WASM GEMM kernel is cost-optimal; 45 claims: 28 formalProof, 12 open, own terminal `WorkloadIncomplete`. | "None of this proves anything about r4's own kernels"; integration = claim-vocabulary bridge only. | Retained formal reference; not a generation mechanism. |
| **Geometric mixer/decoder spikes (T4, #950/#951/#958)** | Replace SmolLM2 layer-29 attention with a bounded learned mixer; prime-route manifest substrate. | G0 reachable; G1 4.44% < 5% and operator/token losses flat → REDESIGN_REPRESENTATION; #958 RETAIN_STORAGE_RECALL_ONLY (product probe never run). | Closed Aug 26. |
| **GI synthetic route attention (T5, #952–#989, #953, #973 Aug 27–28)** | Prime-route/ordered-S³/H4 exact-table selectors on 2–12-decision authored fixtures with target-free gates and permutation controls. | Series of 2/2 vs 1/2 vs 0/2 "witnesses"; every attempt at transfer beyond the fixture 0/6; the one corpus-scale test (document placement) scored below shuffle; the only accepted gain is a +0.95pp tie-break over a count table. | Superseded Aug 28 → T6. |
| **R4 softmax / HELM-D / Lorentz / gauge (T6)** | Ordinary transformers with heads split into R4 blocks and H4 frame transport; HELM-D Lorentz variants; retained-attention K/V memories; readout/binding ladders. | Parity with donor (expected under orthogonal transport); intrinsic/Lorentz variants fail; ordinary attention load-bearing (#1014: 2.68 nats); #1017 NLL 1.5728 on TinyStories; retained path competitive with equal-param softmax (H4 specificity unevaluated); every geometry-attribution test null or negative; V5 STOP_WITHOUT_GENERATION. | Parked Sep 1–3; "#1017 remains the working source-backed `r4 generate`" — an ordinary float transformer. |
| **Grounded correctness (T7, #954)** | SFT/LoRA adapters on #1017. | SB0–SB5 all fail their frozen gates; aggregate-equivalent shortcut found in SB4. | Blocked. |
| **Zoology / controlled English (T8, #1043–#1102)** | Stock HazyResearch attention cell on MQAR, then a 4-fact/1-question controlled English world; R4 transport as a parameter-free adapter. | Stock cell reproduces released result (99.17%) but not on project data (12–98.5%); English binding fails until explicit roles are supplied; R4 transport preserves predictions (max logit diff 8.8e-6) with a weak token-frame control. | Superseded Sep 4 by the "owner-directed native plan". |
| **Native geometric model (T9)** | Rust count tables + learned signed-H4 selectors + exact copy/add on ~0.5–1.4 MB authored/TinyStories fragment; Sep 5–7 accretion of bounded heads (#1136–#1140). | Dozens of bounded template positives (e.g. 24/24 vs 13/24); geometry ≈ exact-code in most; Rust generation never compiles unaided; general prose absent ("Where is selra."). Sep 8 audit withdrew alpha and all M1 claims. | Artifact `15baec48` retained, "parked"; memory repair "parked" Sep 12. |
| **Shared geometric core (T11)** | Isolated H4 finite-table recurrence + Hamming reads + byte/EOS trees; coordinate/proposal search; then typed-curriculum learned primitives. | ~15 consecutive FAIL gates on 672/249/381-position authored corpus (Sep 12–13); then ~20 PASS gates on typed template curricula where ExactIdentity always equals Full; independent neighbor panel FAIL 2,016/2,304 (Sep 14). | Active as of `a5655e93`. |
| **GPT-2 dense (#704, T12)** | Certified native lanes for the offline GPT-2 teacher forward. | 0.695× conventional time, bit-exact. | Adopted offline; unrelated to serving. |

---

## 5. METHODOLOGICAL ASSESSMENT

### 5.1 Train/eval leakage and template circularity

- **Same-template construction and evaluation is the norm for T9/T11.** The project says so itself: "authored construction-family development evidence, now opened"; "All development feature rows were seen during training; this is compositional trajectory transfer, not unseen-feature … learning" (`current-state.md:169, 225`); "The same grammatical forms occur in both splits. This is lexical transfer within fixed syntax" (`native_geometric_language_relation_973.md:47`); "The fresh panel changes numbers while retaining the same names and wording; it does not establish lexical or task transfer" (`recovery-2026-09-08.md:69`). Evaluation names are drawn from the same finite lexicon families; when a genuinely novel lexical draw was finally used (Sep 14), 288/2,304 cases failed on a role alias that the authored panels never exercised.
- **Open panels were iterated against.** "Open feedback informed construction design" (`recovery-2026-09-08.md:71`); "That panel was independent at first evaluation; it is now exposed development evidence" (`README.md:195`). The project reserves a "separate independent final draw after design selection" but has never executed one for T9/T11.
- **T1's "held-out" is dominated by exact-context lookup**: EXCT resolves 85.1–100% of positions; Rule 1+2 is argmax-identical to the store on every EXCT row; the M.V.G. judge "measures exact-context memory" (`BASELINE.md:340-347`). The 24.3%/11.94-bit broad number is therefore mostly retrieval of seen 8-token contexts from the same articles' training split (content-hash split *by article*, so no cross-article leakage, but within-distribution n-gram overlap is the mechanism).
- **T6 is the cleanest**: story-level `BLAKE3 mod 100` splits, tokenizer on train only, sealed test opened once, checkpoint selected on dev. Even there, the "fresh-language" validation for the readout ladder is "nonsealed" and the same 512 prompt directions are reused across V1→direct→layerwise→associative→V5 with new floors each time.
- **Teacher-in-the-loop labels**: T1 uses teacher-argmax agreement as a headline metric; the external referee shows the project's own teacher-floor pipeline is 7.33 bits worse than the same model scored externally (`EXTERNAL_REFEREE_300.md`), so early bits/token floors are pipeline artifacts.

### 5.2 Multiple comparisons / garden of forking paths

- **Sheer volume**: `current-state.md` alone carries ~65 dated verdict sections (Sep 4–14); RESEARCH.md ~120 experiments; the T11 chain alone logs ~35 gates in 3 days, several with 3–4 sealed "attempts" before a PASS (relational attention 14→16→42→64; span 3 attempts; relative language 384→768). The project's discipline (frozen acceptance, no-retry, byte-sealed reports) constrains *within-experiment* tuning, but nothing constrains the *choice of the next experiment*, which is always made after seeing the previous result on the same authored data. **[Inference]** With hundreds of bounded experiments and a rule that each PASS is retained and each FAIL is "preserved" and then circumvented by a redesign, the accumulated PASS list is a selected set, and preservation of exact prior outputs (e.g., "663/663 retained answers") is a regression test, not evidence of generalization.
- **Thresholds are frozen but chosen by the same team** and often barely missed or barely met (V5 0.03897 vs 0.04332; layerwise 0.02870 vs 0.04332; #951 4.44% vs 5%; #1057 98.52% vs 99%; #1017 1.5728 vs 1.50; #1043 2.19e-5 vs 2e-5). Missed floors are respected — a real strength — but each miss is followed by a new mechanism with a new floor, so the ladder never accumulates a reject of the overall hypothesis.
- **Effect sizes**: the only "geometric advantage" on real text is +0.95pp from tie-breaking; T9's advantages are on n = 24–96 template items; T6's retained-vs-ordinary Δ is 0.0035 nats. None comes with confidence intervals except the T1/T3 paired ‰ intervals, and the T3 arms with intervals are not geometric.

### 5.3 Are the "preserved outputs" populations meaningful?

- Counts such as "6,688 actual candidate traces", "13,248 old outputs", "19,968 scheduling rows" are **cross-products of a few dozen authored families × query directions × record rotations × control arms** (e.g. 2,304 = 8 shapes × 4 directions × 2 prefixes × 4 rotations × 3 variants × 3 outcomes). The README concedes: "These are retained comparison populations, not a claim of that many independent semantic tasks" (`README.md:190`). **[Inference]** As regression suites they are valuable; as evidence of capability they measure that a deterministic program still produces the same bytes on the same inputs, and their growth inflates the apparent scale of evaluation by ~two orders of magnitude relative to the number of distinct linguistic situations.

### 5.4 Adequacy of controls

Strengths: matched permutation/derangement controls, ReadDisabled/UpdateDisabled, ExactIdentity, order-shuffled, byte-exact replay, independent fresh-process verification, target-free census before label join, seeds drawn from OS entropy after acceptance freeze, "instruments that could not fail" hunted explicitly. This is far above typical research-code hygiene.

Weaknesses:
- **ExactIdentity equals Full on essentially every T11 PASS** (Sep 13–14), i.e. the geometric distance is never load-bearing versus exact identity on the tasks chosen. The project reports this honestly but continues to build on the geometric path.
- **Controls are "within-artifact disables", not matched refits** in T9 ("These are within-artifact feature effects, not a matched geometry-free refit"), so disabling geometry after training on geometry penalizes the ablation unfairly; where matched refits exist (exact-code selector, equality selector), geometry mostly ties.
- **Retained-attention vs ordinary (T6) is equal-parameter but not equal-compute** (2× score pairs, 10× wall), and the decisive scramble arm ("H4 specificity") was deliberately not run.
- **HELM-D-R4 parity** is a change-of-basis identity check; the "live" frame-permutation control only shows the code path is exercised.
- **No non-geometric strong baseline on the target task**: nothing in T9/T11 is compared to a tiny n-gram/finite-state/regex system or a small standard transformer trained on the same 0.5 MB template corpus, which would almost certainly solve these template tasks at 100%.
- Human rubric judgments (4/5, 5/5 "subject or scene") are operator-scored, unblinded, n=5.

### 5.5 What a rigorous external reviewer would demand before accepting any capability claim

1. **One fixed public benchmark, scored once, with the geometric-native artifact and matched baselines**: bits-per-byte on a held-out natural corpus (e.g., a TinyStories or Simple-Wiki test split the team has never opened) for `15baec48`/`60d19679` alongside (a) the T1 table bundle, (b) a Kneser-Ney 5-gram, (c) GPT-2 124M / SmolLM2-135M zero-shot, (d) a standard 7M transformer trained on the same tokens. Today no such number exists for any geometric-native artifact.
2. **Free-running generation evaluated blind**: ≥100 held-out prompts, outputs judged by raters who do not know the arm, alongside n-gram and small-transformer baselines; report distinctness and cycle rates as in #841.
3. **A pre-registered, one-shot independent evaluation** on a task family *designed by someone else*, with the acceptance criterion and analysis code published before data generation, and no subsequent iteration on that panel.
4. **Matched non-geometric strong baselines on the authored tasks**: exact-identity/finite-state/regex and a tiny transformer trained on the identical construction data, to establish whether the template tasks discriminate anything.
5. **The "H4 specificity" scramble arm** for every retained-attention/H4 claim; and equal-compute (not just equal-parameter) accounting.
6. **Complete-request M1 measurement**: wall-clock and energy (external power meter or `powermetrics`) for load + encode + generate N tokens + persist, versus llama.cpp running SmolLM2-135M/GPT-2 124M at a *matched quality level*, since the project's own rule is "a throughput number never travels without its quality number".
7. **Effect sizes with intervals** for every headline delta; a multiplicity policy (e.g., a pre-registered primary endpoint per campaign rather than per bounded gate).
8. **Independent reproduction on a fresh machine**: the repository states that a clone "is source, not the complete local experiment" and that models, sealed traces and receipts live in ignored `.uor-models/`/`.uor-handoff/` paths on the author's laptop (`README.md:234`; every evidence JSON points at `/Users/casey.allard/...`). No result in Tracks T9–T11 can currently be re-executed by an outsider.

### 5.6 Bottom line

The repository is exceptionally candid: it preserves ~90 explicit negative or withdrawn results, and its own summaries ("It is not yet a working geometric language model", `docs/README.md:130-133`; "Does this show a geometric predictive advantage? Not on this panel", `README.md:192`) match the evidence. What the ledger shows is that (i) the only systems that ever produced readable free-running prose are ordinary float transformers with a cosmetic R4 head split or SmolLM2 itself; (ii) the only multiply-free system with real-corpus numbers is a teacher-distilled exact-context lookup table that generalizes at ~1.8% top-1 off exact context and cannot free-run; (iii) every geometric-vs-matched-control comparison on a population larger than ~100 items is null or negative, with the sole exception of a +0.95pp tie-breaking gain; (iv) no energy, latency-per-request, perplexity, or independent evaluation exists for any geometric-native artifact; and (v) the September 8 audit found that the one set of published efficiency and alpha claims rested on hardcoded constants and non-tests. The stated goal — frontier-quality local LM with no transformer and no matmul at inference — has, on this record, not moved measurably closer than a table-lookup n-gram store.
