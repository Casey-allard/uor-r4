# ORDER: one-model-end-to-end, claude-307

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md). Base: `2a661d900`.

## Goal

The item's done-when reads:

> 1. **The census (laptop, counts only, CHARTER §4.5).** A standalone Rust instrument under `docs/labs/one-model-end-to-end/instrument/` must:
>    - resolve the TRAIN size conflict, and build the TRAIN reply index (exact replies and reply 8- and 13-grams);
>    - report for DEV, Fresh V2 and `m2-fresh256`: the share of reply n-grams in TRAIN replies, the share of prompt templates seen in TRAIN with values masked, and the 8-token key share;
>    - read only the manifest of the K16 fresh slice, and leave the slice unopened.
>
>    It also says which panel, if any, holds new templates.
> 2. **The overturn check (GPU, under a prep sheet with a fresh FIT and a pre-run review).** Score turn-memory-reply claude-194's (r), (r0), and (r) with its memory off.
>    - **Pinned in the sheet:** the checkpoints by store path, `SHA256SUMS` and step; the binary's sha; a lease and cost inside the caps.
>    - **The pod** also seals a float reference for the read's parity: claude-194's (r), with its memory off, at both seeds. That is every DEV row's writer and nearest pick, and its 64 greedy samples. The parity itself, kept sets and top-1, is `integer-hamming-read`'s step 5, on the checkpoint that item's prep sheet names; the first greedy divergence is done-when 3's integer-artifact measure (claude-205's decision review, on #3407).
>    - **Rows:** DEV's 3,406 later rows the reply table does not name, and its 32 unkeyed openings, judged pooled (3,438).
>    - **Band, fixed in advance:** `max(2·√(b+c), the two-seed spread)`, where b and c are the paired discordant rows.
>    - **The cells:**
>      - (r) ahead at both seeds, beyond the band: the memory recipe stays the training default;
>      - (r) behind at both seeds, beyond the band: a memory-dropout arm joins the training run;
>      - mixed (the seeds disagree, or either seed is inside the band): both arms train.
>    - **Descriptive only:** Fresh V2, and 64 greedy samples per arm scored against the census index.
> 3. **The protocol.** A prep sheet, with a fresh FIT, written before the training run of the model to ship. It holds:
>    - **Strata:**
>      - keyed rows;
>      - unkeyed rows;
>      - new-template rows, from a panel no decision has touched. Fresh V2 is never one.
>    - **Arms:** (r), memory-off, (r0), tables only, and a longest-suffix table over TRAIN.
>    - **Scores:**
>      - teacher-forced top-1, per stratum;
>      - free generation: the share of replies that are verbatim TRAIN replies, new n-grams for n = 4–8, distinct-1 and distinct-2, the repeated-n-gram rate, and the table hit rate under generation against teacher forcing;
>      - on the integer artifact: kept-set agreement, first divergence, and tok/s against context.
>    - **Decode rules:** greedy, and a Flock rank rule with the harmonic weights of `stack/flock.rs`.
>    - **The verdict rule, written in advance:**
>      - "memorized": the gains appear only on keyed rows, and generated replies are mostly TRAIN replies;
>      - "generalizes": the model beats (r0) and the longest-suffix table on unkeyed and new-template rows, with novelty near DEV's own replies.
>
>    The training run itself (ASSESSMENT §6, step 5) adds:
>    - the transport controls: frozen and identity SpiralCore routes, under the fixed backward;
>    - the read-by-meaning arms: the Flock read, and the type spin.
> 4. **Lead with tokens per second** (decode against context). One notebook row per result; each REPORT negatives first.

**Claim check.**
- **(a) The step:** the one track (Casey, #2028 6675163438), which is this item's done-when 3 training run on human text.
  - claude-306 (#3814) read row 6 of its outcome map: not met on D4 at these seeds, so the trunk keeps D3, and the next unit is the tokens lever on D3.
  - Casey approved this run in this chat on 7 Nov: "Run it, ~$9 (Recommended)" (recorded on #2028, 6709527956). The recommendation was (b64L) on D3 at 32,768 steps, 2 seeds, on RTX 5090s, after its own sheet, a non-author FIT and a D67 review.
- **(b) What it changes in the model:** the trunk's step count. If the tokens lever reads `ahead`, the trunk trains for 32,768 steps in place of 16,384, so the model that ships trains twice as long.

**The sheet:** [`one-model-end-to-end-d3-tokens-2026-11-07.md`](../../../../harness/prep/one-model-end-to-end-d3-tokens-2026-11-07.md).

## Cancel probe (run first)

Every check passes, or the unit is cancelled:
1. `uor-store ls results/claude/one-model-end-to-end-d3-tokens-2026-11-07` holds nothing, so no run of this sheet exists.
2. D3's inputs are in the store: `results/claude/scale-probe-2026-11-05/inputs/` (claude-301's job fetched them there), and D3's sealed accumulation runs at `results/claude/one-model-end-to-end-d3-accum-2026-11-06/runs-2/`.
3. The pin `ff5156bcc9539aa5bccd0319df3d96ee5dd5a71f` is an ancestor of `origin/main`.
4. No open PR or claim by another lab trains (b64L) on D3.
5. `uor-pod status` leaves room for two pods inside the caps (4 pods, $8/h).

**Result (7 Nov): every check passes. The unit goes ahead.**
1. `uor-store ls` returns a 404 (exit 1), so the path holds nothing.
2. Both are listed. `runs-2/s1` holds its `SHA256SUMS`, its list and a 134.6 MB tar. claude-301's local copy of the inputs, `local/claude/kept/results/claude/scale-probe-2026-11-05/inputs/`, holds `SHA256SUMS`, `d1`, `d2` and `d3`.
3. The pin is an ancestor of `origin/main` (at `f66f023d1`).
4. The only open PR on this item is this unit's draft, #3816.
5. 0 pods are running, at $0/h. There are 16 RTX 5090 offers, the cheapest $0.563/h.

## Files in scope

- `docs/labs/one-model-end-to-end/**`
- `docs/harness/prep/one-model-end-to-end*`
- `docs/loop/tasks/one-model-end-to-end/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`
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

## Done when

1. **The sheet is committed,** with a non-author FIT posted on #3407, and a D67 pre-run review of the job, chain, seal and watch scripts posted on #3407. Each one's findings are applied.
2. **The job's laptop gate tests pass,** and each mutation is caught.
3. **Both seeds of (b64L-D3) at 32,768 steps are trained, scored and sealed** to `results/claude/one-model-end-to-end-d3-tokens-2026-11-07/runs/s{1,2}/`, and read back. Every pod is down. A seed that cannot run is recorded as NOT_RUN under the sheet's Stop rule.
4. **The REPORT reads, negatives first:**
   - I-r and I-s;
   - the bar against D3's T2 and unigram, on all rows and on unseen rows, by sub-panel;
   - the tokens lever against claude-301's (b64L-D3) at 16,384 steps;
   - the mid-run checkpoint's reading (descriptive);
   - the outcome the sheet's map names, and the spend.
5. **One notebook row** in `docs/research/notebook/float-training.md`.

## Stop and ask if

Nothing in this unit waits on Casey (§5.1). These go to a decision review, recorded in the REPORT:
- **The spend passes the sheet's cap:** extend the cap only for a healthy run (the charter's extend rule), and send Casey one [claude, for Casey] note on #2028. Casey approved about $9.
- **The two seeds' pods cannot be had on RTX 5090s inside the sheet's price ceiling:** wait for an offer, and use no other card class without an amended sheet.
- **A change to the bar, the outcome map or the stop rule after any reading:** this needs an amended sheet and a fresh FIT round, never a quiet edit.
