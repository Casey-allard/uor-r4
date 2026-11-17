# REPORT: written-state-count-model, claude-420 (the sealed per-split D87 token streams)

Lab: claude. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **Four departures from the ORDER** [claude]:
  - Records run in the plan's order (shard paths sorted, then line order), not in split-key order. That is the order claude-411's split and audit count in. It does not depend on argument order or thread count, and it avoids holding TRAIN (about 4 GB of ids) in memory to sort it.
  - The byte rule is "a record's targets decode to its text bytes minus its dropped bytes". The ingest counts dropped bytes in `text_bytes`, but no token holds them. So the plain rule in the ORDER's first draft would fail on any record with an unemittable byte. Dropped bytes are reported apart. On the real split there are none.
  - The negative "the novel rule over 12 tokens" cannot be planted as written, because `pack` asserts 13 ids. It was planted as the window shifted one token onto the target, which tests the same window.
  - `read_uort_tokens` was not linked. Its three conditions (the `UORT` magic, the count at bytes 8..16, and a payload of exactly 2·count bytes) were checked on every real stream with `head`, `od` and `stat`. The reference test proves the layout equals the ingest's.
- **My first wick job failed before it did anything** [claude]. The kit's `put` makes its destination a folder, so `tar` found a directory. That was an instrument error, fixed by the path; the second job is the run reported here.
- **One hand count in my own test was wrong** [claude]. DEV-tune showed 7 seen targets where I expected 6. A debug print showed the code was right: the frame gram "\nAssistant: t" also occurs in TRAIN, because `u1`'s answer began with "t", like `t3`'s. I changed the fixture's answer, not the expectation, and the fixture now says why.
- **The order test exercises only `plan`'s sort** [claude, from the review]. Both runs hand `write_streams` the same sorted plan. The streams-layer order is pinned by the reference test, whose reverse-join negative fails.

## What changed

- `docs/labs/d87-split-audit/instrument/src/streams.rs` (new) [claude]:
  - `write_streams`:
    - Pass 1 keeps DEV records whole and builds their gram sets.
    - Pass 2 tokenizes TRAIN shards in parallel into per-shard parts and marks DEV grams.
    - The parts are joined in shard order behind a UORT header equal to `Ingest::finish`'s.
    - The DEV splits are written with their novel classes.
  - `id_bytes`: the per-id byte table, with BOS, EOS and UNK at 0 and lengths over 255 refused.
  - `target_bytes`: the per-record byte check.
  - `novel_of`: claude-411's rule.
  - It refuses a shard under `--out` (canonical paths) and an existing `parts/` folder.
- `src/bin/d87-streams.rs` (new) [claude]: like `d87-audit`, it refuses shards that do not reproduce `split-summary.json`. It takes the tokenizer's sha256 into every summary.
- `src/lib.rs` [claude]: `Tok.dropped_bytes`, and `pub mod streams`.
- `src/tests.rs` [claude]: `coder_missing`, a tokenizer without chosen bytes, plus six tests:
  - streams = Ingest;
  - order and threads;
  - novel classes = the audit plus a hand count;
  - bytes and dropped bytes;
  - a wrong byte table refused;
  - the out and parts guards.
- Committed records [claude]: `run1/*.json` (counts and sha256 digests only) and `logs/`. No stream, mask, id list or corpus text is committed.

## Measurements

Wick, CPU only, 262 D87 shards read only from `/workspace/in/d87`, split v1 from `/workspace/out/c411/split`, tokenizer sha256 `d36d3e87…` [claude]:

| Split | Records | Tokens | Targets | Target bytes | Response targets | Dropped bytes |
|---|---|---|---|---|---|---|
| TRAIN | 519,580 | 2,091,560,001 | 2,091,040,421 | 4,258,944,309 | 1,835,726,031 | 0 |
| DEV-tune | 2,670 | 10,810,087 | 10,807,417 | 21,928,475 | 9,523,958 | 0 |
| DEV-test | 5,321 | 21,354,897 | 21,349,576 | 43,326,982 | 18,751,676 | 0 |

DEV novel classes (targets / bytes):

| Split | Seen | Novel | Short context |
|---|---|---|---|
| DEV-tune | 3,209,203 / 5,859,877 | 7,566,174 / 15,995,822 | 32,040 / 72,776 |
| DEV-test | 6,490,497 / 11,853,818 | 14,795,227 / 31,328,319 | 63,852 / 144,845 |

