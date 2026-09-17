# UOR-R4 — Go-forward execution plan and agent briefing (rev. 1, 2026-09-16)

**Audience:** Casey, and every agent (Antigravity, Codex/Astra, Claude, Gemini) working in `~/uor-r4`. Read `00-project-brain-index.md` first, then `08-outlook…` for the reasoning behind this plan. This document is the *work order*: it converts the critique into pre-registered experiments with controls, budgets and kill criteria, and it fixes the process errors identified in `08` §4/§7. Everything here is a proposal for Casey to accept, edit or reject; nothing below is authorized until he says so.

---

## 0. Ground rules for agents (the errors we are correcting)

These replace the "do not stop at another general plan, proceed with the next step" cadence that produced ~210 PRs and ~50 template gates in 14 days without a capability result.

1. **No experiment without a signed card.** Every run starts from a one-page card (template in §5) that names hypothesis, matched control(s), metric, dataset (sealed, never iterated against), budget and kill criterion. A human (Casey) signs the card before compute is spent. Agents may *draft* cards; they may not self-approve them.
2. **Matched non-geometric baseline is mandatory.** ExactIdentity / regex / n-gram / tiny transformer on identical data. If the baseline hits ~100%, the task is discarded as non-discriminating, not the result celebrated.
3. **Templates are diagnostics, never headlines.** Report template count and slot entropy; every synthetic number is paired with a natural-data number.
4. **Measure, don't estimate.** tok/s, `powermetrics` J/token, resident bytes/token, named hardware/OS, against `llama.cpp` and `bitnet.cpp` on the same machine. No constants in profilers, ever again (see recovery-2026-09-08).
5. **One evidence table, one two-page plan, one dated decision log.** Retire `current-state.md`-style accretion (438 KB) as the authority; it becomes an archive. New results go into `docs/integration/EVIDENCE.md` (one row per card) and decisions into `docs/integration/DECISIONS.md`.
6. **Kill criteria are honored.** A FAIL against a pre-registered kill criterion retires the path; it is not "preserved and circumvented by a redesign" without a new signed card.
7. **Weekly human decision point.** One hour: read the evidence table, continue/kill per path. Agents do not choose the next experiment between those points.
8. **Claims discipline stays.** Keep `check_claim_wording.py`, sealed roots, frozen acceptance, OS-entropy seeds — the good parts of the current process.
9. **Local hygiene before any model work.** `~/uor-r4` must be at origin/main (it is 52 commits behind); `git lfs install` so `git status` is clean; the two stashes are inspected and either applied or dropped by Casey.

---

## 1. Decision D0 — what "no matmul" means (blocks everything)

Casey must choose one, in writing, in `DECISIONS.md`:

- **D0-a (strict):** no mathematical linear map at inference, as the README states today. Consequence: BitNet/T-MAC-class kernels are excluded; the only learnable representation on the table is P3 (learned finite-group embedding + learned 120×120 compatibility tables + differentiable offline surrogate). Run P3 as kill-or-continue; if killed, the native predictor is retired and P2 is the product.
- **D0-b (measured):** "bounded integer/ternary accumulate; no floating point at inference; no dense projection wider than N; ≤ X J/token and ≤ Y bytes/token on M1, measured." Consequence: `uor-matmul`-style exact integer GEMM and ternary weights become legal serving substrate; the project competes directly with BitNet b1.58 on its own hardware and keeps H4/exact memory as state and storage substrates.

Recommendation in `08` §4 R1 is D0-b. Either is coherent; the current wording is not achieving the actual goals (energy, no GPU, local, auditable).

---

## 2. Experiment cards (pre-registered, ready to sign)

