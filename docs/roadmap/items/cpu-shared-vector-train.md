---
id: cpu-shared-vector-train
title: Train the CPU shared-vector path with a lawful rank margin
milestone: research
area: training
status: ready
open_to: lab
needs_gpu: false
depends_on: [cpu-shared-vector-forward]
notebook: docs/research/notebook/cpu-training.md
done_class: host
scope: [crates/uor-r4-cpu-train/**, Cargo.toml, Cargo.lock, crates/uor-r4-harness/src/ratchet.rs, scripts/no-softmax-baseline.json, scripts/no-matmul-baseline.json, scripts/no-discrete-float-baseline.json, docs/loop/tasks/cpu-shared-vector-train/**, docs/research/notebook/cpu-training.md, docs/roadmap/items/cpu-shared-vector-train.md]
---

## Why it matters

This is the CPU form of direct token-to-vector training. It matters only if the forward is already parity-checked and the trainer preserves admission rather than optimizing a disconnected score.

## Progress

The component trainer is delivered but the item is not complete:

- #3518 added exact tied-token-table rank-margin updates with full-forward acceptance, Hamming refresh, admission guard and deterministic checkpoint/resume.
- #3522 added exact BVG-K counterfactual groups for normalization gains, convolution/gate biases, decay offsets and all six FBS route tables.
- #3523 added matched panel evaluation, exact CPU-versus-serving query/token/kept-cell gaps, checkpoint/resume panel identity and a gate that refuses target-admission or export loss.
- The raw-FBS unit adds exact unsnapped Hamilton-gate/transition parity, Hamming and exact-memory composition, and both public dual-c131 trained seeds over 192 positions with finite rank-margin/panel controls.

The first three are synthetic finite component results. The public FBS check now binds genuine trained artifacts, but it remains a finite component/control result: none is a corpus-training, complete-reply, fresh-phrasing or language result.

## What is missing

Run the bounded pilot registered under `cpu-training-pilot` after its dependencies close. Bind a trained artifact and query panel, compare a float shadow with the exact integer server on the same rows, and preserve complete-reply and fresh-phrasing controls. The private-store parity prerequisite for the separate discrete path remains #3502/#3501.

## Done when

- Rank-margin learning is finite, geometric and free of squash/matmul code.
- Two deterministic resumes reproduce the same checkpoint bytes and next-step losses.
- Admission, top-1/top-8 and exact-export readings are logged alongside training loss.
- Training and evaluation use the same context length and token representation.
- A bounded pilot has a pre-registered bar, controls, seed count and stop rule before it runs.

## Cancel if

Forward parity misses, the trainer cannot preserve admission, or the only apparent gain disappears in exact integer evaluation. Record the configuration negative; do not switch to dense or softmax arithmetic.

## How to start

Start from the reports named under Progress and rerun their parity/resume tests at the pilot head. Prepare the bounded `cpu-training-pilot` registration only after the trained-artifact/query input and remaining dependency state are explicit; save it before any corpus-scale step.

## Rules that apply

- Rank margin only; no implicit vocabulary normalization.
- No dense matrix multiplication, including hand loops and wrappers.
- Loss is reported beside admission, exact export and complete-reply outcomes.
- Reuse only independently reimplemented lawful patterns; the new crate has no dependency edge to `uor-r4-training` and receives its own scanned allowlist and ratchet entry.

## Sources

- `crates/uor-r4-cpu-train/src/train.rs`, `counterfactual.rs` and `panel.rs`
- `crates/uor-r4-training/src/joint_optimizer.rs` and `joint_campaign.rs` for reusable checkpoint/cursor patterns
- `crates/uor-r4-float-train/src/in_space_long.rs` for the current serving-aligned training objective
- `docs/model/pipeline-map.md`, stages 6-8
