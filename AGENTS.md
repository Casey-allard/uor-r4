# AGENTS.md — uor-r4

**Owner-directed recovery (2026-09-04):** Build native geometric AI in Rust.
The [project plan](docs/integration/project-track.md) is the canonical goal and
development plan. The [current state](docs/integration/current-state.md) names
the active implementation; live GitHub owns issue status. Historical stage
orders, one-task stops and fixed experiment windows do not override the owner's
current scope. Preserve the evidence those experiments produced.

## Native geometric AI agent policy

<!-- agent-execution-policy:start -->
The repository-wide mode is `native_geometric_ai`. Its stable machine contract
is [agent-execution-policy.json](docs/integration/agent-execution-policy.json),
with the readable [execution policy](docs/integration/agent-execution-policy.md).
This section overrides historical process and scheduling rules below. Technical
invariants still apply to the runtime where they are declared.

- Use Rust for data preparation, training, artifact construction and inference.
  Training may use floating point and matrix multiplication. Final inference
  executes learned geometric operators through bounded routing, state updates
  and integer/table lookup. A dense transformer hidden behind lookup is not the
  target. Preserve old Python/dense references as evidence; do not add a Python
  model implementation or product dependency.
- Prime addresses and ordered n-lets, fixed zeta-zero phases, R4/S3/H4 state and
  transport, exact `Z[phi]`, chirality/polarity, typed paired-H4/icosian geometry
  and UOR identity are primary mechanisms. Name their implemented roles and
  missing pieces. Architectural priority does not imply measured predictive
  advantage; a failed experiment does not demote the whole architecture.
- Develop both conversation/memory and coding/reasoning toward alpha using the
  same native model path. Keep external research optional and inspect the
  actual source when adopting a mechanism.
- Continue within the owner's authorized objective. A request for the whole
  plan permits its necessary successive tasks; do not stop after one historical
  issue by default. Use an isolated full worktree, coordinate independent
  subtasks, preserve user material and deliver through protected pull requests.
- Configure context/training/evaluation windows and wall-time, RAM, new-storage,
  thread and checkpoint limits for the available machine. Charge cumulative
  work across preparation, training, evaluation, retries and resumes. Diagnose,
  correct and retry within the remaining budget when it can advance the result.
  There is no global 15-minute cutoff or one-retry quota. Stop/checkpoint at
  configured limits; do not silently increase the budget or incur external cost.
- Compile and exercise the changed Rust path. Use focused tests for meaningful
  causal, arithmetic, serialization and interface risks and relevant broader
  checks when needed. Do not require a blanket full suite or proof dossier for
  every edit. A queue compatibility acknowledgement is not a test result.
- Keep open development evaluation available during learning and final held-out
  evaluation separate after design selection. Preserve prior results at their
  exact artifact/data/operator/control/budget/decision scope. Distinguish proof,
  measured behavior and hypothesis; `UNAVAILABLE` is not model-quality evidence.

Changes to these stable goals require owner direction and protected delivery.
A task template, stale skill or agent judgment cannot silently change them.
<!-- agent-execution-policy:end -->

## What this repo is

UOR-R4 is a local, CPU-first geometric AI project. It aims to learn useful
conversation, memory, reasoning and coding through prime/zeta/R4 geometric
state, routes and operators, with a Rust training/artifact/inference lifecycle
and a bounded integer/table serving target. This is the objective, not an alpha
or frontier-capability claim.

Existing compiler/runtime, geometric, proof, dashboard and learned-reference
components are reusable parts and evidence. Evolve the actual native path; do
not turn a comparator, mechanical checkpoint or visible shell into a capability
claim. See the current-state pointer for what has actually executed.

## Workspace layout

- `crates/uor-r4-core` — active prime-route math/manifest/attention foundation + historical transformerless runtime
- `crates/uor-r4-router` — active geometric memory/router + historical word-Markov decoder and dashboard backend
- `crates/uor-r4-graph-format` — R4G1 packed artifact format, two-stage validation, borrowed `GraphView`
- `crates/uor-r4-graph-compiler` — offline graph-compiler stages (observation, cover induction, packing)
- `crates/uor-r4-graph-certify` — offline certification/measurement (Gate C `score` harness, `score_runtime` reference scorer, certificates)
- `crates/uor-r4-graph-runtime` — `no_std` allocation-free R4G1 graph runtime (engine, routing, patch chains)
- `crates/uor-r4-graph-cli` — `r4 transformerless …` CLI stage dispatch (convert-r4g1, scenarios, corpus tools)
- `crates/uor-r4-model-source` — offline source teacher/comparator and historical forward/KV/trace runtime
- `crates/uor-r4-proof-model` — executable proof obligations + proof-status matrix
- `crates/uor-r4-api` — typed compile + engine library façade for downstream consumers (wraps the CLI-shaped stages; see its README)
- `crates/uor-r4-workbench` — source-only native Four-fact host candidate with a private same-executable worker and comparison adapter; unbuilt and runtime-unverified
- root package `uor-r4-wasm-router` — façade + `r4` CLI + local server/chat
- `docs/` — plan, RFC (`transformerless/R4G1.md`), baseline, threat model, explainers,
  and the per-issue measurement records (`docs/<topic>_<issue>.md`)