### Card P1 — Ground truth on the M1 (measurement only; no model work)
- **Hypothesis:** none — this is the denominator every later claim divides by.
- **Runs (same machine, same session, 3 repeats):** `bitnet.cpp` + BitNet b1.58 2B4T; `llama.cpp` + SmolLM3-3B Q4_K_M and Qwen3-4B Q4_K_M; `r4 geometric` + `15baec48`; the TLA/R4G1 bundle. Optional: Kneser-Ney 5-gram (kenlm) trained on the same TinyStories slice as `15baec48`.
- **Held-out set (sealed now, opened once per artifact):** 2,000 TinyStories-V2 stories and 500 Simple-Wiki articles, hash-split by `BLAKE3(text) mod 1000 ∈ {0..9}`, never used for development. Store the manifest + digest under `docs/evidence/heldout-2026-09/`.
- **Metrics:** bits-per-byte (teacher-forced), complete-request wall time (load + encode + 256-token generate + persist), `powermetrics` J/token, peak RSS, resident bytes/token, tok/s.
- **Controls:** none needed; the incumbents are the control.
- **Budget:** ≤ 2 days engineering, ≤ 3 h machine. Storage ≤ 6 GB for weights.
- **Deliverable:** `docs/integration/EVIDENCE.md` row per artifact; one table in README replacing all historical efficiency language.
- **Kill criterion:** n/a.

### Card P4 — Geometry controls (before any more geometry)
- **Hypothesis H4a:** zeta-zero phase features beat *same-count random fixed phases* on next-piece accuracy in `15baec48`'s count model. **H4b:** `prime % 120` placement beats *random token→root assignment*. **H4c:** replacing the 120-landmark Hamming distance with the 120×120 angle-class table changes no output (mathematical identity check).
- **Data:** the existing recovery corpus (391,725 positions) and the sealed P1 held-out set.
- **Controls:** random-phase, random-placement, geometry-disabled — each a *matched refit*, not a within-artifact disable.
- **Metric:** held-out next-piece accuracy and BPB; report deltas with bootstrap 95% CIs.
- **Budget:** ≤ 1 week, ≤ 6 h machine (fits are counting; cheap).
- **Kill criterion:** if zeta and prime placement are within CI of their random controls, move primes/zeta/Hopf/E8 out of the critical path permanently (keep H4 tables as substrate only). Record in `DECISIONS.md`.

