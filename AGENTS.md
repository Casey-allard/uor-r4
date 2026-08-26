# AGENTS.md — uor-r4

Guidance for agents (human or otherwise) working in this repository.
**Current intelligence sequencing is authoritative in the
[`docs/geometric_causal_decoder_plan.md`](docs/geometric_causal_decoder_plan.md)**
— the readable mirror of GitHub programme root #820 and execution tracker #949.
The prior S0–S7 completion plan and the graph-compiler implementation plan are
retained as historical engineering/evidence records; neither decides what is
built next. Terminology lives in `docs/transformerless/GLOSSARY.md`. Keep this
file current when conventions change.

## What this repo is

A local, CPU-first **geometric language-model programme**. The active product
lane reuses the source-model tokenizer/residual/MLP/LM-head stack and pinned
`uor-matmul` projections while progressively replacing causal self-attention
with a learned R⁴ geometric mixer. The existing f64 router supplies
identity-scoped geometric memory, retrieval, and turn persistence.

The multiplication-free TLA/R4G1 compiler, packed graph runtime, certifier,
proof assets, and dashboard remain in the repository as working historical
components and research comparators. They are not the active intelligence
sequencing path.

## Workspace layout

- `crates/uor-r4-core` — R⁴ math + transformerless compiler/runtime (see its README)
- `crates/uor-r4-router` — active geometric memory/router + historical word-Markov decoder and dashboard backend
- `crates/uor-r4-graph-format` — R4G1 packed artifact format, two-stage validation, borrowed `GraphView`
- `crates/uor-r4-graph-compiler` — offline graph-compiler stages (observation, cover induction, packing)
- `crates/uor-r4-graph-certify` — offline certification/measurement (Gate C `score` harness, `score_runtime` reference scorer, certificates)
- `crates/uor-r4-graph-runtime` — `no_std` allocation-free R4G1 graph runtime (engine, routing, patch chains)
- `crates/uor-r4-graph-cli` — `r4 transformerless …` CLI stage dispatch (convert-r4g1, scenarios, corpus tools)
- `crates/uor-r4-model-source` — source forward/KV/trace runtime and the active geometric-decoder seam
- `crates/uor-r4-proof-model` — executable proof obligations + proof-status matrix
- `crates/uor-r4-api` — typed compile + engine library façade for downstream consumers (wraps the CLI-shaped stages; see its README)
- root package `uor-r4-wasm-router` — façade + `r4` CLI + local server/chat
- `docs/` — plan, RFC (`transformerless/R4G1.md`), baseline, threat model, explainers,
  and the per-issue measurement records (`docs/<topic>_<issue>.md`)

Documentation entry points, in the order a newcomer should read them:
`README.md` (what it is, quickstart, CLI/HTTP/config reference) →
`docs/geometric_causal_decoder_plan.md` (current architecture and exact issue
order) → `CONTRIBUTING.md` (the short form of this file) → this file (the full
operating manual) → `docs/RESEARCH.md` (what is measured, closed and open) →
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

## Commands (daily drivers)

```bash
cargo fmt --check
cargo check -p <touched-package> --all-targets --offline
cargo test -p <touched-package> --lib --offline
python3 scripts/check_claim_wording.py      # when claims/docs change
```

There is no universal pre-commit test gauntlet. Run the smallest focused test
that exercises the behavior you changed, plus a compile check for the touched
package. The required CI context performs workspace compilation and library
tests for Rust/build changes; docs-only changes run claim wording only. The
exhaustive workspace, BDD, doctest, no_std, deterministic-rebuild, κ, Gate C,
all-features, WASM, fuzz, Kani, conformance, and audit suites are nightly/manual
certification. Invoke one locally only when the change directly targets that
contract or before a release decision.

The toolchain is pinned in `rust-toolchain.toml`: rustup-managed `cargo`
resolves the pin automatically, so the gates above run the same toolchain
CI does. Caveat: a non-rustup Rust earlier in `PATH` (e.g. Homebrew)
ignores the pin — verify `which cargo` resolves to `~/.cargo/bin/cargo`,
or run gates as `rustup run stable cargo …`. Bump the pin in a dedicated
PR (a bump can shift libm-sensitive teacher logprobs — see Gate E below).

## Execution-lane invariants (do not conflate)

- **Active geometric decoder:** learned floating-point projections,
  `uor-matmul`, allocation, and ordinary source-model residual/MLP/LM-head
  components are allowed. Fixed inputs/checkpoints/decode settings must remain
  deterministic, and library boundaries retain typed errors.
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
  identical historical artifact bytes. New learned decoder checkpoints must
  bind their source, tokenizer, training configuration, and parameters.
- **Errors**: library boundaries return `Result` with focused error enums;
  no `unwrap`/`expect`/panic on recoverable paths. No unsafe in the portable
  runtime or the format crate (`#![forbid(unsafe_code)]` there).
