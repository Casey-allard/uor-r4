# AGENTS.md — uor-r4

Guidance for agents (human or otherwise) working in this repository. Project
direction lives in `docs/r4_graph_compiler_implementation_plan.md`; terminology
in `docs/transformerless/GLOSSARY.md`. Keep this file current when conventions
change.

## What this repo is

A local, CPU-first AI system: (1) a **transformerless engine** that
cross-compiles a pinned Hugging Face teacher into a multiplication-free
table-native artifact with a witnessed integer runtime, and (2) the **R⁴
holographic graph compiler** program that generalizes it into a
multiresolution, overlapping semantic graph with an allocation-free runtime
(the plan linked above). The f64 geometric router (`crates/uor-r4-router`) and
the wasm dashboard are exploratory and stay out of the graph migration path.

## Workspace layout

- `crates/uor-r4-core` — R⁴ math + transformerless compiler/runtime (see its README)
- `crates/uor-r4-router` — geometric router + dashboard backend (f64; untouched by the graph plan)
- `crates/uor-r4-graph-format` — R4G1 packed artifact format, two-stage validation, borrowed `GraphView`
- `crates/uor-r4-graph-compiler` — offline graph-compiler stages (observation, cover induction, packing)
- `crates/uor-r4-graph-certify` — offline certification/measurement (Gate C `score` harness, `score_runtime` reference scorer, certificates)
- `crates/uor-r4-graph-runtime` — `no_std` allocation-free R4G1 graph runtime (engine, routing, patch chains)
- `crates/uor-r4-graph-cli` — `r4 transformerless …` CLI stage dispatch (convert-r4g1, scenarios, corpus tools)
- `crates/uor-r4-model-source` — teacher forward-pass port + pinned Safetensors adapter
- `crates/uor-r4-proof-model` — executable proof obligations + proof-status matrix
- `crates/uor-r4-api` — typed compile + engine library façade for downstream consumers (wraps the CLI-shaped stages; see its README)
- root package `uor-r4-wasm-router` — façade + `r4` CLI + local server/chat
- `docs/` — plan, RFC (`transformerless/R4G1.md`), baseline, threat model, explainers,
  and the per-issue measurement records (`docs/<topic>_<issue>.md`)

Documentation entry points, in the order a newcomer should read them:
`README.md` (what it is, quickstart, CLI/HTTP/config reference) →
`CONTRIBUTING.md` (the short form of this file) → this file (the full operating
manual) → `docs/RESEARCH.md` (what is measured, closed and open) →
`docs/MODEL_LIFECYCLE.md` (the multi-hour compile chain) →
`docs/CONFIGURATION.md` (every environment knob).

**Keep them true.** When a measurement revises a claim, correct it where it is
asserted — README, `docs/RESEARCH.md`, and the record itself — rather than
letting a superseded number survive because it lives in three places. Records in
`docs/` are appended to, not rewritten: the history of what was believed and when
is part of the evidence.

UOR standards (`uor-addr`, `UOR-Framework`) are **pinned git dependencies** in
`Cargo.toml` — a fresh clone builds with no extra checkouts. The
`uor_standards/` directory is legacy local material (gitignored; not required
to build).

## Commands (daily drivers)

```bash
cargo test --workspace --offline           # all suites
cargo clippy --workspace --all-targets --all-features --offline -- -D warnings
cargo fmt --check
cargo check -p uor-r4-graph-format --no-default-features            # no_std ladder
cargo check -p uor-r4-graph-format --no-default-features --features alloc
```

All four must be clean before every commit. CI (`.github/workflows/ci.yml`)
runs the same plus `cargo nextest`, doc tests, deterministic-rebuild, cargo
audit, and nightly fuzz smoke — keep it green.

The toolchain is pinned in `rust-toolchain.toml`: rustup-managed `cargo`
resolves the pin automatically, so the gates above run the same toolchain
CI does. Caveat: a non-rustup Rust earlier in `PATH` (e.g. Homebrew)
ignores the pin — verify `which cargo` resolves to `~/.cargo/bin/cargo`,
or run gates as `rustup run stable cargo …`. Bump the pin in a dedicated
PR (a bump can shift libm-sensitive teacher logprobs — see Gate E below).

## Normative invariants (do not weaken)

- **Runtime kernel**: XOR/AND/OR/shift/rotate/popcount/int add-sub/compare/
  table reads only. No multiply, divide, or float in the deployed kernel —
  enforced by a machine-checked source scan (`transformerless/mod.rs` P-4).
  Compiler/certifier code may use floats and allocation; runtime code may not.
- **Allocation**: the prediction hot path is allocation-free in steady state
  (asserted by `crates/uor-r4-core/tests/allocation_census.rs`).
- **Determinism**: identical pinned inputs ⇒ identical artifact bytes. No
  HashMap-iteration-order, clock, or RNG dependence in compiler outputs;
  parallelism partitions by content-addressed sample ID with ordered
  reductions (plan §4.1).
- **Errors**: library boundaries return `Result` with focused error enums;
  no `unwrap`/`expect`/panic on recoverable paths. No unsafe in the portable
  runtime or the format crate (`#![forbid(unsafe_code)]` there).
- **Claim language**: `docs/formal_vocabulary.md` (v0.1.0+) is normative —
  equations are labeled Definition/Objective/Guarantee/Assumption/Empirical
  Criterion, guarantees carry a proof-matrix status, and
  `python3 scripts/check_claim_wording.py` (CI-enforced) blocks
  "machine-verified"/exact-equivalence wording without a linked proof artifact.

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

