# UOR-R4 Project Knowledge Base — index (read this first)

Built 2026-09-16 at Casey's request from `UOR-Foundation/uor-r4` main @ `a5655e93` (2026-09-14), the local checkout on caseys-macbook-pro-local, the live literature, and Casey's public corpus. Purpose: a shared math / CS / engineering / SWE brain for the project so that humans and agents work from the same verified facts.

## How to use it
- Start with **08 (outlook)** for the current judgement, roadmap critique and attack plan.
- Before touching the learner, read **02 (architecture as implemented)** and **04 (mathematics audit)**.
- Before claiming any result, read **03 (results ledger)** §2 and §5 and **05 (literature brief)** §I and "Lessons".
- Before citing the project's own status text, read **01 (current-state record)** §7 — it lists the contradictions between README / ROADMAP / project-track / CONTINUE / current-state.
- Every number in 01–07 is quoted from a cited repo file or a verified external source; reviewer judgement is marked [Inference] / [ASSESS].

## Documents
| Doc | What it holds | Source report |
|---|---|---|
| `claude/01-current-state-record.md` | Dated chronology Jun→Sep 14 2026; artifact identities (15baec48, 60d19679) and lineage; every Sept 2026 bounded experiment with numbers and controls; resource ledger semantics and balances; the stated next action; governance (Codex/owner/protected PRs); 19 documented contradictions/gaps | A1 |
| `claude/02-architecture-as-implemented.md` | What the code computes on the serving path (retained model vs experimental core), learning algorithms with file:line, training data, scale limits, artifact schema, controls (ExactIdentity), dead code map, engineering quality, scalability assessment | B |
| `claude/03-results-ledger.md` | Complete ledger of 12 tracks (TLA/R4G1, router, S0–S4, mixer, GI, R4-softmax, grounding, Zoology, native, recovery, shared core, GPT-2 dense); what has ever been shown vs not; ~90 negatives; methodological assessment; what an external reviewer would demand | A2 |
| `claude/04-mathematics-audit.md` | Mechanism-by-mechanism audit (primes, zeta phases, R4/S3/Hopf, H4, Z[φ], E8=H4×H4, Hamming, r(i,j), SpiralCore, bundles, LUT4, shared core); computed facts (33 reachable roots; 9-level Hamming distance); cited-literature check; role-separability obstruction; capacity accounting; verdict and missing mathematics | C |
| `claude/05-literature-brief.md` | Sept 2026 state of the art: matmul-free/ternary/LUT inference, non-transformer models, logic/weightless networks, memory-augmented LMs, geometric/hyperbolic methods, VSA, expressivity theory, small local frontier on Apple Silicon, methodology norms; 15 lessons; ~90 verified citations | F |
| `claude/06-external-corpus.md` | Casey's six Zenodo preprints (rigor-assessed), Casey-allard repos, UOR-Foundation and afflom repos, what UOR is as a standard, arXiv "paper1" status, external reception, MUDBench/SkyNetwork/DARPA traces | E |
| `claude/07-github-live-state.md` | Live refs/tags/branches, 13 roadmap issue bodies (What/Why/Acceptance), PR authorship and cadence (Codex share), workflows, Pages site contents, sibling repos; access caveats; addendum with live page numbers | D |
| `claude/08-outlook-and-roadmap-critique-2026-09-16.md` | The judgement: are we on the right track; assets; root causes R1–R5; issue-by-issue roadmap critique; ranked paths P1–P6 with first experiments and kill criteria; process reforms; decisions for Casey; unverified items | synthesis |

## Memory
Durable project facts are also filed in the Project memory (`areas/uor-r4.md`, `areas/uor-r4-research-context.md`, `people/ari.md`, `people/alex-flom.md`, `people/maura.md`).

## Gaps (updated 2026-09-16 evening)
- Codex/Astra local context has now been read (`~/.codex/sessions`, `CODEX_RESUME.md`); nothing newer than GitHub HEAD `a5655e93` exists locally — see doc 08 §9.
- Local checkout `~/uor-r4` is 52 commits behind origin/main; LFS filters not applied locally (fix before any work: `git pull --ff-only && git lfs install`).
- Live GitHub API was blocked for agents; issue comments could not be read (GitHub page fetches confirmed #973/#820 have no comment threads).

## Also in this folder (added 2026-09-16 evening)
| `09-factcheck.md` | Adversarial fact-check of doc 08: 43 confirmed, 1 wrong (fixed), 8 imprecise (fixed) | verification |
| `10-execution-plan-agent-briefing.md` | The work order: agent ground rules, decision D0 (contract), pre-registered experiment cards P1–P6 with controls/budgets/kill criteria, roadmap re-sequencing, multi-agent protocol, card template | synthesis |

This folder is committed to the repo as `docs/integration/review-2026-09-16/` so Antigravity, Codex and Claude read the same brain.
