# Configuration reference

Every environment variable the workspace reads, with its default and the module
that owns it. Most have a CLI flag equivalent; where both exist, the flag wins.

**The override contract.** The capacity and sampling knobs
(`capacity_override_usize` / `capacity_override_f64`) share one rule: **unset is
κ-neutral — behaviour is bit-identical to not setting it. Set-but-invalid, or
zero, panics.** A knob that silently did nothing would be indistinguishable from
a knob that does not work, and both failures are invisible. Underscore separators
are accepted (`R4_GATE_C_SAMPLE=10_000`).

## Paths and inputs

| Variable | Meaning | Default |
|---|---|---|
| `TLESS_CHECKPOINT` | llama2.c reference teacher checkpoint | `/tmp/ref/out/model.bin` |
| `TLESS_ARTIFACTS` | TLA artifact container | `/tmp/tless_artifacts.bin` |
| `TLESS_STORE` | Graded store | `/tmp/tless_store.bin` |
| `TLESS_TOKENIZER` | llama2.c tokenizer | `/tmp/ref/tokenizer.bin` |
| `TLESS_MODEL` | Model name or CID for `ask` / `chat` | newest `models/*.json`, else `smollm2-135m-instruct` |
| `TLESS_CORPUS_META` / `TLESS_CORPUS_RECS` | Dashboard R4G1-compiler corpus | none |
| `R4G1_ARTIFACT` | Scored R4G1 for serving | `graph/score.r4g1` beside the artifacts |
| `R4_CORPUS_META` / `R4_CORPUS_RECS` | Override the corpus pair everywhere — compiler and every measurement harness | `/tmp/c_meta.bin` / `/tmp/c_recs.bin`; harnesses default to the committed fixtures |
| `R4_ARTIFACTS` | Override the TLA container in harnesses | the committed fixture |
| `R4_STORIES` | `stories.jsonl` giving the construction/held-out split | derived 80/20 split |
| `R4_SCORED_R4G1` | Scored R4G1 for `router_reconnect` | **required, no default** |
| `R4_CODES_PATH` | κ-keyed per-record code sidecar | `code_sidecar::CODES_PATH` |
| `UOR_MODEL_STORE` | CID object store root | `.uor-models` |
| `UOR_R4_HOST` / `UOR_R4_PORT` | Server bind address | `127.0.0.1` / `8000` |
| `UOR_R4_MANIFOLD_CACHE` | Router manifold cache | `manifold_cache_rust.json` |

## Determinism and teacher math

These are κ-relevant: changing them can change artifact bytes.

| Variable | Meaning | Default |
|---|---|---|
| `TLESS_CANONICAL_DETERMINISTIC` | Any value ≠ `0` selects portable libm math and scalar reductions. **Required for the cross-platform Gate E claim.** | off |
| `TLESS_EXACT_SCALAR` | Any value disables the SIMD/Accelerate fast matmul | unset (fast path on) |
| `TLESS_TEACHER_VFORCE_EXP` | macOS only; `0` disables Accelerate `vvexpf` | on (macOS); off under canonical mode |
| `R4_TLESS_TLA6` | `0` opts a compile out of TLA6 emission | on |
| `R4_TLESS_TLA7` | `0` opts a compile out of TLA7 (residual) emission → TLA6 | on |
| `TLESS_REPIN_WRITE` | `=1` regenerates the κ fixture container. **Maintainer decision only** — see the κ re-pin procedure in [AGENTS.md](../AGENTS.md) | off |

## Thread counts

| Variable | Meaning | Default |
|---|---|---|
| `R4_COMPILER_THREADS` | Graph-compiler rayon pool (CLI `--jobs` overrides) | rayon default |
| `R4_CAP_THREADS` | `capacity_scaling` harness | 2 |
| `R4_SCALING_THREADS` | `cover_scaling` harness | 2 |
| `R4_TS_THREADS` | `two_sided_context` harness | env-derived |

## Cover induction and compiler capacity

