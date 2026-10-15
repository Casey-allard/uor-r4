# #433 — Broad-corpus frontier: why the substrate caps at ~26-27% under topic diversity (successor to #399)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-05T16:19:11Z
- **Updated:** 2026-08-05T17:09:25Z
- **Closed:** 2026-08-05T17:09:25Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/433
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Successor program to #399 (falsifier-2 record complete there). Established facts: on broad-topic corpora (wiki10k and its 500k subsample) the substrate scores ~26-27% at EVERY measured capacity (starved 26.1, subsample 26.0, ratio-scaled 26.5 with a healthy 69-node cover); the fwd-anchor channel inverts regardless of capacity; EXCT precedence is mildly negative under diversity (Rule 1 27.1 > Rule 1+2 26.5); the 36.5-era positives are redundancy-regime records. Open questions: (1) what carries the remaining ~73% — is the ceiling evidence-sparsity (most held-out contexts genuinely novel at this corpus size) or representation (codes/cover can't separate diverse contexts)? (2) does the EXCT-precedence sign flip warrant a diversity-conditional Rule 2 gate? (3) capacity sweep shape: entropy-floor (R4_COVER_ENTROPY_GAIN_BITS) and depth sweeps are now cheap via #432's env knobs. First measurement candidate: evidence-sparsity audit — for held-out misses, does the construction split contain ANY matching evidence at any key (upper bound on what better representation could recover)?

## Comments (2)

### comment 5194742464 — 2026-08-05T16:55:05Z — Casey-allard

Evidence-sparsity audit ANSWERS the ceiling question — and re-attributes it (harness PR pending, run on both corpora):

**wiki10k, teacher-argmax target (gate C's target):** unigram floor of the TARGET STREAM itself = 26.82%; any-k exact-context oracle = 27.51% (+0.7pp). The measured ~26% sits AT the evidence ceiling. Root cause: **P(sampled-next == teacher-argmax) = 6.4% on wiki (vs 70.2% on the original corpus)** — the stories15M teacher is far off-distribution on broad text and its argmax stream is nearly degenerate (~27% one dominant token). The broad-corpus 'collapse' is substantially a TEACHER-SIGNAL artifact; representation cannot help and the corpus-composition finding is largely teacher-distribution mismatch. Match-rate collapse also real (k=3: 11.8% vs 68.2%) — both sparsity and teacher degeneration are in play. Observed-token target confirms: any-k oracle just 5.87% (sampled-next on wiki is mostly entropy at this temperature).

**Original corpus:** oracle any-k 52.7% / longest-match 45.2% vs measured 36.5% — **a real ~16pp representation gap exists in the home regime** (the recoverable prize for representation work: #422/#434/#435 now have a quantified target).

**Consequences:** (1) #320 (teacher upgrade) is the CRITICAL PATH for all broad-corpus work — exactly as that issue's own framing predicted; broad-corpus measurement should pause until a suitable teacher generates the corpus. (2) #435 stratification and #434 should first be graded in the home regime against the 52.7% oracle ceiling.

### comment 5194889960 — 2026-08-05T17:09:24Z — Casey-allard

Closing: the frontier question is ANSWERED (PR #437's audit) — the broad-corpus ceiling is the teacher signal (P(next==t_argmax) 6.4% on wiki vs 70.2% home; measured 26% = target-stream ceiling), and the home regime carries a quantified ~16pp representation gap (oracle 52.7 vs measured 36.5). Successors all tracked: #320 = critical path for broad-corpus work (escalated); representation prize = #422 (Ari)/#434/#435; stratification broad-corpus re-measure scoped on #435. No orphans.


---
_Comment/review ids in this file: 5194742464, 5194889960_

