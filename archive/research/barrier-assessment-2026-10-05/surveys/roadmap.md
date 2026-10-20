# UOR-R4 roadmap survey: governance, live state, blockers and next steps

I read the repo at `71b54adf` (merge of PR #1705, 2026-10-05 01:22 UTC), all 364 comments on #820 (the last 30 in detail), the last 25 of 272 comments on #1552, and the latest comments on #1511, #1512 and #1515. There are 22 open issues and no open PRs. Every figure below is labelled **measured** (with its scope), **design/decision**, or **hypothesis**. Nothing was re-run.

## 1. Mission and governance

**Mission.** The project is building a Rust "native geometric" language model: prime/zeta/R4/S3/H4/Z[φ]/UOR identity, exact addressed memory and shared typed operators. The near goal is useful conversation and memory plus coding and reasoning. The long-term goal is frontier-like capability on an M1-class laptop (`AGENTS.md`; `README.md`; `docs/integration/project-track.md` "Active programme").

**Serving rules** (`ROADMAP.md` §0):
- **R1:** no floating point in served code.
- **R2:** no multiply or divide instruction (D0-b, D11).
- **R3:** no dense per-token weight access as the end state (D5). Low-bit add/shift/table maps (≤4-bit) are allowed as labelled interim steps.
- **R4:** no transformer backbone.
- **R5:** energy is claimed only from measured J/token on the M1.

Offline Rust training may use floats and matmul (D0-b, D8).

**Which document governs what:**
- D19 (`docs/integration/DECISIONS.md:804`, 1 October) makes `project-track.md` the single active plan. `current-state.md` owns results; ROADMAP, STATUS and the lab entry are navigation only.
- The owner's 3 October direction ([#820 comment 5971681179](https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-5971681179)) ended transformer baselines and withdrew the 0.05-nat kill rule. Comparisons are now geometric against geometric, and against the previous best geometric model.
- D12: gates promote, never kill. D9: no unchanged repeats. D14: durable labs and a council.
- Merge rule: exact-head review (self-review allowed) plus actual scoped tests, then a protected merge (D19 §5).
- `docs/formal_vocabulary.md` §1–2 requires claim-class and status labels (Definition, Objective, Guarantee, Empirical Criterion; Structural, Witnessed, Empirical, Unproven). Its automated wording check is "dormant by default" (§2.1).

## 2. Milestones and acceptance criteria

| Milestone | Stated acceptance criterion | Source | Status |
|---|---|---|---|
| A1 retrieval in the stack | MQAR ≥0.9 at distances 16/64/200 **and** open-relation ≥0.9 on the M-world v2 dev split | `docs/labs/plan-2026-09-29.md` stage 1; #1511 (1 Oct) | Failed. D18 outcome D is preserved and A1 training does not resume (STATUS). |
| Grounded compiler → store → emitter (D19 next deliverable) | A saved, learned relation/act/span compiler feeding the exact versioned store and the real emitter, on predicted inputs and the model's own generated history | `current-state.md` "Active execution contract" | Partial (§3). |
| §8 first product milestone | 40 responsive multi-turn, 30 updated-relation and 30 instruction conversations, each ≥80% of scored turns; identical greedy continuation after a fresh-process reload; declared context, eviction and output limits; a cost report; scored on an owner-held sealed panel | `project-track.md` ~L436; plan stage table | Not attempted. The sealed panel stays closed. |
| Stages 0–6 (recover → retrieval → memory/context → faithful native execution → useful conversation → coding/reasoning/efficiency → scale/release) | One evidence requirement per stage | `plan-2026-09-29.md` (superseded schedule; criteria retained) | Stages 1–2 are open. Stage 3 is partial (D11 GPTQ arms). |
| Alpha | Same native artifact; coherent prose and instruction following; causal contextual attention; grounded answers with abstention and conflict handling; durable isolated memory; novel compositional reasoning; executed coding tasks; full-path M1 cost (cold load through persistence, RAM, energy per useful task); then API, WASM and Pages Studio | `project-track.md` "Alpha acceptance" (L1226) | Open (#965). |
| Order of work | compiler/store/emitter (#1552, #1508, #973) → durable grounded conversation (#962, #954) → faithful integer serving (#964) → useful language → reasoning/coding (#955, #1088) → API/WASM/Studio (#1172, #1173, #965). Selected access (#963) runs alongside. Track B (#1509) is parked. | `ROADMAP.md` "Order of work" | See §5. |

## 3. Live measured state (key items only)

- **Scale ladder.** Claude lab, #820, measured on TinyStories validation NLL:

  | Rung | NLL | Report |
  |---|---|---|
  | 20M | 1.3882 / 1.3858 (two seeds) | #820 5974333259 |
  | 29M, lr 5e-4 | 1.2857 | #820 5974333259 |
  | ~96M (`geo-100m`, 95,957,184 params, 1.5B tokens) | 1.1729 | #1552 5980927216 |

- **D19 grounded session** (300 conversations, 1,075 turns, exact store with log sieve): 29M scores 972. The 96M Arm C scores **1008** (908 with the store off) and Arm B scores 984 (#1552 5980927216). The gain is mostly instruction following (68 → 78/98).
- **Open 232-request panel** (qwen2.5:7b judge): 43 acceptable at 29M; 45 and 46 at 96M B and C. Neither 96M arm is significantly better than 29M (p = 0.87 and 0.76; same comment). This is the "flat 43–46" result.
- **Integer serving (D11).** At 96M, GPTQ integer 43 versus float 46 acceptable, p = 0.71. At 29M the gap was 10 replies, p = 0.053 (#820 5981279097 and 5977516230).
- **In-context recall with the network alone.** With the sieve off, MQAR is 3/109; with it on, 105/109. Re-checked from the sealed L12 reports (#820 5983928570). The lab itself warns that MQAR is only ~0.8–2% of the fine-tune mix.
- **MQAR bench.** PR #1698 (`f92ba0f7`): ~1.36M params, width 128, 6 layers, context 512.
  - All-read stack: 0.807 / 0.176 / 0.010 / 0.029 at d16 / d64 / d200 / d400.
  - Zeroing the age bias removes most recall.
- **Key shift.** PR #1701 (`3aa8ba16`) adds `k_t + j·k_{t-1}`. It gives 1.000 at every distance and 1024/1024 on a held-out pairing class. The probe shows one layer-0 head putting 0.99–1.00 of its weight on the value.
- **Key-shift correction.** PR #1704 (`2cdae05a`) found:
  - `rrarra` *without* the shift fails on seed 1 (0.007) but reaches 1.000 on seed 2. The earlier "recurrence crowds out reads" reading is withdrawn.
  - All-read fails on both seeds.
  - With the shift, every pattern and seed tried scores 1.000.
  - `key_shift` is now a saved, trainable option. Integer and QAT paths **refuse** shifted models; the D11 serving form is a design note only (estimated at ~200 lines).
  - Scope: 1–2 seeds at width 128. The lab calls it the standard previous-token route, "not by itself evidence of a geometric advantage."
- **Codex native signed-H4 line** (`current-state.md`, Oct 4 entries; PR #1705):
  - Cue fit: development cross-entropy 1.146 → 1.040; banks 1 → 2/64; fresh 0/32 complete answers (the cue-only control gets 1/32).
  - On fresh rows, the current source has the highest Copy score in 29/32, but 21 of those choose an internal offset, and seven replies loop.
  - All of this is supplied-record consumption with a 128-token context cap, not whole-history recall.
- **Relation compiler wiring** (#1552, 4–5 Oct; unsigned, and it addresses "Claude" as reviewer):
  - The memory panel reaches 137/200 against a pre-registered ceiling of 140 (baseline 84).
  - The `world=v2` gate is unchanged at MQAR 22/23, open 11/12, closed 3/3.
  - The 60 remaining failures are exactly the ceiling's 60 unrecoverable rows.
  - It was reported as uncommitted at the 23:21 UTC handoff. Its false-write rate on non-relation turns is not yet measured.

## 4. What each lab is doing now

**Claude lab** (#820; STATUS lists it on #1511/#1552). It runs the CUDA pod (PR #1649 `dc28b495`), the scale ladder, D11 GPTQ serving evaluations and the MQAR bench with the key shift. In progress: a pod A/B with the same new base and the same Arm C fine-tune, key shift off versus `key_shift=add`, then D19 MQAR with the sieve off against 3/109. It was due after the base finishes, around 1:30 AM ET on 5 October (#820 5986128519). Also owns `examples/mqar-bench.rs`, φ-phase storage/summary and learned ranking (#820 5983925761). The φ-phase exact-address read is **design only**: Fibonacci phases with Zeckendorf codes, exact prime/n-let admission, then geometric ranking (#820 5983648331).

**Codex** (#1552, #1515). The native integer signed-H4 occurrence bank, cue carrier, and Copy/Period/Stop emitter. Active work card (#1552 5986523941): an ordered-prefix directed H4 relation, `inverse(responseprefix)*sourceprefix` in a 120-bin signed q4 lookup, compared against a source-prefix unary control with matched capacity.
- It requires zero-update parity and a nonzero within-source witness.
- The deadline is 03:23 UTC on 5 October. No fit has been admitted yet.
- The cumulative ledger stands at 1,227,435,028 of 1,228,200,000 ms, extended repeatedly under the standing 2026-09-06 allowance.
- It also owns the geometric admission/routing adapter for the bench (`CandidateAdmission`).
- It calls the key shift "a donor principle, not native serving evidence" (#1552 5986476709).

**DeepSeek** (#1512). OpenCode was retired on 3 October and DeepSeek now runs in its own harness (#1512, 22:16 UTC 3 Oct). Its 3 October work, all on #820:
- Surface-form breadth: the statement side generalises (37.8% → 94.6%); the question side does not (58.1% → 63.5%).
- Query conditioning: confirmed that the query is used as a retrieval key.
- Distance curriculum: a negative result (3/120 versus 14/120).
- A disjoint panel, plus a finding that inherited panels contain contradictory duplicate rows.
- It was asked to run the 96M disjoint-panel scoring (#1552 5980927216).

The 4–5 October relation-compiler wiring (§3) matches this lane's reporting style but is not signed. This attribution is an inference.

**Anti-Gravity / Gemini "support lab".** Documentation and issue triage only, read-only. It posted a triage of open issues and proposed new titles (#820 5972101700 and 5973943125). The #1552 title now matches its proposal.

## 5. Blocked items and what blocks them

- **#962 / #954, durable grounded conversation:**
  - The network alone does not recall (3/109).
  - The open panel is flat from 29M to 96M.
  - The compiler has 60 unrecoverable "What is it?" rows and an unmeasured false-write rate.
  - The §8 milestone needs the sealed panel and the 40/30/30 ≥80% thresholds.
- **Geometric attention (Codex line):** blocked on within-source ordering. Source-wide cue scores cannot reorder offsets inside a record; fresh complete answers are 0/32. Whole-prefix admission waits on the bench interface.
- **Key shift into serving (#964):** there is no D11 form, so served paths refuse it (PR #1704). Whether it fixes chat recall waits on the pod A/B.
- **#963 D5 selected access:** dense layer maps and the full vocabulary head are still read per token (README "Architecture"). There is no active work card.
- **#955, #1088, #1172, #1173, #965:** blocked behind useful language and the grounded path (ROADMAP order).
- **Track B #1509:** parked; the #1518 parity failure stands.
- **Infrastructure:** removing the five acknowledgement status names needs an org admin (AGENTS.md L172). Hosted runners are shared with other org repositories (#820 5971306869).

## 6. Contradictions between documents

1. **GPU and paid compute.**
   - "No CUDA/external GPU is authorized by this plan" (AGENTS.md, Resources) and "No external GPU/CUDA or paid compute is authorized by this plan" (`project-track.md` "Practical iteration") disagree with STATUS, ROADMAP and AGENTS.md L172's "approved GPU pod" and the rented 2×4090 Runpod.
   - D19 §6 says "No paid compute".
   - DECISIONS.md has **no entry** recording the pod authorization (grep finds none).
2. **Transformer controls.** README "Architecture" ("An ordinary transformer control runs on the same kernels") and `crates/uor-r4-training/README.md:53` predate the 3 October no-transformer-baselines direction.
3. **Lab roster.** The AGENTS.md D19 banner and `docs/labs/README.md` say "OpenCode–DeepSeek" and treat Anti-Gravity as historical (D19 §5). STATUS says OpenCode was removed on 3 October and lists Antigravity as an active support lab. The D14 banner in AGENTS.md still lists all four as available.
4. **Stale status.**
   - ROADMAP and STATUS say the ~96M run has "no result yet". Results exist (NLL 1.1729, session 1008, panel 45–46, D11 43 versus 46).
   - STATUS puts Claude on "#1511/#1552 compiler/emitter integration". #1511's last comment is from 1 October, and Claude's actual work is the ladder and MQAR.
   - STATUS's Geometric attention row ("Next: typed selected-record occurrence→actual-BPE consumer") is about 20 Codex steps out of date.
5. **Two definitions of "geometric attention".**
   - `project-track.md`'s active section and every 4 October entry in `current-state.md` cover only Codex's supplied-record native H4 bank (128-token cap).
   - The production geometric-stack read layer, the MQAR bench and the key shift are absent from `current-state.md` apart from a one-line pointer ("Claude #1704 is separate floating Stack/MQAR evidence").
   - `project-track.md` says Claude's #1552 work "does not replace this mechanism track".
   - So the canonical plan does not reconcile which attention mechanism the served chat model will use.
6. **Project map.** `docs/PROJECT_MAP.md` (navigation dated 25 September) never mentions `geometric_stack` (0 hits), although README says it "covers every crate".
7. **README memory claim.** README says AERM "is not yet in the served model". STATUS and the D19 sessions use the exact store and sieve at session level. Both may be true by scope, but neither says so.
8. **Superseded documents still cited.** `plan-2026-09-29.md` and `model-direction-2026-09.md` carry "superseded" banners while still being cited for acceptance criteria. D18's "one retrieval question for 1–14 October" coexists with D19.

## 7. What the plan says the next three steps are

**Canonical plan** (`project-track.md` "Current geometric attention experiment"; `current-state.md` top "Next"):
1. Freeze R64 plus cue48 and build the ordered-prefix directed H4 relation against the source-prefix unary control. Zero-factor parity and a within-source witness come before any fit (NOT_RUN).
2. If admitted, run a paired fit with parent-inclusive full128 complete-answer+EOS selection and a fresh panel frozen before update 1. Then coordinate the full-prefix admission adapter with the shared MQAR instrument.
3. Integrate the shared session on #1552 only after that. General generation, chat, compiler/store, reasoning and laptop energy remain obligations.

**Roadmap / product order:** finish the learned compiler/store/emitter (#1552, #973), then durable grounded conversation (#962, #954), then faithful integer serving (#964).

**Claude lab's posted next steps** (#820; not yet in the canonical plan):
1. The pod key-shift A/B on D19 MQAR with the sieve off.
2. A D11 serving form for the key shift (a signed permutation plus an integer add).
3. The φ-phase exact-address read arm on the bench.

**Status of the "canonical token lineage" claim.** Per #1704, the measured result is that the key shift makes recall robust; it was not strictly missing, because the width-4 conv route works on some seeds. Calling it a missing mathematical piece of the architecture is still a **hypothesis**. It would be tested by the pod A/B and by a served D11 form.