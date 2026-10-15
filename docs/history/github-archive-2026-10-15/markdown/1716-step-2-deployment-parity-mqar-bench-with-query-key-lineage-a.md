# #1716 — Step 2: deployment-parity MQAR bench with query+key lineage and learned-shift controls (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-05T04:57:45Z
- **Updated:** 2026-10-05T06:01:03Z
- **Closed:** 2026-10-05T05:01:23Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1716
- **Merged:** 2026-10-05T05:01:23Z
- **Merge commit:** 0df35259358c7f3c404cc756bb2882d0b868ec17
- **Base / Head:** main / UOR-Foundation:codex/step2-parity-bench
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Step 2 of the 2026-10-05 barrier assessment (`docs/research/barrier-assessment-2026-10-05/final-assessment.md` §3 Step 2) plus `completeness-critique.md` points 2 (query-side lineage), 5 (rule ordering) and 6 (bench context no larger than served context). This PR adds the bench code, the arms, the decision-table generator and the pod script. The full grid has **not** been run. Only build, unit tests and smokes ran on the laptop. No result here is evidence about recall.

## What changed

**`StackModel::set_read_lineage` (`crates/uor-r4-training/src/geometric_stack.rs`)**
- Adds research-only key/query lineage controls. The setter is opt-in and mirrors `set_read_key_shift`. It must be called before the optimizer is created, because the learned variants add variables.
- The variants are in `ReadLineage`:
  - `QueryKeyJ` (bench arm `qk`): queries `q_t + j⁻¹ q_{t−1}` with keys `k_t + j k_{t−1}`, as the critique specifies.
  - `QueryKeyJSame` (`qk_jj`, an extra diagnostic arm): queries `q_t + j q_{t−1}`. See caveat 1.
  - `IdentityShift` (`identity`): `k_t + k_{t−1}`.
  - `RandomSo4 { seed }` (`so4`): `k_t + R k_{t−1}`. `R` is fixed and block-diagonal, with one Haar-random SO(4) block per four-channel lane. Each block is `x ↦ p x q̄` with seeded unit quaternions `p` and `q`. It is not learned.
  - `LearnedConv` (`conv`): a learned depthwise width-4 causal convolution on k, `Σ_{i<4} w_i ⊙ k_{t−i}`, the RWKV/H3 token-shift control. It starts at `w0 = 1` and `w1 = 1` (`learned_init=lag1`, the default) or `w1 = 0` (`zero`). It is not weight-decayed.
  - `LearnedPrev` (`wprev`): `k_t + W k_{t−1}`, with `W` learned, width × width. It starts at the identity (`lag1`) or at zero. It is not weight-decayed.
- With the default `lag1` initialisation, every lineage arm starts with a full-strength lag-1 channel. The arms then differ only in the operator applied to `k_{t−1}`.
- `save` refuses a model with a lineage, as do served mode, the integer export, the grid reference (`stack_export.rs`) and the native replay host check (`geometric_attention_native.rs`). F2 and a lineage are mutually exclusive. F2 keeps its saveable form unchanged.
- Adds `quaternion_j_inv_left` and `random_so4_blocks`.

**`mqar-bench layout=fact` (`examples/mqar-bench.rs` plus `examples/mqar_bench_step2/fact.rs`)**
- **Tokenizer.** The #1017 tokenizer is required (`tokenizer=`). The bench records its sha256 `d36d3e87…` and a sha256 of the derived pools.
- **Keys** are 1–3 BPE pieces: a lowercase space-prefixed word piece plus 0–2 continuation pieces. A tuple is kept only if the tokenizer re-encodes its decoded text to exactly those pieces.
- **Gap.** A copula gap of `g ∈ {0,1,2,3}` pieces separates key and value. The gap pool has 15 single-piece function words (`" is"`, `":"`, `" the"`, …; `" ="` is not one piece and is dropped) plus 16 random word pieces from the key/value pool, so the vocabularies overlap.
- **Values** are one word piece, a canonical two-piece word, or a numeral `" d"`. A numeral is written as the tokenizer writes it: a bare `" "` piece, then a digit piece.
- **Reply forms:**
  - REHEARSE: the query is `key ++ gap ++ value`, so the predicting position holds the copula, as in D19 "{k} is".
  - BARE: the query is `key ++ "? It's" ++ value`.
