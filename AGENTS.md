# AGENTS.md — uor-r4

Guidance for agents (human or otherwise) working in this repository.
**Post-#958 intelligence architecture and sequencing are authoritative in
[`docs/geometric_intelligence_programme.md`](docs/geometric_intelligence_programme.md).**
The geometric causal decoder plan, prior S0–S7 completion plan, and
graph-compiler implementation plan are retained as historical
engineering/evidence records; none decides what is built next. Native GitHub
relationships now mirror the programme: #961 closed with reversible S0 state;
#952 stopped at `REDESIGN_ORDERED_ROUTE_SUMMARY`; #967 repaired the ordered
state but terminated `RETAIN_STATE_ONLY`; #970's corrected, target-free A1P
gate produced the bounded paired-H4-derived exact R4-heatmap result
`RETAIN_H4_STATE_ONLY_ADVANCE_MULTICHANNEL_A1Q` and closed through protected
PR #972. #969's mechanism-first pivot delivered one causal R4/S3 least-cost
route-attention mechanism and one matched two-unit decoded smoke at
`PROCEED_TO_I1_WITH_CAUSAL_R4_PATH_ATTENTION`. #953 implemented the first
bounded provider-free decode/render/append loop, but its initial smoke was an
exact rank-preserving lexical relabel of #969 and terminated
`REVISE_I1_GENERATOR_IN_PLACE`: it did not supply incompatible natural choices
or qualify grammar. #953 remains active and #973 remains blocked, followed by
#954–#955 → #962–#965.
Terminology lives in
`docs/transformerless/GLOSSARY.md`. Keep this file current when conventions
change.

## What this repo is

A local, CPU-first **geometric intelligence programme**. Geometry is the route
and the route is the data location. The active lane uses a pinned lexical codec,
registered prime atoms, semiprime transitions including `p^2` self-loops,
ordered n-lets, fixed-zeta R4/S3 state, Hopf observation, torsion, exact
`Z[phi]` radial shells, and the required structural/storage project bridge
`E8 = H4 x H4`, realized in code and serialization by the golden/Galois-coupled
icosian pair `H4 ⊕ phi H4`. The qualified local attention mechanism is
narrower: natural schema-2 adjacency supplies candidates, an ordered
unit-quaternion path on S3 supplies causal prefix memory, and exact path closure
plus lease age selects or abstains. #953 wraps that selector in one bounded
canonical decode/render/append loop. H4 is an exact finite S3 codebook here;
`H4 ⊕ phi H4` / E8 remains structural storage and control rather than the
attention score.

The intended destination is frontier-like useful local intelligence without
transformers, MoE/sparse learned routing, or dense matrix intelligence. That is
an aspirational research target, not a current capability claim. Spherical
harmonics are the project-level model for overlapping spin-state storage and
transport; R4/S3 and Hopf/S2 are the bounded compute/observation charts used to
operate on that field.

Source weights are offline teachers/comparators only. The final serving path
loads no source weights and contains no transformer/self-attention, dense
matrix intelligence kernel, MoE, or sparse learned router. The learned
four-coordinate mixer remains only the negative G0/G1 comparator recorded by
#950/#951; #958 is retained positive foundation evidence at
`RETAIN_STORAGE_RECALL_ONLY` scope.

The multiplication-free TLA/R4G1 compiler, packed graph runtime, certifier,
proof assets, and dashboard remain in the repository as working historical
components and research comparators. They are not the active intelligence
sequencing path.

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
- root package `uor-r4-wasm-router` — façade + `r4` CLI + local server/chat
- `docs/` — plan, RFC (`transformerless/R4G1.md`), baseline, threat model, explainers,
  and the per-issue measurement records (`docs/<topic>_<issue>.md`)

Documentation entry points, in the order a newcomer should read them:
`README.md` (what it is, quickstart, CLI/HTTP/config reference) →
`docs/geometric_intelligence_programme.md` (current architecture, sequencing,
and claim boundaries) → `CONTRIBUTING.md` (the short form of this file) → this
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

## Decision checks (dormant by default)

```bash
cargo fmt --check
cargo check -p <touched-package> --all-targets --offline
cargo test -p <touched-package> --lib --offline
python3 scripts/check_claim_wording.py      # when claims/docs change
```

These commands are references, not automatic pre-commit work. Testing and QA
remain dormant until a product or release issue names the exact check, decision,
fixture identity, outcome actions, and resource budget. Do not run a focused
test merely because code changed. Do not run broad suites to create confidence
without a decision they can change.

Source-free attention probes, anti-recall controls, bounded product
transcripts, and serving censuses are activated by their programme stage.
Workspace, BDD, doctest, no_std, deterministic-rebuild, kappa, Gate C,
all-features, WASM, fuzz, Kani, conformance, audit, and corpus-scale suites stay
dormant unless the active product/release decision explicitly requires them.
Automatic QA is disabled. Pull-request and merge-group events emit only five
instantaneous ruleset-transport acknowledgements with no checkout or
verification work. They exist because immutable ruleset `19597522` requires
the historical names and queue; they are explicitly **not PASS evidence**.

