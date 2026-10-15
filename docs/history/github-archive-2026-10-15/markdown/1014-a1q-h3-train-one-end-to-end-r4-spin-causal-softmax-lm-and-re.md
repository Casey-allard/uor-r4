# #1014 — A1Q-H3: train one end-to-end R4/Spin causal-softmax LM and require autonomous text

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-30T17:20:35Z
- **Updated:** 2026-08-30T20:46:50Z
- **Closed:** 2026-08-30T20:46:50Z
- **Milestone:** Native Rust geometric intelligence — conversation, memory, reasoning and coding
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/1014
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Context

#1012 completed at `INSUFFICIENT_SUPPORT_COVERAGE`: aggregate primary coverage was 62.026%, but one fold covered only 34.691%. More importantly for project direction, the full current-step Q/K/V probe was 0.000346 nats worse than the suffix baseline with 0/4 favorable folds and unchanged top-1. It separated from the rotated-label control, but supplied no useful predictive gain over suffix.

Close the trace-distillation/localization path here. Do not repair its support, add another probe, or create another comparative boundary ladder.

Parent: #973. Predecessor: #1012. Programme root: #820.

## Objective

Train one complete causal language model from random initialization on raw next-token loss, export it as a local checkpoint, and autonomously generate coherent decoded text through ordinary dot-product/stable-softmax attention executed in coherent R4/Spin coordinates.

This establishes the practical learned baseline required before recurrence, resonance, softmax replacement, or exact lowering resumes.

## Pinned precedent

Adapt and credit the MIT-licensed `llama2.c` TinyStories learning loop at commit `350e04fe35433e6d2941dce5a1f53308f87058eb`:

https://github.com/karpathy/llama2.c/tree/350e04fe35433e6d2941dce5a1f53308f87058eb

Copy the known-working training design, not its runtime format or claims.

## Frozen architecture

- tokenizer: train-only byte-level BPE, vocabulary 4,096;
- hidden width: 288;
- layers: 6;
- query heads: 6;
- KV heads: 6;
- head width: 48 = twelve R4 blocks;
- FFN width: 768;
- context: 256 tokens;
- tied embedding/LM head;
- RMSNorm, RoPE, SwiGLU;
- learned Q/K/V/O at every layer;
- ordinary complete-prefix scaled dot product;
- ordinary stable causal softmax;
- ordinary weighted-value aggregation;
- approximately 7.16M learned parameters.

Train all parameters jointly. No teacher logits, trace artifacts, suffix tables, frozen SmolLM weights, Ollama, or hosted inference may participate.

Canonical-gauge training is permitted because qualification and generation, not redundant gauge materialization during every gradient step, are the goal. Qualification and autonomous generation must execute every learned attention layer through the existing R4/Spin transport seam.

## Fresh untouched population

Freeze one content-addressed TinyStories snapshot. Partition stories before tokenization by `BLAKE3(story_bytes) mod 100`:

- buckets 0–89: train;
- buckets 90–94: development;
- buckets 95–99: sealed test.

Train the tokenizer on training stories only. Train on at most 30M deterministically ordered training tokens. Use at most 250k development tokens for checkpoint selection. The trainer must have no filesystem access to the sealed test partition.

After the final checkpoint CID is frozen, reveal at most 250k test tokens. Select five autonomous prompts as the first 24 tokens of the five lowest-CID test stories.

## Implementation

1. Add an isolated offline trainer adapted from the pinned `llama2.c` design, with pinned local dependencies and Apple-Metal support; export standard Llama-family `config.json`, `model.safetensors`, `tokenizer.json`, and a CID-bound training manifest.
2. Generalize the existing reference-generation core without changing its frozen #973 behavior.
3. Add `r4-softmax-local-generate --model <directory> --prompt <text> --max-new-tokens 128`.
4. Load the exported checkpoint with `HuggingFaceLlamaOracle`.
5. Execute all six learned layers through `R4SpinCausalAttentionTransport` with ordinary causal softmax.
6. Record checkpoint, tokenizer, dataset, split, training, R4-policy, decoded-output, and audit CIDs.