- Every value piece is a target, teacher-forced after the first.
- **Uniqueness.** Within a window, key pieces are distinct from every other key, value and gap piece, and filler excludes all key and value pieces. A query therefore identifies its fact uniquely.
- **Context** is at most 384 (the default is 384). The context is refused above that.
- **Held-out split:** keys whose first piece id is ≡ 0 mod 4.
- **Scores**, per cell `(form, g, key pieces)` and per marginal (form, gap, key pieces, value kind, distance bucket):
  - first piece;
  - first content piece;
  - full value;
  - NLL of the first content piece;
  - the reference rule.
- **Reference rule** (non-learned, rehearse + R-nlet): back off over n = 4…1. Find the latest earlier occurrence of the last n pieces before the value and copy the pieces that followed it. For BARE, the rule first strips the fixed bare prefix it knows. It is told the value's piece count.
- **Read probe** (once, after training): the weight of each read head on the fact's first value piece and on its last key piece, by gap.
- **New options:**
  - `lineage=none|f2|qk|qk_jj|identity|so4|conv|wprev`. `key_shift=true` is kept as an alias for `f2`.
  - `learned_init=lag1|zero`.
  - `save_model=true` writes `model/` inside the report root. An arm without a saved form records its refusal and the run is not failed.
  - `device=cuda` uses the existing `cuda` feature.
- The synthetic layout is unchanged, apart from the arm plumbing and two added task-record fields (`vocab` from the run, `save_model`).

**`mqar-bench mode=decide runs=DIR out=NEW_ROOT` (`examples/mqar_bench_step2/decide.rs`)**
- Verifies every sealed root under `runs` against its manifest. Incomplete or failed roots are listed as skipped. Two complete roots for the same (pattern, arm, seed) are refused.
- Applies the frozen rules. The thresholds are constants: pass 0.95, tie 0.03, rule tolerance 0.01.
  - **Decision cells:** g ≥ 1 with a 2- or 3-piece key, for both forms (12 cells).
  - **Metric:** full-value accuracy on the final in-class fresh pairings. A cell passes when every seed scores ≥ 0.95 and the arm has its required seeds: 3 for none, f2, qk and conv; 2 for the others.
  - **Order, with the critique's fix:**
    - R1 `none` passes → no lag problem; stop lineage research. This is checked first.
    - R2 else `f2` passes → F2 is sufficient.
    - R3 only if `none` fails: a conv tie within 0.03 of F2 on every decision cell retracts the j claim.
    - R4 every arm passes REHEARSE and fails BARE → the gap is reply form/data.
    - R5 capability = the cell passes **and** the arm meets the reference rule (also reported: `meets_rule`, `beats_rule`).
    - R6 conv fails a cell that some lineage arm passes → Step 2b is justified.
    - R7 (informational) cells where qk passes and f2 fails.
  - R1–R3 are decided only when none, f2 and conv have all their seeds. R1–R3 and R6 are applied to both forms together and to each form alone.
  - `rararr` and `rrarra` are decided. `aaaaaa` is tabulated as diagnostic only.
- Writes `decision.json` and `decision.md` to a new claimed root and seals it.

**`scripts/pod/step2-parity-bench.sh`**
- Checks the tokenizer sha256 and builds with `--features cuda`.
- Records the source SHA, dirty-path count, binary sha256, rustc version and GPU in `pod-manifest.txt`.
- Runs 60 jobs: 3 patterns × (none, f2, qk, conv at 3 seeds; identity, so4, wprev, qk_jj at 2 seeds). It keeps `K` jobs in flight, round-robin over `GPUS`.
- It can be re-run. It skips every (pattern, arm, seed) that already has a complete sealed root, and gives a failed one a new `-aN` root. A root is never reused.
- It then writes a new sealed `decision-<UTC>` root.
- `PATTERNS` and `ARMS` select a subset, for example for a calibration job.

