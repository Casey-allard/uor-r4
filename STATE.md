<!-- rendered by uor-harness loop render from dd96293762198fdc2ebb1bc168c32ee3403426d4; do not edit -->
# STATE

Rendered from committed inputs at the commit in the stamp above.

## Milestone bars and holds

Source: `docs/harness/milestones.json`.

| Milestone | Status | Headline bar | Training hold |
|---|---|---|---|
| M1 Language base: a geometric stack language model good enough to carry chat | complete | M1 closing bars: 6 of target 6 (out of 6 closing bars) | released |
| M2 Grounded reply from exact memory: the native learner completes correct replies | in-progress | complete correct replies on one accepted saved model (dev): 0 of target 256 (out of 512 replies) (direction: open) | released |
| M3 Durable conversation memory: provenance, conflict, abstention, reset and restart | in-progress | typed outcomes qualified on the saved model: 0 of target 1 (out of 1 qualified milestone) (vacuous) | not recorded |
| M4 One served model: the chosen model through D11 integer serving and a CLI | not-started | accepted model served through D11 and CLI: 0 of target 1 (out of 1 qualified milestone) | released |
| M5 Laptop cost (D5): full-path M1 cost and selected parameter access | not-started | full-path cost and selected parameter access qualified: 0 of target 1 (out of 1 qualified milestone) (vacuous) | not recorded |
| M6 Reasoning and coding on the same native model | not-started | unseen compositions and changed constraints on one model: 0 of target 1 (out of 1 qualified milestone) (vacuous) | not recorded |
| M7 Distribution: capability API, WASM runtime and the Pages Studio | in-progress | native model served through the API, WASM and Studio: 0 of target 1 (out of 1 qualified milestone) (vacuous) | not recorded |
| M8 Alpha release: the same model for conversation, memory and coding | not-started | one labelled release on the same native path: 0 of target 1 (out of 1 qualified milestone) (vacuous) | not recorded |

## Roadmap items by status

Source: `docs/roadmap/items/*.md` frontmatter.

| Status | Items |
|---|---|
| done | 114 |
| held | 11 |
| ready | 31 |
| **total** | 156 |

## Newest notebook row per research line

Source: `docs/research/notebook/*.md`; the row with the latest date, the last such row on a tie.