## Only allowed intervention

Evaluate the same frozen checkpoint once with every attention output zeroed before residual addition.

No plain/geometric comparison, transport permutation, label permutation, fold ladder, recurrent cell, resonance mechanism, route family, or alternative architecture is allowed.

## Necessary gates

Before the main run, one five-minute 64-sequence overfit smoke must reduce loss by at least 80%. This validates the trainer and export path; it is not the research result.

After export, compare one 32-token prefix between the training implementation and the Rust loader; require the same top-1 token and the existing bounded logit tolerance.

Then run exactly one frozen training campaign. Resuming the same interrupted checkpoint is allowed; hyperparameter sweeps are not.

## Definition of done

All must pass:

- sealed-test NLL no greater than 1.50 nats/token;
- attention-off sealed-test NLL at least 0.10 nats/token worse;
- every learned decoder layer executes through coherent R4/Spin transport;
- exact causal work audit with zero future reads;
- zero teacher, provider, Ollama, or previous-trace calls;
- five autonomous 128-token test-prompt continuations are archived;
- at least four of five retain the prompt subject or scene under the frozen local rubric;
- all outputs are valid UTF-8 and none enters a period-1 through period-4 loop;
- checkpoint reload reproduces the selected seeded generation.

## Hard stop

Maximum main-run budget: 30M tokens and six wall-clock hours on the M1 Metal backend. If the projected run exceeds that budget, do not launch it on a slower CPU path.

If the frozen checkpoint fails the Definition of Done, record its curves and exact decoded outputs and close this issue negative. Do not open another diagnostic ladder or modify the architecture inside this issue.

If it passes, the next issue is model scaling or instruction tuning. Exact integer/table lowering remains deferred until coherent generation exists.

## Runtime boundary and nonclaims

Training and qualification may use floating point, multiplication, allocation, autograd, Metal, and ordinary softmax. The local Rust generator remains an experimental floating-point runtime.

This issue does not claim transformerlessness, geometric advantage, multiplication-free execution, reasoning, browser-WASM readiness, or release readiness. Exact/table lowering is the subsequent deployment programme, not an entry condition for learning.

Depends on #1012
References #973

## Comments (4)

### comment 5470299855 — 2026-08-30T17:50:56Z — Casey-allard

Activated from live `main` after #1012 / PR #1015 merged at `46dad4e4844e7a1716e91cdf8201d6389c7bceed`.

Clean implementation branch/worktree:
- branch: `issue-1014-end-to-end-r4-softmax-lm`
- base: exact live `origin/main` at `46dad4e4`

Immediate action is implementation intake for the single frozen end-to-end model: validate the Apple-Metal training environment, pin the isolated trainer dependencies and TinyStories acquisition/split path, then implement the five-minute 64-sequence overfit smoke. This is trainer/export-path validation, not a comparative research ladder. No main training run launches until its duration projection fits the six-hour M1 Metal budget.


### comment 5470615521 — 2026-08-30T18:54:40Z — Casey-allard

### Admission update — 2026-08-30

This track has not opened another comparative-test ladder. The two predeclared implementation gates are now complete and positive:

- Pinned TinyStories source verified: revision `f54c09fd23315a6f9c86f9dc80f725de7d8f9c64`, SHA-256 `6418d412de72888f52b5142c761ac21a582f7d1166f0bfbdb5f03ccfdec90443`, 2,227,753,162 bytes.
- One deterministic preparation pass produced 30,000,000 train tokens, 250,000 dev tokens, and the exact 250,000-token sealed reveal budget. Dataset manifest CID: `blake3:3e4d2ddb006771e5be0d4c76580c8971e6c67a23f8e223da8d81668d03bd9a01`.
- The single 64-sequence MPS overfit smoke **PASSed**: loss `8.412553 -> 1.613438`, reduction `80.8211%` against the fixed `>=80%` rule, in `132.36s`. Smoke manifest CID: `blake3:264704b2b36f0f443ac58100a169d5b884c9e8be518fd6dec1fdcf9838149fc1`.
- The real one-prefix, two-arm Python/Rust R4 qualification **PASSed**. Enabled maximum absolute logit error: `0.0000171661`; attention-off error: `0.0000123978`; both are within `0.005` and preserve top-1.

