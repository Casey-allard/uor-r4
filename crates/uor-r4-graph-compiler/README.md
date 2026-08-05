# uor-r4-graph-compiler

The offline R⁴ Holographic Graph Compiler pipeline.

## Overview

This crate implements the offline compilation stages that cross-compile pinned Hugging Face teacher models (`SmolLM2-135M-Instruct`) into multiplication-free, content-addressed R4G1 semantic graph artifacts.

## Key Components

- **Observation Pipeline** (`observation.rs`, `observation_shards.rs`): Content-addressed observation extraction with deterministic shard spill/resume (ChatML prompt formatting lives in `uor-r4-graph-cli::scenarios`).
- **Cover Induction** (`induction.rs`): Spherical K-means clustering over teacher representation spaces with calibrated overlapping region radii.
- **Score & Residual Accumulation** (`residual.rs`, `pack.rs`): Pre-quantized fixed-point `ScoreQ` residual computation across resolution depths and R4G1 packing.

The Gate C scoring harness and the `instruction-eval.json` evaluation report envelope are emitted by `uor-r4-graph-certify::score` and the root `evaluate-report` command, not by this crate.