| Line | Date | Number | Bar | Verdict |
|---|---|---|---|---|
| cpu-training | 2026-11-16 | Per step: 2,048 forward words, 768 loss words, 48 choices; Flock 16 refusals on both sides. **0 mismatches in all four steps.** Commits `Route(0)`, `Route(1)`, `VectorBlock(56)`, `VectorBlock(59)`, each with a byte-equal data section. The CPU keeps the stale `snapshot_sha256` (claude-372). Planted target shift caught at the losses step (the planted pass runs proposals only). The CPU oracle costs about 4.8 s per parameter. | every compared word equal, or each difference reported (ORDER) | positive |
| data-tokenizer | 2026-11-16 | **Baseline A0 241.9 kept tok/s (A0-repeat 233.9)**. No plan-keeping lever won: `-np 64` 252.2 (1.04×), with larger batches 251.2, short system message 222.2, no `/tokenize` 240.5, `n` = 4 192.6, greedy + `n` = 3 187.5, trimmed CPU chain 236.6; `-np 128` failed at start (out of memory). `--backend-sampling` 280.3 (1.16×) but read "not active". **Final-channel pre-fill 418.3 (1.73×), 931 kept per 1,000 generated against 482**, 0 truncated, 4 markup of 4,824. Sizing: 100M kept tokens = 116.8 h of wick keeping the plan, 66.4 h with the pre-fill. | The sheet's B1–B3 (smokes; ≤ 5% truncated + markup, ≤ 1% errors, 0 tokenize errors; A0 within ±15% of 220.7 and within 10% of its repeat) and the win line (≥ 1.10 × A0 and beyond the A0 spread). | **Measured: B1–B3 hold (reading P).** Slot, batch, prompt and request-shape levers: negative for these configurations; decode-bound at 32 slots. Pre-fill: wins, but changes the plan and conflicts with D87 (reasoning kept). Backend sampler: open (the check could not detect it). Superseded for production by D87 (DeepSeek V4.1 Flash on rented pods). |
| float-training | 2026-11-16 | Outcome 0 exact (49,152 rows a state). `U` = 0.42–0.46, against the bar's 0.106–0.120; `K = 1` on 1.6–8.2% of queries. C2′ puts 94–99% of its picks on the 8 fact rows, spread close to evenly (asked 73/67/70/69, against 60–63 under an even spread; `A ≤ 2P` in every cell). On `K = 1` rows C2′ hits the other fact 1/35 and 3/42, below its `K ≥ 2` rate | The ORDER's outcome map | **inconclusive** overall: outcome 1 does not hold; outcome 3 at ext-s1 b1; outcome 4 ("control fault") in all four cells, so bar 4b's `chose_target` half cannot discriminate on this task. claude-395's verdict stands under its bar |
| harness-governance | no rows yet | | | |
| legal-construction | 2026-10-24 | Speed-up only: Unit 11's rational outputs reproduced bit-for-bit, `solve_ms 69,140`, `disjunctive_ms 146,897` (Unit 10: `264,832` / `1,116,550`). With the sort fix: tree lower bound `≈ -21.02881252`, gap `≈ 0.00685092` (`0.03259%`), `6,406/28,800` codes eliminated, 115 fixed, 186 binary-or-fixed; `solve_ms 57,057`, `disjunctive_ms 148,896` (taken under machine load) | none registered (instrument run on the saved guard-constrained linear problem, not a panel score) | **certified discrete gap narrowed to `0.006851` (`0.03259%`); `disjunctive_ms` cut 86.66% from Unit 10 with exact outputs** — item stays ready until the gap is closed |
| m1-discrete-attention | 2026-11-16 | **0 mismatches everywhere compared.** All 256 root pairs: distance = L1/2. CUDA read and trace = CPU oracle on 264 fixture rows. One 64-row D87 student batch at full size, both transports: read, trace and 768 loss words over 48 Table and Gate parameters. CUDA = served artifact on 66 prefixes (read and greedy token). CUDA = CPU scorer on 38 Table and Gate parameters per transport (608 loss words, 38 choices); every Route refused on both sides. Planted copy picks position 1 at distance 0. Planted faults (swapped transport, shifted targets) caught | the ORDER's: word-for-word parity on fixtures and one real batch, serving parity, the refusals | **positive** for one exact read shared by train and serve; no learning claim, and no CPU applier for Pick yet |
| m1-prime-zeta | 2026-10-14 | 4,096 of 4,096 tokens get distinct vectors (0 collisions, 256 of 256 writer cells occupied); 17 of 17 base and 6 of 6 prime-zeta CUDA controls pass; 0 updates applied, 0 corpus rows | Bit-exact table across seeds 0, 1 and 2; no floating-point instruction in the GPU code; 0 collisions over all 4,096 tokens; 17 of 17 base controls; the codebook must separate structurally from both nulls | positive (construction only; the learned comparison against the nulls is deferred to a training sheet) |
| m2-grounded-reply | 2026-10-26 | `TEST FITNESS: NOT_FIT`: Stage T/trunk required arguments and binary staging absent, trunk Step 0 precedes its checkpoint, anchor build/Stage T grade absent, no M2 student, no candidate-specific GPU reply evaluator or D11/native-bank bridge. Bar 1 is an M1 composite, not a faithful M2/Amendment bar; best antecedent top-8 `4,674 < 4,775`. M2 student cost/power `UNDEFINED`. Superseded recipe matches the #3300 base-blob sections with zero diff. | A FIT must describe an executable end-to-end configuration; no compute authorized and no mechanism/bar/hold/source change. | **negative** (`INSTRUMENT_NOT_FIT`; #2851 remains a pins-delivery close only; #2853 must provide student/evaluator/bridge under a new ORDER; not a model or mechanism negative) |
| m3-durable-memory | 2026-11-02 | Both seeds emitted `Memory: Pickle.` and `Memory: tomatoes.`. `6/6` exits 0, empty stderr, `15/15` valid protocol-2 stops (10 EOS, 5 token-cap), exact normalized seed-1 repeat; **0/6** factual replies contained the supplied value. Peak RSS 11,888 KiB; 0.17–0.22 s per process. | FIT/PRE-RUN registered instrument: both expected memory lines on both seeds, deterministic repeat and valid processes; model quality reported separately with no bar. | mixed at serving scope: `AUTO_MEMORY_OBSERVED`, `MODEL_RECALL_NOT_OBSERVED`; not M3 qualification or useful chat |
| m4-serving | 2026-11-17 | Host tests **6 / 6**, counter **21 / 21**. The 4,096 masses of every history sum to M minus a deficit well inside the 3(N+1)·4,096 bound: the largest seen is **4,225 / 11,554** units at N = 1 / 4, against bounds 24,576 / 61,440. Scorer = oracle on every target of three 6,000-token streams at N = 1..4. Negatives: γ = 0, a BOS target backing off (sum > M), an unseen context giving 0, and raw count-of-counts below N each fail their test. | none registered (finite host control) | **positive, finite only.** No BPB; bits move to claude-420's instrument, because `lawful_likelihood` is compiled out of this crate's test build. |
| m6-reasoning-coding | 2026-11-07 | **80 / 80 rows passed**, covering all **320 / 320** held-out vector assertions; 7 verifier integration tests passed in 10.06 s. | none registered (instrument qualification) | **positive (instrument QA):** the frozen bar's literal `rustc --test` cross-check now runs in CI; this is not model-emitted code, a capability result or useful-chat evidence. |

## Mechanisms

Source: the ACTIVE / READY / PARKED tables in `docs/mechanisms/README.md`.

- ACTIVE: Address cells (A8)
- ACTIVE: Admission
- ACTIVE: D11 integer serving
- ACTIVE: Discrete geometric learning
- ACTIVE: Flock read
- ACTIVE: Hamilton gate (G2)
- ACTIVE: Prime-zeta codebook
- ACTIVE: Rank-margin hinge
- ACTIVE: Relation read
- ACTIVE: Routed read
- ACTIVE: SpiralCore/FBS routes
- ACTIVE: Tokenizer
- READY: Hopf observation: kept-fibre map
- READY: Object store (D19)
- READY: Recurrence gates
- READY: Spin-native weights
- READY: Ternary/XOR
- PARKED: Carrier credit
- PARKED: E8 orbit memory
- PARKED: Exact memory
- PARKED: Geometric stack
- PARKED: Hopf observation: sector routing
- PARKED: Legal construction
- PARKED: Native geometric engine
- PARKED: Pointer key
- PARKED: R4G1 graph
- PARKED: Read binding
- PARKED: VSA

## Lab registry

Source: `docs/harness/labs.json`.

| Lab | Vendor | Client | Open PR limit |
|---|---|---|---|
| antigravity | Google | Antigravity | 3 |
| claude | Anthropic | Claude Code | 3 |
| codex | OpenAI | Codex | 3 |
| deepseek | DeepSeek | DeepSeek harness | 3 |
| kimi | Moonshot AI | Kimi Code | 3 |
