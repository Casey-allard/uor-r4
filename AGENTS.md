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
- `crates/uor-r4-proof-model` — executable proof obligations + proof-status matrix
- `crates/uor-r4-api` — typed compile + engine library façade for downstream consumers (wraps the CLI-shaped stages; see its README)
- root package `uor-r4-wasm-router` — façade + `r4` CLI + local server/chat
- `docs/` — plan, RFC (`transformerless/R4G1.md`), baseline, threat model, explainers

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
- Run: `cargo test -p uor-r4-core --release --offline --test kappa_reproduction
  -- --ignored` (skips vacuously if the checkpoint is absent — check
  /tmp/ref/out/model.bin exists before trusting a green result).
- The baseline is **macOS-pinned** (libm-sensitive teacher logprobs); the
  container and bundle-derived pins are not expected to reproduce on Linux
  until the D2 canonical deterministic compile mode lands.
- Re-pinning is a **maintainer decision**, done via
  `dump_baseline_kappa` (`--nocapture`) → review diff → adopt →
  `TLESS_REPIN_WRITE=1` regenerates the fixture container. Compiler redesigns
  legitimately change κs; drift from nondeterminism never does — investigate
  first (double-compile determinism check), then re-pin.

## Process conventions

- **Merge workflow (since 2026-07-22): NO direct pushes to `main`.** A ruleset
  ("main: required checks", id 19597522) protects `main`: all changes land via
  PR, and the five CI checks (`fmt / clippy / tests / no_std / κ`,
  `cargo audit`, `fuzz smoke`, `wasm-pack build`, `Gate C trend alarm`) must
  pass with the branch up to date (strict policy). GitHub's merge-queue rule
  type was unavailable via the API when this was set up — if the Settings UI
  toggle gets enabled later, PRs go through the merge queue instead of plain
  merge; the workflow below is identical either way.
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
