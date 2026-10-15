# #1782 — Phase 0: MiniLM-teacher clause retrieval (static E8 tables) vs lexical baseline (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-06T04:16:17Z
- **Updated:** 2026-10-09T13:09:03Z
- **Closed:** 2026-10-06T04:17:50Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1782
- **Merged:** 2026-10-06T04:17:50Z
- **Merge commit:** b29abd1e9526d61a301068b202489681b0b0440d
- **Base / Head:** main / UOR-Foundation:claude/minilm-phase0-clause-index
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #820. Phase 0 asks one question: can an offline MiniLM teacher, turned into static multiplier-free tables, pick the right clause of a conversation log for a memory question? MiniLM (`sentence-transformers/all-MiniLM-L6-v2`, Apache-2.0) is used only as an offline teacher and comparator. Nothing here is served.

## Frozen decision: DROP
- Pre-registration: `docs/research/minilm-phase0/PREREG.md`, commit `c840d259`. It was pushed before any arm was scored.
- The owner-requested Hopf arm (H) was registered in the same commit. No arm had been measured at that point, so H is not exploratory.

| set | n | reachable | same-clause | L | M | S | S_raw | **E** | **H** | H_F | H_E8root |
|---|---|---|---|---|---|---|---|---|---|---|---|
| generator-dev | 376 | 77.9% | 20.5% | **55.3%** | 54.8% | 50.8% | 51.3% | **51.6%** | **49.7%** | 50.8% | 40.7% |
| v3 memory (descriptive) | 30 | 100% | 0% | 80.0% | 76.7% | 80.0% | 80.0% | 80.0% | 73.3% | 76.7% | 80.0% |

Both E and H score below 75% and below L, so each is DROP and the overall result is DROP.

E8 quantization of the centered static table:

| quantity | value |
|---|---|
| scale | 2/rms = 5.909 |
| global relative error | 0.134 |
| per-token cosine | 0.990 |
| max doubled coordinate | 27 |
| distinct block codes | 1.28 M (pair table ≈ 1.6×10¹² entries) |

What the result shows (details in `RESULTS.md`):
- **Segmentation limit:** the frozen clause segmentation caps any selector at 77.9% on generator-dev. In 20.5% of dev questions the expected value and a distractor sit in the same clause, so BUILD could not be reached under this segmentation.
- **Quantization is not the cause:** E8 quantization costs nothing measurable (E is within 1 point of float S).
- **Table realization does not hold at this scale:** the pair-table form of the integer inner product needs a bounded codebook, which Phase 0 did not test.
- **The Hopf fiber matters:** H scores 1.1 points below S (49.7% vs 50.8%). Retaining the fiber (H_F) recovers exactly that gap.
- **Post-hoc and descriptive only:** semantic arms beat lexical on `binding` questions (E 89.6% vs L 80.5%) but fail on `assistant_stated` questions (E 0/20). This is a hypothesis for a separate pre-registration, not a result.

## Code
- `crates/uor-r4-training/src/clause_probe.rs` (Rust only, no new dependencies) contains:
  - the BERT WordPiece tokenizer;
  - the 6-layer MiniLM forward with candle-core ops;
  - static-table distillation;
  - the frozen segmentation, stopword list, value matching and lexical scorer;
  - E8 quantization (reuses `b3_e8_codecs::e8_nearest`) with an exact integer cosine-order comparator;
  - the quaternionic Hopf S7→S4 map in the icosian Z-basis order (`h4.1..k, phi_h4.1..k`), with fiber and E8-root variants.
- `crates/uor-r4-training/src/bin/clause-index-probe.rs` provides `parity` and `run`. `run` claims and seals its report root, and it refuses any `conversational-v4` path.
- Results: `docs/research/minilm-phase0/RESULTS.md` and the sealed `run3/` (`report.json`, `rows.jsonl`, `manifest.json`).

## Checks run at this head
- `cargo test --release --offline -p uor-r4-training --lib clause_probe -- --include-ignored` passed 7 of 7 with `UOR_MINILM_DIR` set:
  - segmentation, including non-ASCII text;
  - value matching and hits;
  - lexical scorer and recency ties;
  - E8 round-trip bound and lattice membership;
  - integer cosine order and 240 roots;
  - Hopf base invariant under common right multiplication by a unit quaternion, lands on S4, fiber section reconstructs the pair;
  - MiniLM parity against the published sentence-transformers quickstart cosines (max deviation 4.5e-5) and expected token ids.
- `cargo fmt --check` reports no diffs in the touched files. `scripts/check_claim_wording.py` passes.
- The changed binary was exercised: run 3, 134 s on laptop CPU.

## Execution notes
- Two commits after the pre-registration changed implementation only, not rules:
  - a flattened matmul, for speed (parity unchanged);
  - an ASCII-only separator scan, after run 2 panicked on an "é".
- Attempts:
  - Run 1 was stopped before any arm was scored.
  - Run 2's partial root is preserved and was not read.
  - Run 3 is the sealed result.
- Generator dev: `generator=v2`, `dev_seed=1000003`, 300 dialogues, with `panels=` pointing at symlinks that exclude `conversational-v4*`. Its sha256 is `f501281c…c5c1`.
- `data/panels/conversational-v4*` was neither read nor evaluated.
- Storage: the teacher files are in cloud-store as `minilm-l6-v2-weights` and the local copy is in the Trash. The run roots are in cloud-store as `minilm-phase0-runs-20261006`. New laptop data is under 2 MB.

Not merged; awaiting exact-head review.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6009226039 — 2026-10-06T04:16:57Z — Casey-allard

Exact-head check at 2f47a9d8 (Claude lab): rustfmt clean on the 3 touched .rs files; `cargo test --release -p uor-r4-training --lib clause_probe` 6 passed, 1 ignored (MiniLM parity needs weights; author ran it: max deviation 4.5e-5 vs published sentence-transformers cosines); check_claim_wording exit 0. PREREG committed (c840d259) before scoring; v4 never read. Frozen decision DROP (E 51.6% / H 49.7% vs lexical 55.3%; clause reachability 77.9%, same-clause 20.5%). Merging as a recorded negative (offline-teacher probe only; nothing served). Approve at this head.


---
_Comment/review ids in this file: 6009226039_

