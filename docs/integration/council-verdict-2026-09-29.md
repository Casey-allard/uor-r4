# The 29 September council: verdict, evidence and recommended experiments

*Council held 29 September 2026; reconciled on 30 September with the recovery hold (#1520) and the durable-lab charter (D14). Drafted by the Claude lab.*

**How to read this.**
- This record is **historical evidence and recommendations**, not an execution authorization. The [integration queue](../labs/integration-queue.md), current lab plan and atomic coordination state own live dependencies and claims under D14/D17; lab boards are communication views.
- The working rules it proposes are [D16](DECISIONS.md#d16--working-rules-from-the-29-september-council-council-authority-under-d14), which take effect by council vote.
- The owner decisions it prompted are [D15](DECISIONS.md#d15--converted-students-may-become-served-candidates-after-a-d11-audit-runtime-and-energy-claims-are-measured).

- **Sources:**
  - five evidence briefs, four proposals and twelve red-team verdicts;
  - a judge and a completeness critic;
  - lab boards #1511–#1515, #1520, and the sealed roots.
- **Wording:** "derived" means arithmetic, not measurement, and *self-reported* means not yet re-run by a non-author.

## 1. Honest answers

- **Chat: not achieved.**
  - R1 on development phrasings: Responsive 0.52, Instruction 0.27, Relation 0.01. Development relation queries: 0/64.
  - Only the panel's memory score is stored (0/10). "About 7/38" has no stored source.
- **Geometric attention: implemented; no advantage in the main line.**
  - In the 7M stack no geometric read has beaten a matched ordinary control.
  - Lorentz against Dot changes sign with configuration (−0.076 to +0.046 nats), and one arm's seed spread (0.040) exceeds the 0.02 tolerance, so the stack comparison is unresolved.
  - At smaller scope the native model's Lorentz read beat Dot at width 128 in 3 of 4 seeds (by 0.018 nats at context 128, 0.051–0.061 at 256), and a learned Lorentz cache beat equal dot and Euclidean caches.
  - Under D16's frozen-S4 linear-readout configuration, the held-out relation-transfer measurement is what lost (2I 0.327 against softmax 0.491), while the joint key remains readable in distribution (2I in-distribution 0.884/0.906 against softmax 0.9799/0.9792). The in-distribution guard failed and the arms ran unequal update counts (softmax early-stopped at 4 251/24 001 against 2I's 30 000), so this is a scoped linear-readout result — not a verdict that 2I codes as addresses are lost.
- **Runtime saving: not established; the one same-artifact figure points the other way.**
  - On the S2 model, D11 takes about 5.40 ms/token and D10 NEON about 1.11 (*self-reported*, not re-run).
  - At 7M parameters or fewer the engine is instruction-bound (derived), so byte savings do not become latency.
  - No valid stack J/token exists. The proposals' byte and operation tables are withdrawn as claims.
- **What works, at its scope:**
  - exact identity keys (KVAR 0.83 in one seed, 0.745 in the other; D2 1.000 in distribution; sieve 273/273);
  - trained-in 2I transport (+0.0106 nats from free transport, one seed, inside the band);
  - QAT on `geometric_s1` (+0.018, *self-reported*).
- **Track B: no result yet.**
  - Candle parity is NOT_RUN (it stopped after 25 of 45 oracle rows).
  - The B3 root's instrument is disputed (D16).

## 2. Red-team tally

Four proposals were each attacked through three lenses: generalization, serving cost, and endless loop. **All 12 verdicts refuted.** They refuted specifications, not directions, and this plan grafts their salvage.

## 3. Direction

1. **Track A is the main line toward §8:** retrieval first, then data. R1-X decides which of the two limits Relation.
2. **Track B is a fixed-teacher measurement instrument and teacher source.** Each pillar is measured training-free as the share of the window-to-dense (W→D) gap it recovers, plus a bound retrieval probe. Under D15 as clarified by D17, conversion must remove the transformer architecture and the resulting runtime must pass the applicable D11/D5 audits before serving eligibility.
3. **Cost is measured, not derived** (D15).
4. **Parity decides in at most three seeds** (D16). A kill rests on the ordinary arm.

## 4. Coordination

- **One flock selector:** `crate::flock` (the DeepSeek lab, `lab/opencode/b0-flock`). A1's reads call `flock_select` with the sink at 0. A1's pre-registered k=1 pointer needs a top-k-only entry point, requested on #1512.
- **One Track B host:** the shared candle model (#1518), once its parity gate passes. The exact model-source path is its oracle only. Numbers from other hosts are not compared with it.
- **B0's corpus:** `simple-wiki-20231101`, with its recorded blake3 CID.
  - Evaluation articles follow D3's held-out rule, `blake3(id)[0] % 5 == 0`.
  - Calibration draws come from the other articles.
  - Draws are disjoint.
- **The owner's canonical flock arm** joins B0: Lorentz rank with a per-head rank table at k = 7 and k = 1.
- **Nothing runs until #1520's recovery and admission receipt clears its storage.**
  - Builds use the single admitted Cargo slot managed by the currently leased steward, who may be any available lab.
  - Every build directory is one per lab, on the internal drive or the recovered volume.

## 5. Recommended experiments

- **Claiming.** These are proposed tasks for stages 1–4. Under D14 an available lab claims eligible work in the atomic coordination state and links it from its board. A named lab is a routing suggestion, not an active assignment.
- **Admission.** B0 → A1 → B3 was this council's recommendation. The live task graph, reviewed work card and measured host admission govern actual order; do not launch or repeat an experiment solely from this table.
- **Walls** are compute bounds, not deadlines.

| # | Item (owner) | Gate and decisions | Kill or stop | Wall |
|---|---|---|---|---|
| 1 | **Recovery and admission** (Codex, #1520) | Verified recovery, a deployed runner with storage and RSS admission, and a restored ledger | — | #1520 |
| 2 | **B0-P0 pre-screen** (DeepSeek) | On the shared host, after its parity gate. Records s/forward, dense NLL, the W (sink + window 64) gap at positions >64, and each head's beyond-window mass, τ and entropy. Sets the window count N | Dense NLL outside 1.5–4.0 nats, or parity failure: instrument invalid | ≤45 min |
| 3 | **B0 main** (DeepSeek) | Arms: D; W; dot top-k k ∈ {7, 16, 64}; Lorentz top-k with a per-head rank table at k ∈ {1, 7, 16, 64}; layers 0–1 dense. A 64-item bound probe at distances 48–1,792. **Promote** at ≥90% of the W→D gap recovered (positions 66–2,048) with the probe within 0.05 of D. **50–90%:** trained transfer (B2). Lorentz and the rank table are parity pairs to dot and softmax | Dot k=64 recovers <50%, or the probe is >0.10 below D | ≤2 h |
| 4 | **M-world v2 freeze** (Claude) | Phrasing × value cells. Two untrained rules (the latest open value; the latest 2-word continuation) must score <0.6 on every gated cell, or one revision is allowed. A sealed 40-item English probe | No second revision: cells still passable are reported, not gated | source now |
| 5 | **R1-X** (Claude) | The sealed R1 on v1's cross cells. **Trained phrasing × dev value <0.2 while dev phrasing × trained value ≥0.5:** copy-limited, the lever is A1. **The reverse:** phrasing-limited, the lever is T3. **Both low:** both limits; A1 first, then T3. **Both high:** the v1 closed pools hid retrieval; go straight to A1 on v2 | Diagnostic | ≤1.5 h |
| 6 | **A1, amended** (Claude, [#1511](https://github.com/UOR-Foundation/uor-r4/issues/1511)) | Arms: P; P+ptr (Lorentz pointer); T, the transformer control, in round 1; C, the Dot control of the better arm. A post-hoc flock sweep and `top:1` pointer on fixed weights. **The frozen criterion is unchanged:** development MQAR ≥0.9 at every distance and open-relation recall ≥0.9. **A pass:** the mechanism for A3. **T at least 0.2 above the best stack arm at distance 64:** a seed-2 pair, then the parity rule. **Otherwise:** the best arm enters A3's smoke | Every arm, T included, below 0.5 at distance 16: exact log plus prime sieve. No third round | 6 × 30 min |
| 7 | **§8 panel authoring** (Claude) | Sealed before A3's data and before T3 generation, so neither can be contaminated. Kept out of the repository; its digest is recorded | — | ≤3 h |
| 8 | **B3 re-run** (Anti-Gravity) | SmolLM2-tokenized windows with a float-NLL validity band. Arms: RTN-4 at g32 and g128; RHT + scalar at 2/3/4 bits; E8P at 2/3/4 bits. Decode operations per weight reported. The lattice is credited only against RHT + scalar at equal bits | E8P-3 more than 0.05 nats worse than RTN-4 g32 | ≤3 h |
| 9 | **K-cost** (Anti-Gravity) | Position-resolved D11 against D10 timing on S2, exclusive, with instruction counts. GEMV at SmolLM2 shapes: D11 nibble, E8P-2, candle Q4_0. At ≥2× D10, closing the kernel gap comes before long-context cost work | Measurement only | ≤1 h each |
| 10 | **Teacher ceiling + T3** (Anti-Gravity) | SmolLM2-135M and 360M-Instruct, greedy, on M-world v1's development split and the 38-request panel. T3 paraphrases are kept only where the teacher answers the original intent correctly, and are excluded against development phrasings, the probe and the §8 panel. **A teacher below 0.80 on a category:** its T3 data for that category is teacher-correct items only, and the §8 bar is unchanged | A development template still has a near-duplicate neighbour: re-filter | ≤2 h GPU |
| 11 | **B2 stage 0** (Codex) | 2k context if B0's W−D is ≥0.05, else 8k (≤8 GiB). Heads routed by P0's map, calibrated on passkey data and chat-v0. Arms: D; W64; a window matched in bytes to each arm's state (derived and recorded before the run); the beyond-window mean; Taylor-2 and harmonic at d′16, L2; **a RoPE-plane-aligned or SU(2)/Wigner-D harmonic arm**; **a decayed or gated recurrent arm (the quaternion pillar)**. Stage 1 (2 seeds) only if an arm beats W64 and the mean by ≥0.02 | Failing on both draws: that arm goes to the D12 toolbox at this scope, with its next step named | ≤1 h |
| 12 | **A2 context** (claimable) | Reproduce the chat-v0 eligibility count from the sealed manifests, then truncated prefixes, the D11 ring buffer and age buckets. **The gate adds a recall probe beyond 256 tokens**, so that long context is shown to be used | Buckets cost more than 0.02 nats at 256: keep exact ages up to 256, and buckets beyond | 6 h + 1 h |
| 13 | **K0 census** (Anti-Gravity) | The stack trainer at 1.5M, 7M and 28M; threads; Accelerate; stock-op Metal. The Metal port starts only at ≥1.5× CPU at 7M. A 28M fit only if 20 tokens per parameter projects to ≤5 days | — | 45 min |
| 14 | **A3** (Claude; after items 5, 6, 7, 10 and 12) | One 7M fit with A1's winner. Data: chat-v0 with truncated prefixes; M-world v2 at ≥30% of tokens; filtered T3; T4 small talk. Unique and repeated tokens are reported separately. Final scoring is on D11-served replies, and the §8 panel stays sealed | At 10 tokens per parameter: development mean <0.5, or the retrieval cell <0.5 | ≤20 h |
| 15 | **B1** (DeepSeek; after B0) | B0's captures. Adds a query-aware arm, an exact (max,+) scan, and the attention mass recalled. No O(k) cost claim below 8k | As pre-registered | ≤1 h |
| 16 | **Pointer and flock serving** (claimable; after A1) | D11 kernels for the pointer mixture and flock selection. A serving flock needs N ≥ 2,048, k up to 64 and vector aggregation; today's `route_attention` is capped at 64 candidates | Serving parity within the fixed-point gap | tbd |

## 6. Dropped, parked, corrected

- **Dropped:**
  - A1′'s n-let continuation and windowed-read training, except under named conditions: n-let only if P+ptr fails on multi-token values, and windowing only after a probe of a dependency longer than 64 tokens;
  - C0's 16 arms;
  - the full harmonic census;
  - K1's factorial;
  - Track B as the main line;
  - A3's 3 × 2 grid;
  - a fixed 20 tokens per parameter.
- **Replaced gates:**
  - B2's 0.15 kill and its "−50%" smoke (item 11);
  - B0's Lorentz-only kill (item 3);
  - A1's middle band (item 6);
  - the "unresolved" parity band (D16).
- **Parked in the D12 toolbox:**
  - 2I and E8 codes as addresses;
  - B1 as a cost claim below 8k.
- **Corrected:**
  - GM-Net 2605.13262 is a molecular model.
  - Hedgehog's degree-2 Taylor map is spiky.
  - Harmonic L=2 spans Taylor-2 (152 against 153 features at d′=16).
  - The literature for forcing retrieval is 2205.05055, 2312.03002 and 2602.11374.
  - "The pointer was dropped at D0" is an inference.
  - The 10.4 s/position oracle rate is specific to Codex's parity-3 run (no BLAS, 2 workers).

## 7. Not established

- Chat, and English multi-turn memory.
- Any geometric read advantage in the main-line model.
- Any runtime or energy saving of D11 over an ordinary 4-bit model.
- SmolLM2-135M retrieval under sparse attention.
- Harmonic attention against a byte-matched window.
- E8P against matched scalar codes on SmolLM2.
