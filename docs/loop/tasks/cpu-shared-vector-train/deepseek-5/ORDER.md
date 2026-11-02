# ORDER: deterministic masked-corpus driver, deepseek-5

Lab: deepseek. Item: [cpu-shared-vector-train](../../../../roadmap/items/cpu-shared-vector-train.md) (#3505). Base: `af1979886`.

## Goal

Add a deterministic, host-only corpus driver that turns a pinned token stream and response mask into causal rank-margin samples, trains a genuine public FBS artifact under an explicit bounded step budget, checkpoints and resumes exactly, and reports exact ranking, admission and export outcomes alongside throughput.

This unit builds and tests the driver. It does not run the bounded pilot, which needs the separate `cpu-training-pilot` pre-registration and FIT.

## Scope

- `crates/uor-r4-cpu-train/**`
- `Cargo.toml`
- `Cargo.lock`
- `docs/loop/tasks/cpu-shared-vector-train/deepseek-5/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-shared-vector-train.md`

No serving edit, corpus-scale run, GPU run or language claim.

## Done when

- The driver strictly parses little-endian `u16` tokens and byte response masks, refusing length mismatches, masks other than `0` or `1`, out-of-vocabulary tokens and an empty target set.
- Causal samples are built at every response-masked non-BOS target from only its document prefix, truncated to the artifact's context length, with BOS used as the document separator.
- Training and evaluation samples are disjoint by source document under a deterministic pinned split, and both use the same context and token representation.
- A bounded run accepts a genuine public dual-c131 artifact, records baseline and final exact CPU/serving export, target admission, top-1/top-8, margin loss, accepted/refused updates, target visits, step time, wall time and peak RSS.
- Checkpoints occur at a configured interval and two deterministic resumes reproduce identical final artifacts, readings and next-step choices.
- The public conversational panels may be used only for deterministic qualitative output; the driver does not label open-ended generations as correct replies.
- Sealed fresh-panel scoring remains credential-gated and `NOT_RUN`; no pass/fail language bar is invented.
- Full release tests, clippy, formatting, D23/D25/D32 ratchets, local PR check and all loop gates pass.
- A fresh exact-head adversarial review approves the change before merge-queue delivery.
