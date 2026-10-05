# UOR-R4 project history, June to October 2026: eras, decisions, failure patterns and the stuck period

Every claim below comes from repo files, commits, issues or PRs, read-only, at `origin/main` 71b54adf. Labels: **M** = measured at the stated scope, **P** = proof or exact check, **H** = hypothesis, **S** = self-reported or unverified.

---

## 1. Timeline of eras

### E0 — Browser prime-router demo (9–11 June, 36 commits, 1cf03c36 to 5ede5748)
- **What it was:** a WASM `UorR4Router` web app with prime/zeta vectors, Hopf telemetry and corpus retrieval.
- **Who wrote the text:** Ollama, WebLLM Gemma-2B and briefly Gemini did. See commits 4eeb9630, a19d82ac and 78d8febb.
  - `docs/integration/architecture-2026-09/engines.md` §1 confirms that `ollama_generate` "can author final text".
  - The geometry was retrieval and telemetry around a transformer.
- **Why it ended:** commits stop for five weeks. Nothing generated text natively.

### E1 — Compiled transformerless engine and R4G1 graph runtime (18 July to about 22 August)
- **Start:** 399dc11c imports the Codex "transformerless" engine (PRs #6 and #7).
- **What it was:** TLA/R4G1 compiles a teacher's embeddings and n-gram transitions into no_std XOR/popcount/table graphs. It added Gate C, κ pinning and certification. July authors were Casey, Ari and Copilot.
- **Measured results:**
  - Hopf sector occupancy: 7/512 sectors (#305). After the fix, occupancy rose to 456/512, but sector-filtered MRR fell to 0.0045 against 0.0743 (`docs/RESEARCH.md` "What is closed"). (M)
  - E8 store keying was negative (#403). Two-pass generation was refuted: 16.4% against 26.5% (#399). Cayley–Dickson executed 0 of 1,998 times. (M)
  - Retrieval had been comparing routing vectors against content vectors (#486), so its cosine was at chance for months. Fixing the query object raised MRR 0.7179 → 0.8542, and 0.8763 with the lexical weight at zero (#490, #502). (M)
  - The programme's own conclusion: "every lever that added key resolution failed".
  - Roughly 40 PRs (#524 to #588) went to "R5 totality" refactor tranches.
  - S4 planning was LIMITED (12/20 cells). W33 geometry recorded **NO GEOMETRIC ADVANTAGE**, and #846 had 0 certifiable final samples, so reasoning was "NOT ESTABLISHED" (22 August).
- **Why it ended:** compiled count graphs cannot learn context. `RESEARCH.md` "Which track can produce coherent text" pivots to learned attention.

### E2 — Learned R4/Spin softmax attention (25 August to 3 September; #952, #953, #973)
- **R4PredictiveBlockDeltaPromptCapacityV5:** prompt gain 0.03897 against a 0.04332 floor. Geometry was not attributed: own NLL 3.5420 against plain delta 3.5184. Verdict `STOP_WITHOUT_GENERATION`. (M)
- **#1014** (7,155,360 parameters): sealed NLL 2.1274. Removing attention gives 4.8048, a 2.68-nat penalty, so attention is load-bearing. The quality gate failed. (M)
- **#1017** (continuation to 149,995,520 tokens): sealed NLL 1.5728 against a <1.50 gate. It was frozen as the reference. (M)
- **#1019** (13.1M parameters): UNAVAILABLE under budget. GPU work was excluded.
- **#1043:** MQAR 30/87,360. **#1045** (role-tagged Zoology curriculum): 87.1% against 99%. **#1050** reproduced Zoology's released configuration. (M)
- **#1041:** supplied-history binding failed, 0/2.
- **Why it ended:** the attention was ordinary softmax that happened to be R4-framed, the NLL gates failed, and there was no compute.

### E3 — Native geometric learner on authored curricula (4–17 September)
- **Start:** #1125 "Restore native Rust geometric learning".
- **What it was:** about 50 PRs of bounded skills: relation writes, NoRead, Copy/Add, version intent, spans and roles (`RESEARCH.md` lines 12–589).
- **Measured:** neighbour transfer 2,016/2,304. The later 2,304/2,304 (#1281) used authored syntax and predicate rules (takeover review, line table). (M)
- On 8 September, PRs #1179–#1194 landed "durable memory", "Rust coding", "serving guarantees" and "qualify, release" in a single day.
- **Why it ended:** the 19 September takeover found these were "not general prose or an integrated chat model".

### E4 — TinyStories `.rgm`, the takeover, and the reader series (18–24 September)
- **The 555M-token run (#1283):** 1.2372 bits/byte training-time. The deployed scorer measured 1.8055, and greedy output was repetitive. (M)
- **Takeover (Zed and DeepSeek), `takeover-review-2026-09-19.md`:**
  - It found three disjoint model paths.
  - It withdrew the claim that "the entire context is the residue pair".
  - It flagged a 12-bit empty-route floor and double gradient normalization.
  - Decisions D0, D0-b, D1, D2 and D3 followed.
- **Reader series (#1319 to #1333):** H4 gained on constructed tasks but harmed natural text.
  - +1.29 bits/token (#1321), then +0.119 after the fit/serve repair (#1330).
  - #1332 proved an information collision: at most 74 of 145 required answers were reachable. (P)
  - The order-2 count reference reached 3.97 bits/target against the learned 7.14. (M)
- **24 September:** decisions D4–D8 after the external stuck-point review (`stuck-point-review-response-2026-09-24.md`). In A4, every update was correct locally (1,336/1,336), but the loaded model kept only 28/167.

### E5 — D8 ladder, multiple labs, choice of the stack (25–28 September)
- **Admission and serving:** orthant64 admission failed source retention (D9). The full256 integer bridge was delivered (#1397).
- **#1400** (Anti-Gravity) announced "VICTORY CONFIRMED": 80% entity recall, 1.897 ms per token. (S)
- **Serving decisions:** D10 adopted SmolLM2 (26 September). D11 superseded it the next day.
- **Stack chosen as the main line:** it beat its transformer control 1.998113 to 2.011149, one seed each (#1437). (M)
- **First dialogue stack, S2:** best dialogue NLL 2.5209, but only 2/38 replies answered and 0/10 memory recalls. (M)
- **D2 AERM:** the store scored 1.000 in distribution, but the learned read did not generalise. G v1 scored 0.158. G-binding with text-tag masking lifted held-out Updated to 0.967, with the text guard failing at +0.119 nats (#1495). (M)
- **S1 D11 engine:** bit-identical to D10 and passed the ARM64 audit (#1467). (P)
- **QAT Result C:** greedy agreement 5/58 against a gate of 29.

### E6 — Governance and the A1 decisive test (29 September to 1 October)
- **Process:** D13 and D14 created durable labs and a 23-agent council. D15–D17 followed, including a policy-migration deadlock (#1529).
  - Main carried about 110 compile errors because PR checks "had been executing nothing" (#1547).
  - Heavy work was gated for about 13.4 h (`direction-review-2026-09-30.md`).
- **D18 A1 reached outcome D** (1 October). (M)
  - Every arm was below 0.5 at d16, including the transformer at 2× steps, which scored **0**.
  - Pointer arms scored 0.28–0.44, falling as about 1/N, close to the "most recent value" rule's 0.36.
  - The §8 premise also failed: conversations need up to 317 positions, against a 256 window.

### E7 — D19 grounded memory, the scale ladder, and MQAR (1–5 October)
- **External memory path:** the log-plus-prime-sieve memo (#1559) and session recall (#1600) lifted session MQAR from 42 to 79/109. The emit-6r session reached MQAR 108/109 and Responsive 647/752, on an authored world, one seed. (M)
- **Same-day finding (#1580):** the "identity carry" study showed predecessor identity in q and k gives 192/192.
- **8M ladder** (`docs/history/current-state-2026-10-03-overnight.md`): geometric arms beat a one-seed transformer by 0.010–0.05 nats. On 3 October the owner ended transformer controls (#820 comment).
- **CUDA ladder:**
  - Training NLL: 20M at 1.3882, 29M (learning rate 5e-4) at 1.2857.
  - The 232-request panel scored 43 acceptable at 29M float and 46 at about 96M float. D11 GPTQ at 96M scored 43, not significantly different (p 0.71). (M)
- **Network alone:** with the sieve off, 3/109; with it on, 105/109 (#820, 4 October). The chat fine-tunes contained only about 0.8–2% MQAR-format data.
- **MQAR bench (#1698), F2 (#1701), correction (#1704):** see §4.
- **Parallel Codex line (#1680–#1705):** native signed-H4 "bank" reads on a 64-row supplied-record fitter.

---

## 2. Decisions and their stated reasons (`docs/integration/DECISIONS.md`)

| Decision | Date | Content | Stated reason |
|---|---|---|---|
| D0-a | 19 Sep | Serving is multiplier-free and sparse per token | Remove wasted compute |
| D0-b | 19 Sep | Bounded ≤4-bit integer/ternary linear maps allowed; supersedes D0-a | D0-a "excluded every published precedent" (BitNet, MatMul-free, T-MAC) |
| I1–I5 | 19 Sep | Measured efficiency invariants | — |
| D1 | 19 Sep | Falsification before extension | — |
| D2 | 19 Sep | Ablation is a measurement, not a verdict | Owner: small misses must not retire mechanisms; adds mechanism classes and decision-flip rate |
| D3 | 19 Sep | Takeover ratification; offline matmul OK | — |
| D4 | 24 Sep | Goal R (geometric advantage) becomes a gated hypothesis | "No geometric mechanism has beaten a matched ordinary control on any tested task" |
| D5 | 24 Sep | Per-token parameter sparsity is the terminal invariant | Dense 4-bit streaming "is a GEMM in disguise" |
| D6 | 24 Sep | Target becomes a long-range probe; order-2 counts are a control | Proxy objective |
| D7 | 24 Sep | Integrated attention-language programme | — |
| D8 | 24 Sep | Reference → joint learner → discretize ladder; stop selector tuning | Stuck-point review: no language-to-context credit path |
| D9 | 25 Sep | Anti-loop work cards; recent64 withdrawn | Owner challenge after the orthant64 repeat |
| D10 | 26 Sep | SmolLM2 backbone for chat | — |
| D11 | 27 Sep | Native, multiplier-free, transformerless serving for every lab; supersedes D10 | Owner: "Keep the native, multiplier-free serving target" |
| D12 | 28 Sep | Gates promote, never kill; toolbox | Near misses were becoming family eliminations (B1 at +0.069 nats; D2 margin; confounded index contest) |
| D13 | 29 Sep | Two-track plan | — |
| D14 | 29 Sep | Durable labs and council | — |
| D15 | 30 Sep | Converted students may serve after an audit | — |
| D16 | 30 Sep | Council parity rule | — |
| D17 | 30 Sep | Conversion must remove the transformer architecture; parity v2 | — |
| D18 | 30 Sep | One retrieval question for two weeks | Every memory score 0/10; relation recall 0/64 |
| D18 outcome | 1 Oct | Outcome D | — |
| D19 | 1 Oct | Grounded conversation and durable memory first; admissibility policy | Owner priority |

Undated owner rulings: 3 October, no transformer baselines (#820); 30 September, self-review allowed (D18 §9).

---

## 3. Recurring process failures

**a. Instrument defects found after verdicts were issued.**
- #486: the dead cosine made #480 and #484 report false "inert" verdicts for months.
- #1301: permutation and bootstrap defects.
- #1328: fit-time and serve-time thresholds differed, leaving 24/32 gap-band entries untrained.
- #1043: a 2.19e-5 logit drift against a 2e-5 bound left the result uninterpretable.
- #846: zero certifiable samples.
- B3's 9.45 nats/token came from #1017 token IDs fed to a 49k-token model (D16 §4).
- Panels with identical turns but different expected answers (#820, 3 October).
- CI "executing nothing" while main carried about 110 errors.
- D18's 256-token premise (the real requirement was 317).
- A1's transformer control scored 0 at d16. That makes the instrument and budget suspect, yet the result was read as "no learned retrieval at this scale".

**b. Over-reading results.**
- #1400 "VICTORY CONFIRMED", when S2 three days later scored 2/38.
- 1.2372 BPB quoted when the deployed scorer measured 1.8055.
- 617 tok/s against Qwen at unequal quality.
- The 1 October "capacity ceiling" claim, partially retracted.
- #1698's "recurrence crowds out reads", corrected within hours in #1704 as a seed-1 artifact.
- One-seed geometric-versus-transformer margins.

**c. Micro-fixture loops.**
- About 50 authored-curriculum PRs (E3).
- The reader series, #1319–#1333.
- A4 local fits that did not survive loading.
- About 25 "Learn …"/"Retain …" PRs on 4 October against a 64-row supplied-record fitter (`current-state.md` lines 3–194). Codex's own note says this "does not measure finding a record across the whole conversation".
- D9 exists because of this pattern.

**d. Kill/promote oscillation and parity-rule churn.**
- D1 (falsify) → D2 (don't retire) → D4 (gated hypothesis) → D12 (never kill).
- The parity rule was rewritten in D13, D16 and D17.

**e. Governance consuming research time.** Councils, coordinators, the leases deadlock (#1529), and 13.4 h of gating with no decisive number (`direction-review-2026-09-30.md`).

**f. Fragmentation and rediscovery.**
- Three model paths coexisted (takeover review).
- Predecessor identity was found by Codex on 1 October in #1580 ("identity carry", `geometric-attention-binding-2026-10-01.md`) and found again by Claude on 4 October in #1701.

**g. Baseline whiplash.** Transformers moved between reference (#1017), served backbone (D10), banned (D11), decisive control (A1 T) and removed entirely (3 October).

---

## 4. The stuck period: geometric attention and in-context retrieval (late August to 5 October)

| Attempt | Measured result (scope) |
|---|---|
| #952 A1 / #967 / #970 recursive H4 state | Order retained, but scalar Cayley readout tied 6/6; heatmap readout not identifiable |
| V5 predictive block-delta | Below the capacity floor; geometry unattributed |
| #1014 / #1017 R4-framed softmax | Attention load-bearing (2.68 nats); NLL gate failed |
| #1043 position K/V | MQAR 30/87,360 |
| #1045 Zoology curriculum | 87.1% against 99% |
| Reader series #1319–#1333 | Constructed gains (e.g. 49→103/119 payload reads); natural text worse by +0.12 to +1.29 bits/token |
| A4 (24 Sep) | 1,336/1,336 local, 28/167 after loading |
| Orthant64 admission (25 Sep) | Source retention failed |
| #1438 finite read kernel; T2 | Confounded by magnitude (external Codex audit) |
| D2 AERM | Store 1.000; learned read failed; G v1 0.158 against a dense control's 0.47–0.81 |
| G-binding masking | 0.967 held-out, text guard failed |
| A1 (1 Oct) | All arms below 0.5 at d16; pointers ≈ recency |
| #1580 predecessor carry (1 Oct) | 192/192 against 0–3/48 jointly correct pairs; authored one-token-key grammar, width 32 |
| #1642 distance curriculum | 3/120 against 14/120; negative |
| Delta-rule prototype (#1648) | Six negative logs; not qualified |
| Chat models 8M→96M | Sieve off 3/109; panel flat at 43–46/232 |
| #1698 MQAR bench (1.36M parameters, context 512) | All-read l2: 0.807 / 0.176 / 0.010 / 0.029 at d16 / d64 / d200 / d400; dot gives the same; zeroing age drops d16 to 0.115; recency baseline 0.445 at d16 |
| **#1701 F2: `k_t + j·k_{t−1}`** | 1.000 in every bucket by step 400; held-out class 1024/1024 against 0/1024 |
| #1701 probe | Without F2, no head puts more than uniform weight on the key; with F2, value mass is 0.99–1.00 |
| F1 age-slope spread | Hurt: 0.398 at d16 |
| #1704 correction | `rrarra` without the shift solves at seed 2 (0.9995); `rararr` (14-layer stand-in) solves without the shift; all-read fails on both seeds |

**Synthesis.**
- Every in-network success shares one feature: the query is given a channel for "the predecessor of this position" (#1580, #1701), or admission is exact and external (sieve, D2 store). (M)
- This matches the induction-head and short-conv/token-shift literature (H):
  - Olsson et al., arXiv 2209.11895;
  - H3's shift SSM, arXiv 2212.14052;
  - RWKV token shift, arXiv 2305.13048;
  - Zoology, arXiv 2312.04927;
  - Based, arXiv 2402.18668.
- The `j` rotation is the project's fixed, parameter-free, D11-friendly form of that idea. It is not shown to be the unique or geometric cause.
- **Reasons not to over-read F2 as the missing piece:**
  1. Production-like patterns solve the synthetic task without it on some seeds (#1704).
  2. The chat models saw only about 0.8–2% MQAR-format data, so the 3/109 may be a data deficit (#820, 4–5 October).
  3. F2 has no served form yet. Every D11 path refuses it (#1704).
  4. Neither the D19 grounded cell nor any chat model with F2 has been measured.
  5. The open-panel failures are mostly incoherent open-domain content (#820, 4 October). That is a capability and scale limit that retrieval alone does not address. (H)

**Decision-relevant next measurement, reinforced by every pattern above:** a chat model trained with the key shift on a mix with substantial MQAR, measured on the sieve-off D19 cell with ≥2 seeds and a no-shift paired arm. Report the open-panel score separately.

Key files:
- /Users/casey.allard/uor-r4/.worktrees/claude-d11-chat/docs/integration/DECISIONS.md
- /Users/casey.allard/uor-r4/.worktrees/claude-d11-chat/docs/RESEARCH.md
- /Users/casey.allard/uor-r4/.worktrees/claude-d11-chat/docs/integration/takeover-review-2026-09-19.md
- /Users/casey.allard/uor-r4/.worktrees/claude-d11-chat/docs/integration/direction-review-2026-09-30.md
- /Users/casey.allard/uor-r4/.worktrees/claude-d11-chat/docs/history/current-state-2026-09-25-to-2026-10-02.md
- /Users/casey.allard/uor-r4/.worktrees/claude-d11-chat/docs/history/current-state-2026-10-03-overnight.md
- /Users/casey.allard/uor-r4/.worktrees/claude-d11-chat/docs/integration/geometric-attention-binding-2026-10-01.md
- /Users/casey.allard/uor-r4/.worktrees/claude-d11-chat/docs/integration/architecture-2026-09/engines.md