# #338 — chore(transformerless): #327 re-pin fixtures to the 500k-corpus TLA7 era

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-01T21:53:24Z
- **Updated:** 2026-10-09T14:17:13Z
- **Closed:** 2026-08-01T22:07:15Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/338
- **Merged:** 2026-08-01T22:07:15Z
- **Merge commit:** fcd8595334e10655afcdd915971d57dc5935cb91
- **Base / Head:** main / UOR-Foundation:issue-327-corpus-500k
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #327. Adopts the 500k-token corpus as the pinned era, with the #318 Phase B residual wiring (TLA7) captured in the same pin.

**Era note (2026-08-01)**: corpus 150,000 → 500,000 tokens (2,507 stories, 100,306 held-out). New artifact: TLA7 container, 1,346,836 bytes, κ `blake3:ef6a20f385b9d1c78d3789cf5645f6a2ff5634056b1df4b53e52fc3034821fad`. Token-side pins byte-identical by construction; threshold/context-codebook/class-signature/container κs move with the corpus.

**Full certificate (macOS)**: teacher floor 1.4260 bits/token, ceiling 70.5%; **A (shipped) 34.7 / 39.0 / WB 8.0249 / 179,068 keys** (prior era 28.3/31.5); A-f32 35.7/40.3; P5 container witness byte-identical; long-context cert clean. Determinism: two independent compiles produced identical container bytes; **κ-reproduction 3/3 PASS** (checkpoint present, no vacuous skip).

**Files**: 4 fixtures (c_recs 1.8 MB → 24 MB: 12-byte legacy records → 48-byte; loader auto-detects) + `baseline_kappa.json` (certificate_macos rewritten, old record quoted in era_note) + era-sensitive assertion update in `src/tless_uor.rs` (depths [4,4,4,1] under residual-wired assignment, era-chain comment) + PROOF.md §P2/P5 era notes + BASELINE.md §3.1/§3.4.

**Gates**: workspace tests (76 suites), clippy `-D warnings`, fmt, no_std ladder, claim-wording — all clean. Gate C trend improved vs the 2026-07-30 pin (36.47%/8.3534 vs baseline 39.09%/8.4461, no alarm; pin bump left as a separate maintainer decision).

Full details in the #327 comments. C serving row skipped by design (no compiled bundle in the re-pin worktree — recorded, not silent).


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