Documentation entry points, in the order a newcomer should read them:
`README.md` (what it is, quickstart, CLI/HTTP/config reference) →
`docs/integration/project-track.md` (canonical goal and development plan) →
`docs/integration/current-state.md` (current implementation) →
`docs/geometric_intelligence_programme.md` (architecture and claim boundaries)
→ `CONTRIBUTING.md` (the short form of this file) → this
file (the full operating manual) → `docs/RESEARCH.md` (what is measured, closed
and open) →
`docs/MODEL_LIFECYCLE.md` (active decoder and historical compile lanes) →
`docs/CONFIGURATION.md` (every environment knob).

**Keep them true.** When a measurement revises a claim, correct it where it is
asserted — README, `docs/RESEARCH.md`, and the record itself — rather than
letting a superseded number survive because it lives in three places. Records in
`docs/` are appended to, not rewritten: the history of what was believed and when
is part of the evidence.

UOR standards (`uor-addr`, `UOR-Framework`) are **pinned git dependencies** in
`Cargo.toml` — a fresh clone builds with no extra checkouts. The
`uor_standards/` directory is legacy material excluded from the workspace
build (`Cargo.toml` `exclude`); its `.gitignore` entry blocks new additions,
but ~1,100 legacy files remain tracked in the tree (recorded 2026-08-18,
baseline audit).



## Development checks

Compile and run the changed product/model path. Typical checks are:

```bash
cargo fmt --check
cargo check -p <touched-package> --all-targets --offline
cargo test -p <touched-package> <focused-test> --offline
```

Choose checks for the actual risk. State/update and arithmetic changes need
focused verification; a model behavior change needs a representative execution.
Do not repeat broad workspace, BDD, no_std, fuzz, WASM, conformance or release
checks unless their boundary is affected. Run the claim-wording check when
changing claims. Record actual commands and outcomes, including unrun checks.

The protected queue retains five historical required names. In the
[current workflow](.github/workflows/ci.yml), pull requests and merge groups
run `fmt / clippy / tests / no_std / κ` as formatting, the
Rust architecture-policy check and focused native model, context, allocation,
CLI/service tests. It does not run every check named in that historical label.
The other four statuses explicitly acknowledge compatibility only; they do not
run audit, fuzzing, WASM or Gate C. The broader legacy verification jobs remain
available through manual `release_qa` dispatch. Their retention is not evidence
that they ran or that they certify the new model. Report actual job steps.

The toolchain is pinned in `rust-toolchain.toml`. Use rustup-managed cargo
(`~/.cargo/bin/cargo`); a Homebrew binary earlier in PATH can ignore the pin.
Pin changes belong in a dedicated reviewed change because teacher/reference
floating-point results can be sensitive to toolchain math.

## Execution-lane invariants

- **Native training:** Rust may use floating point, matrix multiplication and
  gradients to learn geometric read/write/selection and nonlinear operators.
  Bind the source/configuration, tokenizer, data and learned parameters to each
  artifact. An offline teacher can be a declared comparator or training source;
  it cannot author serving responses.
- **Native inference:** build toward learned geometric transitions and bounded
  routing/lookup over the primary prime/zeta/R4 representation. Name any
  unfinished operation and its measured cost. Runtime source-model/provider
  access and dense transformer attention/MLP disguised as a lookup are excluded
  from the target. A prototype is not promoted merely because it uses Rust.
- **Typed geometry:** keep R4/S3 compute, Hopf observation, retained fiber and
  torsion, and paired-H4/icosian representation distinct. Preserve exact
  `Z[phi]`, chirality/cosine polarity and artifact-bound zeta identities. Do not
  invent a semantic metric from hash bits or silently drop orientation.
- **Frozen TLA/R4G1 runtime:** its normative kernel remains XOR/AND/OR/shift/
  rotate/popcount/integer add-subtract/compare/table reads, with no multiply,
  divide, float or steady-state allocation. New native training does not weaken
  this existing scoped contract.
- **Artifact determinism:** identical pinned compiler inputs must retain their
  declared artifact determinism. New learned artifacts bind the actual data,
  training seed/configuration, parameters, geometry and format version; do not
  claim bitwise cross-backend training reproducibility without measuring it.
- **Errors:** return `Result` with focused enums at library boundaries; no
  `unwrap`/`expect`/panic on recoverable paths. Preserve `forbid(unsafe_code)`
  in portable runtime and format crates.
- **Claim language:** [formal_vocabulary.md](docs/formal_vocabulary.md) remains
  normative for proof and capability claims. Labels distinguish definitions,
  assumptions, objectives, guarantees and empirical criteria. No blanket proof
  campaign is required to implement or train the next native mechanism.

## Historical product and research rules

The bullets below preserve earlier programme decisions and evidence boundaries.
They do not override the active project-track sequence above.

