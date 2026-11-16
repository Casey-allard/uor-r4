# REPORT: m2-donor-corpus-generation, claude-398 (the donor throughput probe on wick, D86)

Lab: claude. Item: [m2-donor-corpus-generation](../../../../roadmap/items/m2-donor-corpus-generation.md) (#2848), referenced, not closed. Order: [ORDER.md](ORDER.md). Sheet: [d41-donor-throughput-2026-11-15](../../../../harness/prep/d41-donor-throughput-2026-11-15.md) (FIT, round 2). Probe at instrument commit `9679a0e48a6ea68cd4062207490efc9da2ddf32e`; smoke at `9d7c62536956ee3ae9d7f0c42993d876d5e56668`.

**Since this unit was dispatched, D87 (Casey, 16 Nov, [PR #4027](https://github.com/UOR-Foundation/uor-r4/pull/4027)) has replaced the production path it sizes.** Production now uses DeepSeek V4.1 Flash on rented pods and keeps the teacher's reasoning text. This probe was finished as registered, and its numbers stand as measurements of gpt-oss-20b on wick.

## What did not work, first

All rates below are kept-answer tokens per second over the 300 s window. The bar's line is 266.0 tok/s: 1.10 × A0, which is also above max(A0, A0-repeat) by more than their spread of 7.9 tok/s.

- **No lever that keeps the corpus plan won.** None of the slot, batch, system-message, tokenize or request-shape arms reached the win line:

  | Arm | Kept tok/s | × A0 | Generated tok/s | Mean busy slots | Reading |
  |---|---|---|---|---|---|
  | A1 `-np 64` | 252.2 | 1.04 | 535.0 (1.07×) | 62 | no win |
  | A2 `-np 64 -b 4096 -ub 1024` | 251.2 | 1.04 | 536.2 (1.07×) | 62 | no win |
  | A4 short system message | 222.2 | 0.92 | 466.6 | 31 | negative (configuration) |
  | A5 no per-answer `/tokenize` | 240.5 | 0.99 | 502.5 | 31 | negative (configuration) |
  | A6 `n` = 4, 16 workers | 192.6 | 0.80 | 389.0 | 30 | lost at 16 workers, mean busy slots 30 |
  | A7 greedy + `n` = 3, 16 workers | 187.5 | 0.78 | 378.3 | 27 | lost at 16 workers, mean busy slots 27 |
  | A8a `--samplers temperature` (CPU) | 236.6 | 0.98 | 497.5 | 31 | no win |

  Doubling the slots raised busy slots from 30 to 62. Generated tokens/s rose only 7%, and request p50 almost doubled (3.07 s to 5.65 s). So with gpt-oss-20b at this load, wick's limit is the GPU's decode step, not the number of free slots. The window's cut-off bias (about 2% at A0, more at 64 slots) is not large enough to hide a 1.10× win: even A1's generated-token rate, which is less affected, gains only 1.07×.
- **A3 was not measured.** `-np 128 -c 262144 -b 8192 -ub 2048` failed at server start, when allocating its 6 GiB KV cache (`cudaMalloc failed: out of memory`; A0's settings already use 16.3 GB of the 24 GB). It is reported as `server-failed`, not rerun, and is not a negative for 128 slots.
- **A8b's check could not tell whether the GPU sampler ran.**
  - A8b measured 280.3 tok/s (1.16 × A0), with request p50 2.63 s against 3.07 s and mean GPU utilization 69% against 46%.
  - The pre-registered activeness check found 0 "setting backend sampler" lines and byte-identical `smoke-temp` and `smoke-temp-bs` shards.
  - So under the sheet, A8b is read as **"not active"**. It is neither a win nor a negative, and it sizes nothing.
  - The check itself turned out to be uninformative. At `b12000`'s default log level, no server log holds any line about backend sampling, so the grep could never have found one.
  - A per-prompt comparison of A8a and A8b cannot settle it either. At 32 concurrent slots even greedy answers differ from run to run: A0 and A8a share the greedy answer on only 122 of 554 prompts, and neither ran the GPU sampler.
  - The 1.16× is therefore an unexplained measurement, and it stays open (see "Still open").
- **One slip in my own checking.** A diagnostic command I sent to wick held a stray `python3 -` line, against the no-ad-hoc-Python rule. It read no file and printed nothing. The rerun used `jq` only, and nothing in the results came from it.

## What changed

- **The instrument** (`docs/labs/d41-driver-math-pilot/instrument/`) gained:
  - `--instrument-commit` (required, written into `run.json`);
  - `--mode seq|n4|g-n3|final`;
  - `--tokenize 0` with a `count` command run afterwards;
  - `--system-file`;
  - window counters (`metrics.json` `window`, `window.json`).
  - `final` renders the prompt through `/apply-template`, checks that it ends at `<|start|>assistant`, appends `<|channel|>final<|message|>`, and sends the pilot's four requests raw to `/completion`.
  - The pilot's sampling values, record format, `seal` and `verify` are unchanged. Tests cover every mode's seed plan and the system-message swap.
- **`wick-throughput.sh`** (new) runs an arms file on wick. Arms with the same server arguments share one `llama-server`. It polls `/metrics` busy slots and GPU use every 5 s, runs the smoke checks and the backend activeness checks, and writes `summary.tsv`.
- **`throughput/`** holds `arms-smoke.txt` (the six smokes as they ran), `arms-probe.txt` and `short-system.txt`.
- **The sheet** passed its FIT in two fresh non-author rounds. Round 1 was NOT FIT, with one blocker; round 2 was FIT. Every finding is folded in and listed in the sheet.

## Measurements

wick, RTX 4090 (`GPU-7c813ad0…`), llama.cpp `b12000`, gpt-oss-20b MXFP4, the pilot's prompt set (`6a6630b2…`), seed 20261115, `--max-tokens 1024`. Each arm was a 300 s dispatch window from the start of the prompt set; the in-flight tail then finished and was left out of the window rate.

| Arm | Prompts in window | Kept tok/s | Generated tok/s | Kept per 1,000 generated | Request p50 / p95 (ms) | Truncated / markup / errors | GPU % / peak MiB |
|---|---|---|---|---|---|---|---|
| A0 base (`-np 32`) | 749 | **241.9** | 501.5 | 482 | 3,069 / 5,159 | 0 / 0 / 0 | 46 / 16,264 |
| A0-repeat | 727 | **233.9** | 484.1 | 483 | 3,109 / 5,490 | 0 / 1 / 0 | 46 / 16,264 |
| A1 `-np 64` | 788 | 252.2 | 535.0 | 471 | 5,653 / 9,704 | 0 / 0 / 0 | 49 / 16,838 |
| A2 `-np 64`, larger batches | 789 | 251.2 | 536.2 | 468 | 5,715 / 9,671 | 0 / 0 / 0 | 40 / 17,706 |
| A3 `-np 128` | — | not measured (server out of memory) | — | — | — | — | — |
| A4 short system message | 685 | 222.2 | 466.6 | 476 | 3,328 / 5,722 | 0 / 0 / 0 | 51 / 16,264 |
| A5 no `/tokenize` (+ `count`) | 752 | 240.5 | 502.5 | 478 | 3,055 / 5,193 | 0 / 0 / 0, 0 tokenize errors | 51 / 16,264 |
| A6 `n` = 4 | 570 | 192.6 | 389.0 | 495 | 7,713 / 14,187 | 0 / 1 / 0 | 51 / 16,264 |
| A7 greedy + `n` = 3 | 569 | 187.5 | 378.3 | 495 | 4,100 / 6,659 | 0 / 0 / 0 | 59 / 16,264 |
| A8a trimmed chain, CPU | 735 | 236.6 | 497.5 | 475 | 3,080 / 5,276 | 0 / 0 / 0 | 47 / 16,264 |
| A8b trimmed chain, `--backend-sampling` | 871 | 280.3 ("not active") | 582.4 | 481 | 2,627 / 4,423 | 0 / 0 / 0 | 69 / 16,534 |
| **A9 final-channel pre-fill** | 1,174 | **418.3** | 448.9 | **931** | 1,955 / 3,946 | 0 / 4 / 0 | 50 / 16,264 |

**The bar.**
- **B1 holds.** All six smokes at `9d7c62536` passed, and so did the three in-job smokes (`smoke-final`, `smoke-temp`, `smoke-temp-bs`): 16 answers each, with 0 markup and 0 errors.
- **B2 holds for every measured arm.** Truncated plus markup answers are at most 0.08% (A9: 4 of 4,824), errors are 0, and tokenize errors are 0.
- **B3 holds.** A0 is 241.9 against the pilot's 220.7 (+9.6%, inside ±15%), and A0 and A0-repeat are 3.3% apart.

The reading is therefore **P**.

**A9, reported on its own, as the sheet says.**
- **Rate:** it won at 1.73 × A0. Of every 1,000 generated tokens it kept 931, against A0's 482, because the analysis channel is skipped.
- **Answers:** 0 of 4,824 were truncated, so every answer reached a stop. 4 held markup (0.08%), where A0 had none.
- **Plan:** skipping the analysis channel changes the corpus plan. It also conflicts with D87, which keeps the reasoning text.

**The combined arm C was not run.** C takes only winners that keep the plan, and none won (A8b is "not active"), so C would have been A0 again.

**Production sizing**, hours of wick = 100,000,000 / (kept tok/s) / 3,600:
- **Keeping the corpus plan:** no lever won, so the rate is the mean of A0 and A0-repeat, **237.9 tok/s: 116.8 hours (4.9 days) for 100M kept tokens**, and 1,168 hours (48.7 days) for D87's 1B.
- **Best overall (A9, which changes the plan):** **418.3 tok/s: 66.4 hours for 100M**, and 664 hours (27.7 days) for 1B. D87 rules this path out by keeping the reasoning.
- **For reference only, not a sizing number:** if production's own sheet showed A8b's backend sampler active and its greedy answers equal to the CPU's, A8b's 280.3 tok/s would give 99.1 hours for 100M.

**Wall-time rates (recorded, no bar):** kept tokens / wall time: A0 241, A0-repeat 232, A1 254, A2 252, A8b 281, A9 420 tok/s.

## Checks run at the head

- **Laptop, at `9679a0e48`:** `cargo test --release` in the instrument passes 8 lib and 2 bin tests, clippy gives 0 warnings, and `cargo fmt` is applied. The instrument's source holds no float type or float literal. `bash -n` passes on `wick-throughput.sh`. `uor-harness prep-check` shows the sheet as FIT and complete.
- **wick, job `d41-tp-dry2` (at `25f13c283`, a 20 s window):** rc 0. The window counters, `window.json`, `count`, busy-slot polling and the summary all worked, and `smoke-final` passed.
- **wick, job `d41-tp-probe` (at `9679a0e48`):** rc 0. The instrument was built from the exported commit; the files on wick had the same sha256 as the commit's. The prompt-set hash was checked. Every `run.json` names `instrument_commit` `9679a0e48…` (14 arms) or `9d7c62536…` (the 6 earlier smokes). `count` for A5 gave 72,148 window kept tokens with 0 tokenize errors.
- **Store:** `uor-store put` of 130 files to `results/claude/d41-throughput-2026-11-15/`. A fresh `uor-store get`, then `shasum -a 256 -c SHA256SUMS`, gave 129 of 129 OK. The folder holds every arm's `metrics.json`, `run.json`, `window.json`, `count.json`, the shards, the server and GPU logs, `backend-active.txt`, `summary.tsv`, the job log, and a `NOTE.md` that marks the answer texts as probe output: not corpus, never training data.
- **Pre-run review (§5.10), in this session:**
  - The chain is D86, then the pilot (claude-390), then this probe.
  - Every input was present: the engine, the GGUF and the prompt set.
  - P, I, Q, A9 and H-sampler each had a fixed reading.
  - The sheet had its FIT.
  - `wick.sh gpu` showed 4 MiB used and no process. The cost was $0, with no pod.
- **Owner terms (D86):** the probe stored no logits, probabilities, scores or ranks, and no output was sealed. Only this job's own processes on wick were stopped (the script's server and watcher PIDs).

## Skills used

- The loop's ORDER/REPORT form, with a one-page sheet and two fresh non-author FIT rounds.
- The wick kit (`local/claude/wick/wick.sh`: put, job, wait, tail, get), with a completion watcher on the job.
- `scripts/pod/uor-store` for the upload and the checked readback.
- `native-space-cycle`: not applicable. This unit measured an external engine and touched no model state.

## Still open

1. **D87 replaced the wick production path.** Production (claude-400) uses DeepSeek V4.1 Flash on rented pods. The sizing above is a measurement of gpt-oss-20b on wick and no longer plans a run. D87 leaves open wick's role after the first corpus. If that is decided, the plan-keeping 237.9 tok/s is wick's measured rate for gpt-oss-20b.
2. **The final-channel pre-fill (A9) conflicts with D87's reasoning-kept scope.** It is 1.73× faster only because it drops the analysis channel that D87 keeps. Its measurement stands. It is not a production lever under D87.
3. **A8b's 1.16× is unexplained.** The activeness check could not detect a GPU sampler at the default log level. A later sheet that wants this lever needs a check that can fail: a higher server log level, or the engine's own timing counters, plus the sheet's greedy-equality test at `--parallel 1`.
4. **Concurrency changes the answers.** At 32 slots, the same prompt, seed and settings gave the same greedy answer in only about a quarter of prompts across two runs (A0 vs A8a: 122 of 554). A sealed corpus is fixed by its hashes and cannot be reproduced by generating again. Production's receipt should say so.
5. **A3's 128 slots were not measured.** `-c 131072` with `-np 128` (1k tokens per slot) would fit in memory, but A1 shows decode-bound scaling, so it is unlikely to win.

## Plain summary

I measured how fast gpt-oss-20b on Casey's 4090 can write kept answers. The re-measured baseline is 242 tokens per second, close to the pilot's 221. None of the settings that keep the corpus plan as written beat it by the 10% the sheet required:
- more slots gave +4%;
- one request per prompt was slower;
- skipping the tokenize call made no difference.

At the baseline, 100M kept tokens would take about 117 hours of wick. Pre-filling the final channel, so the model skips its reasoning, gives 418 tokens per second (66 hours). But that drops the reasoning, which the new owner decision D87 keeps. D87 also moves production to DeepSeek on rented pods, so these numbers describe wick and no longer plan the production run. One setting, the GPU sampler, ran 16% faster, but my check could not show that it was really on, so it counts for nothing until a later sheet checks it properly.