## Tests (at `fd82c6f7`, macOS M1, CPU)
- `cargo test -j 2 -p uor-r4-training --lib -- lineage read_key_shift the_j_lineage random_so4 the_qk_lineage`: **13 passed, 0 failed.** This includes the 4 new tests and the existing F2, export-refusal and lineage-named tests.
  - `the_j_lineage_channels_are_orthogonal_and_j_inverse_undoes_j` checks that `⟨x, jx⟩ = 0` per lane and that `j⁻¹ j = id`. It also checks `⟨ja, b⟩ = ⟨a, j⁻¹b⟩`, `⟨ja, jb⟩ = ⟨a, b⟩` and `⟨j⁻¹a, jb⟩ = −⟨a, b⟩`.
  - `random_so4_lineage_blocks_are_rotations` checks `RᵀR = I` and `det R = +1`, and that the blocks are seeded.
  - `read_lineages_are_causal_learnable_and_research_only` covers all 8 variant/initialisations. It checks:
    - the logits shape;
    - row 0 unchanged;
    - causality: changing token 7 leaves rows 0–6 bit-identical;
    - that the zero-initialised learned variants start equal to plain;
    - the parameter counts;
    - non-zero gradients on the lineage variables;
    - one AdamW step;
    - refusal of save, of F2 combined with a lineage, and of served mode;
    - that removal restores the inventory.
  - `the_qk_lineage_turns_the_query_and_the_key_shift_alone_does_not`.
- `cargo test -j 2 -p uor-r4-training --release --example mqar-bench`: **13 passed, 0 failed.** Among them:
  - The fact-window test runs on the real tokenizer over 128 windows (train and held-out). It checks spans, distances and buckets, that the predicting position holds the copula, the bare prefix or the last key piece, and that the key tuple occurs exactly twice. It also checks that gap pieces are never key pieces, that weights fall only on value pieces, that all 24 cells occur, and that the rule solves every rehearsed query. If the tokenizer is absent the test prints SKIPPED; that did not happen here.
  - The n-let rule unit test.
  - Three decide-rule tests, covering the ordering, the conv tie and its guard, the reply form, Step 2b, incompleteness and capability.
- `rustfmt --check` is clean on every touched file. `cargo fmt --check` reports only `src/relation_compiler.rs`, which is unformatted on `main` and untouched here.
- The touched code adds no new warnings. The lib's existing dead-code warnings and clippy errors (`b3_e8_codecs.rs`) are on `main`.
- The CUDA build was **not** compiled here (macOS has no CUDA toolkit).

## Smoke (laptop CPU, RAYON_NUM_THREADS=3, release; not evidence)
| run | steps | step time | result |
|---|---|---|---|
| rararr f2, save_model=true | 30 (batch 8) | 0.58 s median | complete, sealed; `model/` saved (config.json, model.safetensors) |
| rararr conv, save_model=true | 30 | 0.59 s | complete; save refused and recorded: "the research-only read lineage conv has no saved form" |
| rrarra qk | 30 | 0.62 s | complete, sealed |
| rararr none | 287 of 400 (max_seconds=230) | about 0.8 s | value NLL 7.76 → 5.84; recall at chance (full 0.036). 300 CPU steps is far too short to reach recall, and was not expected to. |

- **Script, CPU, tiny settings** (STEPS=2, K=3):
  - 15 roots completed and sealed before my 290 s smoke limit stopped the grid.
  - `mode=decide` on them verified the roots and skipped the unsealed ones. It decided R1–R3 for `rararr`, because none, f2 and conv each had 3 seeds.
