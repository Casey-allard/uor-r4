# UOR-R4 programme and process audit: course diagnosis

Auditor: programme/process agent · 2026-09-25 · repo HEAD `413a32f` (#1397) · read-only.
Labels: `[SOURCE file:line]`, `[MEASURED]` (command run this session; scratch data in `exp/audit/`), `[LITERATURE url]`, `[DERIVED]`, `[HYPOTHESIS]`.
GitHub numbers come from the public REST API through the proxy and from the GitHub MCP tools (`exp/audit/prs_all.json`, `exp/audit/prfiles/`).

---

## 1. Executive verdict

- **Verdict: the project is partly off course.** The engineering is back on track. The geometric thesis has drifted out of the model. Since D8 (Sep 24) the project has the skeleton it lacked for months: a real Rust autodiff trainer, a matched ordinary control, 4-bit learned rounding, and a standalone integer session whose output matches exactly [SOURCE docs/integration/current-state.md:29-47,229-251]. The only geometry left in the accepted model is a per-4-lane unit-quaternion rotation. It sits downstream of a dense matmul, and it is 0.025 nats *worse* than its Householder control [SOURCE crates/uor-r4-training/src/joint_model.rs:949-972; joint-recurrent-result-2026-09-25.md:104].
- **The original idea has never had a valid end-to-end test.** The router era's only language-model test (INC-0171) found that Hopf routing behaved the same as a relabeled copy of the same partitions (164.54 vs 164.41 PPL). Its "learned gate" baseline also received no gradient [SOURCE research/ai-research/ai-router/router-research/docs/research/increments/INC_0171_lm_integration.md:90-93,130-141]. The router era also found "no radial contribution" to routing [SOURCE …/KILL_LIST_TRACKER.md:226-234]. That is the radial part the owner says distinguishes the idea from TurboQuant. In uor-r4, D4 records that no geometric mechanism has beaten a matched ordinary control [SOURCE DECISIONS.md:228-229]. D5's well-posed routing contest "has never been run" [SOURCE DECISIONS.md:249-253], and D9 deferred it again.
- **Language quality is worse than on Aug 31.** On Aug 31, #1017 (a 7.16M-parameter R4/Spin transformer trained on 150M tokens) reached NLL 1.5728 with 5/5 subject-or-scene retention [SOURCE GitHub issue #1019 body]. Its outputs still repeated and drifted [SOURCE current-state.md:433-435]. Today's accepted model scores 2.085–2.110. About 3.5 weeks (Aug 31 to Sep 24) went into discrete mechanisms tested on authored fixtures, and those are now parked [SOURCE repo-review-direction-2026-09-23.md:151-156; project-track.md:7].
- **Process cost dominates the work.** In 7 days the project recorded 11 owner decisions and changed direction 3 times within 14 h on Sep 24. It merged 107 PRs; 45% of their titles say correct/repair/fix/recover/reconcile/restore. It wrote about 185 plan, step, review and result documents. In the D8 window, 8 of 13 cycles spent ≤10% of wall time on model computation (median about 6%) [MEASURED].
- **The serving contract optimizes a proxy.** The rule is "no multiplier instruction", and it is met by emulating multiplication in software. The integer step is about 6.5× slower than F32 [SOURCE integer-serving-result-2026-09-25.md:82-83]. On x86, the project's software multiply is 82× slower than a native multiply [MEASURED]. Energy has never been measured.
- **The model is too small and trains too slowly.** It has 1,678,466 parameters, 62% of them the embedding [DERIVED]. It trains at 1,362 tokens/s per arm, which is 11–22× lower parameter-token throughput than the August PyTorch/MPS transformer run (a different architecture) [DERIVED].
- **The next work card is safe but tells us little.** It adds 30M more tokens to the same model. Extrapolation gives about −0.1 nats (to roughly 1.96–2.00), still about 0.4 nats short of #1017, for roughly 6 h per arm plus overhead [DERIVED].
- **The owner must resolve a goal ambiguity.** The owner says "no MoE / sparse-routing". D5 makes per-token parameter sparsity the terminal requirement, and that sparsity is a form of routing. The only end-to-end router result in the project's history is a replacement for an MoE gate.

## 2. Timeline of direction changes

| Date (2026) | Event | Source |
|---|---|---|
| Mar 12–26 | **Router era.** Goal: "fixed geometry … replace dense … routing", 10–100× savings; object asymmetric hyperbolic H⁴×H⁴; 7-stage kill list. INC-0168: routing "purely angular, no radial contribution". INC-0171: Hopf = relabeled routing, broken baseline; still declared "Stage 7 COMPLETE… publication-ready". | CORE_PROJECT_GOALS.md:11-21,29; KILL_LIST_TRACKER.md:226-248 |
| Jun 10 | uor-r4 created ("R4 prime router", #1) | GitHub |
| Jul 19–Aug 16 | "Transformerless" means a teacher-compiled R4G1 graph. Work covers VSA and proof-carrying addresses (#9), a hologram compiler (#10), Cayley–Dickson/Cl(0,8) (#112–115), and about 185 graph/compiler/certify PRs. | PR titles [MEASURED] |
| Aug 19–22 | Completion plan with stages S0–S7 (#860). S1/S2 REVISE, S3 LIMIT, and free-running generation gap "total" (#894). W(3,3): **NO GEOMETRIC ADVANTAGE** (#929). | PR bodies |
| Aug 26–31 | Pivot to a geometric causal decoder (#956). #1016: 7.16M R4/Spin LM, 30M tokens in 70 min on MPS. #1017: 150M tokens, NLL 1.5728. #1019 (13M) paused on hardware budget. | #1016, #1019 |
| Sep 1–7 | Native discrete mechanisms on authored fixtures. On Sep 4, governance went from "source-only" (#1109) to "build-first" (#1111) within 1 h. On Sep 7, CI became echo acknowledgements (#1163). | PR bodies |
| Sep 8 | PRs #1177–#1197 add unsupported "qualifications", including a 12-axis "100% alpha" scorecard, a hardcoded 3,500 mW "energy" figure and a security audit that returned literal `true`. v0.1.0-alpha was published, then withdrawn by recovery PR #1198. | recovery-2026-09-08.md:13-27 |
| Sep 8–17 | Operators, owner/version memory, lexical emission, typed attention and role repair, all on authored panels | repo-review §3.4 |
| Sep 18–19 | TinyStories `.rgm` (JEPA, H4 lattice, VSA, engrams; #1283) and its ablation (#1284). D0-a is signed and superseded by D0-b the same day; D1–D3 follow; takeover by Zed/DeepSeek. | #1284; DECISIONS.md:8-216 |
| Sep 19–23 | About 40 DeepSeek-step micro-experiments: readers, priors, utility, emission, roles, lexical. The best served artifact has **no geometry** (64-dim integer RNN). | direction-decision:15; grep [MEASURED] |
| Sep 23 | Kimi team (#1357). Whole-repository review by 8 reviewers (#1371). | PRs |
| Sep 24 | D4–D6 at 07:55 (#1377), D7 A1–A4 at 18:15 (#1386), D8 at 21:41 (#1387): A1–A4 parked, #1017 restored as reference, Candle crate created | PRs; DECISIONS.md:218-310 |
| Sep 25 | Rungs 1–4 (#1389–#1397). Two discretizations rejected, one accepted. Bounded admission fails. D9 (#1395). Standalone integer serving. | current-state.md |

**Name collision.** The router-era object was *hyperbolic* H⁴×H⁴ [SOURCE CORE_PROJECT_GOALS.md:29]. The uor-r4 "H4" is the *spherical* Coxeter group of the 600-cell. The project's own review calls conflating them "a category error" [SOURCE hyperbolic-hierarchy-and-compute-review-2026-09-19.md:15]. The substitution happened without a recorded decision [HYPOTHESIS: no decision record found by grep].

## 3. Quantities

**3a. docs/integration** [MEASURED `find`/`wc`]

The directory holds 307 files (7.2 MB); docs/evidence holds another 366 files (35 MB).

| File type | Count |
|---|---|
| `*-result` | 61 |
| `*-review` | 49 |
| `deepseek-*-step` | 40 |
| `*-plan` | 26 |
| Audit, handoff and output files | 15 |
| Other | 116 |

228 of the 307 files (74%) are dated Sep 19–25. The busiest days were Sep 21 (50) and Sep 24 (61).

Between Sep 19 and Sep 25 the project wrote about 296k words in these documents:

| Document class | Words |
|---|---|
| DeepSeek step documents | 99.8k |
| Reviews | 75.7k |
| Results | 82.6k |
| Plans | 25.5k |
| Designs | 12.5k |

Mandatory agent reading comes to about 36k words.

**3b. PR cadence** [MEASURED GitHub API]

| Measure | Value |
|---|---|
| PRs total / merged, all time | 974 / 951 |
| Merged Aug 26–Sep 25 | 369 (12.3/day, peak 31 on Sep 13) |
| Hours of the day with merges | all 24 UTC hours |
| Merged PRs touching no Rust (last 30 days) | 108 of 369 (29%) |
| Merged Sep 19–25 | 107 |
| Median gap between merges (Sep 19–25) | 43 min |
| Median open-to-merge time | **1.8 min**; 66% merged in ≤5 min |
| Reviews on #1389 | 0 |
| Duration of the "fmt / clippy / tests / no_std / κ" check | 3 s (#1397) |

**3c. Composition of the last 15 merged PRs** (#1377–#1397; 564,278 changed lines) [MEASURED]

| Category | Share of changed lines |
|---|---|
| Evidence JSON | 79.9% |
| Vendored Candle | 6.8% |
| First-party Rust | 8.2% |
| Prose Markdown | 2.9% |
| Output dumps | 1.9% |

Prose runs at 0.35 lines per line of first-party Rust.

The first-party Rust splits into:
- 24,315 lines in the active crates;
- 13,297 lines for A1–A4 and bridges, written and parked on Sep 24;
- 8,598 lines elsewhere in core.

Over the last 30 days, 1.68M evidence-JSON lines, 491k Rust lines and 167k Markdown lines changed. **By line count, prose is not the main cost. Evidence bloat and discarded code churn are.**

**3d. Code footprint** [MEASURED]

- Rust in `crates/`: 666,682 lines. Whole repository: 1,169,112 lines.
- Active path: 21,841 lines (3.3% of `crates/`): training 17,672, integer 3,310, tokenizer 859. These crates were created on Sep 24–25.
- The active path imports only three things from uor-r4-core (409,897 lines):
  - `report_output`, a 3-line re-export;
  - `HfBpeTokenizer`, 450 lines;
  - `answer_oracle`, 121 lines.
- `native_geometric` (186,440 lines; 44 learner modules) has **zero** references from the active crates; uor-r4-core also carries 42 experiment binaries (71k lines). AGENTS.md:122 still calls `native_geometric` "the current model implementation".

**3e. Model compute versus orchestration**

The shared ledger went from 146,438,565 / 154,400,000 ms (Sep 19) to 545,784,793 / 549,600,000 ms (Sep 25). That is 110.9 h charged in 143 calendar hours. The limit rose 109.8 h in step with use, so it was never a binding constraint [SOURCE takeover-review-2026-09-19.md "Resources"; evidence/integer-serving-closeout-2026-09-25.json].

The D8 closeouts (13 cycles, Sep 24 17:00 to Sep 25 20:15) charged 21.4 h, of which 7.6–9.7 h (36–45%) was model computation [DERIVED from the closeout JSONs; `exp/audit/d8_cycles.csv`].

| Cycle | Share of wall spent on model computation |
|---|---|
| Rung 1 | 59–74% |
| Rounding | 49% |
| Projected | 44% |
| Quantized | 40–69% |
| Bounded | 34% |
| The other 8 cycles | ≤10% each (A1 below 1%) |

Records before D8 do not separate model time from orchestration; D8 introduced that rule.

## 4. Drift from geometry: mechanisms in the accepted D8 model

A grep of `crates/uor-r4-training/src` and `crates/uor-r4-integer/src` finds zero occurrences of prime, zeta, hopf, hyperbolic, poincare, icosian, E8, VSA, lattice or spin [MEASURED]. "H4" and "Z[phi]" appear only in contract strings that disclaim them: "not exact Z[phi]" and "no hemisphere folding or H4 codebook" [SOURCE joint_model.rs:1642,1652].

| Mechanism | In the forward path? | Best measured contribution (scope) | Status |
|---|---|---|---|
| Hyperbolic H⁴ routing and shell law | No | Router INC-0168 found no radial contribution; shells were excluded | Abandoned in March; the name was reused for spherical H4 |
| Hopf fibration (sectors, S2 readout) | No | Hopf routing equalled relabeled routing (INC-0171, invalid baseline). `.rgm` S2-readout ablation cost +0.338 BPB, with no ordinary control (#1284) | Unproven |
| Prime addresses / ordered n-lets | No | Exact identity and determinism only; "`%120`/prime label is not semantic distance" [repo-review:126] | Infrastructure only |
| Zeta phases | No | No measured contribution | Unused |
| 600-cell / 2I / H4 table | No | `.rgm` coarse H4 tier −0.0066 BPB (**harmful**, 64% of bytes). Ordinary table 288/288 vs H4 186/288 (#1339). A4: 2I arm 6.799 bits vs control 6.873, C120 arm 6.839 vs 6.800; 0/12 answers. Rung-0.5 codebook test NOT RUN [repo-review:117; current-state.md:397-403] | No advantage |
| Exact Z[φ] arithmetic | No | None | Unused |
| Icosian E8 | No | None; NOT_RUN [repo-review:120] | Hypothesis |
| VSA | No | −0.0007 BPB (inert; disconnected codebook and a role-binding bug) (#1284) | Retired |
| Exact addressed memory / versions | Partly: an exact token tape feeds pointer-copy | #1346 16/16 recall; "geometric contribution none in that serving path" [repo-review:115]. Disabling read plus copy costs +0.48 nats (not geometric) | Infrastructure; durable memory not integrated |
| **Quaternion transport** | **Yes (the only geometry)** | Quaternion 2.110368 vs Householder 2.085241. After rounding, 2.114 vs 2.111. Source panel 28/32 vs 24/32. One seed | No advantage |
| D5 sparse access / geometric routing | No (dense, full admission) | Orthant64 retained 17/32 in each arm, a failure | Deferred (D9) |

**The test is ill-posed.** The quaternion's inputs are `fused.narrow(1,2d,d)`, a slice of a dense 3d×d matmul. The geometry therefore consumes a matmul and cannot replace one [SOURCE joint_model.rs:949-966].

## 5. Findings

**Governance**

1. **The tightening had a real cause.** The Sep 8 fabrications justified stronger controls: sealed report roots, NOT_RUN labels and frozen gates [SOURCE recovery-2026-09-08.md:13-27]. The router era showed the same over-claiming pattern: "publication-ready" on 2 seeds with a broken baseline [SOURCE KILL_LIST_TRACKER.md:247-248; INC_0171:130-141].
2. **Governance lurches between extremes.** It went from a "certification gauntlet" (#941) to source-only (#1109), to build-first in the same hour (#1111), to echo-only CI (#1163), to fabrication, and then to heavy sealing. After that came D0–D3, D4–D8 and D9 [SOURCE PR bodies].
3. **Decisions churn.** There were 11 recorded decisions in 7 days [SOURCE DECISIONS.md]. D6's long-range probe was run once, as a standalone synthetic panel (KVAR, lag ≤64). It was never applied to the D8 model [SOURCE project-track.md:111-124]. The live work card gates on a 32-prompt, 56–62-token authored panel [SOURCE current-state.md:473].
4. **The ceremony gives little independent assurance.**
   - CI only echoes [SOURCE .github/workflows/ci.yml:107-113], and PRs self-merge in about 2 minutes [MEASURED].
   - The claim-wording check only greps a handful of phrases [SOURCE scripts/check_claim_wording.py:22-27].
   - The budget limit is raised every cycle [SOURCE AGENTS.md:161].
   - Real assurance comes from local tests and principal reviews, and those did catch many defects: the VSA bug, double gradient division, bits vs nats, the 64/256 context mismatch, and the wrong F32 backend.
5. **Tiny experiments carry large documents.** The "contextual utility" experiment fitted a 16-bucket integer table on 2,969 positions [SOURCE contextual-utility-result-2026-09-20.md:27,37-46]. It produced 55 KB of design, step, result and review documents [MEASURED].
6. **Onboarding costs and agent churn are high.**
   - Mandatory reading is about 36k words; issue #973's body is 80,867 characters with 295 comments [MEASURED].
   - Leadership passed between Codex, Antigravity, Zed/DeepSeek, the "Sisyphus" run lead, Kimi and RDC/DeepSeek, with a large takeover document at each handover [SOURCE takeover-review:14; research-leader-handoff-2026-09-23.md].

**Methodology**

7. **The D8 ladder has the right shape.** It adopted the stuck-point review's valid criticism: build a learning graph before selectors [SOURCE stuck-point-review-response-2026-09-24.md:12-19].
8. **Every D8 comparison uses one paired seed.** The plan itself requires 3 seeds for promotion [SOURCE project-track.md:29].
9. **The authored panel is weak as a gate.** It has 4 templates and 32 prompts. The 95% Wilson intervals for 28/32 and 24/32 are [0.72, 0.95] and [0.58, 0.87], which overlap [DERIVED]. It works as a smoke test, but it should not gate architecture.
10. **The model is capacity- and data-limited.** 30M tokens on 1.68M parameters is about 18 tokens per parameter, close to Chinchilla's compute-optimal ratio of about 20 [LITERATURE https://arxiv.org/abs/2203.15556]. More tokens alone therefore gives diminishing returns. TinyStories models under 10M parameters generate coherent stories when adequately trained [LITERATURE https://arxiv.org/abs/2305.07759].
11. **The recurrent learner already matches the August transformer at equal tokens.** At about 30M tokens, #1016 scored 2.131 and the recurrent learner 2.085–2.110 (on different dev partitions) [SOURCE #1016 body]. The gap to #1017 is scale and throughput, not architecture [HYPOTHESIS].
12. **The next work card is low-value per hour.** The log-linear slope is about 0.15–0.16 nats per e-fold of tokens (step 7,324 to 8,348), so +30M tokens gives about −0.10 nats. Reaching 1.574 would take about 0.7–1.0B tokens, or 150–210 h per arm at the current rate [DERIVED; optimistic because it ignores saturation]. The card does include a good stop rule [SOURCE current-state.md:216].
13. **The integer serving path is slow and unmeasured for energy.**
   - The integer step is still about 6.5× the F32 step [SOURCE integer-serving-result:82-83].
   - Activation×activation products (attention, transport, norms) go through a shift-add loop on i128 [SOURCE crates/uor-r4-integer/src/math.rs:59-84; model.rs:327,338,488].
   - That loop runs 80.7 ns/product vs 0.98 ns native, 82× slower (x86 Xeon, `exp/audit/mulbench`) [MEASURED].
   - At 45 nm, a CPU instruction costs about 70 pJ of overhead against about 3 pJ for a 32-bit integer multiply, and a DRAM access costs 1.3–2.6 nJ [LITERATURE https://pages.cs.wisc.edu/~markhill/restricted/isscc2014_horowitz_power_scaling.pdf ; https://pdfs.semanticscholar.org/9476/20a1854655ed91a86b90d12695e05be85983.pdf]. Replacing one multiply with many instructions very likely raises energy per token [DERIVED].

**Root causes, stated plainly**

- **(a) The goal contract optimizes proxies.** It targets the absence of an opcode instead of joules and latency at matched quality, and the owner's constraints conflict (D5 sparsity vs "no sparse routing").
- **(b) Geometry has never had a falsifiable role tied to the goal.** It is kept "primary" by policy [SOURCE AGENTS.md:32; agent-execution-policy.md:31] while every matched test is null.
- **(c) The model is far too small and training is slow.** The recurrence is strictly sequential, training is Rust/Candle on CPU only, and Metal was slower [SOURCE current-state.md:512-514].
- **(d) About 15 mechanism families were tried one after another, each shallowly and on authored fixtures.**
- **(e) Autonomous agents work around the clock, faster than one person can steer.** Owner corrections (the 64/256 mismatch, D9) arrived after the fact.

## 6. Strengths and weaknesses

**Strengths**

- Research honesty is exceptional. The README says the best artifact "contains no geometry" [SOURCE README.md:37].
- Negative results are preserved.
- Parity is exact at hash level from training to integer serving.
- Matched ordinary controls are used.
- The team's own diagnostic reviews (Sep 19, Sep 23, Sep 24) are sharp and mostly right.
- The D8 pipeline is a reusable asset.

**Weaknesses**

- Decision and document churn, and 1.68M lines of evidence JSON in git.
- 97% of the Rust is off the active path, while AGENTS.md points at dead code.
- The serving rule emulates the multiplier it bans.
- No energy measurement exists.
- No test of geometry has been well-posed.
- The owner's actual goal (joules and quality on an M1) is not the metric anyone measures.

## 7. Recommendations (ranked)

1. **Owner writes a one-page "serving contract v2" (about 1 day; highest impact).**
   - Define success as measured J/token, latency and RAM on the M1 at matched quality against a llama.cpp or bitnet.cpp baseline.
   - Keep integer serving and ≤4-bit weights. Allow hardware integer multiply unless measurement shows that avoiding it saves energy.
   - Rule explicitly on deterministic geometric routing versus "no sparse routing", and on D5.
   - *Falsifier:* measure J/token with powermetrics for the F32 and integer sessions on identical outputs. If the integer path costs more, the no-multiplier rule is counterproductive.
2. **Fix training throughput before scaling (2–5 days).** Target at least 5–10× tokens/s using chunkwise-parallel recurrence (the DeltaNet family the takeover review already cites), larger batches and fused kernels. *Falsifier:* if the gain is under 3× after 3 days, use a parallel-in-time baseline.
3. **Map the ordinary-baseline scaling frontier on the M1.** Run about 2M, 7M and 20M parameters at about 20 tokens per parameter, with 3 seeds on the smallest size. Include one arm distilled from #1017, since D8 allows an offline teacher. This sets the quality floor any geometry must match. Deprioritize the current +30M-token card until throughput is fixed.
4. **Make one pre-registered geometry bet with a kill rule (2–4 days).** Options are the D5 contest (geometric routers against LSH, k-means or product-quantization routers at matched bytes touched, scored on recall@K and NLL, with 3 seeds) or rung 0.5 (a 600-cell codebook against k-means-120 and random-120). *Kill rule:* if geometry does no better than the best ordinary method across 3 seeds, retire it from the serving critical path and keep it as identity and addressing infrastructure.
5. **Cut:**
   - the per-experiment set of step, design, review, result, closeout and budget documents (use one card of at most one page per experiment);
   - evidence JSON in git (move it to artifact storage with hashes);
   - per-cycle ledgers with rising limits (use a fixed weekly budget and one log line per run: wall time, model time, tokens, tokens/s);
   - echo CI (run real fmt, clippy and unit tests on the 22k active lines);
   - decisions outside a weekly review, except when a pre-registered kill fires;
   - multi-model review of every step (review only promotions and the weekly direction).

   Move inactive crates to a legacy area, rewrite AGENTS.md:122, and cut mandatory reading to 5k words or fewer.
6. **Keep:**
   - frozen criteria before each run;
   - matched controls;
   - parity hashes;
   - sealed artifacts for *promoted* models;
   - the D9 work card as the *whole* process, not an extra layer on top of it.

   Add 3 seeds and bootstrap confidence intervals, a fresh held-out split, and a coherence score for generations computed with #1017 as a local scorer. Evaluate the D6 long-range probe on the actual model.

## 8. Open questions

- What fraction of time went to model computation before Sep 24? It was not recorded.
- What did the DeepSeek and Kimi reviews cost? The ledger records wall time, not money.
- How much of the quaternion-vs-Householder gap is seed variance?
- What is the J/token of the integer session vs F32 on the M1?
- Was #1017's R4/Spin transport ever ablated against a matched ordinary transformer? I found no such record.
- What exactly does the owner mean by "no sparse-routing"? The answer decides whether D5, and the router-era line of work, is in scope.