- Follow the reconciled #820 dependency chain. #961 closed GI-1/S0 lexical
  geometry at reversible-state scope. #952's A1.0 gate preserved candidate and
  value reachability but stopped before a scorer because the reusable
  non-digest summaries erase earlier order. #967's A1R delivery added the exact
  associative ordered fold and passed the frozen scope, global, fold,
  incremental, and support contracts. Its full arm produced distinct `ll`/`rr`
  relative states on 6/6 queries and changed the same-candidate state in 5/6
  paired comparisons, but the scalar shortest-Cayley-distance readout collapsed
  both candidates to energy 2 and tied on 6/6. Its terminal verdict is
  `RETAIN_STATE_ONLY`
  (report `blake3:f0db7a5d5c81d51ebf3b4bf8a2715c4960ec16b14161e8bf7598d7b98c48c881`).
  #970's corrected target-free preflight then enumerated the complete paired-H4
  domain: 120×120 = 14,400 ordered pairs, 120 relative `D=X*Y^-1` rows, 45
  exact signed `(1,i)` R4-heatmap classes, and 480 typed-null pairs. Across 36
  fixture decisions it exercised 14 classes; construction coverage was 12/12
  and pure, construction classes covered 10/12 validation decisions, the
  no-class-splitting oracle ceiling was 10/12, strict construction transfer was
  0/6, and eight exact heatmap classes were incompatible. The hard gate stopped
  before scalar search; every downstream selection, control, and placement row
  is `NOT_RUN_IDENTIFIABILITY_HARD_STOP`, not PASS. The terminal literal is
  `RETAIN_H4_STATE_ONLY_ADVANCE_MULTICHANNEL_A1Q`. Its contract, universe, and
  report identities are respectively
  `blake3:2daacf538c022fab9580d1e124af6c18d0b06da04604fbc962a01bda57f08a98`,
  `blake3:dca725c0ec6060166bcd0023df956e1ff029661b5fa7800ccb9f20808712b796`,
  and `blake3:5f9239150dea8c0c27c4dfa6ad2e4d0068bc3d18afc127b315c0ec358ceddb3f`.
  This is only a bounded heatmap-readout identifiability negative: fixed-zeta
  phases, ordered n-lets, exact `phi` radial transport, and typed geometry
  adapters remain structural, diagnostic, or control state. It does not
  promote attention or generation. #970 closed through protected PR #972.
  #969 then delivered one causal R4/S3 least-cost route-attention mechanism and
  one matched decoded smoke. #953 has implemented a bounded source-free
  library/CLI decode/render/append loop. Its rank-preserving relabel smoke
  terminated `REVISE_I1_GENERATOR_IN_PLACE`; the later
  `PrimaryThenAdjacentSpinFallbackV1` repair recovered exact `{still}` then
  `{run,runs}` primary support under equal work while still consulting and
  truthfully tracing non-admitting adjacent-spin rows. The one permitted
  four-arm run produced `still run` for both full-path prompts and `still runs`
  for both state-disabled prompts, with deterministic replay. The terminal
  therefore remains `REVISE_I1_GENERATOR_IN_PLACE`, now localized to
  candidate-relative representation/scoring rather than admission or state
  starvation. The first frozen local candidate-placement preflight then
  reproduced 7/7 construction prototypes with zero class collisions, but real
  placement selected 0/2 intended candidates while its same-artifact cyclic
  placement control selected 2/2; generation and replay were `NOT_RUN`. #953
  was then blocked by #983, whose independently frozen
  `ConstructionCausalReturnV1` produced pure usable construction classes but
  transferred to 0/6 held-out decisions. #983 stopped
  `UNAVAILABLE_ZERO_CONSTRUCTION_TRANSFER` before deployed selection, payload
  inversion, or #953 generation and is now closed as bounded negative evidence.
  #986 then stopped `UNAVAILABLE_FRAME_OR_POPULATION`: its raw corpus
  reproduced, but the exact population/codec commitment and a complete
  same-frame lexical SpiralCore operator map were unavailable. Placement,
  Gate 0, labels, selection, and the historical #953 path were `NOT_RUN`. The
  later B0/#989 reset and separate matched #953 table-tie intervention have
  since passed. #973 Gate 0 has now retained one bounded exact-candidate
  prior-prefix copy-attention mechanism, the frozen paragraph slice retained
  one exact-descriptor/entity-binding stored-phase path selector, and the
  frozen conversation slice retained one exact-descriptor cross-turn entity-
  role stored-spin path selector. The first bounded-global relation failed
  target-free; its V2 repair later passed, the first natural placement failed,
  and the bounded gated-delta core trailed plain delta. Direct-attention V2 was
  non-promotable; equal-manifold-budget V3 then rejected the current mixed-gauge
  H4 projection/connection/optimizer combination against a working plain path
  and an inference-time coherent alternative-connection swap. V4 then passed
  construction covariance but failed held-out functional binding. `HELM-D-R4`
  ordinary-softmax parity in transported R4/Spin frames subsequently passed and
  remains qualified. Intrinsic Lorentz V1 attempt 02 stopped unavailable before
  D3 on covariance, with diagnostic NLL worse than donor and flat.
  Source-faithful learned-manifold V2 then produced a valid non-D3
  construction-validation negative: learned Lorentz failed retention and
  matched parity, while its controls established sensitivity only. The 8/8
  contract's attempt stopped at its two-document preflight and rejected tangent
  readout. Provider-free-at-execution, source-backed
  `R4SoftmaxReferenceGeneratorV1` (`HELM-D-R4`) generation and its explicit
  opt-in, loopback-only dedicated native HTTP endpoint now pass. Dashboard
  wiring/readiness and static/WASM-isolation checks pass; hosted Pages remains
  static/offline without a functioning chat backend/artifact lowering. The Q16
  suffix trace student is complete with bounded distillation but looping output.
  #1019 is an optional frozen 12-layer, 13,130,784-parameter ordinary-softmax
  R4/Spin quality-capacity improvement. The model-side
  population/smoke/parity subgates passed; MPS stopped
  `UNAVAILABLE_HARDWARE_BUDGET` on time for the frozen eight-hour offline
  implementation, and full training through replay remains `NOT_RUN`. The
  fused-AdamW/deferred-logging fast path was slower (`4.485223` versus signed
  `3.491307 s/step`), so #1019 tuning/full-run work stops and remains
  optional/paused. The #1017 `r4 generate` path remains the working prototype;
  #954 C1-SB2 and C1-SB3 are preserved negatives. C1-SB4's independently frozen
  full-source structured-margin attention arm reached only `70/126` fit and
  `35/63` sealed exact records and stopped before Rust/checkpoint/development/
  product; do not retry it. C1-SB5 then fit `56/56` paired records but reached
  only `14/28` sealed, with bit-exact row-swap equivariance and `0/28`
  mean-query/attention-off controls. Its products remained unopened and the rung
  retired before checkpoint/head/Rust/development work. CUDA and external GPU
  execution are out of scope.
  Intrinsic/readout alternatives,
  resonance-based softmax replacement, full-model recurrent lowering, and
  exact deployment are parked. D3 remains `NOT_RUN`.
  #954's final source-free terminal remains blocked behind #973, and #955 remains
  blocked behind #954. #954 and #955 own correctness and reasoning respectively.
  #962 owns durable multi-turn CLI/HTTP chat, persistence, isolation, and
  hive-memory; #963–#965 then own optimization, formal closure, and release.