- **Rule-3 guard.** That decide run is why caveat 3's guard exists. Under the literal rule, two arms both at chance "tie", and that read as "J_CLAIM_RETRACTED". It now reads "R3 NOT_INFORMATIVE: F2 does not pass (literal tie within 0.03: true)".
- **Calibration filter.** `PATTERNS=rrarra ARMS=none:1 K=1` ran one job and wrote the manifest, with 0 dirty paths at `fd82c6f7`.
- **Reference rule on these sets:** about 1.0 on REHEARSE cells and low on BARE g ≥ 1 cells. The overall rule accuracy is about 0.60.
- **Laptop resources:** at most 3 threads, about 0.69 GB RSS per process, 52 MB new storage under `~/uor-r4-local/step2-parity-bench/smoke-20261005` (kept). The job lock was registered and then removed.

## Pod run script
```bash
# On the GPU pod (2x RTX 4090, CUDA 12.6 as in docs/compute/cuda-training.md), from a clean clone:
git fetch origin codex/step2-parity-bench && git checkout fd82c6f7dbd0093613255ae6e72f1394b6af66fb
export PATH=/usr/local/cuda/bin:$PATH
# optional calibration job first (its root is reused by the full grid):
TOKENIZER=/workspace/inputs/1017/tokenizer.json OUT_ROOT=/workspace/step2-parity-bench-20261005 \
  PATTERNS=rararr ARMS=none:1 K=1 GPUS="0" DEVICE=cuda STEPS=4000 BATCH=16 LR=0.001 WARMUP=200 \
  EVAL_EVERY=250 FINAL_SEQUENCES=256 MAX_SECONDS=3600 THREADS=2 CUDA_COMPUTE_CAP=89 \
  bash scripts/pod/step2-parity-bench.sh
# full grid (60 runs) + decision table:
TOKENIZER=/workspace/inputs/1017/tokenizer.json OUT_ROOT=/workspace/step2-parity-bench-20261005 \
  K=8 GPUS="0 1" DEVICE=cuda STEPS=4000 BATCH=16 LR=0.001 WARMUP=200 EVAL_EVERY=250 \
  FINAL_SEQUENCES=256 MAX_SECONDS=3600 THREADS=2 CUDA_COMPUTE_CAP=89 \
  bash scripts/pod/step2-parity-bench.sh
```
The tokenizer must be the file with sha256 `d36d3e87…0f89`; the script refuses any other. Each job issues exactly this command:
```
mqar-bench out=$OUT_ROOT/runs/<pattern>-<arm>-s<seed> layout=fact tokenizer=$TOKENIZER device=cuda context=384 \
  pairs_per_bucket=4 batch=16 steps=4000 lr=0.001 warmup=200 min_lr=0.1 weight_decay=0.1 clip=1.0 \
  eval_every=250 curve_sequences=16 final_sequences=256 probe_sequences=16 probe_steps=final \
  seed=<seed> max_seconds=3600 save_model=true gaps=0,1,2,3 forms=rehearse,bare \
  arm=stack pattern=<pattern> read=l2 rotation=true width=128 heads=4 mlp=384 age=default \
  lineage=<arm> learned_init=lag1
```
The model has about 1.83M parameters, of which 0.52M are the 4096 × 128 tied embedding. Each step sees 16 windows × 12 facts.

**Expected wall time: about 2–3 h for the 60 runs on 2× RTX 4090 at K=8.** This is a projection, not a measurement:
- Single-job throughput on a 4090 at 8M was 55k tokens/s (docs/compute/cuda-training.md §6). That gives about 0.11 s per 6144-token step, so 4000 steps take about 7.5 min, plus about 1–2 min of evaluation and probe.
- Four jobs sharing one GPU are assumed to run 2–3× slower each, which gives about 20 min per run and 7.5 waves.
- The bound is `MAX_SECONDS=3600` of training per run. In the worst case, 8 waves of about 65 min would take about 9 h.
- The calibration job gives the real per-step time in its `median_step_seconds`. If it is much slower than 0.2 s, adjust `K` before the full grid.
- CPU fallback (`DEVICE=cpu`): about 1.1 s/step at batch 16 on 3 threads, so about 75 min per run.

