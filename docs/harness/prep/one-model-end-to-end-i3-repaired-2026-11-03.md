# Prep sheet: does a model trained as served now serve as trained? I-3 at the repaired head (one-model-end-to-end, claude-263, 3 Nov 2026)

TEST FITNESS: FIT - fresh non-author adversarial review (round 2) of this sheet and `serve-i3.sh`, after round 1's seven findings (round 1 was NOT FIT) were fixed. Round 2's four minor findings are fixed in this text: I-1 names #3579's nine files and how each hunk was read; the laptop tree must equal `0073cd07b` in its integer and tokenizer crates and Cargo.lock; `serve-i3.sh` refuses a seed whose score or snapshot exit code is not 0, and prints PASS only when I-5 also holds (step 6); and step 2 spells out the unpack mapping. No bar moved. (The wording of the third fix was corrected on 3 Nov, at the exact-head review, to match step 6; the fix itself is unchanged.)

PRE-RUN: PASS - fresh D67 review of `job-i3.sh`, `serve-i3.sh` and this sheet. Its one fix (E could read "files differ: 0" when both E runs failed) is in `job-i3.sh` SHA-256 `d350e94af43a8b3e71a2e593b83da2b8d295149390ce84c27497b75e38f4ed64`, the file launched. E does not run in this unit, since #3583 was not merged at claim time.

## Question

claude-261 found that its served-trained checkpoints and their integer exports gave different first tokens on 295–305 of DEV's 310 prompts, against a bar of 8. claude-262 (`fac0131bc`) then repaired the main defect in the float forward, a missing `PRODUCT_EXP`. It never ran the float-against-integer check on a trained export. This unit runs that check on claude-262's own served checkpoints. No new training.

## Base artifact

The checkpoints are claude-262's served runs, `r/served-s1` and `r/served-s2`, at `models/step-000256/`. They come from the seal `results/claude/one-model-end-to-end-served-repair-2026-11-03/claude-262.tar` (SHA-256 `d04505ac64af7c9f37c60c1e26b00943cad16b3bd36aebde37541da8cabb00e3`) and were trained at `4465685531a6bc02548466485c3a5b9a2201160f`.

`4465685531` was squash-merged as `fac0131bc`, and the two have identical trees for both crates.