- Sequence strictly from the current reset: working source-free table baseline
  (#989, established) → one matched geometric intervention (#953, accepted) →
  direct geometric attention (#973: retained bounded scope evidence → H4
  scaffold/V4 held-out negatives → HELM-D-R4 full-decoder softmax parity →
  intrinsic Lorentz V1 construction-unavailable → valid non-D3
  learned-manifold V2 construction-validation negative → tangent-readout
  localization rejection → qualified provider-free-at-execution,
  source-backed `R4SoftmaxReferenceGeneratorV1` (`HELM-D-R4`) generation →
  verified opt-in, loopback-only dedicated native HTTP endpoint (dashboard
  wiring/readiness and WASM isolation PASS; hosted Pages static/offline without
  chat backend/artifact lowering) → construction-only layerwise oracle traces
  [COMPLETE] → source-free Q16 suffix student [BOUNDED DISTILLATION; LOOPING] →
  `R4SoftmaxTraceStateStudentV1` [COMPLETE; FAIL PROMOTION] → #1012
  observability [COMPLETE; INSUFFICIENT SUPPORT] → #1014 direct attention
  [ATTENTION PASS; QUALITY FAIL] → #1017 exposure continuation [NLL-ONLY FAIL]
  → #1019 frozen 12-layer parameter-capacity campaign [MODEL SUBGATES PASS;
  FROZEN OFFLINE MPS IMPLEMENTATION OVER 8 H; FUSED FAST PATH SLOWER; OPTIONAL/
  PAUSED; FULL CAMPAIGN NOT_RUN; CUDA/EXTERNAL GPU OUT OF SCOPE]) →
  working bounded #1017 `r4 generate` prototype → #973 qualified retained
  language path → rejected paired-H4/direct/layerwise/learned-associative
  capacity seams → V5 predictive write/binding
  [`PREDICTIVE_BINDING_NO_TERMINAL_CAPACITY`; `STOP_WITHOUT_GENERATION`] →
  #954 source-span pointer development negative → C1-SB2 source-relative
  relation preflight negative → C1-SB3 bounded attention transfer / exact
  preflight negative → C1-SB4 structured-margin negative → C1-SB5 paired-query
  binding negative → final source-free correctness terminal [BLOCKED BEHIND
  #973] → reasoning [BLOCKED ON POSITIVE CORRECTNESS] → optimization/purity/
  release. The older placement/transport
  sequence is retained as evidence, not an active implementation queue.
- Kappa is canonical identity/serialization, never the tokenizer or semantic
  distance. The pinned lexical codec is provenance-bound but opens no weights.
- Preserve the project shorthand `E8 = H4 x H4`. Its concrete implementation
  and serialization is the golden/Galois-coupled icosian pair
  `H4 ⊕ phi H4` with fixed basis, glue, forward map, and inverse witness.
- Keep **required structural/storage representation** distinct from a
  **qualified semantic scoring term**. Paired-H4/icosian coordinates may remain
  mandatory for canonical storage, address reconstruction, and inverse
  witnesses without being valid ranking features. Hopf, H4, zeta, icosian,
  SpiralCore, trajectory, hypersphere, winding/window, projection-energy,
  shared-factor, and resonance terms remain storage fields, diagnostics, or
  future hypotheses. #969 establishes only its local ordered-S3 path mechanism
  as load-bearing. #953's historical path code establishes reusable
  decoded-loop plumbing and tiered admission, but its unchanged full-path
  choice did not establish natural grammar. The separate B0 table-tie
  intervention now establishes only its bounded geometric accuracy increment
  and decoded comparison. #973 Gate 0 adds one exact-candidate prior-prefix
  copy mechanism. Its paragraph slice adds only one construction-bound exact-
  descriptor/entity-binding stored-phase path selector, and its conversation
  slice adds only one construction-bound exact-descriptor cross-turn entity-
  role stored-spin path selector. They do not qualify a general entity,
  paragraph, or conversation model. The first global exact-spin relation failed
  target-free; only a newly frozen #973 repair may qualify bounded-global state.
- Keep **candidate admission** separate from **harmonic influence**. In #953,
  `PrimaryThenAdjacentSpinFallbackV1` uses I1/I2/ordered-sentence plus divisor
  as the primary admission tier. Adjacent-spin rows are always consulted and
  report physical presence truthfully, but admit candidates only when that tier
  is empty. Do not delete or disguise a physical adjacent-spin hit.
  A newly frozen #973 repair may apply one global-epoch/operator-bound result to every immutable
  reference in the same exact signed-S3/Hopf/fiber/torsion class. Similar but
  non-identical states require a separately frozen finite relative-angular
  kernel built independently over exact classes. The existing adjacent-spin
  rows remain retrieval fallback/diagnostics, not operator coefficients.
  Neither operator mechanism may widen #953 support.
- An exact kappa miss must not collapse unseen global history to a suffix-only
  default, but global ordered-state behavior is tested on an independently
  frozen global-snapshot permutation rather than by mutating session history.
- The completed #969 evidence compares exactly full retained path, last-only,
  and state-disabled. #953's repaired natural agreement run carried full path
  and state-disabled arms under equal support/work: both full-path prompts chose
  `still run`, while both disabled prompts chose `still runs`. That deterministic
  negative localized the next revision to candidate-relative
  representation/scoring; it did not qualify incompatible natural choices.
  Do not retroactively add a broad construction/validation programme, channel
  census, weight sweep, control matrix, or higher-scope fixture to the completed
  #969 smoke or #981 tiered-admission decision. The frozen #953 placement
  revision permitted exactly one tiny pre-frozen construction/evaluation
  separation: construction-only observed transitions compiled the overlay and
  a label-free, selection-blind raw relation census froze before expected
  continuations were attached; frozen evaluation labels could not tune it.
  Full-history disjointness did not supply operative-representation anti-recall:
  the decisive suffixes exactly recalled shorter construction subhistories. Its
  exact preflight selected the opposite candidate on both prompts while the
  placement-permuted control selected both intended candidates, so it stopped
  before decoded generation or replay. That failure does not authorize a second
  representation, a wider split, or broader scope under the same contract.
- Teacher output may label or compare only after a source-free report freezes.
  It is never substituted for the product response.
- #973 Gate 0 selections emit their exact prefix-coverage and matched-work
  witnesses. The retained paragraph selector emits its exact two-fact binding,
  stored-phase path, and matched-work witnesses. The retained conversation
  selector emits its cross-turn binding, lower/global-scope isolation, stored-
  spin path, and matched-work witnesses. The first global relation emitted its
  detached-carrier/exact-class census and commuting-fold negative; later
  hierarchy selections must emit
  their corresponding scope coverage witness. Exact recall,
  grammatical generation,
  correctness, and reasoning remain separate gates.
- Start with the smallest product artifact that can falsify the stage. A
  negative stops or redesigns it; it does not authorize a larger harness.
- Do not add a graph section, proof lane, benchmark framework, BDD suite, or
  corpus-scale run before the active product decision requires it.
- Testing/QA is dormant by default. Activate only named product/release checks;
  missing or unrun evidence remains `NOT_RUN` or `UNAVAILABLE`.

## Historical release-only κ reproduction reference (dormant)

Do not run this during ordinary development. It is retained only for a future
release issue that explicitly activates the cross-platform κ decision.

- Setup (once per machine): `curl -sL -o /tmp/run.com
  https://github.com/trholding/llama2.c/releases/download/experimental/run.com
  && cd /tmp && unzip -o run.com out/model.bin -d ref`
- Run: `TLESS_CANONICAL_DETERMINISTIC=1 cargo test -p uor-r4-core --release
  --offline --test kappa_reproduction -- --ignored` (the canonical mode is
  required for the cross-platform Gate E claim; check
  /tmp/ref/out/model.bin exists before trusting a green result).
- The certificate fixture is re-pinned under the portable canonical math path.
  Legacy accelerated teacher builds remain platform-sensitive and are not the
  cross-platform reproducibility claim.
- Re-pinning is a **maintainer decision**, done via
  `dump_baseline_kappa` (`--nocapture`) → review diff → adopt →
  `TLESS_REPIN_WRITE=1` regenerates the fixture container. Compiler redesigns
  legitimately change κs; drift from nondeterminism never does — investigate
  first (double-compile determinism check), then re-pin.

## Historical TLA/R4G1 teacher-parity certification (dormant)

The commands below are records of the historical lane. They require an
explicit product/release decision before execution.

The suite below remains valid for the frozen compiled-runtime evidence lane. It
is not an entry gate for geometric-decoder development and must not be added to
routine decoder PRs.

`features/suites/teacher_parity_benchmarks.feature` (steps in `tests/bdd.rs`)
runs the live SmolLM2-135M teacher against both compiled runtimes (legacy TLS
store and R4G1 graph) on teacher-forced accuracy (top-1 / top-8 recall /
Δbits), generation speed, and kernel invariants (zero-multiply op census,
zero-alloc hot path, witness self-consistency), κ-pinning every input. A
corpus-replay scenario (S6) additionally measures in-distribution top-1
against the recorded teacher labels in the bundle's `corpus.meta` /
`corpus.records` through the deployed paths — no live teacher — reporting
next to Gate C's anchors (Gate C scores a held-out partition with the
compiler-side plain baseline; S6 replays recorded positions, so its ~0.43
figures sit above the 0.181 anchor by construction). It runs
in the default `cargo test --test bdd` when `.uor-models/sources/
smollm2-135m-instruct` and the compiled bundle are present. If a conditional
fixture is absent, that evidence is **UNAVAILABLE** even when the enclosing test
process exits successfully; never report the unexercised parity scenario as
PASS.
Budgets: `R4_PARITY_POSITIONS` (256), `R4_PARITY_GEN_TOKENS` (8, a hard
adaptive ceiling), `R4_PARITY_RUNS` (1), `R4_PARITY_CORPUS_POSITIONS` (1000).
Thresholds are pinned empirical floors with ~20%
margin; the ~1% top-1 figures are out-of-distribution honesty, not a bug —
the suite's 8 prompts are novel text, unlike Gate C's same-corpus replay
(see the comment above the constants in `tests/bdd.rs`).

