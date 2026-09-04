# Configuration reference

For the current Rust model, use the CLI options and examples in the
[native geometric workflow](native_geometric_workflow.md). The explicit
`r4 geometric` command family configures preparation, training, evaluation,
generation and the local service; model context and operator settings travel
with the artifact. The [canonical plan](integration/project-track.md) owns
architecture and budget policy, while
[current-state.md](integration/current-state.md) owns implementation status.
Native options do not wait for a historical stage or later Rust port.

## Retained compiler and reference configuration

The inventory below covers existing compiler/runtime and measurement lanes
across earlier architectural eras. `TLESS_*`, `R4G1_*`, teacher and release
variables apply to their named paths. Defaults for plain `ask`/`chat`, source
tokenizers and older HTTP engines do not select the `r4 geometric` model.
These references preserve compatibility and reproducibility; they do not
reintroduce a Python model dependency or change the current project goal.

The listed environment variables give their defaults and owning modules.
Where a listed CLI flag and variable both exist, the flag wins.

**Legacy override contract.** The capacity and sampling helpers
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
| `R4G1_ARTIFACT` | Scored R4G1 for research loading. Production admission additionally requires the schema-2 release envelope and its content-bound full-census deployed-quality report | `graph/score.r4g1` beside the artifacts |
| `R4_CORPUS_META` / `R4_CORPUS_RECS` | Override the corpus pair everywhere — compiler and every measurement harness | `/tmp/c_meta.bin` / `/tmp/c_recs.bin`; harnesses default to the committed fixtures |
| `R4_ARTIFACTS` | Override the TLA container in harnesses | the committed fixture |
| `R4_STORIES` | `stories.jsonl` giving the construction/held-out split | derived 80/20 split |
| `R4_SCORED_R4G1` | Scored R4G1 for `router_reconnect` | **required, no default** |
| `R4_CODES_PATH` | κ-keyed per-record code sidecar | `code_sidecar::CODES_PATH` |
| `UOR_MODEL_STORE` | CID object store root | `.uor-models` |
| `UOR_R4_HOST` / `UOR_R4_PORT` | Server bind address | `127.0.0.1` / `8000` |
| `UOR_R4_MANIFOLD_CACHE` | Router manifold cache | `manifold_cache_rust.json` |

## Tokenizer adapter selection

Registered source tokenizers are selected by CLI/API fields, not environment
variables. CLI commands that accept `--source` also accept the atomic pair
`--tokenizer-family FAMILY --tokenizer-version N`. Both flags must be present
or absent. When absent, discovery succeeds only when the source directory has
one unambiguous supported definition; a directory containing both
`tokenizer.json` and `spiece.model` requires the explicit pair. Unknown pairs
and any attempt to combine a registered adapter with legacy `--checkpoint`
input fail closed.

For raw SentencePiece sources, use family `sentencepiece-unigram`. Version 2
is the current, reference-correct adapter; automatic discovery of one
unambiguous `spiece.model` selects it. Version 1 remains registered only as the
immutable published behavior whose unknown id decodes to the literal model
surface `<unk>`. `N` is part of the semantic identity, not a compatibility
range: consumers do not substitute a newer or older version. Both versions
read the pinned 32,000-piece `spiece.model`; neither applies the T5 Hugging
Face wrapper's 100 sentinel tokens or EOS-appending post-processor.

The host adapter performs text normalization and encoding. The deployed
tagged `tokenizer.bin` is decode-only and carries the complete adapter binding
(family, version, source-model CID, and policy digest). Serving requires a
matching host encoder for prompt text. Missing/mismatched identity, unavailable
encode, invalid token ids, and decode failures are errors; an explicitly tagged
tokenizer does not fall back to a legacy tokenizer. SentencePiece observation
records have no original-source byte anchors because normalization and unknown
collapse do not preserve a total input-offset map.

## Determinism and teacher math

These are κ-relevant: changing them can change artifact bytes.

