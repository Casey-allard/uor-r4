# Survey B: current track cost and risk (PI review, 16 Nov 2026) - PARTIAL

I stopped early at the context mark. What I read: end-to-end-gap-analysis-2026-11-16.md, and claude-407 REPORT lines 1-51 (by grep). What I did not read: SYNTHESIS.md, the trainer map, the claude-406 REPORT and the notebooks. Items marked OPEN are left for a fresh agent.

## 1. Definition of "1B-param-equivalent" (proposal)
- Quality bar Q has two parts:
  - bits per byte on held-out D87 DEV, at or below the yardstick;
  - an exact-answer panel (the claude-346 kinds panel), at or above a fixed pass rate.
  No transformer is run. The yardstick comes from published numbers only.
- Yardstick (hypothesis, from memory, NOT checked this session):
  - GPT-2 XL (1.5B) scores a zero-shot WikiText-103 perplexity of about 17.5 (GPT-2 paper).
  - Pythia-1B and TinyLlama-1.1B reach about 0.85-1.0 bits/byte on web text.
  OPEN: confirm these with sources. Then convert them to D87's tokenizer by measuring bytes per token on DEV.
- Budget B:
  - serving state of at most 4 GB resident on the 16 GB M1, which leaves room for the OS and the context;
  - decode of at least 20 tok/s at 2k context.
  The headline metric is decode tok/s.
- "Equivalent" means the model meets Q within B, at any parameter count. The parameter count is not the target.

## 2. Cost on the current track (line C)
- Corpus against scale (estimate):
  - A compute-optimal model wants about 20 tokens per parameter (Chinchilla).
  - D87 has 1.0096B tokens (checked, memory note on D87). So the compute-optimal size is near 1.0096e9 / 20 = 50M dense parameters.
  - A real 1B dense model wants about 20B tokens, which is 20x D87.
  - So a 1B-equivalent trained on 1B tokens has to get its size from exact memory or n-gram rows, which are cheap to fill, not from trained dense coordinates.
- Scoring throughput (checked):
  - At 4,096 rows the GPU is full at about 54,000 row-coordinates/s for K >= 8 (claude-407 REPORT line 14).
  - At K = 1 it runs about 32k targets/s (from the brief, claude-406/407).
- One pass over D87 at 32k targets/s, for ONE coordinate (estimate): 1.0096e9 / 32,000 = 31,550 s = 8.8 h. That is the counterfactual for one coordinate, not a training step.
- One sweep of P coordinates over a 4,096-row batch (estimate) takes P x 4,096 / 54,000 s:
  - P = 1e9: 4.1e12 / 5.4e4 = 7.6e7 s = 879 days per sweep. Not feasible.
  - P = 1e7: 7.6e5 s = 8.8 days per sweep.
  - P = 1e6: 21 h per sweep.
- These sweep times are lower bounds:
  - they leave out carry, choose, apply and DEV evaluation (claude-407 REPORT line 13);
  - the trainer still refuses more than 64 rows (REPORT line 7);
  - the trainer scores one parameter at a time (REPORT line 12, training.rs:3089).
- Serving memory (estimate):
  - 1e9 parameters take 1.0 GB at int8 and 0.5 GB at int4, which fits budget B.
  - If every parameter is read for each token, decode is bounded by memory bandwidth: about 68 GB/s on the M1 / 1 GB = about 68 tok/s at most.
  - Routed or sparse reads, which touch a few rows per token, are much faster.
  - OPEN: the measured M1 tok/s of discrete_d11 and of line A's uor-chat.
- Best quality in each line (A, B, C): OPEN, not read. The gap analysis gives:
  - line B: read recall of 96.5-98.4% (claude-388) and pick recovery of 98-99.6% held out (claude-395);
  - line A: "the best replies so far";
  - line C: no language-quality evidence cited.