The fixture-present live-teacher work is required to be an exact-parallel,
multi-stream host measurement, not a single-stream latency benchmark hidden
behind an intra-forward thread pool. `S = R4_PARITY_STREAMS` is the independent
private-state trajectory/batch width; `W = R4_PARITY_WORKERS` is the one
persistent exact output-row worker pool. Scientific coverage stays fixed at
eight canonical lanes in an `S = 8` shared-weight batch. `S` and `W` are
independent: the bounded tuner compares the host's all-logical-CPU width with
its four-worker candidate (deduplicated when equal) over the same eight-lane
work and selects the faster exact point. On the binding M1 these candidates are
`W = 8` and `W = 4`; neither width is a utilization quota or performance goal.
A physical teacher
batch must advance all `S` states through shared immutable weights while the
`W` pool divides output rows only; no worker may split or reassociate a row's
pinned exact dot-product reduction. Compiled candidates must receive the same
lane seeds and logical workload, and all results must reduce in canonical
prompt/position order. The shared teacher transcript also retains the S4 prefix
states, eliminating duplicate teacher prefill and the independent S4 warm-up.

Every live run must emit flushed JSONL progress events, deterministic evidence,
and a final JSON report with fixture identities/status, actual tokenized work,
configured/effective/current/peak stream and worker occupancy, complete
physical-batch/logical-forward/matrix/tile/cell/scalar-term accounting,
per-lane state/output identities, elapsed/rate/ETA basis, CPU/RSS readings, a
retained-workspace capacity/growth ledger, and a typed final `PASS`, `FAIL`,
`UNAVAILABLE`, `ABORTED`, or `NOT_RUN` verdict. Model, transpose/output, and
per-worker exact scratch buffers are prepared outside timed work; any capacity
growth during a measured forward fails the steady-state evidence. A heartbeat
must continue while an individual exact forward is in flight; its liveness and
ETA use monotonic in-flight exact scalar-term progress (worker-task progress is
the fallback), while completed-forward throughput remains a separate rate. The
bounded live tuner compares equal S=8 work at W=available/W=4 without full-model
candidate warm-ups, establishes exact trace equality plus owner-plan
reconciliation, and selects the faster exact point. W=1/2/4/8 equality remains
a focused structural gate. Speedup and CPU utilization are recorded diagnostics
rather than admission floors. Full work launches only when the selected exact
point has complete evidence and a safety-adjusted projection below the
configured hard wall ceiling, capped at eight hours. S4 starts with one causal
decode step per lane and extends through 2, 4, then 8 only while more work can
change its verdict. Any missing or failed evidence refuses the full run. See
`docs/teacher_parity_parallelism_932.md` and `docs/CONFIGURATION.md`.