## Caveats
1. **The QK sign.** The specified `qk` arm turns the query predecessor by j⁻¹ while the key predecessor is turned by j. Because `⟨j⁻¹a, jb⟩ = −⟨a, b⟩` (tested), the predecessor-to-predecessor term enters the score with the opposite sign of the current-to-current term, under the same W_q/W_k. If the intent was positive two-hop matching, that form is `qk_jj` (j on both sides). It runs at 2 seeds as an extra diagnostic arm, and no frozen rule refers to it.
2. **The reference rule is at ceiling on REHEARSE.** With unique key pieces, the n-let backoff solves every rehearsed query (tested). On REHEARSE cells an arm can therefore only meet the rule, never beat it. R5 accordingly counts a capability as "passes and meets the rule (−0.01)", and reports `beats_rule` separately. On BARE g ≥ 1 cells the rule mostly fails, because it copies the gap.
3. **Rule-3 guard (an addition to the frozen rule, stated in the decision output).**
   - When F2 does not pass, R3 is "not informative" instead of "retracted", and the literal tie is still recorded.
   - A conv that exceeds F2 is never more than 0.03 below it, so it also retracts the claim.
   - R1–R3 are not decided until none, f2 and conv have all their seeds.
4. **CUDA.** The `cuda` build of this example was not compiled locally. The lineage operations are ordinary candle tensor ops (narrow, cat, broadcast_mul, matmul), and the fused read is unchanged, but the first pod build is the first CUDA compile of this code.
5. **Steps are not calibrated.** 4000 steps × 16 windows were chosen by analogy with the synthetic bench, which needed about 1800 steps × 8 windows. Nothing local shows this budget reaches recall. A null at this budget is a budget-scoped result. The calibration job's curve shows whether recall is rising.
6. **Not implemented here:**
   - the assessment's "case and prefix-space variants". The capitalised rehearsed key from 0b is Step 4's copy-boundary work, and every key here is lowercase as asserted;
   - multi-lag lanes (Step 2b);
   - a lineage-hashed admission arm (critique point 7).
   The value side (critique 6, lag +1 on the value) is exercised only implicitly, through teacher-forced multi-piece values.
7. **Probe cost.** The final probe makes one forward pass per head × gap × source: 64 passes over 16 windows. That is about 30 s per run on one CPU thread and minor on a GPU. Set `probe_steps=none` to skip it.
8. **What the scores are.** All of this is offline float training of research-only arms, and no serving claim follows from it. Under D11 only F2 has a saveable form, and its serving port is Step 3.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (3 entries; chronological: conversation comments, reviews, review comments)

### comment 5988660033 — 2026-10-05T05:27:03Z — Casey-allard

Adversarial review at fd82c6f7dbd0093613255ae6e72f1394b6af66fb:

**Verdict: request changes.** There is one blocking issue, in the decision table's budget and config integrity. The rest is sound, and I could reproduce it.

## What I ran (exact head)
- `cargo fmt --check` on a `git archive` of the head: only `src/relation_compiler.rs` differs. That file is unformatted on main and untouched here, so the claim holds.
- `cargo test -j 2 -p uor-r4-training --lib -- lineage read_key_shift the_j_lineage random_so4 the_qk_lineage`: **13 passed, 0 failed.**
- `cargo test -j 2 -p uor-r4-training --example mqar-bench`: **13 passed, 0 failed.**
  - The #1017 tokenizer (sha256 `d36d3e87…0f89`, checked) was at the test's default path, so the fact-window test ran rather than skipped.
  - My `--release` build ran out of disk (ENOSPC, the host volume was at 100%), so this run is a **debug** build. I ran it from the author's clean worktree, which is at this exact head with 0 dirty paths.
  - I removed my own scratch copy afterwards. I deleted nothing else. The volume is at about 0.4 GiB free, below a safe margin for further builds.
- I did not compile CUDA, as with the author.

