# ORDER: one-model-end-to-end, claude-320

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `7fea6c761`.

## Goal

**Survey the dialogue corpora (b64L) could train on, and name the most varied, before any mix sheet.** This is claude-319's decision review (row 4, #3848):
- At × 3, (b64L) on D3 + `mw-balp/train` stops at both seeds (234 and 204 of 256).
- Its stopped replies collapse onto one stock text (88 and 101, against the bar of 64).
- Every ratio that stops has collapsed at some seed.
- So the next corpus must carry more distinct replies per scored EOS.

The unit is laptop-only and counts only (CHARTER §4.5: no GPU, no training, no FIT needed):
1. **A Rust instrument, `reply-census`,** under `docs/labs/one-model-end-to-end/reply-census-2026-11-08/`. It reads a UORT split (`tokens.u16`, `response_mask.u8`, `manifest.json`). For each split it reports:
   - documents, tokens, scored targets and EOS targets;
   - replies: each maximal run of mask-1 positions, ending on EOS or not;
   - distinct replies by exact token sequence;
   - the top reply counts, and the share of scored EOS that close a reply seen more than once;
   - distinct reply 8-grams;
   - the median reply length.

   Against another split (`against=`), it reports the share of replies, and of reply 8-grams, that also appear there. Integer arithmetic only; any inconsistency is a refusal.
2. **The census,** on every candidate in the store:
   - `mw-balp/train`;
   - OASST2's `train/` from `results/claude/real-text-corpus-2026-11-05/`, which is Apache-2.0, split by source id, and screened against nine panels (that root's README);
   - against `m2-fresh256`'s replies and OASST2's own `heldout/`.

   Its JSON is committed beside the instrument.
3. **The reading:** which corpus, or which union, gives the most distinct replies per scored EOS, at what size. Its licence and generator must be recorded. The next unit's mix sheet takes this unit's pick, against (b64L-D3), with the stopping (128) and collapse (64) bars unchanged.

Claim check:
- **(a)** This is Track A of "Both in parallel" (#2028, 6715318106), under "pipeline first" (#2028, 6709947526). The next training run is judged by the served-reply score.
- **(b)** It decides only which corpus the next sheet names. It changes no bar.

## Cancel probe (run first)

1. No open PR builds a reply census or a dialogue-corpus mix sheet. Open PRs: #3851 (this) and #3842 (claude-317, scale-probe width slope). DeepSeek's deepseek-31 sheet (on `main`) repeats M-World in reply-SFT: a different lever, not this census.
2. The store holds `data/ft-balp.tar` and OASST2's `train/` and `heldout/` (`tokens.u16` 7.2 MB, `response_mask.u8` 3.6 MB, `manifest.json`), with the root's `SHA256SUMS`.
3. OASST2's licence is recorded: Apache-2.0, OpenAssistant (LAION), in the corpus root's README. mw-balp's licence and generator are not recorded (claude-314). The census says so beside its row.
4. The laptop has about 5 GB free. The census reads under 400 MB.

**Result, 8 Nov:** 1 to 3 hold as written. 4 holds: 5.0 GiB free.

## Files in scope

- `docs/labs/one-model-end-to-end/**`
- `docs/harness/prep/one-model-end-to-end*`
- `docs/loop/tasks/one-model-end-to-end/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`
- `crates/uor-r4-float-train/Cargo.toml`
- `crates/uor-r4-float-train/src/bin/packet-parity.rs`
- `crates/uor-r4-float-train/src/hamilton_sheet.rs`
- `crates/uor-r4-float-train/src/geometric_stack_native.rs`
- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `crates/uor-r4-float-train/src/flock_read_memory.rs`
- `crates/uor-r4-float-train/src/flock_read_hamming.rs`
- `crates/uor-r4-training/examples/m-world.rs`
- `crates/uor-r4-float-train/src/fbs_export.rs`
- `crates/uor-r4-integer/src/fbs_snapshot.rs`
- `crates/uor-r4-integer/src/stack/format.rs`
- `crates/uor-r4-integer/src/stack/hamming.rs`
- `crates/uor-r4-integer/src/stack/synthetic.rs`
- `crates/uor-r4-integer/src/stack/tests.rs`
- `crates/uor-r4-training/src/bin/m-world-recall-corpus.rs`
- `crates/uor-r4-training/src/bin/mix-chat-corpus.rs`
- `crates/uor-r4-training/Cargo.toml`
- `crates/uor-r4-integer/src/bin/fbs-toks.rs`
- `crates/uor-r4-training/src/milestone_world_v2.rs`
- `crates/uor-r4-cpu-train/src/fbs.rs`
- `crates/uor-r4-cpu-train/src/lib.rs`

This unit touches only the first three paths and the notebook.

## Done when

1. `reply-census` builds offline and locked, and its tests pass. The tests use a synthetic split with known replies, EOS ends, duplicates and an `against=` overlap. A mutation of the reply-run rule, or of the EOS count, fails them.
2. The census JSON for each candidate is committed, with each input's sha256.
3. The REPORT names the pick, negatives first, with one notebook row.

## Stop and ask if

Nothing waits on Casey (§5.1). If no candidate in the store is more varied than mw-balp, the REPORT says so. Its decision review then chooses between sourcing a new corpus, which must have a recorded licence, and a decode-time lever. Bound: one unit.