- **Claim language**: `docs/formal_vocabulary.md` (v0.1.0+) is normative —
  equations are labeled Definition/Objective/Guarantee/Assumption/Empirical
  Criterion, guarantees carry a proof-matrix status, and
  `python3 scripts/check_claim_wording.py` (CI-enforced) blocks
  "machine-verified"/exact-equivalence wording without a linked proof artifact.

## Active product and research rules

- Work only the earliest unblocked child of #949. #950 is the first engineering
  issue after the roadmap PR; downstream issues stay unassigned.
- Every active decoder engineering leaf ends in a runnable product result, not
  only a format, harness, proof, or metric. Governance and administration work
  may instead close with their explicit repository/GitHub outcome.
- Establish the coherent local `uor-matmul` control before debugging geometry.
- Bind persistent memories to the exact source tokenizer and qualify their
  deterministic layer projection before progressive replacement.
- Geometry must execute before token choice, change real support/logits, and
  beat a disabled or permuted-geometry null before it is scaled.
- Student-prefix free-running output is required whenever generation is
  claimed. Teacher-forced top-1 alone cannot promote a decoder stage.
- Before closing a stage with `REDESIGN` or another negative verdict, reblock
  its successor on the replacement design or close the downstream work as not
  triggered. A closed blocker is not itself a promotion.
- Start with the smallest batch/prompt set that can falsify the mechanism. A
  negative result stops or redesigns it; it does not automatically authorize a
  larger harness.
- Do not add a graph section, proof lane, benchmark framework, BDD suite, or
  corpus-scale run before the active vertical slice demonstrates decision
  value.
- Routine verification is one focused behavior test, touched-package compile,
  and bounded product transcript. Add certification only when its exact
  contract is changed or a release decision requires it.

## κ-reproduction (Gate E) — how to run and re-pin

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

## Historical TLA/R4G1 teacher-parity certification

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

- **Merge workflow (since 2026-07-22): NO direct pushes to `main`.** A ruleset
  ("main: required checks + merge queue", id 19597522) protects `main`: all
  changes land via PR. `fast build + product smoke` is the single substantive
  gate; five zero-work compatibility contexts mirror its result until a
  repository administrator updates the required-context list. **The merge queue is
  ENABLED (since 2026-07-31)**: PRs merge through the queue, and a queued
  PR's head branch is LOCKED — pushes are rejected ("branches that are
  queued for merging cannot be updated") until the PR merges or is
  dequeued. Follow-up work for a queued PR goes on a fresh branch off
  `main` after it lands (the #323 lesson), not as extra commits on the
  queued branch.
- **CI critical-path budget (issue #940, 2026-08-25).** Pull requests and
  speculative merges have one substantive gate: `fast build + product smoke`.
  Docs/non-build changes run claim wording only. Rust/build changes additionally
  run fmt, `cargo check --workspace --all-targets`, and
  `cargo test --workspace --lib`. Do not add certification, research, proof,
  fuzz, cross-target, or corpus-scale work to this context. Those suites run on
  the nightly schedule or by manual dispatch. Target budgets are under two
  minutes for docs and under eight minutes for ordinary warm-cache Rust changes.
- **Per issue**: assign yourself (WIP signal) → branch `issue-<n>-<slug>` →
  work + run focused checks for the changed behavior → open PR → merge when checks are
  green → close the issue with the DoD evidence and the merge commit
  reference. Milestones mirror plan phases.
- **PR review** (incl. Copilot-generated): never merge unverified. Run the
  focused checks appropriate to the changed behavior; run κ or another
  certification suite only when its contract is affected. Resolve conflicts
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
  consolidation ahead of #949.

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

**Cross-target checks are scoped certification.** A native workspace check does
not build WASM. Run `cargo check --target wasm32-unknown-unknown -p
uor-r4-wasm-router --lib` when the change touches the WASM boundary or before a
release; it is not required for every core edit. A
filesystem-touching helper gated `#[cfg(not(target_arch = "wasm32"))]` needs a
wasm counterpart, or every caller has to become cfg-aware; prefer the
counterpart. This was found the expensive way on PR #470, where PR checks were
green and the queue build failed.

**`gh pr merge` returning nothing usually means queued, not failed.** Verify
with `git ls-remote origin 'refs/heads/gh-readonly-queue/*'` or by re-reading
the PR state before concluding anything is wrong, and never reach for
`--admin` on a shared repo.

**Issue hygiene that goes with it.** Every issue filed mid-run gets an owner
and a named next action, or it gets closed with its record. Assignment means
actively-working-now; unassign when a track parks so the board reads true for
everyone. A PR that ships only part of an issue's scope says "References #N",
never "Closes #N" — GitHub will auto-close the issue on merge and the
unfinished half loses its home.

## Batch flow for small issues (process amendment, 2026-07-29)

Small, low-risk issues (docs, help text, certifier-side rows, test
harnesses, telemetry) are worked on ONE integration branch (`batch-N`)
with one commit per issue (message refs `#N`), and focused checks + the
merge queue run ONCE per batch of 3-6 issues — not per issue. Authoring
feedback during a batch is `cargo check` on a warm shared target
(`CARGO_TARGET_DIR`); exhaustive certification is nightly/manual.
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