All follow the override contract above.

| Variable | Default |
|---|---|
| `R4_COVER_DEPTHS` | 3 |
| `R4_COVER_K0` | 8 |
| `R4_COVER_REGIONS_BUDGET` | 256 |
| `R4_COVER_MEMORY_BUDGET_MB` | 512 |
| `R4_COVER_MIN_SUPPORT` | 64 |
| `R4_COVER_ENTROPY_GAIN_BITS` | 0.25 |
| `R4_COVER_SPLIT_CRITERION` | `absolute` (`absolute\|relative\|mdl`; unknown panics) |
| `R4_COVER_RELATIVE_THETA` | — |
| `R4_COVER_MDL_PENALTY_BITS` | — |
| `R4_COVER_SCALE_K0` | `0` (flag: `0/false/1/true`, else panics) |
| `R4_COVER_SCALE_REGIONS_BUDGET` | `0` |
| `R4_COVER_CAPACITY_ALPHA` | — |
| `R4_COVER_CAPACITY_REF_N` | — |
| `R4_CTX_SAMPLE` | `CTX_SAMPLE` |
| `R4_RVQ_SAMPLE_CAP` | 10,000 |

> **Trap.** `R4_COVER_CAPACITY_REF_N` defaults near the 500k fixture's train
> count, so the `scaled-k0` arm passes *degenerately* on any ~500k corpus. Use
> the `@ref50k` arms when that matters.

## Gate C scoring

| Variable | Meaning | Default |
|---|---|---|
| `R4_GATE_C_SAMPLE` | Deterministic **stride** subsample of the held-out split. The sample size and binomial standard error travel with every rate, so a sampled number cannot be read as a census. Unset is the full pass, bit-identical to the pre-sampling evaluation. `<1` or non-integer panics | unset (census) |
| `R4_GATE_C_SKIP_ARMS` | Comma-separated arm-group names not to build. Only known group: `right_context` — the #446 two-sided and latent families, i.e. the closure of the whole-corpus right-context code pass (~60% of a sampled run's wall clock). **Unknown names panic.** Skipped arms report as absent (`null`), never as zeroed rows | unset (skip nothing) |
| `R4_TRANSITION_OUT_DEGREE` | Per-source out-degree cap for forward edges | 8 |
| `R4_EMISSION_ENTRIES` | Per-region emission list bound | 64 |
| `R4_ROOT_TOP_B` / `R4_EXCT_TOP_X` | Candidate list bounds | 64 / 64 |
| `R4_CONTEXT_ENTRIES` | Packed NGRAM context row entries | 64 |
| `R4_FWDA_ENTRY_CAP` | Forward-anchor row cap | 64 |
| `R4_LATENT_CLASS_DEPTH` | Bytes of the right code forming the latent class (clamped to `1..=STAGES`) | 1 |

## Certification

| Variable | Meaning | Default |
|---|---|---|
| `R4_CERTIFY_C_ONLY` | ≠`0` runs only the C serving row, no teacher load | off |
| `R4_CERTIFY_ROWS_ONLY` | ≠`0` skips the C serving row. Explicitly **not** a full certificate | off |
| `R4_CERTIFY_PHASE_C_ONLY` | ≠`0` certifies phase C only | off |
| `R4_CERTIFY_R4G1_BUDGET_SECS` | Readiness-probe wall clock | 120 |
| `R4_CERTIFY_R4G1_EVAL_BUDGET_SECS` | Subsampled-eval wall clock | 600 |
| `R4_CERTIFY_SERVING_BUNDLE` | Bundle for the certify C row | scans under `.` |

## Measurement harnesses