The exact teacher, pinned `uor-matmul` crates, and both compiled S4 engine paths
have narrow `profile.test.package` opt-level 3 overrides in the root manifest.
Do not remove them and then interpret an opt-level-0 BDD rate as serving
performance. The rest of the workspace retains the normal test profile.

Before spending any live-teacher work, run
`R4_PARITY_PREFLIGHT_ONLY=1 cargo test --test bdd --offline`. This teacher-free
gate parses the tokenizer and every compiled prerequisite, exercises all eight
canonical legacy and graph seeds through typed deployed decisions, and writes a
content-bound `uor-r4.teacher-parity-preflight/1` success or refusal artifact
before exiting. The ordinary BDD fixture loader publishes the same artifact
before it can open the teacher. Refusals retain the exact reason, safe input
paths/CIDs, `teacher_source_opened=false`, and `teacher_forwards=0`; an
unwritable artifact path is itself a visible failure. A failed preflight blocks
the tuner and full suite; it is not bypassed as a fixture skip. The artifact's
`authorizing_contract_cid` binds the current executor, BDD, model, manifest,
and toolchain sources. Direct tuner invocation validates that binding plus the
selected paths and current compiled-input plus complete production-admission
CIDs before loading teacher weights.

## Current delivery and long-running work

Use a named branch, stage files by name and deliver through a protected pull
request. Never directly push `main`, bypass protection with `--admin`, or write
fabricated status checks. Review the actual diff and meaningful checks before
merging; resolve conflicts hunk by hunk. A partial issue delivery says
"References #N", not "Closes #N". Keep active assignments and issue status true.

