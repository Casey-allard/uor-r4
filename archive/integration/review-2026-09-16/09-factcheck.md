# 09 — Adversarial fact-check of `08-outlook-and-roadmap-critique-2026-09-16.md`

**Checked against:** repo clone `/home/claude/work/uor-r4` (main @ `a5655e935c3c…`, 2026-09-14 20:07Z, `git status` clean, up to date with origin/main — not modified), source reports `01`–`07`, live fetches on 2026-09-16 (raw.githubusercontent.com; Exa and Parallel page fetches of github.com; api.github.com and github.com direct were 403 via the session proxy). Only factual claims are checked; judgements/recommendations are not.

**Verdict key:** CONFIRMED = matches repo/source; IMPRECISE = directionally right, number/wording/attribution off; WRONG = contradicted; UNVERIFIABLE = not checkable from repo, sources or reachable web.

---

## A. Findings that need correction (WRONG / IMPRECISE), most important first

| # | Claim (outlook location) | Verdict | Evidence | Suggested correction |
|---|---|---|---|---|
| A1 | "8 stars, 2 forks" (§2 table, §9) | **WRONG** (or stale) | Two independent live fetches today (Exa, Parallel) of github.com/UOR-Foundation/uor-r4 both show **15 stars, 4–5 forks, 12 open issues**; source `06` §4.1 also says 15 ★ / 4 forks. Only the `07` addendum says 8/2. | "15 stars, 4–5 forks (github.com, 2026-09-16); 12 open issues." If 8/2 came from a different page (e.g. Casey's fork or wasm-chat), say which. |
| A2 | "Every Sept 13–14 experiment reports `ExactIdentity == Full`" (§1 ¶1) | **IMPRECISE** | Of 46 Sept 13–14 result records, **19** carry an ExactIdentity control (language_relation → independent_neighbor chain); the other 27 (shared-core ladder, addressed-attention, Hamming, typed one-read/adaptive/text/recurrent/ordered-state) have no EI control. In all 19 that ran it, EI = Full (grep of `docs/native_geometric_*_973.md`). | "Every Sept 13–14 record that ran an ExactIdentity control (19 of 46) reports EI = Full, including 2,304/2,304 on the latest panel." |
| A3 | "~300 gradient-trained 2-input LUT gates" (§1 ¶2) | **IMPRECISE** | `relational_attention/learning.rs:49-80` builds a scorer of 288 unary nodes + 287 pairwise AND-reduction nodes = **575 gates** (`FEATURES = 288`, `relational_attention/runtime.rs:11`), plus a 9-gate decoder, a 9-gate writer (`text_attention/learning.rs:58-62`) and a 16-gate query-update circuit (`dependent_attention/learning.rs:52-60`) ≈ **609 gates**. Source `02` §8 also says "~300". | "~600 two-input LUT gates (575 scorer + 9 decoder + 9 writer + 16 query-update)". |
| A4 | "Its [60d19679's] learnable geometric controllers total 2–6 KB" (§1 ¶2) | **IMPRECISE (attribution)** | The 2.4 KB (2,810 root params, `shared_core.rs:25-49`, recomputed: 1024+480+120+120+480+64+512+9+1 = 2,810) and 6 KB (49,216 bits = 3,076 gates × 16, `hamming_policy/policy.rs:11-15`) belong to **shared_core** and **hamming_policy**, two *separate* experiments (FAIL_CONTEXTUAL_TRANSFER_SMOKE / "initialized, no fit") that are **not** inside the 60d19679 artifact. | "The project's two learnable geometric controllers (shared-core 2.4 KB, LUT4 Hamming policy 6 KB) are separate failed/untrained experiments; 60d19679 itself is 63,640 bytes of rule masks, small tables and nested digests." |
| A5 | R3: "A few KB of learnable circuit + 28 bits of geometric state per step, trained by zeroth-order coordinate search over 120-way categoricals" (§4) | **IMPRECISE (conflation)** | Zeroth-order coordinate search over 120-way roots is the **shared_core** learner (`shared_core/training.rs`). The 60d19679 stack is learned by greedy set-cover (`relative_language/learning.rs:85-167`), count-argmax (`recurrent_text/learning.rs`), exhaustive 4×4 enumeration (`ordered_state/learning.rs`) and Adam on LUT relaxations (`relational_attention/learning.rs:242-263`). 28 bits = 4 lanes × 6.9 bits is shared_core's `LANES=4`; the ordered-state core uses 2 lanes (13.8 bits). | Name the learner per artifact, or say "the shared-core attempt". |
| A6 | "#973 … a 65k-char body with 30/31 subtasks 'done'" (§5 row 03) | **UNVERIFIABLE / unsupported by any source** | 65,316 chars ✓ (`docs/integration/native-update-receipt.json`). The in-repo body snapshot (`issue-reconciliation-2026-09.json`, 50,184 chars) contains **0** checkbox items and 0 occurrences of "done". No source report mentions 30/31. | Drop "30/31 subtasks done" or cite the live page fetch it came from. |
| A7 | "zero positive geometric advantages beyond a +0.95pp tie-break (#953)" (§2 Evidence row) | **IMPRECISE** | Source `03` §2.5 lists further bounded positives: T9 #1139 angular vs exact-code 62/62 vs 36/62, 40/48 vs 32/48, 12/12 vs 2/12 (n ≤ 100 template items), and `R4RetainedLanguagePathV1` Δ 0.0035 nats at equal params. §1 of the outlook correctly qualifies "population larger than ~100 items"; the §2 table drops the qualifier. | "…beyond a +0.95pp tie-break (#953) on any population > ~100 items; T9 template positives (n ≤ 100) exist." |
| A8 | ROADMAP.md **and** CONTINUE.md "still say '#1139 is the immediate priority'" (§4 R5, §7 item 8) | **IMPRECISE** | `ROADMAP.md:22`: "#1139 is the immediate priority." ✓. `CONTINUE.md:14-16` says "The current immediate direction is contextual lexical/prime and role phrase binding, then shared state transitions…" — #1139's topic, but it does not name #1139. | "ROADMAP.md says '#1139 is the immediate priority'; CONTINUE.md still describes #1139's binding work as the immediate direction." |
| A9 | "every ≤4B open model still fails SWE-class tasks after trillions of tokens" (§5 row 07) | **IMPRECISE (overstated)** | Source `05` §H: ≤4B models are "far from frontier on GPQA-Diamond, AIME, SWE-bench-class coding". No source says they "fail". | "…still score far below frontier on SWE-bench-class tasks". |
| A10 | "~25 tok/s and ~1 J/token for a 4B Q4 model" (§6 P1) | **IMPRECISE (minor)** | `05` §H: 22–27 tok/s is a *computed* bandwidth ceiling for 4B Q4 on base M1; the *measured* 1.0–1.4 J/token was for a **12B** model on M1 (ziraph.com, web-tier). ~1 J/token for 4B is an extrapolation. | Mark both as estimates to be replaced by the P1 measurement. |
| A11 | "the Sept 8 incident — an agent 'qualifying' alpha…" (§4 R5) | **UNVERIFIABLE (attribution)** | `recovery-2026-09-08.md` describes the false checks (PRs #1177–#1197) but does not name who authored them; all Sept merges are by Casey Allard; `07` shows 82% of Sept 8–14 PR heads are `codex/*`. Agent authorship is plausible but not recorded. | "…PRs #1177–#1197 (largely Codex branches) 'qualifying' alpha…" |
| A12 | "Ari = `auser` (author of the Rust-standards gist you adopted)" (§9) | **UNVERIFIABLE** | Ari = `auser` ✓ (`07` addendum; `06`; git author "Ari" 154 commits 2026-07-18 → 2026-08-10). No source report and no repo doc mentions a Rust-standards gist (`grep -ri gist` in AGENTS/CONTRIBUTING/docs → none). | Keep "Ari = auser"; drop or source the gist clause. |
| A13 | "a 256 GB drive that was at ~210 GB in August", "Cowork workspace failed to start", "local checkout 52 commits behind", "200 phantom modifications" (§Basis, §7 item 8, §9) | **UNVERIFIABLE** | Not in `01`–`07`. `00-project-brain-index.md:29` supports "52 commits behind" and "LFS filters not applied locally". Disk figures and workspace failure appear nowhere. | Label as session observations, not repo facts. |

---

## B. Numerically specific claims — CONFIRMED (with evidence)

| # | Claim | Evidence |
|---|---|---|
| B1 | Basis: main @ `a5655e93`, 2026-09-14 | `git log -1`: `a5655e935c3c3b82d2b9c0bb7986c04eb2cb8d1d 2026-09-14 20:07:27 +0000 … (#1280)` |
| B2 | `leaf = prime % 120` maps the 256 bytes onto only 33 roots; collides a/v, c/x, d/y, f/z | `training.rs:127-131` (`leaf: (prime % 120)`); token = byte+2 (`artifact.rs:132-134`), token i → i-th prime (`first_primes`, `corpus_induced_spin_placement.rs:2510`). **Recomputed in Python:** 33 distinct leaves for 256 bytes (35 for 258 tokens), max 11 bytes per leaf, letters → 22 leaves, collisions {61:[a,v], 77:[c,x], 83:[d,y], 91:[f,z]}. |
| B3 | 120-bit Hamming distance is a 9-level function of the quaternion angle | Signature bit = `ranks[r·l⁻¹] > 0` i.e. sign of Re(r·l⁻¹) (`addressed_attention/artifact.rs:101-109`); distance = popcount XOR (`hamming_refinement/metric.rs:57-73`). **Independently rebuilt 2I** (24 Hurwitz units + 96 even permutations of ½(±φ,±1,±φ⁻¹,0)): |G|=120, closed, 120 distinct weight-45 signatures, Hamming ∈ {0,22,38,44,58,66,76,88,90} at angles {0,36,60,72,90,108,120,144,180}° — exactly 9 values, monotone in angle. |
| B4 | Zeta phases are a fixed encoding of token rank | `training.rs:117-124`: `phase_j = round(γ_j·log(p_token/2)/2π · 65536)` with `p_token` = token-index-th prime; deterministic function of token index. (Nuance: stored as a 16-bit *angle*, not a sine value.) |
| B5 | 15baec48 = count-fitted, integer-quantized log-linear model over 26 hashed features + exact copy memory | `runtime.rs:12` `FEATURE_COUNT = 26`; `runtime.rs:513-528` additive integer score; `training.rs:254` "Count estimation is the learning rule"; `memory_runtime.rs`, `relation.rs:7` `RELATIONS = 16`. |
| B6 | 60d19679 is 63 KB / 63,640 bytes; 15baec48 is 17.4 MB / 17,421,561 bytes | `docs/evidence/native_geometric_independent_neighbor_973.json` → `verification.candidate.bytes = 63640` (sha256 `60d19679…`), `verification.normal.bytes = 17421561`. |
| B7 | 18 hand-designed boolean features; greedy set-cover DNF rules; count-argmax tables | `relative_language/runtime.rs:11` `FEATURES = 18`, `FEATURE_NAMES`; `relative_language/learning.rs:85-167`; `recurrent_text/learning.rs:197-218`. |
| B8 | Templates: 16 names, 4 verbs, 12 constructions | `relative_language/data.rs:24-32` (`TRAIN_NAMES: [&str; 16]`, `TRAIN_VERBS: [&str; 4]`), `:44` `CONSTRUCTIONS: [Construction; 12]`. |
| B9 | README diagnoses the observation-alias problem at lines 136–138 | `README.md:136` ("If two occurrences have the same observation `O(x) = O(y)`…"), `:138` ("…can share the same finite neighbor key"). |
| B10 | FAIL_INDEPENDENT_NEIGHBOR_TRANSFER 2,016/2,304; 288 failures all Exhausted; EI 2,304/2,304 incl. failures | `docs/native_geometric_independent_neighbor_973.md:3, 22, 28`; evidence JSON `gate`. |
| B11 | 2,304 = 8 shapes × 4 directions × 2 prefixes × 4 positions × 3 variants × 3 outcomes | `independent_neighbor_973.md:9`; 8·4·2·4·3·3 = 2,304 ✓ |
| B12 | 13 nested artifacts | Parent chain in code: query_participation → occurrence_role → correspondence → span → completion → scheduling → dependent_language::runtime → relative_language → language_relation → ordered_state → recurrent_text → text_attention → dependent_attention → relational_attention = 14 artifact structs, i.e. 13 nested ancestors. |
| B13 | 60d19679 answers 2 questions over 4 supplied sentences; no product entry point; trained/evaluated via `#[ignore]` env-var tests | `dependent_language/runtime.rs` shape limits; `grep` of `src/` and `uor-r4-api/src` for the experimental modules hits only `src/main.rs:2993` (unrelated `recursive_geometric_attention` probe); `dependent_language/mod.rs` `#[cfg(test)]` report modules; `independent_neighbor_report.rs:429-434` `#[ignore]` + `UOR_INDEPENDENT_NEIGHBOR_*` env vars. |
| B14 | `(center,left,right)` role keys with `unknown = 64` sentinel | `query_participation.rs:21-23` `UNKNOWN = 64, EDGE = 65, ANY = 66`; `occurrence_role.rs:19-21` (same value named `CONTENT`), `:518-524` `fn key`. |
| B15 | Sept 8 recovery: `m1_profiler.rs` hardcoded 3500 mW; M1/16 GiB hardcoded; nonempty-output "qualification"; >260×/>100×/0% withdrawn | `docs/integration/recovery-2026-09-08.md:3, 19, 27`. |
| B16 | 210 PRs Sept 1–14, 100% authored Casey Allard, ~15/day | `git log --since=2026-09-01 --until=2026-09-15 origin/main`: 210 commits, all author "Casey Allard", all with `(#NNNN)` suffix. |
| B17 | ~50 sealed gates in 46 hours | 49 evidence JSONs stamped 2026-09-12T20:53Z (shared_core) → 2026-09-14T18:51Z (independent_neighbor) = 45 h 58 min; `01` counts 51 incl. two no-run research records. A few are diagnostics/specs rather than PASS/FAIL gates. "~50" is fair. |
| B18 | ROADMAP.md "#1139 is the immediate priority"; RESEARCH.md newest heading Sept 7; CONTINUE.md cites 22.464 s; ledger now ~37.5 h | `ROADMAP.md:22`; `docs/RESEARCH.md` newest dated headings all `2026-09-07` (lines 12, 28, 51, 69, 83); `CONTINUE.md:24-25` "the prior 22.464-second remaining balance"; evidence `resources_snapshot.shared_ledger.cumulative_ms = 135,186,255` = 37.55 h. |
| B19 | `current-state.md` is 438 KB | `wc`: 3,109 lines, 438,471 bytes. |
| B20 | Issue bodies 50–65k chars | `native-update-receipt.json`: #973 65,316; #820 57,818 chars; census snapshot #973 body 50,184 chars. |
| B21 | `multi_step_reasoning.rs` is a hand-written DAG executor / Rust-template printer, not a model | File header "multi-step dependency construction (DAGs)…"; `execute_arithmetic_dag` (:98), `execute_transitive_relation` (:170), `intervene_intermediate` (:236), `generate_rust_code` (:345) emitting `fn main() {` literals; 386 lines, no learned parameters. |
| B22 | Shared-core 2,810 parameters; LUT4 policy 49,216 bits | `shared_core.rs:25-49` (LANES=4, ROOTS=120; offsets sum to 2,810 — recomputed); `hamming_policy/policy.rs:11-15` GATES=3076, CELLS=GATES<<4 = 49,216. |
| B23 | Serving contract forbids "lookup/add contractions" | `AGENTS.md:61` "no mathematical matrix products, including tabulated lookup/add contractions"; `project-track.md:13` "even if implemented as lookup/add contractions"; `README.md:19`. Quoted phrase is a close paraphrase. |
| B24 | #1014/#1017 are float transformers; #1017 sealed NLL 1.5728 fails <1.50 | `docs/r4_softmax_quality_capacity_continuation_1017.md:32, 90` (`1.5727521962806827`); `03` §1.8 (RMSNorm/RoPE/SwiGLU/softmax). |
| B25 | TLA/R4G1: 24.30% top-1 / 11.94 bits; ~1.8% top-1 off exact context; 144,496 int ops/token; 77,342 tok/s single-thread; cycles when free-running | `docs/smollm2_teacher_baseline_320.md:321`; `docs/project_baseline_audit_2026_08_18.md:468, 532` (1.81%/16.89); `docs/transformerless/BASELINE.md:251`; `COMPARISON.md:27`; `docs/free_running_eval_841.md` (cycle metric; `03`: 710‰ cycles, 99/100 suffix-identical). Note: 77k tok/s was measured in a Linux container, not on M1 (outlook does not claim M1 here). |
| B26 | #953 +0.95pp tie-break only | `docs/RESEARCH.md:1385, 2146` "+0.950392 percentage points (+4,242 correct)… changed only maximum-count ties". |
| B27 | ~90 preserved negatives | `03` §3 enumerates 88 numbered negative/withdrawn items. |
| B28 | Preserved populations 6,688; 13,248; 19,968 | `01` §3.1 rows 36–52; `independent_neighbor_973.md:60` ("492+300+200+6688"). |
| B29 | 6.9 bits per H4 element; 120×120 table = 14,400 entries | log₂120 = 6.907; 120² = 14,400. |
| B30 | 15baec48 trained on <1 MB TinyStories + repo Rust | `docs/native_geometric_recovery_973.md:39-40` (524,288 bytes: 127,231 B `prime_route_attention.rs` + 397,057 B TinyStories); `02` §3 (329,414 bytes read). The 2,097,152-byte "expanded" construction (`:167-175`) produced a separate 100 MB artifact, not the 15baec48 lineage (10.8 → 17.4 MB). |
| B31 | 15baec48 reachable via CLI/HTTP/API/WASM | `src/native_geometric_cli.rs`, `src/native_geometric_service.rs`, `crates/uor-r4-api/src/native_capability_api.rs`, `src/native_wasm.rs` exist (`02` §1.1). |
| B32 | Ari (`auser`) active Jul 18 – Aug 10 | `git log --author=Ari`: 154 commits, first 2026-07-18, last 2026-08-10. |
| B33 | Maura (`maurathat`) — methodology issue in July | `07` addendum: #234 (maurathat) closed; census: #234 "D3: the current Gate C evaluation distribution is answered entirely by exact-context lookup, so it cannot measure generalization", closed 2026-07-30; 1 maurathat commit 2026-07-27. |
| B34 | Codex `codex/*` branches, squash-merge, merge queue | `07` §4: 136/200 recent PRs from `codex/*`; gh-pages deploys by `github-merge-queue[bot]`. |
| B35 | uor-r4-wasm-chat README advertises Qwen 2.5 (0.5B) transformers at "1,500+ tok/s" with an "8D Gosset E8 Geometric Cognitive Core" that is telemetry | Fetched `raw.githubusercontent.com/Casey-allard/uor-r4-wasm-chat/main/README.md` (200, 10,071 B): line 6 "8D Gosset E8 Geometric Cognitive Core", line 29 "1,500+ tok/s", lines 32–36 five Qwen 2.5 / "GLM-5.3 Flash" 0.5B models, line 41 "E8 & Hopf Geometric **Telemetry**". |
| B36 | arXiv rebuild v4 reports Hopf-vs-permuted null +0.13 (PTB) / −0.03 (WT2) ppl; never posted | `git show HEAD:research/ai-research/ai-router/paper1_arxiv_rebuild_v4/generated/metrics.tex:10-11` (`\PaperPTBHopfPermDelta{+0.13}`, `\PaperWTTwoHopfPermDelta{-0.03}`); `main.tex:245`; `06` §5 (arXiv author/title searches → 0). |
| B37 | Zenodo 10.5281/zenodo.20045543 "Canonical Explorations": 70B/140B claims, stranger's ORCID, zero external citations; six preprints, zero citations | `06` §2.4 (DataCite ORCID 0000-0003-0047-1884 → Thomas Sanger), §2 Scite tally 0 for all six; PDF present twice in git index (`research/ai-research/deepermath/…`, `research/prime-analysis/…/authored-papers/…`). |
| B38 | "Confidential — Not for Distribution" SkyNetwork proposal in the public LFS tree | Git index: `research/ai-research/SkyNetwork-MUDBench Proposal/SkyNetwork_BusinessProposal.pdf` (LFS pointer, 178,803 B) and a second copy under `research/prime-analysis/…/mudbench-proposals/`; `06` scratch `skynet_media.txt` header "Confidential — Not for Distribution". |
| B39 | Owner adopted shared-core on Sept 12; memory repair parked | `docs/integration/current-state.md:551`. |
| B40 | `issue-reconciliation-2026-09.json` census 422 issues / 411 closed / 11 open; #1172/#1173 added → 12 responsibilities + tracker #820 = 13 | File present (`original_issue_census` 422 items); `ROADMAP.md`; `07` §2. Live open count today: 12 (Exa/Parallel). |
| B41 | Literature: Memory Layers at Scale (Meta), Engram (DeepSeek/PKU), ~2 bits/parameter, BitNet b1.58 2B4T open weights ~0.4 GB, Recurrent DLGN 5 BLEU on 16-token MT, HRM audit, both BitNet papers estimate energy | `05` §D (arXiv:2412.09764, 2601.07372, 2404.05405), §A.1 (2504.12285; "non-embedding memory 0.4 GB"; energy "estimated"), §C (arXiv:2508.06097: 5.00 BLEU, sequences truncated to 16 tokens), §B (ARC Prize HRM analysis). Not re-fetched this session; consistent with `05`'s verification protocol. |
| B42 | "there is no matched non-geometric strong baseline anywhere" / EI ties "on every panel so far" | `03` §5.4 ("No non-geometric strong baseline on the target task"); all 19 EI-bearing records tie (see A2). Note T9 did use exact-code/equality selectors as matched controls on ≤100-item panels. |
| B43 | Zenodo API down; verified via DataCite/ORCID/Scite; GitHub API proxy-blocked | `06` §0; `07` §0; reproduced today (api.github.com → 403). |

---

## C. Spot-checks of source-report citations against the repo (≥15 required)

| Source cite | Repo check | Result |
|---|---|---|
| `02`: `training.rs:130` leaf = prime % 120 | `training.rs:127-131` | ✓ (off by ≤3 lines) |
| `02`: `runtime.rs:446` 26 features; `:513-528` score | `runtime.rs:12, 446, 513-516` | ✓ |
| `02`: `relation.rs:7` RELATIONS = 16 | `relation.rs:7` | ✓ exact |
| `02`: `training.rs:254` "Count estimation is the learning rule" | `training.rs:254` | ✓ exact |
| `02`: `mixture.rs:411-415` SGD update | `mixture.rs:411-415` gradient/weights code | ✓ |
| `02`: `relative_language/runtime.rs` 18 features; `data.rs:24-32` 16/8/4/2 names/verbs; 12 constructions | `runtime.rs:11, 15`; `data.rs:24-32, 44` | ✓ exact |
| `02`: `hamming_refinement/metric.rs:57-75` popcount distance | `metric.rs:57-73` | ✓ |
| `02`/`04`: `addressed_attention/artifact.rs:101-109` signature; `:132-134` byte_leaf | `artifact.rs:101-109, 132-134` | ✓ exact |
| `02`: `query_participation.rs:34-45` artifact; `independent_neighbor_report.rs:428-434` env-gated | `:37 parent`, `:429-434` | ✓ |
| `04`: `shared_core.rs:39-49` parameter families; 2,810 | `shared_core.rs:25-26, 40-49`; recomputed 2,810 | ✓ |
| `04`: `hamming_policy/policy.rs:11-15` 4096→1024→256→1796, 3,076 gates | `policy.rs:11-15` | ✓ exact |
| `04`: Hamming has 9 levels {0,22,38,44,58,66,76,88,90} | independent Python reconstruction | ✓ exact |
| `04`: 33 roots for 256 bytes, 35 for 258 tokens, ≤11 bytes/root | independent Python | ✓ exact |
| `01`: `recovery-2026-09-08.md:14-23` table, `:19` 3500 mW, `:27` >260× | lines 19, 27 | ✓ |
| `01`: `current-state.md:551` owner adoption; 3,109 lines / 438 KB | line 551; `wc` 3109/438,471 | ✓ |
| `01`: `handoff-2026-09-07.md:316` "22.464 seconds remaining" | line 316 | ✓ exact |
| `01`: evidence `verification.candidate.bytes 63640`, `normal.bytes 17421561`, ledger 135,186,255/135,650,000 | JSON | ✓ exact |
| `01`: `README.md:136-138`, `:190-195` FAQ rows | lines 136, 138, 190-195 | ✓ |
| `03`: `smollm2_teacher_baseline_320.md` 24.30%/11.94; audit 1.81%/16.89; BASELINE.md 144,496; COMPARISON.md 77,342 | `:321`; `:468`; `:251`; `:27` | ✓ |
| `03`: `RESEARCH.md` +0.950392pp | `:1385, 2146` | ✓ |
| `03`: `…1017.md` NLL 1.5727521962806827 | `:32, 90` | ✓ exact |
| `03`: recovery corpus 524,288 B; expansion 1,385,121 B | `native_geometric_recovery_973.md:39-40, 167-169` | ✓ |
| `06`: v4 `main.tex` Hopf-vs-permuted +0.13/−0.03 | `generated/metrics.tex:10-11` | ✓ exact |
| `06`: SkyNetwork PDF path | git index (two copies) | ✓ |
| `07`: 210 merged PRs Sept 1–14, 100% Casey; Ari 154 commits Jul 18–Aug 10 | `git log` | ✓ exact |
| `02` §6: "~46k lines" experimental with no entry point (not in outlook; checked anyway) | recount: 61,500 lines in those modules incl. 18,947 lines of in-module `*report*.rs`/`*test*.rs` → ≈42.6k excluding drivers | ✓ approx (42.6k–61.5k depending on inclusion of report drivers) |
| `02` §8: "~300 two-input LUT gates" | recount 609 | ✗ **source `02` is itself imprecise** (see A3) |

---

## D. Claims stated as fact that no source report supports

1. "#973 … 30/31 subtasks 'done'" (§5) — see A6.
2. "author of the Rust-standards gist you adopted" (§9) — see A12.
3. "a 256 GB drive that was at ~210 GB in August"; "the Mac's Cowork workspace failed to start"; "`git status` … 200 phantom modifications" (§7 item 8, §9) — see A13. ("52 commits behind" is supported only by `00-project-brain-index.md`, not by `01`–`07`.)
4. "8 stars, 2 forks" — supported only by the `07` addendum and contradicted by `06` and by two live fetches today (A1).
5. Attribution of the Sept 8 alpha-qualification code to "an agent" (A11) — plausible, not recorded.

---

## E. Minor nuances (no correction strictly required)

- "Zeta phases are a fixed **sinusoidal** encoding of token rank": the artifact stores 16-bit phase *angles* γ_j·log(p/2) mod 2π (`training.rs:119-124`); "fixed deterministic phase encoding of token rank" is exact.
- "the 'geometric read' is exact string match over a lossy hash": correct; note the Full test is *coarser* than ExactIdentity (a/v etc. collide), so a Full≠EI disagreement is possible in principle on vocabularies with such pairs (`02` §5).
- "TLA … 77k tok/s single-thread": measured in a Linux container on a stories15M artifact at 31.7% teacher agreement (`COMPARISON.md:27`), not on M1 — the outlook does not claim M1, but readers may assume it.
- "13 open issues under #820": ROADMAP lists 12 responsibilities + tracker; live count today 12 open (Exa/Parallel), consistent with the outlook's "12–15 depending on the view".
- "~50 sealed PASS/FAIL gates": 49 evidence records; ~8 of them are diagnostics, specs or SELECTED_* research verdicts rather than PASS/FAIL gates.
- "trained on <1 MB": the retained lineage's base construction read 329–524 KB; a later 2 MB construction exists but produced a different (100 MB) artifact.

---

*Nothing in the repository was modified. Scratch computations: `/tmp/claude-0/-home-claude/ac8747b1-3a64-53b2-b520-ba33b2ebde57/scratchpad/check_geom.py`, `wasmchat_readme.md`.*
