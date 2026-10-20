# UOR-R4 Geometric Language Model — historical graph compiler

This component preserves its declared graph/formal contracts. The accepted
native model is [uor-r4-core/src/native_geometric](../uor-r4-core/src/native_geometric),
shared by the CLI and native API. Its bounded attention and inference evidence
does not establish general prose completion or general reasoning; graph
compilation and proof status do not qualify those capabilities either. Use the
[project map](../../docs/PROJECT_MAP.md),
[canonical plan](../../docs/integration/project-track.md),
[current artifact and measured behavior](../../docs/integration/current-state.md),
and [model direction](../../archive/integration/model-direction-2026-09.md).

**Build surface (legacy-crate repair, 19 October 2026):** the library now builds from
[src/native_support.rs](src/native_support.rs). It compiles the planning, semantic-state,
stage, budget, packing and probe modules only. The teacher-backed `compile`/`observe`
pipeline in `src/lib.rs` and the observation, induction, route-fit and corpus modules
are preserved and not built. See the
[repair record](../../docs/labs/legacy-crate-repair-2026-10-19/README.md).

**Preserved research lane.** This is the earlier offline R4G1 graph-compiler
pipeline, retained for its observations, artifacts, measurements, and reusable
mechanisms. It is not the compiler for the active route-native geometric
intelligence architecture and does not by itself provide coherent chat. The
current direction and sequencing live in the
[Geometric Intelligence Programme](../../docs/geometric_intelligence_programme.md).

The offline R⁴ Holographic Graph Compiler pipeline.

## Overview

This crate implements the offline compilation stages that cross-compile pinned Hugging Face teacher models (`SmolLM2-135M-Instruct`) into multiplication-free, content-addressed R4G1 semantic graph artifacts.

## Key Components

- **Observation Pipeline** (`observation.rs`, `observation_shards.rs`): Content-addressed observation extraction with deterministic shard spill and transactional raw resume (ChatML prompt formatting lives in `uor-r4-graph-cli::scenarios`). At every whole-story boundary the versioned `raw-committed.bin` checkpoint atomically binds the global teacher state, manifest identity/layout, and the committed base-record length of every shard. Resume first validates every shard and companion without mutation, then trims tentative aligned tails and repairs the 25-byte `state.bin` compatibility mirror. Unfinished legacy raw directories without this authoritative boundary fail closed; fully finalized, κ-validated legacy bundles remain readable without rewriting historical bytes.
- **Cover Induction** (`induction.rs`): Spherical K-means clustering over teacher representation spaces with calibrated overlapping region radii.
- **Score & Residual Accumulation** (`residual.rs`, `pack.rs`): Pre-quantized fixed-point `ScoreQ` residual computation across resolution depths and R4G1 packing.

The Gate C scoring harness and the `instruction-eval.json` evaluation report envelope are emitted by `uor-r4-graph-certify::score` and the root `evaluate-report` command, not by this crate.