## Teacher parity BDD suite

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
smollm2-135m-instruct` and the compiled bundle are present, and vacuously
skips otherwise (κ-test convention — check the fixture before trusting green).
Budgets: `R4_PARITY_POSITIONS` (256), `R4_PARITY_GEN_TOKENS` (128),
`R4_PARITY_RUNS` (3), `R4_PARITY_CORPUS_POSITIONS` (1000). Thresholds are
pinned empirical floors with ~20%
margin; the ~1% top-1 figures are out-of-distribution honesty, not a bug —
the suite's 8 prompts are novel text, unlike Gate C's same-corpus replay
(see the comment above the constants in `tests/bdd.rs`).

## Process conventions

- **Merge workflow (since 2026-07-22): NO direct pushes to `main`.** A ruleset
  ("main: required checks + merge queue", id 19597522) protects `main`: all
  changes land via PR, and the five CI checks (`fmt / clippy / tests / no_std / κ`,
  `cargo audit`, `fuzz smoke`, `wasm-pack build`, `Gate C trend alarm`) must
  pass with the branch up to date (strict policy). **The merge queue is
  ENABLED (since 2026-07-31)**: PRs merge through the queue, and a queued
  PR's head branch is LOCKED — pushes are rejected ("branches that are
  queued for merging cannot be updated") until the PR merges or is
  dequeued. Follow-up work for a queued PR goes on a fresh branch off
  `main` after it lands (the #323 lesson), not as extra commits on the
  queued branch.
- **CI split: expensive verification runs ONCE, in the queue (2026-08-07).**
  `.github/workflows/ci.yml` reports the same five required check names in
  both contexts, but the work differs. On `pull_request`: claim wording, fmt,
  clippy (job `gates-pr`) + `cargo audit`; the other three required names are
  reported by trivial stub jobs. On `merge_group` (and pushes to `main`):
  the full ladder — tests, no_std, deterministic rebuild, κ-reproduction,
  Gate C trend, wasm, fuzz — on the speculative merge, which is the verdict
  that binds. **Do not add a slow step to the PR-side job.** If a check takes
  minutes, it belongs in `gates` (queue-side); the PR trigger exists for fast
  author feedback, not for the binding verdict. Any new required check name
  must be reported in BOTH contexts (real job in one, same-`name:` stub in
  the other) or PRs hang forever waiting on a check that never runs.
- **Docs-only PRs take a fast path.** If a PR's whole diff is `*.md` or
  `docs/**/*.pdf` — excluding `docs/hologram_r4_formal_monograph.md` and
  `docs/transformerless/INFERENCE_OPERATION_CONTRACT.md`, which are
  `include_str!`d into Rust — `gates-pr` runs only the claim-wording gate.
  The guard fails closed (any other path, an empty diff, or an
  uncomputable diff runs everything) and applies only to `pull_request`;
  the queue always re-verifies the merged content in full.
- **Per issue**: assign yourself (WIP signal) → branch `issue-<n>-<slug>` →
  work + verify the four gates locally → open PR → merge when checks are
  green → close the issue with the DoD evidence and the merge commit
  reference. Milestones mirror plan phases.
- **PR review** (incl. Copilot-generated): never merge unverified. Run the
  four gates + κ-reproduction on a merge preview first; resolve conflicts
  hunk-by-hunk — whole-file `checkout --theirs/--ours` has silently dropped
  upstream features before (the TLA5 incident).
- **Committing while subagents work in-tree**: add files **by name**, never
  `git add -A` — in-flight agent work (unregistered modules, half-written
  tests) must not be swept into unrelated commits (the cover.rs incident).
- **Tests that encode era sensitivity**: `src/tless_uor.rs`
  `indexing_and_generation_update_store` asserts resolution depths that depend
  on the fixture artifact's class signatures — update the expected depths with
  an era note whenever the fixture is regenerated.
- **ScoreQ**: there are intentionally two compatible definitions in flight
  (`uor-r4-graph-format::ScoreQ` wire newtype; `uor-r4-core::score_q::ScoreQ`
  with compiler-side f32 conversions). Consolidation onto the format crate is
  a scheduled pre-Phase-5 cleanup — don't add a third.

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
binding. `cargo test -p uor-r4-graph-certify --test capacity_scaling --
--ignored` takes about twelve minutes and prints a SATURATION verdict per
structure. If it reports SATURATED on the structure the experiment intends to
move, the long run does not launch. On 2026-08-06 that instrument reported
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

**Two gates the local checks do not cover.** `cargo clippy --workspace
--all-targets` does NOT build other targets: the merge queue builds wasm, so
any change under `uor-r4-core` needs `cargo check --target
wasm32-unknown-unknown -p uor-r4-wasm-router --lib` before shipping. A
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
with one commit per issue (message refs `#N`), and the four local gates +
merge queue run ONCE per batch of 3-6 issues — not per issue. Authoring
feedback during a batch is `cargo check` on a warm shared target
(`CARGO_TARGET_DIR`); the full workspace suite still gates every merge in
CI, so rigor is unchanged — it just stops running serially per issue.
Runtime-kernel and serving-semantics changes still get individual PRs.
Measurement runs are background science with scheduled harvests; they
never sit between two pieces of code work.

## Things that bite

- `/tmp/ref/out/model.bin` disappears on reboot/periodic /tmp cleanup — κ tests
  skip silently and report vacuous green.
- `crates/uor-r4-graph-format/fuzz/target` must never be committed (gitignored).
- Fuzz targets need nightly (`cargo +nightly fuzz run …`); the stable
  deterministic mutation smoke runs under plain `cargo test`.
- The on-disk compiled store in `.uor-models/` predates the u32 token
  migration (TLS1-u16); `runtime::parse_store_legacy_u16` reads it, and a full
  recompile is needed to refresh it.