The toolchain is pinned in `rust-toolchain.toml`: rustup-managed `cargo`
resolves the pin automatically, so an activated local check and the manually
dispatched workflow use the same toolchain. Caveat: a non-rustup Rust earlier
in `PATH` (e.g. Homebrew)
ignores the pin — verify `which cargo` resolves to `~/.cargo/bin/cargo`,
or run gates as `rustup run stable cargo …`. Bump the pin in a dedicated
PR (a bump can shift libm-sensitive teacher logprobs — see Gate E below).

## Execution-lane invariants (do not conflate)

- **Active geometric intelligence path:** compiler-side floating point and
  allocation are allowed while constructing witnessed charts, but source
  weights, source residual/MLP/LM-head execution, `uor-matmul` intelligence
  projections, transformers, dense matrix intelligence, MoE, and sparse learned
  routers are not serving dependencies. The pinned lexical codec may load
  vocabulary/normalization data without weights. Route manifests, hierarchy
  state, chart selection, and decode settings remain deterministic, and library
  boundaries retain typed errors.
- **Frozen TLA/R4G1 runtime:** XOR/AND/OR/shift/rotate/popcount/int
  add-sub/compare/table reads only. No multiply, divide, or float in its
  normative kernel; its steady-state prediction path remains allocation-free.
  Do not weaken those scoped guarantees while changing decoder code.
- **Transformerless is not multiplication-free.** A decoder may be called
  transformerless only when it invokes no source-attention operator, contains
  no dense full-prefix Q·K matrix/softmax kernel, and uses bounded geometric
  support shown load-bearing by disabled/permuted interventions.
  P-4/table lowering is a later, separately triggered decision.
- **Artifact determinism:** identical pinned compiler inputs still produce
  identical historical artifact bytes. New route manifests and transitional
  decoder checkpoints must bind their source, tokenizer, compiler or training
  configuration, and semantic parameters.
- **Errors**: library boundaries return `Result` with focused error enums;
  no `unwrap`/`expect`/panic on recoverable paths. No unsafe in the portable
  runtime or the format crate (`#![forbid(unsafe_code)]` there).
- **Claim language**: `docs/formal_vocabulary.md` (v0.1.0+) is normative —
  equations are labeled Definition/Objective/Guarantee/Assumption/Empirical
  Criterion, guarantees carry a proof-matrix status, and
  `python3 scripts/check_claim_wording.py` (available but dormant unless a
  product/release decision activates it) blocks
  "machine-verified"/exact-equivalence wording without a linked proof artifact.

## Active product and research rules

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
  library/CLI decode/render/append loop, but its rank-preserving relabel smoke
  terminated `REVISE_I1_GENERATOR_IN_PLACE`. #953 still owns a genuinely
  predeclared natural grammar contrast; #973 remains blocked and later owns
  paragraph/conversation/global qualification. #954 and #955 own correctness
  and reasoning.
  #962 owns durable multi-turn CLI/HTTP chat, persistence, isolation, and
  hive-memory; #963–#965 then own optimization, formal closure, and release.
- Sequence strictly: lexical/address plumbing → local route attention →
  source-free grammatical inference/generation → higher-scope attention →
  correctness/abstention → reasoning → optimization/purity/release.
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
  as load-bearing. #953's current code establishes reusable decoded-loop
  plumbing only; its relabelled smoke does not establish a natural grammar or
  sentence loop. Only a later accepted #953 result can expose #973, and only
  #973 may qualify paragraph, conversation, or global state.
- An exact kappa miss must not collapse unseen global history to a suffix-only
  default, but global ordered-state behavior is tested on an independently
  frozen global-snapshot permutation rather than by mutating session history.
- The completed #969 evidence compares exactly full retained path, last-only,
  and state-disabled. #953's revision smoke carried forward full path and
  state-disabled under identical support/work, but its exact rank-preserving
  lexical relabel cannot qualify incompatible natural choices. Do not add a
  construction/validation split, channel census, weight sweep, control matrix,
  or higher-scope fixture to either bounded decision.
- Teacher output may label or compare only after a source-free report freezes.
  It is never substituted for the product response.
- Once #973 activates higher scopes, every hierarchy selection emits a
  global-context coverage witness. Exact recall, grammatical generation,
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

**A long run must be observable before it starts.** Anything expected to exceed
15 minutes needs a finite work denominator, completed/remaining units,
throughput, an ETA derived from that denominator, durable checkpoints, and a
typed terminal report. A missing denominator, absent ETA, non-resumable
checkpoint, or worker setting that has not passed a one-worker/four-worker
semantic-equivalence, useful-worker utilization, and measured wall-time
improvement canary prevents launch. #958's final schema-2 complete-manifest
canary passed on 2026-08-26; its exact binding may be reused only while semantic
inputs and workload shape remain unchanged. A change must re-establish the
binding before the dependent product decision. Performance
evidence comes from release builds; a debug run cannot authorize larger work.
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
