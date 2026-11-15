# ORDER: one-model-end-to-end, claude-388 (the hard-copy read, eval-only; D41's release recorded as D86)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Base: `origin/main` at `a8cf8e36d`, which holds claude-387 (#3991, squash `f88230fce`).

## Why this unit

On 15 Nov Casey asked for a five-architect review (machine learning, mathematics, networking, systems, platform) of the path to one end-to-end served vector model. All five reviews independently reached the same first diagnosis. claude-386's Flock read picks the asked fact first on 97% of rows (claude-387 REPORT). Its payload, however, is an equal mean of four kept picks, so nothing in it marks which pick was asked, and scaling it only makes it louder. The read needs a hard copy: the first pick's value only.

The review recommends testing that on the saved states before any training (the synthesis is in the session's shared notes, `architect-review-2026-11-15/SYNTHESIS.md`). After the review, Casey answered "Start the next unit: record your D41 release, then run the eval-only hard-copy read?" with "Yes, start it".

## Part 1: record Casey's D41 answers (same day, §5.1)

Casey's words, 15 Nov 2026, in this session. Each answers a question that quoted the choice.

- To "Release D41's one-time donor pass (distill-d1)?": "I would say yes release it, but why is it going to cost money? if we are running it locally on wick, wouldn't it be free?"
- To "may the donor give up to 4 alternative answers per prompt (still no probabilities or scores)?": "Allow up to 4 (Recommended)".
- To the weight-distillation variants: "Defer (b), ban (c)(d) (Recommended)". That is: (b) a one-time donor embedding-table import is deferred; (c) fitting layers or hidden states and (d) logit or distribution targets stay banned.
- To "Which donor?" (gpt-oss-120b needs about 65 GB and cannot run on wick): "gpt-oss-20b on wick (Recommended)".
- To "Keep that order?" (D41 item 3's driver math, then the ~1M-token pilot): "Drop driver math, keep pilot (Recommended)".

These are written as D86: a decision row, a change-log entry for §3.10, a PLAN owner-direction row, and a note on `m2-donor-corpus-generation` and `d41-driver-math-pilot`. CHARTER.md's text is not edited. §3.10 keeps "hard targets only". Up to four answer texts per prompt are still hard targets, with no probabilities, scores or ranks. The donor generation itself is a later unit, with its own sheet and FIT.

## Owner approval

Casey, 15 Nov 2026, in claude's chat. These are the answers quoted in Part 1, as the session's transcript recorded them:
- At 2026-11-15T10:43:04Z: "I would say yes release it, but why is it going to cost money? if we are running it locally on wick, wouldn't it be free?", "Allow up to 4 (Recommended)" and "Defer (b), ban (c)(d) (Recommended)".
- At 2026-11-15T10:53:02Z: "gpt-oss-20b on wick (Recommended)" and "Drop driver math, keep pilot (Recommended)".

After the review, Casey answered "Yes, start it" to "Start the next unit: record your D41 release, then run the eval-only hard-copy read?".

These answers approve the two owner-path edits that record the decision as D86:
- the D86 row in `docs/integration/DECISIONS.md`;
- the 15 Nov entry in `docs/rules/changes.md`.

No other owner path is touched, and CHARTER.md is not edited.

## Part 2: the hard-copy read, eval only

Add one eval-only setting to the float reference read (`vector_reads.rs`), off and skipped by default: **first-pick payload**. With it set, Flock's payload is `w ⊙` its first real kept pick's next-token point instead of the mean of up to four. Every other kind already reads one pick, so for those it changes nothing. Training and the default forward are unchanged, and a test pins that.

The probe (`recall_payload_probe`, `recall-payload-probe`) gains `--first-only`, which runs every scale with that setting. On wick it then runs on claude-386's two 16,384-step Flock-g states (`try-5`, `try-6`) at scales 1, 4, 16, 64, 256 and 1,024. It also runs on the two 4,096-step states (`try-1`, `try-3`), where the first pick is right on only 418 and 474 of 1,024 rows, as a reading of the scorer's limit.

## Sheet (§4.5: one GPU evaluation whose outcome decides the next unit)

- **Bar:** V2(b)'s. `R_ask` ≥ 0.90 in each far bucket (512 rows each) of both seeds' 16,384-step states, at some single scale ≤ 1,024.
- **Prediction registered** in claude-387 REPORT:109: "a first-pick-only payload at a scale where it dominates should decode to the asked value on most first-right rows".
- **Noise:** binomial at 0.90 on 512 rows, about 0.013 standard deviation. A pass needs both buckets at both seeds.
- **Controls:**
  - the same states with the mean payload, which is claude-387's own run reproduced (best `R_ask` 0.367 / 0.330 and 0.363 / 0.328);
  - the read cut (`R^{-v}`).
- **Seeds:** 2 (claude-386's B2 seeds 1 and 2).
- **Cost:** minutes of wick time, $0. No pod and no training.
- **Reading map** (fixed before any number is read; amended before the run by the FIT's findings):
  - **What the run decides.** The run decides the native read operator's write form. It does not decide whether that operator comes next, which it does under P, N1 and N2 alike. The first pick is already right on 1,000 and 990 of 1,024 rows, and the weight is almost all positive (claude-387). So at a dominant scale, `R_ask` ≈ 0.97 × the **decode-to-self rate**: how often a dominant `w ⊙ e_t` decodes back to `t`. P/N1 against N2 is the decisive split. The REPORT gives that rate (`pick_first` on the first-right rows) beside `R_ask` and does not present P as new evidence that the read learns.
  - **One scale.** A pass needs one scale shared by both seeds and both far buckets. A result whose lowest bucket lands in 0.875–0.925 is borderline and reads as D.
  - **P (pass):** the bar is met at one shared scale. The REPORT records that scale and its median `|p|/|x|`. The consequence names the gain the native operator must carry:
    - a fixed factor, when the passing `|p|/|x|` is below 1;
    - a replacing write, when the payload swamps the stream;
    - or a trained size, if a scale near the trained one passes.

    The hard copy becomes the native Q31 read operator's read (the review's step 2, D85). The float reference read is not trained further.
  - **N1 (the scorer's limit):** the bar is missed, but the decode-to-self rate is ≥ 0.90 at some scale, pooled over the 1,024 rows' first-right subset at each seed. The copy works, and the misses are the rows whose first pick is wrong. The next unit is still the native operator, with the scorer's training (hinge) as its open part.
  - **N2 (the place):** the decode-to-self rate (pooled per seed, as in N1) is below 0.90 at every scale where `|p|/|x|` ≥ 1. A single token's point added there does not decode to itself through `final_norm` and the nearest-table decode. The fix is the write's place or form (a replacing write, or after `final_norm`), and the native operator is specified with that write.
  - **D:** anything else, borderline results included, reported as read, with no fix named.
- **FIT:** a fresh-context non-author review on 15 Nov returned `TEST FITNESS: FIT`. Its findings 1, 2, 4 and 5 are folded in above. On finding 6, the default Flock forward is pinned against the mean reference by the existing `the_forward_matches_the_plain_reference_for_every_kind`.

## Native-space five items (SKILL)

None is met. This is a float reference diagnostic (D85) on saved float states: an inventory-class unit with no training run. Its result fixes what the native read operator must do.

## Cancel probe

Before the run: a first-pick payload measurement on `main` for claude-386's states. At base: none (claude-387 REPORT "Still open" item 4 lists it as not measured).

## Files in scope

- `crates/uor-r4-float-train/src/vector_reads.rs`
- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/bin/recall-payload-probe.rs`
- this folder
- `docs/research/notebook/float-training.md` (one row)
- `PLAN.md` (one owner-direction row)
- `docs/integration/DECISIONS.md` (D86 row)
- `docs/rules/changes.md` (D86 entry)
- `docs/roadmap/items/m2-donor-corpus-generation.md` and `docs/roadmap/items/d41-driver-math-pilot.md` (a D86 note each)

## Stop list

No training, no pod, no change to the default forward or to training. No donor model is downloaded or run in this unit. No edit to CHARTER.md.

## Done when

1. D86 is recorded in the four places named above, with Casey's words verbatim.
2. The new setting's tests pass on the laptop (host tests) and on wick (cuda `vector_read` tests). The default forward is bit-identical (a test).
3. `probe.json` exists for the four states, fetched with its sha256.
4. The REPORT names the reading map's letter with the numbers, and the notebook row is written.
5. The PR goes through a fresh exact-head review, the queue and `close --verify`.