- DEV-test response targets: 14,190,847 novel (29,964,782 bytes) and 4,560,829 seen (8,030,634 bytes).
- Time: 376 s at 16 threads; 210 s at 7 threads for the second run, on a warm page cache.
- Digests: `dev-test.tokens.u16` sha256 `023784b7…`, `train.tokens.u16` `c6cba29e…`, `id-bytes.u8` `befb22f6…`. The full digests are in `run1/*.json`.

## What the numbers say

- The streams are the audit's streams [claude]:
  - Every count claude-411 took matches: DEV-test 5,321 records and 21,349,576 targets; 14,795,227 novel, 63,852 short and 14,190,847 novel response targets; TRAIN 2,091,560,001 tokens.
  - So bar Q's novel subset is defined on exactly the targets the audit counted.
- 69.3% of DEV-test targets and 72.3% of its target bytes are novel [claude]. The novel-subset gate on bar Q therefore covers most of DEV-test, not a corner of it.
- No D87 record has a byte the vocabulary cannot emit [claude]. The BPB denominator is the full text, about 2.03 bytes per target on every split.
- TRAIN is 2,091,560,001 tokens, under the counter's 2^31 bound by 55,923,647 [claude]. A whole-TRAIN count fits in one stream, but with about 2.6% headroom.

## Checks run at the head

- G1 [claude]: `cargo test --release` in the instrument gives 34 passed, 0 failed at 6672ce9f3 on the laptop, and 33 at e6eb7de01 on wick. The 34th is the review's guard test. Clippy has no finding in the new files; its two warnings are in existing `lib.rs` lines 600 and 777.
- G2: set-aside negatives, each failing, then restored and rebuilt to 34 of 34 [claude]:
  - TRAIN parts joined in reverse shard order: `streams_equal_the_ingest_writer` fails.
  - The novel window shifted one token onto the target: `novel_classes_match_the_audit_and_the_hand_count` fails on the seen count.
  - Special ids given their surface length: all five stream tests are refused, for example "targets decode to 63 bytes, but its text holds 56".
  - The review's guard fix set aside: `a_shard_inside_out_or_a_stale_parts_folder_is_refused` fails.
- G3, wick, job `c420-run-2` at e6eb7de01 [claude]:
  - All 15 output files are `cmp`-identical between 16 and 7 threads (`logs/cmp.log`).
  - The novel counts, record count and TRAIN tokens equal claude-411's audit (`logs/novel.log`).
  - Each stream has the `UORT` magic, a payload of 2·count bytes, BOS first and a count below 2^31 (`logs/uort.log`).
  - The review's fixes at 6672ce9f3 touch only refusals, so they do not change these bytes.
- G4: `UOR_LAB=claude uor-harness loop pr-check --local`: see the PR [claude].
- G5: only summaries with counts and digests, and logs, are committed [claude].
- Fresh review [claude]: one reviewer, given the ORDER, the declared departures and the diff, returned APPROVE-WITH-LOW and found no correctness defect. It found four LOWs:
  - A weak shard-inside-out guard. Fixed: canonical paths and a prefix check.
  - A stale `parts/` folder could be joined or left behind. Fixed: refused.
  - The library and the binary used different bloom bounds. Fixed: 6..=34 in both.
  - The order test exercises only `plan`'s sort. Accepted, because the reference test carries the order (above).

## Skills used

None invoked [claude]. I followed AGENTS.md: Rust only, the D87 copy read only, nothing written on `/mnt/plexpool`, and no corpus data committed.

## Still open

- claude-421 [claude]:
  - the counts filtered to DEV-test contexts;
  - a GPU scorer equal to claude-419's host scorer;
  - the bits instrument (`lawful_likelihood::bits_q12` of the Q48 masses, asserting that its `PROBABILITY_ONE` equals 2^48);
  - order-2 BPB equal to a CPU oracle on a 1% slice, from these streams and `id-bytes.u8`.
- claude-422: the sheet, FIT, N tuned on DEV-tune, and the sealed DEV-test run on all targets and on the novel subset [claude].
- The streams live at `/workspace/out/c420/run1` on wick (8.0 GB) [claude]. They are a rebuildable cache of D87, not an asset. Two runs reproduce every byte.

## Plain summary

The model and the baseline it must beat both need the same fixed copy of the corpus to learn from and be graded on [claude]. This unit writes that copy: the training, tuning and test portions as token files. It marks every test token as either seen in training (13 tokens of context) or new, and it records how many text bytes each token stands for. Two runs gave identical files. Every count agrees with the earlier audit of the same split, and no text byte is lost. About 70% of the test tokens are new, so the test is mostly about predicting text the model has not seen.