## Correctness: library
- The lineage is applied in `geometric_read_with_source::project`. That is after the projection and before the head split and `fused_read_selected`, so the CPU and CUDA fused reads both see it.
- The geometric-address and latch `Projections` paths bypass `project`, and both are refused.
- `ReadLineage` is opt-in. The default path is unchanged, and the synthetic layout keeps its behaviour: `vocab = VOCAB`, and with no lineage nothing is set.
- Weight decay works as stated:
  - `layers.NN.read.lineage_conv.weight` is excluded by `decayed()`, through its `conv.weight` suffix.
  - `read.lineage_prev` has no `.weight` suffix, so it is excluded too.
- Every refusal path I checked returns `Err`. I saw no new `unwrap`/`expect`/panic outside tests: `save`, served mode, export, grid reference, native replay host check, and the F2 exclusion in both orders.
- `set_read_lineage(Some)` validates before it mutates anything.
- The tests do exercise the risks:
  - causality is bit-exact on rows 0–6;
  - the lineage variables receive gradients;
  - the parameter counts are checked;
  - SO(4) blocks satisfy RᵀR = I and det = +1;
  - the inner-product signs of j and j⁻¹ are tested.

## Correctness: bench
- Placement is correct:
  - `range` is inclusive, so the query fits the context;
  - `distance >= fact.len()`, so a query never overlaps its fact;
  - spans are tracked as free or used.
- Uniqueness holds: keys use fresh pieces against every earlier key and value, gap pieces are never key pieces, and the filler excludes key and value pieces.
- The reference rule works as described:
  - REHEARSE: the 4-let always contains a unique key piece, so the rule is at ceiling.
  - BARE: the rule copies the gap whenever g ≥ 1.
- `report.json` uses the `"status": "complete"` format that the script's `complete()` greps for, and `manifest.json` is written by seal.

## Blocking
1. **`mode=decide` counts runs truncated by the time budget, and mixes runs of different configurations.**
   - `collect()` reads only `status == complete`. It ignores `results.stopped_early_at_max_seconds` and `steps_completed`.
   - A run that hits `MAX_SECONDS=3600` therefore counts as a full seed. Eight jobs share two GPUs, and wprev/conv cost more per step. Under that contention, the frozen R1–R3/R6 verdicts could be set by unequal training budgets rather than by the mechanism.
   - The smoke `rararr-none-s1` root is an instance: `stopped_early: true` at 282/400 steps, and decide still counts it.
   - Decide also never checks that seeds share steps, batch, lr, context and the binary. The PR explicitly offers the calibration root for reuse by the grid, so a calibration run at a different `STEPS` would be mixed in silently.
   - **Fix (cheap):**
     - In decide, skip, or at least flag and exclude from R1–R3/R6, any root with `stopped_early_at_max_seconds == true`.
     - Record `steps_completed` per seed in the table.
     - Refuse roots whose training config differs (`task.common` without seed and out, plus the arm config without lineage). Binding the binary sha would also help.
   - Alternatively, make the script fail loudly. As it stands, a budget-truncated grid would yield a sealed decision.

## Strongly recommended before the pod run (non-blocking)
- **The QK sign.** The test shows `<j⁻¹a, jb> = −<a,b>`. With a shared W_q and W_k, the specified `qk` arm therefore scores the predecessor–predecessor match with the opposite sign of the current–current match. That structurally works against matching a sequence of two pieces.
  - The critique's literal form (`completeness-critique.md` point 2) appears to come from the decomposition `<q, jk> = <j⁻¹q, k>`. It does not read as a positive two-hop term.
  - R7, which asks whether query-side lineage is needed, is run on `qk` at 3 seeds. `qk_jj` runs at only 2 seeds and is outside every rule, so R7 is likely uninformative by construction.
  - **Suggestion:** either raise `qk_jj` to 3 seeds and report R7 for both arms, or record an owner or critique decision on which sign is meant before spending the pod hours. The PR discloses this honestly; the issue is that it is left unresolved.
- **Step budget.** The 4000 × 16 budget is uncalibrated, and the PR says so. Run the calibration job first, as the PR says. If recall is not rising by about 2000 steps, a null across the grid is budget-scoped, and R1 "none passes" cannot fire.

