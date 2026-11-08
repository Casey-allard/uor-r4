# ORDER: one-model-end-to-end, claude-321

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `859621f33`.

## Goal

**Train (b64L) on D3 plus a `mw-balp/train` whose repeated replies are scored at most k times, and see whether it stops without collapsing.** This is claude-320's decision review (#3851):
- `mw-balp/train`'s short replies are what made (b64L) stop (claude-313, 315, 319);
- its repeats are what the model collapsed onto: 72.4% of its EOS targets close a reply seen more than once, and its top reply occurs 4,470 times;
- OASST2, the only more varied stored corpus, carries about 36 times fewer EOS per scored token.

So the lever keeps every mw-balp text as context and scores each distinct reply at most k times.

1. **A Rust tool, `reply-cap`,** under `docs/labs/one-model-end-to-end/reply-cap-2026-11-08/`. It reads a UORT split and writes a new split:
   - `tokens.u16` is copied byte for byte;
   - in `response_mask.u8`, the j-th occurrence of each distinct EOS-ending reply, in file order, keeps its mask if j ≤ k. Every later occurrence has its whole run set to 0. A reply is a maximal mask-1 run, as in `reply-census`. Replies that do not end on EOS are left unchanged;
   - `manifest.json` carries the input's `source`, `revision`, `licence` and `attribution`, the new `mask_sha256`, the input's shas, and a `reply_cap` block with k and the counts kept and masked.

   It uses integer arithmetic only, and any inconsistency is a refusal. `scale-probe mix` must accept its output as a root.
2. **The choice of k, by a rule fixed here:**
   - The sheet's mix is claude-319's layout: D3's four roots, then the capped split three times.
   - Let E(k) be the capped split's EOS targets. k is the smallest value in {1, 2, 3, 4, 6, 8} with 3·E(k) ≥ 212,968, which is claude-315's × 1 mix's mw-balp EOS count. That mix stopped at 177 and 122 of 256.
   - `reply-census` is run on each capped candidate. Its tables (E(k), distinct share, top reply) go into the sheet before the pick.
   - If no k in the set meets the rule, the sheet takes k = 8 and says so.
3. **The prep sheet** `docs/harness/prep/one-model-end-to-end-b64l-d3mwcap-2026-11-08.md`. It follows claude-318's sheet in everything but TRAIN: claude-301's (b64L) recipe, seeds 1 and 2, 16,384 steps, one RTX 5090 Vast pod per seed. It pins:
   - the mix's shas and its build;
   - the job, chain and seal scripts' shas;
   - the export, serve and score path;
   - a fresh, non-author FIT and the session's pre-run review.
4. **The run and the reading:** `m2-fresh256` served replies, scored as in claude-319, with the no-memory panel pair against (b64L-D3). claude-319's × 3 is recorded beside it.

**The bars are unchanged from claude-318's sheet:**
- stopping: at least 128 of 256 EOS stops at both seeds;
- collapse: the most common EOS-stopped reply occurs under 64 times at both seeds;
- the panel: 104 complete and 52 pairs;
- the held-out pair is not `behind`.

Claim check:
- **(a)** This is Track A of "Both in parallel" (#2028, 6715318106), under "pipeline first" (#2028, 6709947526). The run is judged by the served-reply score.
- **(b)** It changes only TRAIN. No bar moves.

## Cancel probe (run first)

1. **No open PR builds a reply-capping tool or a capped mix.** Open PRs: #3855 (this unit) and #3842 (claude-317, the scale-probe width slope).
2. **DeepSeek's deepseek-34 (#3854, merged 07:41Z) does not cancel this unit.** It found that the first reply token is never in the top 32 on DeepSeek's own M-World content-ratio checkpoint. That is a different trunk, corpus and panel. This unit's bars are about stopping and collapse in (b64L) on `m2-fresh256`. The first-token rank is not a bar here.
3. **The inputs are present:**
   - `mw-balp/train` in claude-314's checked local copy (`3dee858f…` / `b5e7352d…`), with its manifest's four provenance fields;
   - D3's four roots at their pinned shas;
   - `scale-probe mix` (bin `31610a00…`);
   - claude-318's job, chain and seal scripts;
   - `reply-census` (merged in #3851).
4. **Compute:** pod C (claude-317) is the one pod up, so two more stay inside 4 pods and $8/h. Expected spend is about $3, as in claude-319 ($2.98).
5. **Disk:** 5.3 GiB free on the laptop. The capped splits are 7.5 MB each, and the mix is about 150 MB.

**Result, 8 Nov 07:50Z:** 1 to 5 hold as written.

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

This unit touches only the first four paths.

## Done when

1. `reply-cap` builds offline and locked, and its tests pass. The tests use a synthetic split with repeated replies, a non-EOS run, k = 1 and k = 2 and a refusal. A mutation of the occurrence count or of the masked span fails them. Its output is accepted by `scale-probe mix`.
2. The sheet is committed with the k table, the mix's shas, its FIT and its pre-run review, before any pod comes up.
3. Both seeds run, are sealed to the store, and are scored. The REPORT gives the outcome row, negatives first, with one notebook row.

## Stop and ask if

Nothing waits on Casey (§5.1).
- **If the pre-run review fails:** the run does not start, and the REPORT says why.
- **If a seed is NOT_RUN for an instrument fault:** one repair and one rerun inside this unit.

Bound: one unit, two pods, $8 of spend.
