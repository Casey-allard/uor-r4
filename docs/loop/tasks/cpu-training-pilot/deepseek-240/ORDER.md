# ORDER: cpu-training-pilot, deepseek-240

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506), supporting
[one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Base: `d43e339502eabf5351c49677929e14c6a8fc5b51`.

## Goal

Make one repeatable CPU checkpoint-grading command for Claude's current geometric
training line. It must accept either a saved checkpoint or an already exported
`UORLUT01` artifact and report:

- checkpoint-to-artifact identity and the existing CPU/serving parity audit;
- a deterministic, bounded sample of the supplied held-out response rows, with
  top-1 success for both the nearest and native-writer heads;
- native-writer top-8 and admission readings when the artifact serves that head;
- bounded complete-reply generation and repetition diagnostics;
- an optional comparison with a baseline artifact over the same sampled rows.

This is local verification and development evidence. It does not replace the GPU
qualification path or make a useful-chat claim.

## Why now

DeepSeek-239 landed the CPU-only b64L export path. Claude-304 stopped width 1,024
without a quality reading, and Claude-305 prepared D4, about 2.65 times the D3
human text, as the next b64L training input. The next accepted checkpoint should
therefore be exported and graded on the laptop while the GPU remains responsible
for qualification.

The existing CPU pieces are separate binaries: `cpu-fbs-export`, the corpus
driver, the complete-reply evaluator and the served-forward audit. This unit
joins their public APIs behind one fail-closed command rather than adding another
training mechanism.

## Claim check

- The CPU item is ready and its scope names the isolated CPU crate.
- Claude owns the trainer and next D4 GPU run; this unit changes neither.
- The claimed branch is `deepseek/checkpoint-grade`, based on merged main
  `d43e33950`.

## Cancel probe

Class: instrument.

1. The merged CPU exporter builds and exports both accepted b64L seeds without
   CUDA.
2. The CPU crate already exposes masked-corpus ingestion, exact panel evaluation,
   complete-reply generation and the independent served-forward audit.
3. The next Claude checkpoint does not exist yet, so this unit is prepared and
   qualified against existing b64L artifacts and synthetic fixtures without
   waiting on Claude.

Result: GO.

## Done when

1. One Rust binary can grade an artifact or export a checkpoint first, using only
   the exact CPU path and refusing ambiguous, existing or unsafe outputs.
2. Held-out row sampling is deterministic, bounded and verified to be
   non-vacuous; nearest and native-writer readings use the served decision for
   their head.
3. The report binds artifact, corpus, manifest, tokenizer and optional baseline
   identities; includes CPU/serving parity and complete replies; and separates
   measured values from diagnostics.
4. Tests cover malformed arguments, missing inputs, bounded sampling, head
   selection, deterministic replay and baseline row alignment.
5. The binary grades both accepted b64L artifacts on an existing held-out corpus
   shard and reproduces the result after a fresh process.
6. A REPORT records commands, hashes, timings and negative findings first. It
   makes no GPU-qualification, float-equivalence or useful-chat claim.

## Stop and ask if

- grading would require opening CUDA or changing Claude-owned trainer sources;
- an artifact head has no exact served decision API available without widening
  the CPU crate's scope;
- the held-out corpus is unavailable or the only available panel would be reused
  outside its registered split;
- evidence would exceed the existing local CPU evidence budget.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `Cargo.toml`
- `Cargo.lock`
- `docs/harness/claims.json`
- `docs/harness/prep/cpu-training-pilot*`
- `docs/labs/cpu-training-pilot-*`
- `docs/loop/tasks/cpu-training-pilot/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`