## Nits
- **R3 tie threshold.** About 128 items per decision cell (256 windows × 12 facts / 24 cells) gives roughly ±0.02 SE per seed near 0.95. TIE = 0.03 on means over 3 seeds is acceptable, but the cell n should appear in `decision.json`.
- **Claim order in decide.** `report_output::claim(&out)` runs after `collect()`. AGENTS asks for claim immediately after argument validation. No model is loaded, so this is minor.
- **Rule-3 guard.** The guard is a documented addition to the frozen rule. It is reasonable, but it should be mentioned in the issue card as a rule change made before the run.
- **so4 seeding.** The `so4` rotation is seeded by the run seed, so each seed draws a different R. That is fine, and it should be stated as intended.
- **Unequal GPU load.** GPUs are assigned round-robin by job index, and skipped jobs still consume an index, so load can be unequal on a resumed run. This is cosmetic.

## Claims in the body
The body's claims are measured or labelled as projections:
- 13 + 13 tests;
- fmt;
- smokes, stated as "not evidence";
- wall time, stated as a projection;
- CUDA, stated as uncompiled.

I found no overclaims.


### comment 5988722961 — 2026-10-05T05:33:35Z — Casey-allard

Claude lab, at exact head 0bcbf67d68a26e4fa732550b3d41f49f2ee0255f: **the reviewer's blocking item is fixed.** `decide` now:
- skips any root that stopped early at `max_seconds` or completed fewer steps than planned, and records the counts in `skipped`;
- refuses to mix roots trained with different configurations. The fingerprint is the task settings plus the fact layout, minus the seed, device, threads, save flag, wall cap and probe.

**Verified on the author's smoke grid:** `rararr-none-s1` (282/400 steps) is now excluded, and the other three roots share a config. `cargo test --release --example mqar-bench` passes 13/13, and decide.rs passes rustfmt.

**Nits, carried to the pod launch:**
- qk_jj gets 3 seeds, and R7 is reported for it, because the specified qk arm's predecessor-predecessor term has a negative sign by construction.
- The calibration job runs first.
- Each cell's n is reported next to TIE=0.03.
- The rule-3 NOT_INFORMATIVE guard is recorded as a pre-run rule change.

**Approved; queuing.**

### comment 5988985508 — 2026-10-05T06:01:03Z — Casey-allard

Adversarial review at fd82c6f7dbd0093613255ae6e72f1394b6af66fb:

**Verdict: request changes at this head.** This PR has already merged (0df35259). The blocking decide-integrity item is the same one the earlier review raised, and #1720 (86e85291) fixes it on main. One script defect that follows from it is **still on main**. Everything else below is non-blocking.

## What I ran, at the exact head (`git archive` into $TMPDIR, `CARGO_TARGET_DIR=~/.cache/uor-claude-target`, `-j 2`, macOS M1)
- `cargo fmt --check`: the only diffs are in `src/relation_compiler.rs`, which this PR does not touch. Touched files are clean.
- `cargo test -p uor-r4-training --lib -- lineage read_key_shift the_j_lineage random_so4 the_qk_lineage`: **13 passed, 0 failed.**
- `cargo test -p uor-r4-training --release --example mqar-bench`: **13 passed, 0 failed.** The #1017 tokenizer is present at the test's default path, with sha256 `d36d3e87…` (I checked it).
- I did not compile CUDA and did not run the grid.

## Blocking at fd82c6f7 (fixed on main by #1720)
1. **`decide` counted truncated or mismatched runs as full seeds.** `collect()` only checks `status == complete`. It ignores `stopped_early_at_max_seconds`, `steps_completed`, and whether the roots share one training config. With `MAX_SECONDS=3600`, an uncalibrated step budget and K=8 processes on 2 GPUs, truncation is plausible, and it can differ by arm (wprev adds a width x width map). The calibration root, which the script says the grid will reuse, could also carry a different budget. Either case lets a run set R1–R3/R6.