## 3. Top three risks
1. Per-coordinate counterfactual scoring does not scale past about 1e7 trained coordinates on one 4090: a 1e9 sweep takes 879 days (section 2). Most parameters must be trained in closed form or by writing: the output error written into addressed rows, or count tables. Scored search should cover only a small core.
2. The gap analysis cites no evidence that line C learns language at scale. Its wins are parity and speed (claude-404, claude-407). OPEN: confirm from the notebooks.
3. D87 is 20x too small for a 1B dense model, so "1B-equivalent" has to rest on exact memory capacity. Quality then depends on gap 3 (conjunction memory) and gap 4 (readout), and line C has neither built yet.

## 4. Alternatives (ranked by estimate; OPEN: check that each engine exists as named)
1. Write-based memory model on the line C state:
   - fill exact n-gram and conjunction rows in one counting pass over D87, estimated at hours on wick, limited by memory bandwidth;
   - train a small scored core of at most 1e6 coordinates, about 1 day per sweep;
   - read with the XOR+popcount Pick from #4045.
   This is the fastest route to a served model and uses the least compute.
2. Line A trunk plus line B read, ported native. It has the best replies today, but the D85 export gap must close first, because D85 forbids a float master as the default.
3. Line B's Q31 native Hamilton state as the base, with line C's transport ported in. A native CUDA trainer exists; it still needs a trunk.
4. The current track as planned (scored training up to 1B). It is the slowest, and risk 1 blocks it.

## B2 (completion), 16 Nov 2026

Read-only. Paths are relative to the repo at origin/main (worktree payload-probe-387) unless they name a notes file. "checked" = I read the line in this session; "estimate" = my arithmetic; "from memory" = not checked against a source.