**The scoring head is pinned to `0073cd07bd5d16021639041aa30c55132d2c6dc9`** (#3579's merge). Between `fac0131bc` and that head, the float and integer changes are claude-261's copy-key plumbing. The copy-key fields default to 1 and are omitted when they are 1, and `joined()` returns the code unchanged when `pair` is `None`. So none of them changes a copy-key-1 forward. I-1 checks this.

The pod builds `train-in-space` and `fbs-snapshot` at that head with `--locked --features cuda`. The laptop builds `fbs-export` and `fbs-toks` at the same head, in this unit's worktree. All four SHA-256 values are recorded.

## Readings

- **I-3, for each seed:**
  - the `fbs-toks` exit code;
  - order mismatches;
  - prompts whose first reply token differs between the float `samples-memory.json` and `fbs-toks --reply-tokens 64` on DEV's 310 prompts;
  - prompts whose whole reply differs.
- **E (support for #3583, run only if #3583 is merged at claim time):** E uses its own `train-in-space` build, made at #3583's merge commit and kept apart from the scoring build, so I-1 is unaffected. It runs `--score-rows` on seed 1's checkpoint twice, with `--batched-decode 0` and with `1`. The reading is a byte comparison of every samples file, plus both wall times. If #3583 was closed rather than merged, E is dropped.

## Bar

**I-3 passes for a seed when** `fbs-toks` exits 0, order mismatches are 0, and the first token differs on at most 8 of 310 prompts (2.9%, claude-261's bar).

**A pass at both seeds** means served training is exact through export. The full-length served run may then use I-3 as its door, and R6 (tok/s) and R7 (transcripts) can be read on a served-trained export.

**A fail** is classed instrument (§4.2), because a mismatch is a parity fault, not a model property. The next unit localizes it to one layer, with no new trace tool. It runs the existing `fbs-toks` and `--score-rows` on a truncated config. claude-262's two one-unit edges (follow-up 5) are named candidates.

E passes when every samples file is byte-identical. It is reported to DeepSeek on #3583 and decides nothing here.

## Instrument checks

- **I-0.** `cargo test --release -p uor-r4-float-train --features cuda --lib` passes on the pod. It includes `pair_halves_puts_the_first_token_in_each_copy_heads_low_half`, owed since claude-261.
- **I-1 (laptop).** `git diff fac0131bc 0073cd07b -- crates/uor-r4-float-train crates/uor-r4-integer` is the nine files of #3579:
  - `train-in-space.rs` (help text), `fbs_export.rs`, `flock_read_hamming.rs` and `in_space_long.rs` in float-train;
  - `fbs_snapshot.rs`, `stack/format.rs`, `stack/hamming.rs`, `stack/synthetic.rs` and `stack/tests.rs` in the integer crate.

  Every hunk of `in_space_long.rs`, `train-in-space.rs` and `fbs_export.rs` must read as copy-key only, with the default of 1, and the reading is logged in the REPORT.
  - It was read on 3 Nov, and that holds: `hamming_copy_key` defaults to 1 and is not written to `config.json` when it is 1, `with_copy_key(1)` changes nothing, and the other additions are a refusal, a test and help text.
  - `git diff 4465685531 fac0131bc` over both crates is empty.
  - Both runs' `config.json` carry `train_as_served: true` and no copy key.
- **I-2.**
  - The tar's SHA-256 matches `SHA256SUMS`, which sits beside it in the store (`d04505ac…00e3`).
  - Each unpacked member used here matches `MEMBERS.sha256`, which also sits beside the tar (`a698f9a3…65ff`). For example, `claude-262/r/served-s1/models/step-000256/model.safetensors` is `c8c8d378…c3d6`.
- **I-5.** Each export's header carries no `copy_key`.

## Data split

DEV, the tokenizer and their pins are claude-261's (its sheet's Data split):

| File | sha256 |
|---|---|
| `ft/dev/tokens.u16` | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` |
| `ft/dev/response_mask.u8` | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
| `tokenizer.json` | `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89` |

No panel is read.

## Steps

1. Build the binaries on one pod at `0073cd07b`, and record their hashes. Run I-0.
2. Fetch the seal, check I-2, and unpack:
   - `claude-262/r/served-sN/models/step-000256/` is the checkpoint `CKPT` for seed N;
   - `claude-262/r/served-sN/config.json` becomes `ROOT/served-sN/config.json`.

   Step 3 writes into the same `ROOT`: `served-sN-scored/`, `served-sN-snapshot.json`, and the exit codes `served-sN.score.exit` and `served-sN.snap.exit`.
3. Score the seeds one after the other, never two processes on the card (Casey, 3 Nov: "Arms in turn"). For each seed, run `train-in-space --data-root DATA --score-rows CKPT --out served-sN-scored --samples 1536 --reply-tokens 64 --retain-rates 1 --seed N`. This is claude-261's scoring command without the panels, so it covers DEV with memory on and out. Then run `fbs-snapshot CKPT served-sN-snapshot.json`. A run whose `samples-memory.json` does not hold 310 samples is an instrument fault, and I-3 is not read from it.
4. If #3583 is merged, run E after the scoring, also one run at a time.
5. Save every output with `SHA256SUMS` under `results/claude/one-model-end-to-end-i3-repaired-2026-11-03`, read it back, then take the pod down.
6. On the laptop, run `zsh serve-i3.sh ROOT TREE OUT`. It is claude-261's `serve-snap.sh` adapted to `served-s1` and `served-s2`, and its text goes into the REPORT. `TREE` is this unit's worktree, with `fbs-export` and `fbs-toks` built there with `--features serving-wasm`.
   - The script refuses unless `TREE`'s integer and tokenizer crates and `Cargo.lock` equal `0073cd07b`'s, and it records `TREE`'s head in `binaries.sha256`.
   - It refuses a seed whose score or snapshot exit code is not 0, or whose sample count is not 310.
   - It runs `fbs-export` with no raw flags (the snapped export), then I-5, then I-3, with the same fields and bar as claude-261.
   - It prints PASS only when I-5 also holds. It writes no transcripts.

## Minimum test power

There are 310 paired prompts for each seed (step 3 refuses any other count). Before the repair, the miss was 295–305 against a bar of 8, so a real repair and a remaining fault differ by two orders of magnitude.

## Cost and stop rule

**The estimate.** claude-261 scored one arm alone in 7,106–7,591 s. That covered DEV (310), p-w (1,202) and p-v (393), memory on and out. DEV is 310 of 1,905 prompts, so a DEV-only scoring takes about 0.16 × 7,350 ≈ 1,200 s.
- Two seeds in turn: about 40 min. With bootstrap, build and I-0, about 60 min.
- E adds seed 1 twice, about 35 min.
- The lease is 2.5 h on one RTX 5090 at under $0.80/h, at most $2.00, inside the caps. A watcher is armed on the job.

**The stop rule.**
- A build, I-0, scorer or `fbs-snapshot` failure stops that step. It is an instrument result, repaired in-unit or recorded.
- A resource fault (OOM, lost pod) is rerun once.
- A healthy run that outlasts the lease has its lease extended, never cancelled, and the extension is noted.
- The pod is never left idle, and it goes down once step 5's read-back matches.
