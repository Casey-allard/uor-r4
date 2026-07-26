# uor-r4-graph-compiler

The offline R⁴ Holographic Graph Compiler pipeline.

## Overview

This crate implements the offline compilation stages that cross-compile pinned Hugging Face teacher models (`SmolLM2-135M-Instruct`) into multiplication-free, content-addressed R4G1 semantic graph artifacts.

## Key Components

- **Observation Pipeline**: Content-addressed observation extraction with ChatML prompt formatting (`scenarios.rs` / `encode_chat_prompt`).
- **Cover Induction**: Spherical K-means clustering over teacher representation spaces with calibrated overlapping region radii.
- **Score & Residual Accumulation**: Pre-quantized fixed-point `ScoreQ` residual computation across resolution depths.
- **Evaluation Harness**: Produces `instruction-eval.json` evaluation report envelopes carrying BLAKE3 CIDs (`tokenizer_cid`, `artifacts_cid`, `store_cid`) for Gate C certification.
