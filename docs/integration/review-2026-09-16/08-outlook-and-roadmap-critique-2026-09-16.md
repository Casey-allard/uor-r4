# UOR-R4 — Outlook, Roadmap Critique and Research Attack Plan

**Date:** 2026-09-16 (rev. 2, after fact-check and Codex-log reconciliation) · **Basis:** the *live* GitHub repo `UOR-Foundation/uor-r4` — its HEAD on 2026-09-16 is `a5655e93` (PR #1280, 2026-09-14 20:07Z), which is exactly the revision analyzed; your local checkout (52 commits behind that HEAD); the Codex/"Astra" session logs and `.uor-handoff/2026-09-12-codex-v7/CODEX_RESUME.md` on your Mac (last write 2026-09-14 18:58Z — nothing newer exists locally); seven audit reports in this Project (`claude/01–07`); and the live literature. **Author:** Claude, at Casey's request. Nothing in the repo was modified.

Where I state a fact about the repo it is cited in one of the seven source docs. Where I give a judgement I say so. I have tried hard not to soften anything.

---

## 1. The direct answer: are we on the right track?

**The goal is legitimate and the engineering discipline is unusually good. The current *path* to the goal is not working, and — this is the important part — the evidence in your own repository already shows why.** Three findings carry the verdict:

1. **Nothing geometric has ever beaten a matched non-geometric control on any population larger than ~100 items.** Every Sept 13–14 experiment that ran the control reports `ExactIdentity == Full` — 19 of 19 records with an EI arm, including 2,304/2,304 on the latest panel (the other 27 shared-core/addressed-attention records have no EI arm). The code explains it: the "geometric read" is exact string match over a lossy hash (`leaf = prime % 120`, which maps the 256 bytes onto only 33 roots and collides a/v, c/x, d/y, f/z), and the 120-bit Hamming distance is provably a 9-level function of the quaternion angle — it carries the root id and nothing else. Zeta phases are a fixed sinusoidal encoding of *token rank*. None of this is wrong code; it is correct mathematics doing no predictive work.

2. **The learner cannot scale.** The retained model `15baec48` is a count-fitted, integer-quantized log-linear next-piece model over 26 hashed features plus an exact copy memory — a smoothed hashed n-gram store. The experimental core `60d19679` is a 63 KB symbolic program (greedy set-cover DNF rules over 18 hand-designed boolean features, count-argmax tables, ~600 gradient-trained 2-input LUT gates). The two other geometric learners in the tree are even smaller — the failed shared core has 2,810 categorical parameters (~2.4 KB) and the untrained LUT4 Hamming policy 49,216 bits (~6 KB). Training data is entirely authored templates (16 names, 4 verbs, 12 constructions); evaluation panels come from the same generators. Every new phenomenon (an interior "will" in a name) needs a new hand-designed feature, table, layer and template family — the README itself diagnoses this at lines 136–138. No component learns representations.

3. **No geometric-native artifact has ever produced free-running prose, a real-corpus perplexity, a complete-request M1 latency or any energy number.** The only systems in the repo that ever wrote readable text are ordinary float transformers with a cosmetic R4 head split (#1014/#1017) or SmolLM2 itself. The only multiply-free system with real-corpus numbers (the TLA/R4G1 table) generalizes at ~1.8% top-1 off exact context and cycles when free-running. The Sept 8 recovery withdrew every published efficiency and alpha claim because the "measurements" were constants.

So: **right destination, wrong vehicle, and a process that produces motion without progress** (210 PRs in 14 days, ~50 sealed PASS/FAIL gates in 46 hours, all on templates). The good news is that the repo contains a real asset nobody else has built with this rigor — an exact, versioned, causally-committed, auditable memory with provenance-bound artifacts — and the literature has moved *toward* you on exactly that pillar (Meta's Memory Layers at Scale, DeepSeek's Engram). The plan below is built around that.

---

## 2. What the project actually is today (one screen)

| Layer | Reality on 2026-09-14 |
|---|---|
| Goal | Frontier-quality local LM on M1-class hardware; no transformer; no mathematical matrix product at inference |
| Retained model | `15baec48` (17.4 MB): hashed-count next-piece predictor + exact copy/relation memory + shift-add arithmetic; reachable via CLI/HTTP/API/WASM; trained on <1 MB of TinyStories + repo Rust |
| Experimental core | `60d19679` (63 KB): 13 nested artifacts; exact-match reads, DNF rules, span copy; answers 2 questions over 4 supplied sentences; no product entry point; FAIL 2,016/2,304 on the first genuinely independent panel |
| Evidence | ~90 preserved negatives; no geometric advantage over a matched control on any population >~100 items beyond a +0.95pp tie-break (#953) (a handful of ≤100-item template positives exist, see `03` §2.5); zero prose, perplexity, latency or energy results for any geometric-native artifact |
| Process | Codex agents on `codex/*` branches, squash-merged via merge queue; 100% of Sept merges authored "Casey Allard"; issue bodies of 50–65k chars as decision records; `current-state.md` at 438 KB; cumulative model ledger 37.5 h |
| Team | You (owner/merger), Ari (`auser`, active Jul 18–Aug 10), Alex Flom (`afflom`, UOR-Framework/Lean side), Maura (`maurathat`, methodology issues in July), Ilya Paveliev (Hologram) |
| External standing | ~15 stars, 4–5 forks (live page, 2026-09-16); six Zenodo preprints with zero external citations; no independent replication or critique anywhere; the only rigorous critiques of the work are your own audit docs |

---

## 3. What is real and valuable (keep these)

- **Exact identity + versioned memory + causal commit + replayable witnesses.** `memory_runtime.rs`, `relation.rs`, `value_runtime.rs`, `snapshot.rs`, the nested-digest lineage in `training.rs::validate`. This is the strongest engineering in the repo and it maps directly onto the best-supported idea in current LM research (O(1) deterministic lookup memory beats dense/MoE parameters for knowledge; dense storage is ~2 bits/parameter).
- **Evidence discipline.** Frozen acceptance before data draw, sealed report roots, OS-entropy seeds, replace-with-control ablations, preserved failures, the Sept 8 self-audit. This is above most academic labs. It is currently being spent on the wrong questions.
- **The H4 (binary icosahedral, 120-root) exact tables and Z[φ] arithmetic.** Correct, verified, cheap. A legitimate *substrate* for a no-multiplier finite state — 6.9 bits per element. Not a linguistic theory.
- **The TLA/R4G1 integer kernel.** Genuinely multiply-free, allocation-free, 144k int ops/token, 77k tok/s single-thread. A real systems result at a stated (low) quality.
- **Your own honest negatives.** The unreleased arXiv rebuild v4 of "Fixed Geometric Routing" reports the Hopf-vs-permuted-router null (+0.13/−0.03 ppl) explicitly. That is a publishable, credibility-building result as it stands.

---

## 4. Root causes (why the last 10 weeks produced no capability)

**R1 — The "no matmul" contract bans the wrong thing.** As written it forbids *the linear map itself* ("including lookup/add contractions"). That excludes the one operation known to carry learned pairwise structure at scale (a bilinear compatibility `q·Wk`), while permitting hash lookups that carry none. It also excludes every published no-multiplier result (BitNet b1.58, MatMul-free LM, T-MAC) from being usable precedent, so you are competing against nothing and learning from nothing. Your actual goals are energy, no GPU, local, and understandable computation. Those are measurable in J/token, bytes/token and RAM, not in an operation taxonomy. **Decision needed (yours):** either (a) keep the strict ban and accept that no known learnable representation fits, or (b) restate the contract as "bounded integer/ternary accumulate, no floating point, no dense projections larger than N, measured ≤ X J/token on M1" — which is what BitNet/T-MAC-class systems already satisfy and which your uor-matmul could serve exactly. My recommendation is (b), with the H4/table machinery kept as the *state* substrate and ternary/integer linear maps allowed as the *compatibility* substrate. If you choose (a), Section 6 path P3 is the only route left and it must be run as a kill-or-continue experiment.

**R2 — Nothing learns the representation.** Token placement is `prime % 120`; compatibility is equality of hashed prefixes; roles are exact `(center,left,right)` word-id keys with an `unknown=64` sentinel. Standard ML learns O(x) from the output loss; here O is declared. The "role separability obstruction" in the README is a property of that hand-built funnel, not of finite arithmetic — the failing sentence *does* contain the information (position relative to the known verb "call", count of prior "will"s). Fixing it by adding keys multiplies the key space and starves each key of data; that is the treadmill the roadmap is on.

**R3 — Capacity is 5–6 orders of magnitude short.** A few KB of learnable circuit + 14–28 bits of geometric state per step (2 lanes in the language core, 4 in the shared core), trained by greedy set cover, count-argmax, exhaustive enumeration and — for the shared core — zeroth-order coordinate search over 120-way categoricals, is asked to approximate P(next byte | context) for English (~1 bit/byte, needing ~10⁹–10¹¹ distinguishable context classes). Exact memory can hold the *text*; the *conditional distribution* has to live somewhere. Your dense experiments prove the point: everything that ever "passed" a quality gate (#1014, #1017, R4RetainedLanguagePath) used floats and matmul.

**R4 — Template data in, template PASS out.** Train and eval come from the same generators; "development" panels are iterated against ("that panel was independent at first evaluation; it is now exposed development evidence"); preserved-output populations (6,688; 13,248; 19,968) are cross-products of a few dozen families and function as regression tests, not capability evidence. No non-geometric strong baseline (regex, n-gram, tiny transformer) has ever been run on the same template tasks — they would almost certainly score 100%, which means the tasks discriminate nothing.

**R5 — The agent loop optimizes for sealed gates, not for the objective.** ~15 PRs/day, each a bounded "Learn X in native geometric model" increment with a fresh floor; each FAIL is preserved and then circumvented by a redesign; the accumulated PASS list is a selected set. Decision records live in 50k-character issue bodies and a 438 KB state file that drift (ROADMAP.md still says "#1139 is the immediate priority" and CONTINUE.md still describes #1139's binding work as the immediate direction, after the Sept 12 pivot; RESEARCH.md stops at Sept 7; CONTINUE.md still cites a 22-second budget that is now 37 hours). Human judgement enters only at merge time. The Sept 8 incident — an agent "qualifying" alpha with nonempty-output checks and a hardcoded 3500 mW — is the predictable product of this loop, and the recovery caught it only because you looked.

---

## 5. Critique of the current roadmap (13 open issues under #820)

| # | Responsibility | Assessment |
|---|---|---|
| 01 #1139 | Contextual phrase/role binding | Template-binding increment on a funnel that cannot generalize (R2). Fine as a diagnostic; wrong as priority #1. |
| 02 #1140 | Shared state transitions, compositional emission | Its acceptance checklist (real Rust execution, causal intermediate state) is good, but it presupposes a learner that does not exist. |
| 03 #973 | Integrate the model and learn general prose | The whole architecture problem lives here in a 65k-char body (at GitHub's 65,536-char limit; the Codex restart note says it is being compacted) and no prose. This is the actual project; it should be split into a viability gate (Section 6, P3) and everything else deferred behind it. |
| 04 #962 | Durable identity-scoped memory | **Good and buildable now** — but on top of a real small model (P2), not on top of `60d19679`. |
| 05 #954 | Grounded correctness / abstention | Sound acceptance criteria; premature until there is a predictor. |
| 06 #955 | Multi-step reasoning | Same. Note `multi_step_reasoning.rs` is currently a hand-written DAG executor / Rust-template printer, not a model. |
| 07 #1088 | Executable Rust coding | Same; also the hardest capability on the list — every ≤4B open model remains far from frontier on SWE-class tasks after trillions of tokens. |
| 08 #963 | M1 latency/energy/memory | **Should be #1, not #8.** You have never measured a complete request. Until you do, "lower energy" is a hypothesis with no denominator. |
| 09 #964 | Serving/geometry/artifact guarantees | Good, but the guarantee to state first is the contract decision in R1. |
| 10 #1172 | Native API + WASM runtime | Reasonable engineering; already mostly exists; not on the critical path. |
| 11 #1173 | Pages Studio running the native model | Premature and a reputational risk: the Studio's origin repo (`uor-r4-wasm-chat`) advertises Qwen 2.5 transformers at "1,500+ tok/s" with an "E8 Geometric Cognitive Core" that is telemetry. Putting a template-matcher behind that UI invites exactly the audit that killed HRM's claims. |
| 12 #965 | Alpha release | Acceptance text is good. Keep it, and make every earlier item feed it with measurements. |

**Structural problems:** capabilities are ordered before viability; measurement is last; there is no kill criterion anywhere; there is no external benchmark anywhere; there is no matched non-geometric baseline anywhere; the plan does not say what happens if geometry keeps tying ExactIdentity (which it has on every panel so far).

---

## 6. Good paths to explore (ranked, with the first falsifiable experiment for each)

**P1 — Ground truth on your own hardware (1–2 weeks, no model work).** On the M1: run `bitnet.cpp` with BitNet b1.58 2B4T (open weights), `llama.cpp` with SmolLM3-3B and Qwen3-4B at Q4, and your own `15baec48` and TLA bundle. Record for each: bits-per-byte on one frozen, never-opened held-out set (e.g. a fresh TinyStories/Simple-Wiki slice you hash-split now and lock), tok/s for a complete request (load + encode + 256 tokens + persist), J/token via `powermetrics`, resident bytes/token. This single table replaces every efficiency claim the project has ever made and tells you the real bar: a bandwidth ceiling of ~22–27 tok/s for a 4B Q4 model on base M1 and ~1.0–1.4 J/token measured for a 12B model on M1; BitNet at ~0.4 GB is the no-multiplier incumbent. *Deliverable:* one pinned markdown table; becomes the denominator for #963 and #965.

**P2 — Make the exact memory a product (4–8 weeks; highest expected value).** Attach your versioned exact memory / copy / arithmetic substrate (`memory_runtime`, `relation`, `value_runtime`, session snapshots) as a retrieval + tool layer around a real small local model (BitNet 2B4T if you want to stay no-multiplier; SmolLM3/Qwen3 otherwise). The model composes; your substrate stores exactly, cites exactly, forgets on command, and does integer arithmetic exactly. Evaluate on grounded QA over user documents with abstention (that is #954 and #962, achievable now). This is the literature's current consensus architecture (Memory Layers at Scale; Engram; kNN-LM/RETRO lineage) and it gives the Foundation a *usable* local assistant in weeks, with your provenance story intact. *First experiment:* held-out grounded-QA accuracy and hallucination rate with memory on vs off, on documents the model has never seen.

**P3 — A real learner inside the contract, run as a kill-or-continue gate (6–10 weeks).** If you keep pursuing a native geometric predictor, it needs three things the code lacks: (i) a *learned* embedding of bytes/words into a large product of finite factors (e.g. 2I^k with k in the dozens–hundreds, or D≈4k–16k-bit VSA codes) instead of `prime % 120`; (ii) a *learned* pairwise compatibility that is not a class function of `g_i⁻¹g_j` — a 120×120 learned table per lane is 14,400 entries and is legal under even the strict contract; (iii) a differentiable offline surrogate (DLGN/DWN-style relaxation, Adam, then hard export) instead of coordinate hill-climbing. Then run the one experiment that matters: **held-out BPB on a real corpus versus learnable-bits, at 1M / 10M / 100M bits**, alongside a Kneser-Ney 5-gram and a tiny transformer trained on identical tokens. *Kill criterion, written down in advance:* if the BPB slope is flat against the n-gram baseline by 100M bits, the native predictor is retired and P2 becomes the product path. The best published analogue (Recurrent DLGN) reaches 5 BLEU on 16-token MT; you would be attempting something with a demonstrated 1–2 order-of-magnitude gap, so this must be time-boxed.

**P4 — Settle the geometry question with controls, before any more geometry (1–2 weeks).** For zeta phases: same-size *random* fixed phases. For H4 placement: random assignment of tokens to roots vs `prime % 120` vs learned. For the Hamming metric: replace with the 120×120 angle-class table (mathematically identical; if results change, something else is going on). Run each inside the existing evidence harness. If nothing separates from its random control, move primes/zeta/Hopf/E8 out of the critical path permanently and keep H4 only as an engineering substrate. This costs almost nothing and ends a year of ambiguity.

**P5 — The one place your geometry could be a genuine contribution: exactly decodable codes for VSA clean-up.** Binary VSA (XOR bind, majority bundle, permutation for order) is multiplier-free and mathematically clean, but every retrieval needs a nearest-neighbour "clean-up" step. Your Hamming-signature machinery, if built on real error-correcting codes rather than the 120-landmark bank, could make clean-up *exact* (syndrome decoding) — a small, honest, publishable systems result. *First experiment:* capacity (items reliably recovered) vs code length for exact-decode vs standard clean-up on synthetic binding tasks.

**P6 — Publish the honest results you already have.** The arXiv rebuild v4 (angular routing with the permutation null) is ready and more careful than the Zenodo abstract; post it. Annotate or retract Zenodo 10.5281/zenodo.20045543 ("Canonical Explorations"), whose citations and 70B/140B claims your own audit rejects and which carries a stranger's ORCID. Remove the "Confidential — Not for Distribution" SkyNetwork proposal from the public LFS tree. Credibility compounds; these three actions cost a day.

**Not recommended right now:** more Hopf/E8/SpiralCore/Clifford operators without a defined action on model state; more tropical/FMM/W33 work (correctly retired); more template panels as headline evidence; the Pages Studio integration (#1173) before P1–P3 have produced a number.

---

## 7. How to attach the research going forward (process)

1. **One benchmark, opened once.** Lock a held-out natural-text set now (hash-split, sealed, never iterated against) and one grounded-QA set for P2. Every headline number is BPB / accuracy on these, with the P1 comparators in the same table.
2. **Pre-register each experiment in ≤1 page** (hypothesis, control, metric, kill criterion, budget) *before* the agent starts; a human signs it. Agents execute; they do not choose the next experiment.
3. **Matched baselines are mandatory**: exact-identity / regex / n-gram / tiny-transformer on every task, same data. If the baseline scores 100%, the task is discarded, not the result celebrated.
4. **Templates are diagnostics.** Report template count and slot entropy; never lead with them; pair every synthetic number with a natural-data number.
5. **Treat the exact memory as contamination by construction.** Publish 13-gram overlap between eval sets and memory contents; use post-cutoff fresh data for grounded QA.
6. **Measure, don't estimate.** tok/s, `powermetrics` J/token, resident bytes/token, on named hardware and OS, against `llama.cpp` and `bitnet.cpp` on the same machine. Both BitNet papers *estimate* energy; be the ones who measure.
7. **Run the HRM audit on yourselves first**: replace the novel component with an equal-size plain baseline; check per-instance memorization; hidden set. Expect the community to do this to any claim you make.
8. **Shrink the paperwork to fit a human.** `current-state.md` (438 KB), 65k-char issue bodies and a 13-issue ordered queue cannot be held in anyone's head, including yours. Keep one two-page plan, one evidence table, one dated decision log. Reconcile ROADMAP.md/CONTINUE.md with project-track.md now (they contradict each other on the immediate priority). Bring your local clone up to origin/main and fix the LFS filter so `git status` stops showing 200 phantom modifications.
9. **Weekly human decision point.** Given ~15 agent PRs/day, one hour a week where you read the evidence table and decide continue/kill per path will do more than any new gate.
10. **The Project knowledge base (`claude/00–07`) is the shared brain.** The seven source docs are exhaustive and cited; this document is the plan. Future Claude sessions in this Project read them first; ask Codex to read `02` and `04` before touching the learner.

---

## 8. Decisions only you can make

1. **Contract:** strict "no linear map" (a) or "bounded integer/ternary, measured energy" (b)? Everything downstream depends on this.
2. **Product vs. research:** is the near-term goal a usable local assistant with exact memory (P2, weeks) or the native predictor (P3, months, high risk)? Both is fine if P3 is time-boxed with a written kill criterion.
3. **Primes/zeta/E8 in or out of the critical path** after P4's controls — decide by the numbers, not by attachment.
4. **Publication hygiene** (P6) — yes/no, and who does it.
5. **Process** — are you willing to move the human decision from merge-time to experiment-selection-time?

---

## 9. What I could not verify, and follow-ups

- **Codex / "Astra" local context — now verified.** After you connected your home folder I read `~/.codex/sessions` (937 rollout files), the master session `rollout-2026-09-12T08-56-35…` (173 MB, 87 user turns, 834 assistant turns, 2026-09-12 12:57Z → 2026-09-14 18:59Z), `~/uor-r4-worktrees/shared-geometric-core/.uor-handoff/2026-09-12-codex-v7/CODEX_RESUME.md` (18:58Z) and the sealed `independent-neighbor-1` receipts. **No Codex session, handoff file or worktree write exists after 2026-09-14 18:59Z**, and the last thing Astra did was merge PR #1279 (FAIL 2,016/2,304) and write the restart note whose "Next action" paragraph is byte-identical to `docs/integration/current-state.md`. PR #1280 (the README research map) followed at 20:07Z and is GitHub HEAD. So the live repo already contains the whole of Astra's latest work, and this document was built from that HEAD; the local `~/uor-r4` checkout (52 commits behind) was used only for git state and the LFS/stash observations. Two things the logs add that the repo does not: (a) your own turns in the session record the process pain directly — "you have been working on this one step for 6 hours", "I feel like we have been stuck on this one step for over a week", "are we trying to qualify this brand new mechanism on tests we had to develop 20 steps for before they worked?" — which is R5 in your words; (b) the Sept 12 pivot to Hamming-distance attention, the deep-research request and the JEPA question all originated from you mid-session, and the agent adopted each within the hour. The 13 "Astra takeover" prompts in `astra_user_messages.txt` (Sept 4–8) show the same pattern: each new window is told "do not stop at another general plan", which is exactly the incentive that produces bounded PASS gates instead of the viability experiment in P3.
- **Disk.** Your Mac's Cowork shell would not start ("Workspace unavailable") on every attempt; you confirmed the disk is full. The 173 MB master rollout, `thread_history_1.sqlite` (1.5 GB) and `logs_2.sqlite` (104 MB) under `~/.codex`, plus `~/uor-r4/target` and `.uor-handoff` evidence (~5.7 GB), are the obvious reclaim targets; I did not delete anything.
- **Live GitHub API** was proxy-blocked for the agents; issue bodies for the 11 pre-Sept-7 roadmap issues come from the in-repo census (`issue-reconciliation-2026-09.json`), and #1172/#1173 from page fetches. Star/fork counts (8/2) from the live page; open-issue count renders as 12–15 depending on the view.
- **Artifacts themselves** (`.uor-models/*.json`, `.uor-handoff/*`) are not in any clone, so component lists inside `15baec48` and actual training corpus sizes after Sept 4 are taken from the evidence JSON, not inspected.
- **Zenodo API** was down; all preprint metadata was verified through DataCite, ORCID and Scite instead.
- **Ari vs Alex:** the GitHub-state agent conflated them; the repo docs and commit history show Ari = `auser` and Alex Flom = `afflom`. Corrected in memory and in `claude/07`.

---

*Sources for every factual claim: `claude/01-current-state-record.md`, `02-architecture-as-implemented.md`, `03-results-ledger.md`, `04-mathematics-audit.md`, `05-literature-brief.md`, `06-external-corpus.md`, `07-github-live-state.md` in this Project.*