An independent review found two lifecycle holes before launch: a completed frozen selection could be rewritten via `train --resume`, and main training did not code-enforce the smoke plus Rust parity admission. Main training remains **NO-GO** until those are sealed and reviewed. This is implementation correction, not a new research experiment.

Explicitly not being run: folds, probes, permutations, alternative architectures, support repair, hyperparameter sweeps, full-corpus Rust replay, or broad workspace tests. After the lifecycle seal, the next action is the one frozen <=30M-token MPS campaign; pass or fail then causes the predeclared reveal/generation outcome.

### comment 5471081181 — 2026-08-30T20:29:57Z — Casey-allard

### Terminal measurement — frozen campaign, 2026-08-30

The one admitted campaign completed exactly once and is terminal for this issue.

#### Frozen training

- 7,155,360 parameters; 1,831 optimizer steps; 29,999,104 train tokens.
- MPS training time: 4,220.42 s (70.34 min), inside the six-hour ceiling.
- selected step: 1,831; development NLL: `2.1313565266`.
- checkpoint CID: `blake3:9c36e109d8dee67deec0362307ba4a471c967ff574835210f87653d628c95c91`.
- weights CID: `blake3:7d7c26e1a71866dc46973cea3b23b819f4b5060b345d2a0ec1bd067aa493bb7d`.

#### Sealed attention result

- enabled test NLL: `2.1274072772`.
- attention-off test NLL: `4.8047998381`.
- attention-off minus enabled: **`+2.6773925609` nats/token** against the predeclared `>=0.10` requirement.
- enabled quality ceiling: **FAIL**, `2.1274072772 > 1.50`.

This establishes that ordinary causal dot-product/stable-softmax attention executed in coherent R4/Spin coordinates is functioning and causally necessary for this learned model. It does **not** establish an H4/geometric advantage over ordinary coordinates or transformerless execution.

Final Python/Rust two-arm qualification PASSed on all 4,096 logits:
- enabled max absolute error: `0.0000076294`;
- attention-off max absolute error: `0.0000123978`;
- both top-1 choices identical; all six layers selected; zero future reads; zero provider/Ollama/prior-trace calls.

#### Autonomous Rust generation

Five sealed prompts were generated locally with seeds 1014–1018 and up to 128 new tokens. All outputs are UTF-8, all avoid period-1 through period-4 loops, and all five replays reproduce the same token sequence, text, output CID, decision CID, and policy CID.

Frozen subject/scene rubric: **3/5**, below the required `>=4/5`:
- prompt 0 cricket: FAIL (subject abandoned);
- prompt 1 Tim/selfish scene: PASS;
- prompt 2 ice-cream scene: PASS;
- prompt 3 Grandma/Grandpa garden: FAIL (subject abandoned);
- prompt 4 Tim/cartoon setup: PASS by retained Tim/child scene, though cartoon detail is lost.

#### Verdict

`ATTENTION_ESTABLISHED__FULL_QUALITY_DOD_FAIL`

#1014 is negative as a complete quality campaign because both enabled NLL and 4/5 subject retention were missed. Per the frozen hard stop: no rerun, tuning, folds, support repair, alternative architecture, or diagnostic ladder will be added inside this issue. Draft implementation/evidence PR: #1016. The next issue must build forward from the now-established attention mechanism and target language quality directly.

### comment 5471163132 — 2026-08-30T20:46:50Z — Casey-allard

Delivered and preserved through PR #1016, merged by the protected queue at `4b21f9af37bbe4918413cf52b3f9fc905dad58dd` after the full merge-group CI passed. Terminal scientific verdict remains `ATTENTION_ESTABLISHED__FULL_QUALITY_DOD_FAIL`; the campaign is closed without rerun or tuning. The sole successor is #1017.


---
_Comment/review ids in this file: 5470299855, 5470615521, 5471081181, 5471163132_

