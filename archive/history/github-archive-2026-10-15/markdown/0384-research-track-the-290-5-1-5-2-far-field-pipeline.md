# #384 — research: track the #290 §5.1/§5.2 far-field pipeline

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-03T17:21:05Z
- **Updated:** 2026-10-09T13:09:08Z
- **Closed:** 2026-08-03T17:40:07Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/384
- **Merged:** 2026-08-03T17:40:07Z
- **Merge commit:** 487a1c6e62d4e927d0c00db1e328d6b63d1a9c4c
- **Base / Head:** main / UOR-Foundation:issue-290-hmatrix-prototype
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Records the #290 §5.1/§5.2 far-field investigation as a tracked, reproducible
pipeline. Research artifacts only — no crate, runtime, or artifact changes.

## Why tracked

The original §5.1 analysis lived in an untracked `scratch/fmm_rank/` and was
destroyed by a working-tree cleanup, leaving a published result on #290 with no
reproducible pipeline behind it. The extractor survived only in a preserved
worktree. Nothing here goes back in `scratch/`.

## Contents

| file | role |
| --- | --- |
| `fmm_dump.rs` | extractor — re-induces the cover, dumps vectors/sigs/prototypes/members |
| `validate_51.py` | rebuilt analysis; reproduces §5.1's published figures as an acceptance gate |
| `hmatrix_proto.py` | §5.2 offline precision gate |
| `README.md` | reproduction status, including two known discrepancies |
| `RESULT-52.md` | the §5.2 negative, as posted to #290 |

## Reproduction status

The dump reproduces **bit-for-bit** — `artifact_kappa`, `corpus_kappa`, and
`cover_kappa blake3:67d957c3…` all MATCH, 362 regions, per_depth
[16, 30, 58, 98, 160], 159,658 observations. It exits non-zero rather than
emitting a usable dump if any check fails, and `validate_51.py` refuses to
analyze a dump whose `reproduction_verified` is false.

Analysis against the figures published on #290:

| figure | published | rebuilt |
| --- | ---: | ---: |
| admissible pairs, depths 1–5 (η=1.0) | 2 / 5 / 16 / 29 / 87 | **exact match** |
| r(1e-2) / r(1e-3) / r(1e-4) at n=1024 | 20.5 / 134.0 / 274.5 | 20.0 / 132.0 / 273.0 |
| growth exponents | 0.11 / 0.45 / 0.64 | 0.119 / 0.438 / 0.637 |
| null control | 57 / 198 / 287 | 58 / 198 / 287 |
| pairs in exponent fit | **37** | **79** — unreconciled |

Two rebuild details are written into the README rather than left implicit: `diam`
must be taken over **full** membership (computing it on the subsample shifts
admissibility and gives 27/85 instead of 29/87 at depths 4–5), and the
exponent-fit pair count does not match — the original filter is unrecoverable, the
exponents agree to ≤0.012, and it is recorded as an open discrepancy rather than
tuned until 37 falls out.

## §5.2 result

`RESULT-52.md` records the negative: measured against the optimal truncated SVD
(Eckart–Young, so an upper bound on any practical scheme), rank-20 operator error
is ~6%, not the ~1% the pre-registration assumed — `r(ε)` counts spectrum above a
threshold and was never an error bound. Structurally, the interaction block is
`V_A V_Bᵀ`, exactly rank ≤ D = 288 by construction, so the exact matvec already
factors in O((n_A+n_B)·D) and a far field buys a constant factor bounded by D
rather than an asymptotic win.

Scoped explicitly to that object: #356's scorer compresses graph prototypes
against emission residuals, which the associativity argument does not cover. The
reconciliation is on #290.

Related: #290, #356, #359, #361.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