| Variable | Meaning | Default |
|---|---|---|
| `TLESS_CANONICAL_DETERMINISTIC` | Any value ≠ `0` selects portable libm math and scalar reductions. **Required for the cross-platform Gate E claim.** | off |
| `TLESS_EXACT_SCALAR` | Deprecated compatibility input. Llama/shared projections already use pinned exact `uor-matmul`, so this variable no longer changes their arithmetic; it does not select a GPT-2 Conv1D/lm-head route. | unset |
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
| `R4_PARITY_WORKERS` | Size `W` of the one persistent exact output-row worker pool. Diagnostic range is `1..=available_parallelism()`; `0`, malformed values, and requests above the host budget fail instead of being clamped. Fixture-present execution adopts the faster exact W=available/W=min(4, available) tuner result, so this input cannot force a different binding selection. The candidate widths are a bounded tuning choice, not a utilization target. | all available logical CPUs before probe selection |
| `R4_PARITY_STREAMS` | Independent private-state trajectory and physical-batch width `S`. `S` is scientific work, not a worker bound: it may exceed or be less than `W`. The binding fixture-present workload requires the eight canonical prompt lanes. | 8 |
| `R4_PARITY_BATCH_PER_WORKER` | Bounded exact output-row task fan-out per worker. This changes scheduling granularity only; it cannot select Accelerate or change exact arithmetic. Requested and effective fan-out are recorded. | 4 |

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
| `R4_CERTIFY_C_ONLY` | ≠`0` runs only the historical C research diagnostic, no teacher load. It is not production-admission evidence; use `r4 deployed-quality` | off |
| `R4_CERTIFY_ROWS_ONLY` | ≠`0` skips the C serving row. Explicitly **not** a full certificate | off |
| `R4_CERTIFY_PHASE_C_ONLY` | ≠`0` certifies phase C only | off |
| `R4_CERTIFY_R4G1_BUDGET_SECS` | Readiness-probe wall clock | 120 |
| `R4_CERTIFY_R4G1_EVAL_BUDGET_SECS` | Subsampled-eval wall clock | 600 |
| `R4_CERTIFY_SERVING_BUNDLE` | Bundle for the certify C row | scans under `.` |

## Normative deployed-quality evaluation (#933)

These knobs drive the teacher-free `R4G1Runtime` evaluator. Worker partitions
are deterministic and reductions are replayed in canonical position order, so
changing the worker count changes wall time but not report bytes. Once an
evaluation phase launches it writes monotonic progress plus its phase terminal;
a missing control, timeout, or absent identity is `UNAVAILABLE`/skipped and
cannot authorize production.

| Variable | Meaning | Default |
|---|---|---|
| `R4_DEPLOYED_QUALITY_MODE` | `full` selects the complete held-out census; every other value selects the deterministic sample | sample |
| `R4_DEPLOYED_QUALITY_POSITIONS` | Sample size when mode is not `full` | 6,000 |
| `R4_DEPLOYED_QUALITY_WORKERS` | Evaluation workers. Results are canonically ordered after parallel execution | available logical CPUs |
| `R4_CERTIFY_R4G1_BUDGET_SECS` | Cheap readiness-probe wall-clock budget | 120 |
| `R4_CERTIFY_R4G1_EVAL_BUDGET_SECS` | Sample/census wall-clock budget | 3,600 |

The historical certify-C diagnostic above and this normative command share
the two budget variable names but retain distinct defaults: 600 seconds for
the non-binding diagnostic, 3,600 seconds for `r4 deployed-quality`. Explicit
CLI flags override their environment variables.

Every invocation against a valid, non-symlink bundle root first creates
`evidence/deployed_quality_invocation_terminal.jsonl`. The create-once JSONL
journal is synced after its `started` row and again after exactly one
`completed`, `failed`, or best-effort `interrupted` terminal row, so argument,
bundle-discovery, cross-surface, witness, sample-gate, projection, and evaluator
failures are all durable. A synced `started` row without a terminal row means
the process was externally interrupted or otherwise unresolved; signals,
power loss, and `SIGKILL` cannot be guaranteed to run cleanup code. An absent,
symlinked, or non-directory bundle cannot safely host in-bundle evidence and
therefore fails before journal creation. Reusing the same staged bundle is
refused rather than overwriting its first invocation.

The invocation journal is local, non-semantic evidence. It is excluded from
the generation CID, production admission, and release archives, as are the
resource sidecar and terminal transcript described below. Phase progress and
phase terminal files under `graph/` remain authoritative only after that phase
actually launches.

Production admission accepts only a schema-2 `release-bundle.json` whose
component digests and compiler/selector identities reproduce from the loaded
graph, teacher artifact, corpus pair, tokenizer, tokenizer adapter, score
report, cover report, and deployed-quality report. A sampled report remains
research evidence even when all measured rows are favorable.