| Variable | Default | Harness |
|---|---|---|
| `R4_CAPACITY_SAMPLE` | 100,000 (`0` = census) | `capacity_scaling` |
| `R4_CAPACITY_TRAIN_SAMPLE` | 0 (census; nonzero is the biased fast look and **disables the sidecar in both directions**) | `capacity_scaling` |
| `R4_CAP_COVER_MAX_TRAIN` / `R4_CAP_SKIP_COVER` | 0 / off | `capacity_scaling` |
| `R4_SCALING_FRACS` | `13,25,50,100` | `cover_scaling` |
| `R4_SCALING_ARMS` | `absolute,relative,mdl,scaled-k0,relative+scaled` | `cover_scaling` |
| `R4_SCALING_MAX_TRAIN` / `R4_SCALING_MAX_HELD` | 0 / 50,000 | `cover_scaling` |
| `R4_TS_RIGHT_R` / `R4_TS_MAX_EVAL` / `R4_TS_SIG_TOP` / `R4_TS_SIG_BINS` / `R4_TS_MIN_SUP` | 4 / 250,000 / 4 / 16 / 1 | `two_sided_context` |
| `R4_TS_GROWTH_FRACS` | `6,12,25,50,100` | `two_sided_context` |
| `R4_RECONNECT_CONSTR_STORIES` / `R4_RECONNECT_HELD_STORIES` | 2,000 / 200 | `router_reconnect` |
| `R4_INFILL_STRIDE` | 4 | `anchor_infill` |
| `R4_CD_SAMPLE_EVERY` / `R4_CD_LABEL` / `R4_CD_AB_BUNDLE` | 5 / `unlabeled` / `/tmp/score.r4g1` | `r4g1_cd_ab` |
| `R4_HOPF_CONSTR_STORIES` / `R4_HOPF_PROBES` | 2,000 / 500 | all router harnesses |
| `R4_HOPF_FISHER_PAIRS` / `R4_HOPF_REDESIGN` | 10,000 / off | `hopf_retrieval_quality` |
| `HOPF_OCCUPANCY_REPORT_PATH` | `target/hopf_sector_occupancy/report.json` | `hopf_sector_occupancy` |
| `R4_SELFMATCH_PROBES` | 100 | `geometry_selfmatch` |
| `R4_LEXW_CONTENT_QUERY` | off (`1` enables content-query mode) | `lexical_weight` |
| `R4_ZETA_ARM` | must be `1` or the harness no-ops | `zeta_state_retrieval` |
| `R4_MEMLIFT_CONSTR_STORIES` / `R4_MEMLIFT_PROBES` | 2,000 / 500 | `memory_lift_corpus` |
| `R4_ARM_SKIP_SAMPLE` / `R4_ARM_SKIP_FIXTURE_DIR` | 10,000 / the committed fixtures | `gate_c_arm_skip` |
| `R4_STATUS_FIXTURE_DIR` | `/tmp/r4-status-fixture` | `status_policy` |
| `R4_PARITY_POSITIONS` / `_GEN_TOKENS` / `_RUNS` / `_CORPUS_POSITIONS` | 256 / 128 / 3 / 1000 | BDD teacher parity |
| `R4_FMM_POSITIONS` / `R4_FMM_RANK` / `R4_FMM_TOLERANCE` | 256 / — / — | BDD FMM |
| `SMOLLM2_SOURCE` | — | `smollm2_adapter` tests |
| `UOR_R4_API_E2E_SOURCE` | — | `uor-r4-api` E2E test |

## Runtime state files

Written under `.uor-models/` (or `UOR_MODEL_STORE`):

| File | Purpose |
|---|---|
| `audit_log.json` | Per-turn question, answer, UOR address, κ + PASS/DRIFT, generation mode, latency. Rendered by `r4 audit` |
| `last_model.txt` / `last_model_name.txt` | Orchestrator's last model selection |
| `last_engine.txt` | Persisted engine preference; **silently pins the cascade** for requests that omit `engine` |
| `sources/<name>/` | Downloaded Hugging Face teacher sources |
| `compiled/<name>/` | Compiled bundles (`score.r4g1`, `tless_artifacts.bin`, `corpus.meta`, `corpus.records`) |
| `corpora/` | Observation corpora |