The [project plan](docs/integration/project-track.md#practical-iteration-and-machine-budget)
owns development-budget guidance. Before a substantial run, use a representative
measurement or valid existing timing to select a feasible dose, context window,
thread count and cumulative machine budget. Track progress and preserve useful
checkpoints using the model's existing runner. Do not create a separate
supervision framework where ordinary checkpointing and visible progress suffice.
A longer useful evaluation window is an ordinary configured development choice;
it does not require changing the mechanism or overwriting old evidence.

Focused verification follows the changed boundary. Cross-target checks matter
when that target's code changes; full release certification is a separate
larger decision. Batch coherent low-risk changes when appropriate, while
keeping independently reviewed model/state changes understandable.

<details>
<summary>Historical process amendments — superseded for current execution</summary>

The following material preserves earlier process decisions and examples. It
cannot impose a stage order, fixed time/retry limit, test prohibition or extra
approval requirement on the owner-directed native recovery above.

## Process conventions

- **PR workflow; queue as transport only.** Do not push directly to `main`;
  use a named branch and PR. Ruleset `19597522` cannot currently be edited, so
  the repository emits its five required names as explicit no-QA
  acknowledgements and uses the forced merge queue only to transport the
  reviewed commit. Never report those acknowledgements as tests or PASS.
- **Dormant governance cleanup (#940).** A future administrator may remove the
  obsolete ruleset and its queue. Until then, contributors with write-only
  permission use the transparent transport shim; they must not fabricate
  external statuses, use `--admin`, or reactivate development QA.
- **Release activation only.** A future product-ready release issue may
  manually dispatch a bounded product/release QA scope and may propose a new
  minimal release ruleset. The old always-on research queue is not restored by
  default.
- **Per issue**: assign yourself (WIP signal) → branch `issue-<n>-<slug>` →
  work + produce the declared product/release evidence → run only checks the
  issue explicitly activated → open PR → merge through the protected path →
  close the issue with its evidence and merge commit. Milestones mirror plan
  phases.
- **PR review** (incl. Copilot-generated): review the changed path and its
  declared evidence before merge. Run no QA by habit; use only the activated
  product/release checks. Resolve conflicts
  hunk-by-hunk — whole-file `checkout --theirs/--ours` has silently dropped
  upstream features before (the TLA5 incident).
- **Committing while subagents work in-tree**: add files **by name**, never
  `git add -A` — in-flight agent work (unregistered modules, half-written
  tests) must not be swept into unrelated commits (the cover.rs incident).
- **Tests that encode era sensitivity**: `src/tless_uor.rs`
  `indexing_and_generation_update_store` asserts resolution depths that depend
  on the fixture artifact's class signatures — update the expected depths with
  an era note whenever the fixture is regenerated.
- **ScoreQ**: there are intentionally two compatible definitions in the frozen
  graph lane
  (`uor-r4-graph-format::ScoreQ` wire newtype; `uor-r4-core::score_q::ScoreQ`
  with compiler-side f32 conversions). Do not add a third or prioritize their
  consolidation ahead of the active route-native intelligence sequence.

## Long-run discipline (process amendment, 2026-08-06)

Compiles and Gate C runs at corpus scale cost hours. The waste is never the
run itself; it is launching one whose result could not have changed what we
do next. Three gates, in order, before any run measured in hours:

**One — reachability arithmetic.** From numbers already in hand, compute the
ceiling on the metric the run intends to move, and write it in the run
contract. Worked example (#460, 2026-08-06): the record showed 97.9% of
held-out positions resolving as ExactContext, so at most 2.1% ever touch the
graph path, so ANY cover-side change is capped at about 2.1pp of headline
movement. That is a five-minute calculation and it invalidates a four-hour
run. If the ceiling is below the effect you are hoping for, do not launch.

**Two — the cheap instrument is a hard gate.** Where an instrument exists
that reports the structural precondition, it runs FIRST and its verdict is
binding. For graph experiments,
`cargo test -p uor-r4-graph-certify --test capacity_scaling -- --ignored`
takes about twelve minutes and prints a SATURATION verdict per structure. For
decoder experiments, use the issue's tiny-overfit, reachability, or short-rollout
preflight instead; do not run a graph instrument that cannot decide the decoder
question. If the relevant instrument fails, the long run does not launch. On
2026-08-06 the graph instrument reported
`records_per_full_key: 36.02 SATURATED` and `exct.supported_record_fraction:
0.9882 SATURATED` before a multi-hour Gate C run that then confirmed exactly
what those two lines already implied.

**Three — pre-declare the decision, not just the exit rule.** Exit rules
("positive if at least 2pp") say how to read the number. A run contract also
says what each outcome CAUSES. If the positive and the negative branch lead
to the same next action, the run has no decision value; drop it or redesign
it until they differ.

**Run contract** — paste into the issue before launching, and post the
outcome against it afterwards:

    metric to move:      <name, current value>
    reachability ceiling: <arithmetic, with the numbers it came from>
    instrument + verdict: <which cheap test, what it must report to proceed>
    exit rule:           <threshold, pre-declared>
    if positive:         <the next action>
    if negative:         <the next action, and it must differ>
    cost estimate:       <wall-clock, and what else it blocks>

**Execution-plan amendment (2026-09-03): measure eligible plans for substantial
offline compute.** Before a deterministic offline training, compilation or
measurement job needs new timing evidence, predeclare a small set of materially
plausible, scientifically eligible plans and benchmark a representative unit:
CPU BLAS/Apple Accelerate, MPS only when the frozen contract allows it, and
selected intra/inter-op thread and process/worker counts. Reuse previously
qualified evidence when its semantic inputs, workload shape, backend and
runtime bindings remain unchanged; do not repeat calibration by habit.
Select the measured-fast stable plan that preserves the declared result and fits
memory. Maximum threads or concurrent arms are not automatically faster; use
sequential arms when shared-memory contention wins. Record hardware, backend and
BLAS provider, intra/inter-op threads, worker processes, utilization, unit
timings, determinism/equivalence evidence, and the selected plan in the run
contract. A substantial job must not default to one core without measured
evidence that one core is fastest or a scientific constraint requiring it.
CUDA is eligible only when the active issue explicitly places it in scope.
Offline acceleration never changes the CPU/table-native deployed-runtime target.

**A long run must be observable before it starts.** Anything expected to exceed
15 minutes needs a finite work denominator, completed/remaining units,
throughput, an ETA derived from that denominator, durable checkpoints, and a
typed terminal report. A missing denominator, absent ETA, non-resumable
checkpoint, or execution plan without the contract's required semantic,
resource and representative timing evidence prevents launch. Compare only the
predeclared eligible plans; four processes, four threads and four concurrent
arms are different choices, not a universal requirement. A scientifically
required single plan must state that constraint and still establish budget
admission. #958's one-worker/four-worker schema-2 complete-manifest result from
2026-08-26 remains historical evidence for that exact path, not a mandatory
comparison for new workloads. Re-establish only the bindings affected by a
change before its dependent decision; reuse unaffected evidence. Performance
evidence for compiled paths comes from release builds; a debug run cannot
authorize larger work.
Eight hours is a hard kill ceiling, not an estimate. Reaching it stops the run
and records `ABORTED`, `NOT_RUN`, or the last completed bounded result; never
continue because the process may be nearly finished.

**Cross-target checks are scoped certification.** A native workspace check does
not build WASM. Activate `cargo check --target wasm32-unknown-unknown -p
uor-r4-wasm-router --lib` only when a product/release decision explicitly names
the WASM boundary; it is not routine implementation work. A
filesystem-touching helper gated `#[cfg(not(target_arch = "wasm32"))]` needs a
wasm counterpart, or every caller has to become cfg-aware; prefer the
counterpart. This was found the expensive way on PR #470, where PR checks were
green and the queue build failed.

**The forced queue is transport, not QA.** The five compatibility jobs perform
no checkout or verification and carry no product evidence. A silent or blocked
merge command is not authorization to use `--admin`, run dormant QA, or
manufacture external contexts; inspect the queue/check state and keep the
status language exact.

**Issue hygiene that goes with it.** Every issue filed mid-run gets an owner
and a named next action, or it gets closed with its record. Assignment means
actively-working-now; unassign when a track parks so the board reads true for
everyone. A PR that ships only part of an issue's scope says "References #N",
never "Closes #N" — GitHub will auto-close the issue on merge and the
unfinished half loses its home.

## Batch flow for small issues (process amendment, 2026-07-29)

Small, low-risk issues (docs, help text, certifier-side rows, test
harnesses, telemetry) are worked on ONE integration branch (`batch-N`)
with one commit per issue (message refs `#N`). Run only product/release checks
explicitly activated by the batch contract, once per batch of 3-6 issues—not
per issue. Do not turn authoring feedback into an implicit compile/test gate;
all other certification remains dormant.
Runtime-kernel and serving-semantics changes still get individual PRs.
Measurement runs are background science with scheduled harvests; they
never sit between two pieces of code work.


</details>

## Things that bite

- `/tmp/ref/out/model.bin` disappears on reboot/periodic /tmp cleanup — κ tests
  may still exit successfully without exercising reproduction; the Gate E
  evidence is **UNAVAILABLE**, not PASS.
- `crates/uor-r4-graph-format/fuzz/target` must never be committed (gitignored).
- Fuzz targets need nightly (`cargo +nightly fuzz run …`); the stable
  deterministic mutation smoke runs under plain `cargo test`.
- The on-disk compiled store in `.uor-models/` predates the u32 token
  migration (TLS1-u16); `runtime::parse_store_legacy_u16` reads it, and a full
  recompile is needed to refresh it.
- After deleting a git worktree, cached rlibs in the shared `target/` can
  carry the dead worktree's baked paths and poison the local register gates
  (#788, AUD-VER-001). `repo_root()` now resolves at runtime, but any other
  compile-time `env!("CARGO_MANIFEST_DIR")` user (fixture-loading tests) has
  the same hazard — `cargo clean -p repo-model -p repo-conformance -p xtask`
  clears the register gates; when in doubt, clean the crate whose test reads
  a repo path.
- `cargo test` is fail-fast at the test-binary level: one poisoned binary
  hides every suite after it. Use `cargo test --workspace --no-fail-fast`
  for local gate runs so a single bad binary cannot mask the rest
  (AUD-VER-002).