The release CLI exposes the same contract directly. `--mode full` cannot
bypass the cheap instrument: it first runs the exact 6,000-position prefix of
the canonical label-free, story-distributed order on the same bundle/evidence
generation. Any structural, reachability,
non-futility, planted-control, cross-surface, or witness falsifier emits
`STOP:` and returns without starting the census. `PROCEED:` launches the
census. A typed `INCONCLUSIVE:` extends the same immutable nested order to
18,000 positions; if that interval still overlaps the gate, it can launch only
the census and only when the measured reachable ceiling remains sufficient.
Before any census launch, the authorizing stage's measured throughput is
scaled to the full population; a projection beyond the configured evaluation
budget or one hour refuses launch and requires a revised run contract. No
sample report can authorize production admission.

```bash
cargo run --release -- deployed-quality \
  --bundle .uor-models/compiled/<model>-staging \
  --compiler-revision <full-40-character-revision> \
  --mode sample --positions 6000 --workers 8
```

For a canonical run, wrap the command so host identity, effective workers,
wall time, peak child RSS, free storage, bundle growth, exit status, and every
graph evidence-file size are durable even when the evaluator fails. A
create-once JSONL beside the summary samples the complete child process tree's
CPU share, RSS, process count, host-memory headroom, and filesystem headroom
every five seconds. The wrapped evaluator still streams its phase counters,
throughput, and ETA. On
the canonical macOS host, `/usr/bin/script -e -F` preserves that stream in a
separate terminal transcript and propagates the child exit status; require a
new transcript path before launch just as the wrapper requires a new sidecar.
The resource summary, live-sample JSONL, and transcript are explicitly
non-semantic and are never admission inputs. The inventory snapshot is captured
when the child exits. If the live-sample JSONL is placed inside the bundle it is
therefore present in that snapshot; the subsequently written resource summary
and the outer transcript's final trailer are not. Both resource paths are
append-only: the wrapper refuses to overwrite either one, and it verifies that
its `--bundle` exactly matches the wrapped command's `--bundle` (evaluation) or
`--bundle-root` (graph emission) before launch. It also records `--workers` or
`--jobs` as the effective requested worker count. Use `--samples-output` to
select the JSONL path; otherwise it is `<output>.samples.jsonl`.

