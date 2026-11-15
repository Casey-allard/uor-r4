# REPORT: one-model-end-to-end, claude-387 (V2(b) follow-up: does the read's payload reach the answer?)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md). PR #3991. Base `origin/main` at `9747a3177`, which holds claude-386 (#3989).

Casey, 15 Nov 2026, in this session: "Yes, run the check" (the payload instrument check on claude-386's saved Flock-g states, before any change to the read).

This unit is an instrument check on saved float states (D85 reference code): no training, no bar, no milestone claim. It reads claude-386's four saved Flock-g states with an eval-only probe: the extension's step 16,384 at seeds 1 and 2, and the 4,096-step arms' last step at both seeds.

**What is now true, at the probe binary built from `37f3b8126`:**
- **The probe reproduces claude-386 to the count, so the instrument reads the same forward.** At payload scale 1 it gives the same held-out `ask` counts per far bucket (seed 1: 7 and 12 of 512; seed 2: 0 and 5; both 4,096-step states: 0 and 0). It gives the same first-pick counts (1,000, 990, 418, 474 of 1,024) and the same cut counts (0 and 2; 1 and 1).
- **The payload is small where it is added.** At the 16,384-step states, on the rows whose first pick is the asked fact, the median `|p| / |x|` is 0.026 at both seeds. That is `|p|` 1.36 and 1.21 against `|x|` 51.4 and 46.3, with a median cosine of 0.07 and 0.08. At 4,096 steps the ratio is 0.0017 and 0.0019.
- **The payload is diluted by Flock's mean.** On every query row of the 16,384-step states but one, exactly one of the four kept picks is followed by the asked value. No row has more than one, because the asked value occurs once per window, and no row has more than one on the other fact's. So two or three of the four point at neither fact, and the asked fact is a quarter of the payload.
- **Scaled up, the payload decodes to its own kept picks, and to the right one about a third of the time.** At 16,384 steps the best `R_ask` over the scales 1 to 1,024 is 0.367 and 0.330 (seed 1, ×16) and 0.363 and 0.328 (seed 2, ×64), against the 0.90 bar. At ×64 the decoded token is one of the four kept picks' next tokens on 1,011 and 946 of 1,024 rows. On the rows whose first pick is right, it is that pick's token on 346 of 1,000 and 339 of 990. The weight is nearly all positive: 9 and 2 of 256 coordinates are negative.
- **The reading map's letter is C by its stated test, but its meaning, a wiring or decode fault, is not what the probe finds.** At 16,384 steps `R_ask` stays under 0.90 at every scale while `|p|` passes `|x|` at ×64 (median ratio 1.69 and 1.65), which is C's test. C's reading is "the payload does not decode to the asked token". The decode-on-kept-pick reading below was added after the first probe read, outside the fixed map, and it finds no fault: the payload decodes to the tokens its picks point at, and the picks' indices are right. What it shows is B's mechanism: a payload too small at scale 1, diluted by an equal-weight mean of four picks. B's third clause fails as written ("scaling lifts `R_ask` less than it lifts the margin"), because the median margin falls past ×4 while `R_ask` rises (see Measurements). At 4,096 steps the letter is D: `|p|` reaches only 0.11 and 0.12 of `|x|` at ×64, and `R_ask` stays under 0.24 up to ×1,024.
- **claude-386's negative stays a configuration negative, not an instrument result.** C's consequence (re-reading it as an instrument result) does not follow, because the probe reproduces its numbers and finds the read wired as designed.

## What did not work, first

1. **No payload scale passes the bar.** Scaling the saved payload by up to 1,024 lifts `R_ask` at the 16,384-step states from 0.000 to 0.023 to at most 0.367 per far bucket, against 0.90. Scaling alone cannot get there: the decode picks among the four kept tokens and lands on the asked one about a third of the time. This is a reading of these two saved states, eval only; no fix ran.
2. **The fixed reading map did not separate dilution from a fault.** Its C test (`R_ask` under 0.90 at every scale while `|p|` dominates at ×64) holds here, yet the cause is the read's average and not a wiring fault. Its B test asked the asked token's margin (its score minus the best other's) to rise faster than `R_ask`. Past ×4 the best other token is another kept pick sitting near the asked one, so the median margin falls (seed 1: −0.00029 at ×1, −0.00018 at ×4, −0.00131 at ×64). I added the decode-on-kept-pick reading and the weight's sign census after the first probe read, to classify instrument against mechanism (no technicality ruling); they are labelled in the code and in `probe.json`. The first job's `probe.json` (scales 1 to 64) has neither; the second job's (scales 1 to 1,024) has both, and the numbers they share are equal.
3. **The 4,096-step states read almost nothing at scale 1.** Their weight's mean size is 0.37, and `|p| / |x|` is 0.0017 and 0.0019. Their first pick is right on 418 and 474 of 1,024 rows, so even at ×1,024 their `R_ask` is at most 0.236.
4. **Not measured:** a payload of the first pick only, or picks weighed by score, or a payload after `final_norm`. Those are fixes, each its own unit with a sheet (ORDER). The payload's share of the decoded point after `final_norm` is not read directly; the decoded margin stands in for it.

## What changed

- **`crates/uor-r4-float-train/src/vector_reads.rs`:** an eval-only probe.
  - `probing_payload` records each row's `|x|`, `|p|`, `x · p` and its real kept picks' next tokens (`PayloadRow`) while it is open.
  - `with_payload_scale` multiplies the payload by a positive, finite scale and skips the multiply at the default 1.
  - `weight_values` gives the weight's coordinates.
  - The forward computes the same tensors in the same order. The added row is now named (`added`) before `flat.add`, and `next_ids` is split out of the payload's `index_select`.
  - Tests: `the_payload_probe_reads_the_forward_and_leaves_it_bit_for_bit` (cuda) and `the_payload_scale_and_probe_restore_what_they_found` (CPU).
- **`crates/uor-r4-float-train/src/in_space_long.rs`:** `recall_payload_probe` and its helpers. It loads a saved checkpoint and its reads under the run's flags, rebuilds the held-out two-class windows from the seed (`recall_query_window`, as `run_recall` does), and decodes every query row with the payload at each scale and with the read cut. It writes summaries per scale over all rows, the first-right rows and the rest, plus every row.
- **`crates/uor-r4-float-train/src/bin/recall-payload-probe.rs`:** the command, with `--checkpoint DIR --run DIR --out FILE [--scales …]`. It reads the seed and trunk flags from the run's `config.json` and the header and held-out count from `recall.json`, refuses a run that is not two-class, and never writes over an existing file.
- Training and the default forward are unchanged; the cuda test pins the forward bit for bit inside a probe and at scale 1.

## Measurements

wick (Casey's RTX 4090, `sm_89`), container `uor-r4-claude`, alone on the card. The inputs are claude-386's saved states, copied as one tar (sha256 `023adc72c238439db48bb61a576fcd4f58de7788aa9ff83c9ec376c416ac5314`, the same on both ends). Each probe took about a second. Held-out: 256 two-class windows per seed, header 4, 1,024 query rows (512 per far bucket). Scores are the nearest-table decode `recall_picks` makes: the negative mean L1 displacement of the decoded point to each table row.

**At the read, scale 1, on the rows whose first pick is right** (medians):

| State | first right | `\|x\|` | `\|p\|` | `\|p\| / \|x\|` (to its p90) | cos(x, p) | weight mean `\|w\|` (negative of 256) |
|---|---|---|---|---|---|---|
| 16,384 s1 | 1,000 | 51.4 | 1.36 | 0.026 (to 0.031) | 0.073 | 3.72 (9) |
| 16,384 s2 | 990 | 46.3 | 1.21 | 0.026 (to 0.031) | 0.085 | 3.47 (2) |
| 4,096 s1 | 418 | 62.7 | 0.109 | 0.0017 (to 0.0023) | 0.112 | 0.37 (1) |
| 4,096 s2 | 474 | 52.3 | 0.099 | 0.0019 (to 0.0024) | 0.112 | 0.36 (0) |

**Kept picks against the facts** (rows by how many of the four kept picks are followed by the asked value; by the other fact's):

| State | asked: 0 / 1 | other: 0 / 1 |
|---|---|---|
| 16,384 s1 | 0 / 1,024 | 756 / 268 |
| 16,384 s2 | 1 / 1,023 | 770 / 254 |
| 4,096 s1 | 171 / 853 | 663 / 361 |
| 4,096 s2 | 160 / 864 | 660 / 364 |

**By payload scale.** `R_ask` per far bucket (of 512); `kept` = the decoded token is one of the kept picks' next tokens (of 1,024); `first` = the decoded token is the first pick's, on the first-right rows; `ratio` = median `|p| / |x|` on those rows; `margin` = the asked token's median margin there.

| State | Scale | `R_ask` | kept | first | ratio | margin |
|---|---|---|---|---|---|---|
| 16,384 s1 | 1 | 0.014, 0.023 | 60 | 18 / 1,000 | 0.026 | −0.00029 |
| | 4 | 0.279, 0.229 | 704 | 254 | 0.106 | −0.00018 |
| | 16 | **0.367, 0.330** | 965 | 349 | 0.423 | −0.00040 |
| | 64 | 0.363, 0.328 | 1,011 | 346 | 1.69 | −0.00131 |
| | 256 | 0.332, 0.256 | 1,023 | 293 | 6.77 | −0.00237 |
| | 1,024 | 0.309, 0.232 | 1,019 | 270 | 27.1 | −0.00245 |
| | cut | 0.000, 0.004 | | | | −0.00041 |
| 16,384 s2 | 1 | 0.000, 0.010 | 20 | 5 / 990 | 0.026 | −0.00027 |
| | 4 | 0.088, 0.090 | 245 | 87 | 0.103 | −0.00019 |
| | 16 | 0.258, 0.238 | 781 | 242 | 0.413 | −0.00021 |
| | 64 | **0.363, 0.328** | 946 | 339 | 1.65 | −0.00061 |
| | 256 | 0.314, 0.273 | 963 | 289 | 6.61 | −0.00117 |
| | 1,024 | 0.285, 0.250 | 916 | 261 | 26.4 | −0.00137 |
| | cut | 0.002, 0.002 | | | | −0.00032 |
| 4,096 s1 | 1 | 0.000, 0.000 | 4 | 0 / 418 | 0.0017 | −0.00444 |
| | 64 | 0.068, 0.061 | 353 | 36 | 0.112 | −0.00250 |
| | 256 | **0.201, 0.205** | 1,007 | 107 | 0.447 | −0.00344 |
| | 1,024 | 0.197, 0.209 | 1,022 | 106 | 1.79 | −0.00650 |
| 4,096 s2 | 1 | 0.000, 0.000 | 3 | 0 / 474 | 0.0019 | −0.00436 |
| | 64 | 0.039, 0.033 | 149 | 23 | 0.120 | −0.00293 |
| | 256 | 0.213, 0.227 | 947 | 123 | 0.481 | −0.00228 |
| | 1,024 | 0.199, **0.236** | 1,023 | 130 | 1.93 | −0.00468 |

The 4,096-step states at ×4 and ×16 are in `probe.json` (`R_ask` at most 0.008). Both 16,384-step states read 186 and 168 at ×64; their held-out windows differ (seed 1's slot rows `[134, 141, 110, 127]`, seed 2's `[125, 121, 138, 128]`), so the match is a coincidence of counts, not one file read twice.

**Results:** `local/claude/claude-387/results/c387/` (job 1, scales 1 to 64) and `c387b/` (job 2, scales 1 to 1,024, with the added readings). Each holds `probe-{ext,arm}-s{1,2}.json` and a `SHA256SUMS` checked on the laptop against the container's. Job 2's files: `ext-s1` `2b8881a9…08876`, `ext-s2` `2a10704b…67459`, `arm-s1` `580a3501…05b9`, `arm-s2` `022a7130…be401`. The kit (`kit/job-387.sh`, `kit/job-387b.sh`), both job logs and both result trees are in `c387-results.tar.gz` (sha256 `4b97c0e04620114796357d75eec2e6a5d37257898c16c0d06dec12fad77ce6e5`), uploaded to the HF store at `results/claude/one-model-end-to-end-payload-probe-2026-11-15/`.

## Checks run at the head

- **Laptop (`aarch64-apple-darwin`), at `37f3b8126`:**
  - `cargo test -p uor-r4-float-train --lib vector_reads`: 9 passed.
  - `cargo test -p uor-r4-float-train --bin recall-payload-probe`: 1 passed.
  - `cargo clippy -p uor-r4-float-train --lib --tests --bin recall-payload-probe`: no finding in the changed code; every warning it lists is in a file or line this unit does not touch.
  - `rustfmt --edition 2021` on the new binary only.
- **wick (`sm_89`), job c387-2, the tree synced at `37f3b8126`:**
  - `cargo +1.97.1 build --release --locked -p uor-r4-float-train --features cuda --bin recall-payload-probe`: binary sha256 `6a343c529fe5e29bc663c632619998f394e4919ff3fb873d3bacfc4fd06dea34`.
  - `cargo test --release --locked … --features cuda --lib vector_read -- --test-threads=1`: 25 passed, among them `the_payload_probe_reads_the_forward_and_leaves_it_bit_for_bit`.
  - The binary's test: 1 passed.
  - Job c387-1 ran the same checks at `be0746349` (binary `7b37e37b…50fd`, 25 and 1 passed) before the added readings.

## Skills used

`native-space-cycle` (read in the ORDER: none of the five items is met, a float reference diagnostic, an inventory-class unit); the loop's ORDER/REPORT form; the wick kit (`local/claude/wick/wick.sh` sync, job, wait and get, with a completion watcher armed for job 1); `uor-store` for the results.

## Still open

- **The fix this points to is the read's average, then its size or place:** a payload of the first kept pick only, or the picks weighed by their scores, at a scale or place where it is not 2.6% of the stream. That fix is its own unit with a sheet (ORDER, reading map B and A).
  - The probe's numbers give that sheet a prediction to register. At the 16,384-step states the first pick is right on 97% of rows, and the scaled payload already decodes to a kept pick's token on 92 to 99% of rows at ×64. So a first-pick-only payload at a scale where it dominates should decode to the asked value on most first-right rows.
  - That is a prediction, not a result: no first-pick payload ran here.
- **Training grows the payload slowly, and the mean caps it anyway.** From 4,096 to 16,384 steps the weight's mean size grew tenfold (0.37 to 3.7) and `|p| / |x|` fifteenfold (0.0017 to 0.026). Even grown past the stream, the mean of four keeps `R_ask` near 0.36 on these states. How far further training would take the ratio is not read here.
- **V2(c), the integer form, waits on a read that passes the bar.**

## Plain summary

claude-386 found that the Flock read learns to look at the right earlier fact almost every time, yet the model's answer does not change. This check measured why, on the saved models, without training anything. Two things stop it. The amount the read adds is about 2.6% of the size of what it is added to, so it is too small to move the answer. And the read averages four places it looked, of which only one holds the right fact, so the right fact is a quarter of what it adds. Turning the added amount up 16 to 1,024 times makes the answer one of those four places' words almost every time, but the right one only about a third of the time: at most 37% correct, against the 90% needed. Nothing is wired wrong: the check reproduces claude-386's numbers exactly. The next unit should make the read add only its best pick, at a size that counts, and test that with its own sheet.