## Still on main (script, unchanged by #1720)
2. **`step2-parity-bench.sh` never retries a truncated seed.** `complete()` counts any root whose report says `"status": "complete"`, and a max_seconds stop reports exactly that. On a rerun the script prints `skip` for that seed. Since #1720, decide drops the same root. The result: that (pattern, arm, seed) stays missing for good, and R1–R3 show `INCOMPLETE`, unless an operator hand-moves the root. That conflicts with the "a root is never reused" contract. Fix: in `complete()`, also require `stopped_early_at_max_seconds == false` and `steps_completed == STEPS` (for example with `jq`). The same check also stops a calibration root that used another budget from being skipped.

## Non-blocking findings
3. **The 3-seed query arm is the sign-flipped one.** With `qk` (query `j⁻¹`, key `j`), the predecessor–predecessor term is `−⟨q_{t−1},k_{s−1}⟩`; your test confirms this. The positive pairing is `qk_jj`, but it has only 2 seeds and R7 ignores it (R7 only asks "qk passes and f2 fails"). Plan §B writes the served score with `⟨j⁻¹q, k_{s−1}⟩`, which is just the adjoint form of `⟨q, j k_{s−1}⟩`. The critique's `j⁻¹` on the query may have been read from that identity. Suggestion: give `qk_jj` 3 seeds and let R7 accept either query arm. Otherwise a qk failure cannot be read as "query-side lineage is not needed".
4. **R3 compares two things at once (hypothesis).** The PR says that, at `lag1`, the arms "differ only in the operator applied to k_{t−1}". That is not true for `conv`: it also has learned lags 2 and 3. At g ≥ 1 the token before the value is a gap piece, so lag 1 alone never reaches a key piece in one hop, while width 4 does for g ≤ 2. A conv ≥ F2 tie (R3 `J_CLAIM_RETRACTED`) would therefore mix "learned vs fixed j" with "receptive field 1 vs 3". This follows the plan's width-4 control, so it is not a defect here. Still, decision.md should say so, or a width-2 learned conv should be added if R3 fires.
5. **Exclusivity is only checked one way.** `set_read_lineage` refuses latch, address and served mode. But `set_read_identity_latch`, `set_geometric_address` and `set_read_identity_carry` do not refuse a model that already has a lineage. `geometric_address_read` also returns before `read_lineage_projection` runs. So the call order lineage → Held latch → address silently drops the lineage. F2 had the same hole on main, and the bench never reaches it.
6. **`decide` hides NaN.** A missing `full_accuracy` becomes NaN (`unwrap_or(f64::NAN)`), and then `reduce(f64::min)` drops it, so the cell can pass on the remaining seeds. Rates are never NaN as the code stands (`total.max(1)`), so this only matters if the schema drifts. Refusing would be safer.
7. **Recorded deviations from the frozen text, all disclosed:** "meets rule within 0.01" in place of "beats rule"; the F2-must-pass guard on R3; a conv that exceeds F2 also counts as a retraction; and no case/prefix-space variants. I agree with the guard. Post the R3 deviation on #820 as an amendment before the grid runs, so the rule is frozen before any data.

## Claims check
- The body correctly labels the smoke runs as not evidence. It also flags the wall-time projection (2–3 h, worst case about 9 h) and the 4000×16 step budget as uncalibrated, so the calibration run is a real precondition, not optional.
- The library adds no `unwrap`/`expect`/panic; the `expect`s are confined to tests. Weight decay is skipped as stated: `lineage_conv.weight` matches the `conv.weight` exclusion, and `read.lineage_prev` does not end in `.weight`. Save, served, export, grid reference and the native host check all refuse a lineage, and the tests exercise this.
- I checked the script's `device=cuda` route: `baseline_protocol::device("cuda")` → `new_cuda(0)` under `CUDA_VISIBLE_DEVICES`, which is correct.



---
_Comment/review ids in this file: 5988660033, 5988722961, 5988985508_