### Card P2 — Exact memory as a product layer (highest expected value)
- **Hypothesis:** the retained exact/versioned memory + copy + arithmetic substrate, used as retrieval/tool layer around a real small local model, produces grounded answers with lower hallucination and correct abstention than the model alone.
- **Model:** BitNet b1.58 2B4T under D0-b (or SmolLM3-3B if D0 undecided); via llama.cpp/bitnet.cpp server, called from the Rust service.
- **Task/data:** grounded QA over 200 user-style documents the model has never seen (post-cutoff text, e.g. project docs written after the model's training date), 1,000 questions with supported / unsupported / conflicting labels, authored by someone other than the implementer; sealed.
- **Controls:** model alone; model + naive BM25 retrieval; memory layer with citations disabled.
- **Metrics:** exact/semantic accuracy, hallucination rate on unsupported, abstention precision/recall, citation correctness, complete-request latency and J/query (P1 harness).
- **Budget:** 4–8 weeks engineering; machine ≤ 20 h.
- **Deliverable:** working local assistant CLI/service (this satisfies most of #962 and #954 acceptance text); EVIDENCE rows.
- **Kill criterion:** memory layer does not beat BM25 control on accuracy *and* abstention → the substrate's product value is retrieval-equivalent; keep as provenance/audit layer only.

### Card P3 — A real learner inside the contract (time-boxed kill-or-continue)
- **Hypothesis:** a geometric-native predictor with (i) a *learned* token→state embedding into a product of finite factors (2I^k, k ∈ {16, 64, 256}, or D-bit VSA codes), (ii) a *learned* pairwise compatibility that is not a class function of g_i⁻¹g_j (per-lane 120×120 tables; legal under D0-a), (iii) trained by a differentiable offline surrogate (DLGN/DWN-style relaxation, Adam, hard export) — shows a *non-flat* held-out BPB slope versus learnable bits.
- **Data:** TinyStories-V2 train (≥ 100 MB), sealed P1 held-out set. No templates.
- **Baselines (matched tokens):** Kneser-Ney 5-gram; a 1M / 10M-param standard transformer; BitNet-style ternary GRU (if D0-b).
- **Metric:** BPB at 1M / 10M / 100M learnable bits; free-running distinctness and cycle rate on 100 held-out prompts (as in #841).
- **Budget:** 6–10 weeks; machine ≤ 60 h across the three scales.
- **Kill criterion (written now):** if BPB at 100M bits is not at least 0.3 bits/byte better than the 5-gram, or the slope from 10M→100M is < 0.05 BPB, the native predictor is retired and P2 becomes the product path. No redesign extends this card without a new signed card.

### Card P5 — Exactly decodable codes for VSA clean-up (small, honest contribution)
- **Hypothesis:** replacing nearest-neighbour clean-up in binary VSA with syndrome decoding over a real error-correcting code recovers more bound items per bit than the standard majority/clean-up recipe.
- **Data:** synthetic bind/bundle tasks (Kleyko-style), D ∈ {1k, 4k, 16k} bits.
- **Controls:** standard random binary VSA with Hamming clean-up; the existing 120-landmark signature.
- **Metric:** items reliably recovered vs. code length; decode cost in integer ops.
- **Budget:** 1–2 weeks. **Deliverable:** short note; candidate arXiv/Zenodo preprint.

### Card P6 — Publication hygiene (one day, no compute)
- Post `research/ai-research/ai-router/paper1_arxiv_rebuild_v4` to arXiv as-is (it contains the permutation null). Annotate or retract Zenodo 10.5281/zenodo.20045543. Remove `SkyNetwork_BusinessProposal.pdf` (marked confidential) from public LFS. Fix the wrong ORCID on the Zenodo record.

---

## 3. Roadmap re-sequencing proposal (for #820)

| Now | Proposed | Why |
|---|---|---|
| 01 #1139 role binding | **D0 decision → P1 → P4** | Nothing else is interpretable until the contract is settled and a denominator exists |
| 02 #1140 transitions | **P2 (memory as product)** absorbs #962/#954 acceptance | Buildable now; literature-backed; gives the Foundation a usable artifact |
| 03 #973 general prose | **P3 as a kill-or-continue card**, then #973 either continues or is re-scoped to "memory + operators around a host model" | Viability before capability |
| 04–07 #962/#954/#955/#1088 | Follow P2's result | Their acceptance texts are good; keep them |
| 08 #963 M1 cost | **Merged into P1 and every later card** | Measurement moves from last to first |
| 09 #964 guarantees | After D0 | The guarantee to state first is the contract |
| 10–11 #1172/#1173 API/Studio | Unchanged code, deferred | Studio integration after a model exists; Studio's README claims need reconciling first |
| 12 #965 alpha | Unchanged text; fed by EVIDENCE.md | — |

---

## 4. Working with multiple agents (Antigravity, Codex, Claude, Gemini)

- **Shared brain = files in the repo**, not chat memory. This knowledge base is committed under `docs/integration/review-2026-09-16/` (10 documents). Every agent reads `00` and `08` before proposing work; `02` and `04` before touching `native_geometric/`.
- **Cards live in `docs/integration/cards/PX-*.md`**; an agent that wants to run something files a card (copy §5 template) and stops. Casey signs by appending `Signed: Casey <date>`; only then does execution begin.
- **Results go to `docs/integration/EVIDENCE.md`** (one row: card, date, artifact digest, metric, control, verdict, receipt path). Narrative goes in the card's `RESULT.md`. Nothing else is authoritative.
- **Division of labor suggestion:** Antigravity (agentic workflows) executes signed cards and drafts new ones; Claude reviews cards/results adversarially before signing (the fact-check pass in `09` is the template); Codex/Astra is retired from open-ended "proceed with the next step" mode.
- **No agent edits this plan or the cards' acceptance sections after signing.** Amendments are new cards.

---

## 5. Experiment card template

```
# Card <ID> — <title>
Owner (human): Casey        Drafted by: <agent>        Date:
Signed: <blank until Casey signs>

Hypothesis (one sentence, falsifiable):
Why this and not something else (link to 08 §):
Data: <source, size, sealing digest, who authored, opened-before? yes/no>
Matched controls: <list; each a refit/baseline of equal size on identical data>
Primary metric + threshold:            Secondary metrics:
Budget: engineering <d>, machine <h>, storage <GB>, wall-clock cap
Kill criterion (pre-registered):
Preservation: <what must not regress; how measured>
Deliverables: EVIDENCE row; RESULT.md; receipts path
```

---

*Sources: `08-outlook-and-roadmap-critique-2026-09-16.md` (root causes R1–R5, paths P1–P6), `02` (code facts), `03`/`04` (evidence and math), `05` (literature norms and baselines).*