### B2.1 What SYNTHESIS.md and the trainer map change in the cost estimate
- **The platform reviewer's speed is 10-100x too high for the scored path.** SYNTHESIS.md:29 says 0.3-3M tok/s and "1B tokens under an hour" (a hypothesis). The measured figure is 32,368 targets/s at 4,096 rows and K = 1, scoring only (claude-407 REPORT:56, checked). Drop the hypothesis.
- **The target count is 1.87B, not 1.0096B.** D87 tokenizes to about 1.87B scored targets in our V4096 tokenizer (claude-406 ORDER:17, checked; it is claude-403's projection, which I did not re-derive). So B section 2's one-coordinate pass is 1.87e9 / 32,368 = 16 h, not 8.8 h. claude-407 REPORT:65-72 gives the same 0.67 days (checked).
- **The donor corpus cost about 4x the estimate.** SYNTHESIS.md:42 estimated 6 h and at most $20 per 100M tokens (hypothesis). D87 cost about $854 for 1.0096B tokens, about $85 per 100M (memory note on D87). That money is already spent, so it does not count toward the cost of the run.
- **There is engineering work before any run** (trainer map lines 7-19, checked against code):
  - No entry point trains at more than 64 rows: `DriverConfig::validate` requires `(1..=64).contains(&batch_rows)` (crates/uor-r4-discrete/src/training.rs:307, checked).
  - `stage-t-run` caps max_batches at `STAGE_T_MAX_BATCHES = 16_404` (stage_t.rs:50, checked), has no resume flag and hard-wires its DEV panel.
  - The general trainer path scores one parameter per call through `propose` or `propose_rows_cached` (training.rs:3087-3091, checked).
  - Only `run_block_transitions` (training.rs:3310, checked) reaches rows-K, and it has no CLI. That CLI is claude-408's planned work.
- **Data is no longer the binding limit for recall training.** SYNTHESIS.md:21 counted about 39k TRAIN windows. D87 replaces that. Section 2's Chinchilla point still holds: a 1.87B-target corpus is compute-optimal for about 50-90M dense parameters (estimate).

### B2.2 Measured serving speed on the M1 (decode tok/s)
| Model / path | Size | Decode tok/s | Source (checked) |
|---|---|---|---|
| Discrete D11, the M1-close saved model, `d11-scorer toks` | 86.5 MB / 84.9 MB | **11,026** (s1), 10,573 (s2); prefill 2.6M | docs/loop/tasks/m1-close-saved-model/antigravity-110/REPORT.md:28-30 |
| Discrete D11, stand-in A8 artifact (V4096, B4, C192), `toks` | n/a | 13,259 (p95 13,176) | m1-d11-artifact-scorer/claude-116/REPORT.md:51 |
| D11 float-trunk export, snapped `lut-stack/4`, `fbs-toks` | n/a | 4,827-4,905 | d11-exporter-float-trunk/claude-181/REPORT.md:13 |
| Same export, unsnapped (`raw_both`), the closing artifacts | n/a | 4,217-4,320 | d11-exporter-float-trunk/claude-182/REPORT.md:6-7 |
| a7r-s1..s3, `uor-chat --stack`, one thread, at load 4-7 | n/a | **1,323-1,333** | m4-export-uorlut01-cli/claude-259/REPORT.md:13-15 |
| a7r R s1, `fbs-toks` | n/a | 1,204 | integer-nearest-readout/claude-247/REPORT.md:7, citing claude-243 |
| Pair rows served (S-pair), loaded machine | n/a | 1,505-1,648 | one-model-end-to-end/claude-384/REPORT.md:128-131 |

- Every measured model is at least 50x above budget B's 20 tok/s. But every one of them is small: the largest is 86.5 MB. The 51 MB pair artifacts were timed only through a bin that re-parses the artifact on every row (claude-311 REPORT:9), so no decode figure exists for them.
- codex-11 REPORT:35-42 reports about 34k tok/s up to a 32k context. That is an indexed test session in a debug build, not a language model, so it is not a serving figure.
- Estimate: a 1 GB int8 model whose decode touches only routed rows stays far above 20 tok/s. A dense read of all 1 GB is capped at about 68 tok/s (section 2).

### B2.3 Measured training-side rates (wick RTX 4090) and what they include
- **claude-406**, 64 rows, checkpoint 0, V4096/B4/C192 Route read, seed 1 (REPORT:36-50, checked):
  - baseline `propose`: 4.73 s per coordinate (13 targets/s);
  - rows-K at K = 1: 82 ms per call, 778 targets/s;
  - rows-K at K = 64: 127 ms per call, 32,337 row-coordinates/s.
- **claude-407**, at 64 / 256 / 1,024 / 4,096 rows, K = 1 (REPORT:51-58, checked):
  - 783 / 2,819 / 12,064 / 32,368 targets/s;
  - at 4,096 rows, K = 8 gives 6,324 targets/s, and K = 64 gives 4.80 s per call at 54,649 row-coordinates/s.
- **What the rates include:** only the time inside `score_rows_k`, the counterfactual losses (crates/uor-r4-discrete/src/rowsk_device.rs:56).
- **What they leave out:**
  - carry, which took 23 ms at 4,096 rows outside the timers (claude-407 REPORT:14);
  - choose, apply, DEV evaluation and data movement (claude-406 REPORT:60);
  - any cost from training past checkpoint 0 (claude-406 REPORT:13).
  So they are lower bounds.
- **The key derived figure** (estimate): a pass tries one coordinate per batch per unit of K.
  - At 4,096 rows and K = 1, one D87 pass tries 1.87e9 / 4,096 = 457k coordinates in at least 0.8 days (0.67 days of scoring plus about 18% for carry).
  - At K = 8 it tries 3.65M coordinates in at least 3.4 days.
  - So scored training reaches about 0.5-1.1M coordinate trials per GPU-day. Touching 1e9 coordinates once takes about 900-2,000 GPU-days, which agrees with section 2's 879 days.
- **Revised cost of the current track (estimate):**
  - about 1-3 engineering units for the 408 CLI and the ingest;
  - then 1-4 GPU-days per D87 pass for a trained core of at most a few million coordinates.
  - The cash cost is power only: about 0.45 kW x 24 h = 11 kWh a day on Casey's own card.
  - The bottleneck is coordinates per day, not money.

### B2.4 Does each alternative's engine exist?
1. **Write-based memory: PARTLY.**
   - The Pick read exists in three places:
     - CUDA: `pick_device` (crates/uor-r4-discrete/src/pick_read_cuda.rs:56), with a test that it matches the served d11 artifact (:352);
     - serving: `DiscreteReadMode::Pick` and `PickTransport` (crates/uor-r4-integer/src/discrete_d11.rs:298, :801);
     - the native trainer of its gates: `train_pick_step` (crates/uor-r4-native-vector/src/read_train.rs:235).
   - A counting trainer exists only as `cpu-trunk-count-driver` (crates/uor-r4-cpu-train/src/bin/cpu-trunk-count-driver.rs:1-5, "exact additive sufficient-statistic training", count tables). It is a D84 CPU research program, so it cannot qualify a GPU bar (§3.5).
   - I found no GPU/native n-gram or conjunction counting pass over a D87 stream. My grep was not exhaustive, so this is not proof that none exists.
2. **Line A trunk plus line B read: YES, but it is float.**
   - Training: `train-in-space` (crates/uor-r4-float-train/src/bin/train-in-space.rs) and `in_space_long.rs`.
   - Export: `fbs-export` (crates/uor-r4-float-train/src/fbs_export.rs, uor-r4-integer bin fbs-export).
   - Serving: `IntegerStackSession` (crates/uor-r4-integer/src/stack/session.rs:2217) and `uor-chat`.
   - The D85 default forbids the float master. The export gap is what has to close.
3. **Line B's Q31 native state: YES, for the read and transport.**
   - CUDA: `native_vector_device.rs` ("AOT CUDA handle for native Q31 Hamilton vector transport and training steps", line 1), including `hamilton_rotate` (:125).
   - CPU: `uor-r4-native-vector` (read.rs, read_train.rs).
   - It has no trunk, as B already said.
4. **The current track: YES, with no CLI.**
   - `Driver::run_block_transitions` (training.rs:3310) calls `score_rows_k` (rowsk_device.rs:56).
   - Its 64-row and one-parameter limits are listed in B2.1.

### B2.5 Yardstick for "1B-equivalent" in BPB (all from memory, unchecked)
- **GPT-2 XL (1.5B), zero-shot on the Pile:** about 1.0-1.1 BPB. The Pile paper's table puts GPT-3 davinci at about 0.72.
- **Pythia-1B on the Pile test set** (in-distribution): about 0.80-0.90 BPB.
- **TinyLlama-1.1B on web text** (SlimPajama-like): about 0.75-0.85 BPB.
- **A KN 5-gram on about 1B tokens:** the One Billion Word benchmark's modified-KN 5-gram reaches a perplexity of about 67.6 per word (Chelba et al. 2013). At about 5.5-6 bytes a word, that is about 1.0-1.1 BPB.
- **Reading:** a pure count model on D87 lands near GPT-2 XL zero-shot. It sits about 0.2-0.3 BPB behind Pythia-1B and TinyLlama. The repo's own M4 bar is 0.90 BPB (claude-259 REPORT:5, checked).
- **Proposal:** Q = at most 0.90 BPB on D87 DEV, matching the existing bar. A count-only model is expected to miss it, so the scored core has to close about 0.2 BPB. Both yardstick numbers have to be re-measured in D87's bytes per token before use.

### B2.6 Top risk (revised)
- **The risk:** the only route to 1B-equivalence that fits on one 4090 is write/count tables plus a small scored core. Scored training reaches about 1M coordinate trials per GPU-day (B2.3). But the count engine that exists is CPU-only research under D84, and no native GPU counting pass over D87 was found (B2.4).
- **Why it matters:** count-only models top out near 1.0-1.1 BPB (B2.5, from memory). So the 1B goal rests on two things that are unbuilt or unmeasured:
  - a native counting pass;
  - measured learning per coordinate trial in the scored core.
- **Next measurement:** BPB against coordinate trials on a D87 slice, at K = 1 and K = 8, with a no-memory control.