```bash
test ! -e .uor-models/compiled/<model>-staging/graph/deployed_quality_full.log
/usr/bin/script -q -e -F .uor-models/compiled/<model>-staging/graph/deployed_quality_full.log \
  python3 scripts/run_deployed_quality.py \
  --bundle .uor-models/compiled/<model>-staging \
  --output .uor-models/compiled/<model>-staging/graph/deployed_quality_full_resources.json \
  -- target/release/r4 deployed-quality \
  --bundle .uor-models/compiled/<model>-staging \
  --compiler-revision <full-40-character-revision> \
  --mode full --positions 6000 --workers 8
```

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
| `R4_LEXW_CONTENT_QUERY` | Off in this historical harness; `1` selects the deployed content-query arm. The router itself defaults content-query mode on (#490/#502). | `lexical_weight` |
| `R4_ZETA_ARM` | must be `1` or the harness no-ops | `zeta_state_retrieval` |
| `R4_MEMLIFT_CONSTR_STORIES` / `R4_MEMLIFT_PROBES` | 2,000 / 500 | `memory_lift_corpus` |
| `R4_ARM_SKIP_SAMPLE` / `R4_ARM_SKIP_FIXTURE_DIR` | 10,000 / the committed fixtures | `gate_c_arm_skip` |
| `R4_STATUS_FIXTURE_DIR` | `/tmp/r4-status-fixture` | `status_policy` |
| `R4_PARITY_POSITIONS` / `_GEN_TOKENS` / `_RUNS` / `_CORPUS_POSITIONS` | 256 / 8 / 1 / 1000. The binding registered work uses exactly eight lanes and an eight-token maximum; S4 starts at one causal continuation step per lane, then may extend through 2, 4, and 8 only while more work can change the verdict. Smaller generation caps are diagnostic only and cannot qualify the full run. Fixture-present execution requires one causal run. The report records actual tokenized/executed work; caps are not forward-count claims. | BDD teacher parity |
| `R4_PARITY_PREFLIGHT_ONLY` | Off. Set to `1` to validate the schema-2 `release-bundle.json` production envelope and deployed-quality bindings, then parse and exercise the tokenizer, legacy artifact/store, graph, graph report, and all eight canonical deployed seeds without opening the live teacher. It writes a `uor-r4.teacher-parity-preflight/1` success or refusal artifact with `teacher_source_opened=false` and `teacher_forwards=0`, then exits before Cucumber. The ordinary BDD fixture loader publishes the same artifact automatically. Any missing or invalid prerequisite blocks expensive teacher work truthfully. | Teacher-free BDD preflight |
| `R4_PARITY_PREFLIGHT_REPORT` | `target/teacher-parity/teacher-free-preflight.json`. Relative paths resolve from the repository workspace root for both the BDD owner and direct tuner. Atomic deterministic output used by explicit and automatic preflight. Non-PASS artifacts retain the exact reason, selected paths, safe per-input presence/CIDs, and current `authorizing_contract_cid` before returning; an empty, non-Unicode, uncreatable, or unwritable path fails the preflight visibly. The direct tuner validates the current contract, exact report/source/bundle paths, and recomputed compiled-input plus complete production-admission CIDs before opening teacher weights. | Teacher-free BDD preflight and exact live admission probe |
| `R4_PARITY_PROGRESS_EVERY_SECS` | 10. Periodic human-readable and flushed JSONL heartbeat cadence, including while no exact forward has completed. In-flight matrix/tile/cell/scalar counters drive liveness and the ETA basis; whole-forward throughput remains separate. Zero or malformed values fail. | BDD teacher parity |
| `R4_PARITY_MAX_WALL_SECS` | 28,800 (8 h). Stops dispatching new work at the ceiling and records `ABORTED`; it never converts partial work into PASS. Values above 28,800 fail: the override may shorten but never widen the hard ceiling. The full run is also refused before launch unless the cheap probe projects completion below this ceiling. | BDD teacher parity |
| `R4_PARITY_REPORT` | `target/teacher-parity/parity-report.json`. Final versioned JSON report; the flushed event stream is written beside it as `parity-report.events.jsonl`. A create/write/flush failure is `FAIL`, not missing telemetry on a PASS. | BDD teacher parity |
| `R4_PARITY_TELEMETRY` | `1` / enabled. `0` exists only for focused planted/unit controls; fixture-present parity refuses to run without telemetry. Invalid booleans fail. | BDD teacher parity |
| `R4_EXACT_PROBE_REPORT` | `target/teacher-parity/exact-multicore-probe.json`; relative paths resolve from the repository workspace root and flushed events use `exact-multicore-probe.events.jsonl`. An empty/non-Unicode path refuses admission. | Exact live admission probe and BDD consumer |
| `R4_EXACT_PROBE_POSITIONS` | 1; valid range `1..=8`. Every configured position is exercised over the same eight canonical states at W=available and W=min(4, available), deduplicated when equal. | Exact live admission probe |
| `R4_PARITY_SOURCE` | `.uor-models/sources/smollm2-135m-instruct`. Teacher source directory used consistently by the exact probe and fixture-present BDD run. Relative paths resolve from the repository workspace root; empty or non-Unicode values fail closed. The teacher-free preflight records this selection but does not open it. | Exact live admission probe and BDD teacher parity |
| `R4_PARITY_BUNDLE` | `.uor-models/compiled/smollm2-135m-instruct`. Compiled schema-2 `release-bundle.json`/deployed-quality envelope plus tokenizer/artifact/store/graph/report bundle used consistently by teacher-free preflight and fixture-present BDD work. A pre-schema-2 bundle is refused before teacher access. Relative paths resolve from the repository workspace root; empty or non-Unicode values fail closed. | Teacher-free preflight and BDD teacher parity |
| `R4_FMM_POSITIONS` / `R4_FMM_RANK` / `R4_FMM_TOLERANCE` | 256 / — / — | BDD FMM |
| `SMOLLM2_SOURCE` | — | `smollm2_adapter` tests |
| `UOR_R4_API_E2E_SOURCE` | — | `uor-r4-api` E2E test |
| `UOR_R4_RELEASE_BUNDLE_PATH` | — | `release_bundle_packager` real-local-bundle test (`#[ignore]`d; mirrors the `UOR_R4_API_E2E_SOURCE` convention) |
| `PORT` | 8000 | `./uor-r4-cli` orchestrator only (shell script; the `r4` binary itself reads `UOR_R4_PORT`) |

Teacher-parity controls govern host-side verification only and do not enter
artifact identities or deployed serving. `S` private sequence states must share
one immutable weight allocation and advance in one physical exact batch; the one
`W`-thread pool must schedule disjoint output rows without nested
oversubscription.
Every output row must keep the pinned `uor-matmul` dot reduction intact;
splitting or reassociating the reduction dimension is not permitted. Results
return to canonical prompt/position order before metrics are reduced.

The 36 logical teacher-forced positions are executed once in six physical
batches with registered widths `8, 8, 8, 7, 4, 1`. That transcript retains each
canonical lane at its distinct final teacher-forced prompt prefix (lengths
`5, 6, 5, 4, 3, 4, 5, 4` for the pinned tokenizer), rather than truncating
different prompts to a colliding common prefix. S4 clones the templates,
appends the already-computed teacher next token to each logical seed, and times
only new causal continuation steps. Exact batches carry the per-lane sequence
positions explicitly, so variable histories do not serialize the cohort.
There is no duplicate live-teacher prefill and no independent full-model S4
warm-up. Preparation, decode, and one-shot elapsed time remain separate
evidence fields.

Before measurement, the harness reserves the bounded model-wide batch buffers,
exact input/output transpose buffers, and one exact scratch slot per dedicated
worker, then exercises the worker pool/backend with a tiny known product. Its
elapsed time, retained capacity, capacity-growth event count, and actual added
capacity bytes are recorded as excluded preparation. Measurement counters are
reset without discarding those buffers. Every transcript and S4 physical
forward must subsequently report zero workspace-growth events and bytes;
otherwise the run fails rather than presenting allocation churn as steady-state
throughput.

Conditional source, artifact, store, graph, tokenizer, or corpus evidence is
reported as `AVAILABLE`, `UNAVAILABLE`, `FAILED`, or `NOT_RUN`. An absent
fixture is never a parity PASS merely because the enclosing test executable
returns success. The progress/event/report schema and the launch gate are
specified in [the #932 run record](teacher_parity_parallelism_932.md).

The durable artifact set is:

| Artifact | Schema id | Default path | Role |
|---|---|---|---|
| Progress events | `uor-r4.teacher-parity-progress/2` | `target/teacher-parity/parity-report.events.jsonl` | Flushed heartbeat, phase, work, failure, and completion snapshots |
| Final run report | `uor-r4.teacher-parity-report/2` | `target/teacher-parity/parity-report.json` | Empirical timing/resource/occupancy verdict and exact reason |
| Deterministic evidence | `uor-r4.teacher-parity-evidence/2` | `target/teacher-parity/parity-report.evidence.json` | Timing-free identities, exact outputs, reductions, and verdict inputs |
| Exact admission probe | `uor-r4.exact-multicore-probe/2` | `target/teacher-parity/exact-multicore-probe.json` plus `.events.jsonl` | Source/host/executor-bound equal-work W=available/W=min(4, available) selection evidence |

For `uor-r4.exact-multicore-probe/2`, `probe_deadline_policy` means that no new
exact forward is admitted after 3,600 seconds; an already-active fixture load or
exact forward may finish, then the probe records `ABORTED`, and elapsed time at
or beyond the deadline cannot qualify. The report's required `events` object
binds the sibling `file_name`, full-byte `content_cid`, `byte_len`,
`record_count`, `final_record_number`, `final_event`, `final_status`,
`final_qualifies_full_run`, and non-cyclic `report_body_cid`. FINAL is synchronized
before the report is atomically committed, carries
`sequence == final_record_number == record_count`, and must be the last JSONL
record. Admission validates the current sidecar before teacher weights load;
missing, truncated, appended, or tampered bytes refuse the run.

Schema `/1` bytes describe the superseded fixed-sweep design and remain
historical evidence. Schema `/2` binds the adaptive candidate set, fastest-exact
selection policy, and the complete registered admission shape: 36 transcript
logical forwards in batch widths `8, 8, 8, 7, 4, 1`; eight continuation tokens
across eight lanes; 100 logical forwards in 14 physical shared-weight batches;
zero-based maximum sequence position 13; and private-state capacity 14. A
smaller operator cap may produce diagnostic evidence but cannot qualify the
binding suite. `reference_workers` names the first measured exact candidate,
`equal_to_reference` binds every candidate's raw trace to it, and the cheap
worker-pool/backend `prestart` is recorded separately and excluded from timed
forward rates. CPU and RSS fields are diagnostics: they must be structurally
valid and truthful, but a platform-reported `UNAVAILABLE` value does not by
itself refuse exact admission. A later change to field meaning, type,
requiredness, units, or artifact partition requires another schema id rather
than reinterpreting an existing record.

## Served model identity (#655-F)

The canonical served model id on every serving surface (OpenAI `/v1`
routes, native `/api/chat`, WS, WASM) is **`r4`**. Requests may omit
`model`, send `r4`, or send the deprecated pre-flip alias `uor-r4`
(accepted for a compatibility window); responses and `/v1/models`
always report `r4`, and OpenAI wire ids are `r4`-prefixed
(`chatcmpl-r4-…`, `resp-r4-…`, `msg-r4-…`, `system_fingerprint:
r4-{mode}`). The CLI `client` subcommand's `--model` default is `r4`.
Per-bundle logical/physical names are metadata (`/uor/v1/status`), not
the served identity. Engine/tier names (`r4g1`, …) are unrelated to the
model id and unchanged.

## Runtime state files

Written under `.uor-models/` (or `UOR_MODEL_STORE`):

| File | Purpose |
|---|---|
| `audit_log.json` | Per-turn question, answer, UOR address, κ + PASS/DRIFT, generation mode, latency. Rendered by `r4 audit` |
| `last_model.txt` / `last_model_name.txt` | Orchestrator's last model selection |
| `last_engine.txt` | Persisted engine preference. Under the `experimental` profile it **silently pins the cascade** for requests that omit `engine`; under `production` (the default) a non-r4g1 value is silently inert |
| `engine_profile.txt` | Serving profile (#655-E2, #789): `production` or `experimental`. Absent, empty, or unparseable ⇒ `production` (fail-safe), under which only the r4g1 tier is admitted on the cascade endpoints, the `/api/tless/*` bypass endpoints are declined (#789-G1; `/api/r4g1/*` stays open), an explicit non-r4g1 `engine` request returns a typed decline echoing the requested string (#789-G3.2), and discovery counts a model active only with a text-ready R4G1 graph (#789-G2). An explicit engine name outside the recognized vocabulary is a typed decline on every profile (#789-G3.1). `/uor/v1/status` reports the active profile in its `profile` field. Read fresh on every request; since #790-6 both this file and `last_engine.txt` resolve against the store root (`UOR_MODEL_STORE`, defaulting to `.uor-models/`), superseding the 2026-08-18 audit's CWD-relative note |
| `sources/<name>/` | Downloaded Hugging Face teacher sources |
| `compiled/<name>/` | Base compiled bundle. It may hold legacy absence, an explicit historical attention/1+dense/1 pair, or a fresh current attention/2 bundle with dense absent. Inventory includes graph/artifact outputs, `tokenizer.bin`, `tokenizer_adapter.json`, `attention_operator.json`, optional `dense_operator.json`, `corpus.meta`, and `corpus.records`. |
| `compiled/<name>-attention-v2/` | Resolver-owned current attention/2 root with dense absent, used when a historical base must remain immutable. |
| `compiled/<name>-attention-v2-dense-v2/` | Resolver-owned current GPT-2 root. It must carry learned-absolute attention/2 plus `gpt2-source-dense/2`; current dense provenance is invalid in either lower-precedence root. |
| `corpora/` | Observation corpora |

### Managed source-execution-era resolver (2026-08-15)

`-attention-v2` and `-attention-v2-dense-v2` are reserved physical-root
suffixes; downloaded source basenames ending in either are refused before
mutation. Suffix stripping uses longest match. The base and exact-suffix reload
names are aliases for one logical model, attach to `sources/<logical-name>/`,
and report the selected physical root plus optional attention/dense records.
A missing source is allowed for the #718 decode-only path; a present-invalid
source is terminal.

Every managed operation inspects all three roots before choosing one. A
malformed/nonregular binding, an unbound populated suffix, a future version,
an impossible attention+dense pair, duplicate semantic identities, or an
unfinished lower-precedence identity is terminal and byte-preserving. Current
startup/reload also requires the canonical corpus pair and
`graph-cover/cover_report.json` to carry the same registry-exact execution pair
as the root sidecars. Historical v1 and dense absence remain readable where
documented. These checks reconcile metadata. The cover-graph PROV/1 byte section itself
landed with #637 phase 3 (PR #738): `cover` now emits a PROV/1 section
unconditionally (a deliberately-empty record when no identity flags are
supplied).
