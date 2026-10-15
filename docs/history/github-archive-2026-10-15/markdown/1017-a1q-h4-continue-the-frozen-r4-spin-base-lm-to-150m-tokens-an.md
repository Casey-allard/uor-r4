# #1017 — A1Q-H4: continue the frozen R4/Spin base LM to 150M tokens and require coherent local generation

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-30T20:43:20Z
- **Updated:** 2026-08-31T06:30:02Z
- **Closed:** 2026-08-31T06:28:57Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/1017
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Context

#1014 established load-bearing ordinary causal attention in the learned R4/Spin path: enabled sealed NLL was `2.127407277216677`, attention-off NLL was `4.804799838144271`, and the penalty was `+2.6773925609275944` nats/token. Final enabled/off Python and Rust logits matched within `0.005`, all six layers executed through coherent R4/Spin transport, and zero future/provider/Ollama/prior-trace reads were recorded.

The full #1014 quality Definition of Done was negative: enabled NLL missed `<=1.50` and subject-or-scene retention was `3/5` rather than `4/5`. The selected development checkpoint was the final step, so the frozen run showed no development plateau. It consumed only `29,999,104` tokens for `7,155,360` parameters: `4.19` tokens per parameter.

Parent: #973. Predecessor: #1014. Programme root: #820. Implementation/evidence delivery: #1016.

## Objective

Complete one independently frozen quality-capacity continuation of the exact #1014 model. Add fresh deterministic training exposure until the cumulative total is `149,995,520` tokens (`20.96` tokens per parameter), then require coherent autonomous local Rust generation on a fresh disjoint confirmation tranche.

This is not another attention experiment. The #1014 attention-off question is closed and must not be rerun here.

## Fixed inheritance

Start from the exact #1014 selected `checkpoints/best.pt`, including its AdamW state:

- checkpoint CID `blake3:9c36e109d8dee67deec0362307ba4a471c967ff574835210f87653d628c95c91`;
- exported weights CID `blake3:7d7c26e1a71866dc46973cea3b23b819f4b5060b345d2a0ec1bd067aa493bb7d`;
- tokenizer CID `blake3:3f42bcfce7728512076549c63b88387e13c8156fe35c0f91d9b112439f3739cc`;
- dataset manifest CID `blake3:3e4d2ddb006771e5be0d4c76580c8971e6c67a23f8e223da8d81668d03bd9a01`;
- split policy CID `blake3:54f0886d3e906a4aeeaa9328ff236440d61d9f16b2f92dcb8c05cac96e54d1aa`.

Do not change the architecture or runtime path: 7,155,360 parameters; vocabulary 4,096; width 288; six layers; six Q/K/V heads; head width 48; FFN 768; context 256; tied head; RMSNorm; RoPE; SwiGLU; learned Q/K/V/O; complete-prefix scaled dot product; stable causal softmax; and weighted-value aggregation. Preserve batch 16, accumulation 4, AdamW betas, weight decay, clipping, tokenizer, story split, Rust sampler, and all-layer R4/Spin generation.

## Fresh deterministic population extension

Using the already-pinned TinyStories source and existing pre-tokenization story split:

1. Continue the deterministic training-story stream after the first 30,000,000-token cap and materialize the next `119,996,416` training tokens. Do not cycle or replay the original 30M store.
2. Materialize the next disjoint 250,000 development tokens after the complete #1014 development tranche. Evaluate the starting checkpoint on this tranche and include it in selection.
3. Before training, seal the next disjoint 250,000 test token IDs after the complete #1014 test tranche. Reserve 249,880 for NLL and use five 24-token prompts from the next five eligible story CIDs not used by #1014.
4. Training and selection must have no filesystem access to the new sealed confirmation paths.

If the pinned snapshot cannot supply these exact disjoint tranches, stop before training with `UNAVAILABLE_FRESH_POPULATION`.

## One frozen continuation

Run exactly 7,324 optimizer steps, adding `119,996,416` tokens and reaching `149,995,520` cumulative tokens.

Use one predeclared phase-two schedule: warm from the inherited `3e-5` floor to the unchanged `3e-4` peak over 100 steps, then cosine-decay once to `3e-5`. Include the starting checkpoint and every fixed development checkpoint in selection. Resume only the same interrupted run; no sweep, retry, alternate schedule, alternate architecture, or seed change is allowed.

Measured #1014 throughput projects the continuation at about 281.4 minutes. Hard ceiling: 5 hours 15 minutes on MPS, with no CPU fallback. If the ceiling is reached, record `UNAVAILABLE_MPS_BUDGET` and do not interpret a partial checkpoint.

## Only necessary gates

1. Before launch, reproduce the inherited checkpoint/data/tokenizer identities, MPS availability, fresh-tranche disjointness, and sealed-path denial.
2. Run the one fixed continuation. There is no new overfit smoke, fold, probe, permutation, support repair, attention-off arm, or alternative model.
3. After export, compare one enabled-only 32-token prefix between Python and Rust; require identical top-1 and maximum absolute logit delta `<=0.005`.
4. Reveal the fresh sealed confirmation tranche exactly once.
5. Generate the five fixed prompts through the all-layer Rust R4/Spin path with seeds 2014 through 2018 and replay each once.

## Definition of done

All must pass:

- fresh sealed-confirmation NLL `<=1.50`;
- at least `4/5` seeded continuations retain the prompt subject or scene;
- all outputs are valid UTF-8 and none enters a period-1 through period-4 loop;
- all five seeded outputs replay exactly after checkpoint reload;
- all six learned decoder layers execute through coherent R4/Spin transport;
- exact causal/projection/R4/output-policy audits with zero future reads;
- zero teacher, provider, Ollama, previous-trace, or #1014 sealed-test reads.

## Decision

If all gates pass, freeze the checkpoint as the first coherent locally trained R4/Spin base model. Only then may a later issue test the frozen multi-resonance/softmax-replacement mechanism against this quality baseline.

If quality still fails, close this issue negative and prohibit further 7.15M exposure extensions or learning-rate tuning. The next decision is one predeclared parameter-capacity increase or external training hardware, not another diagnostic ladder.

## Runtime boundary and nonclaims

This continuation uses floating point, multiplication, allocation, autograd, MPS, dense full-prefix dot products, and ordinary softmax. It does not claim geometry advantage over ordinary coordinates, transformerlessness, multiplication-free/table-native execution, reasoning, chat, browser-WASM readiness, release readiness, or frontier capability.

Depends on #1014
References #973
References #820

## Comments (5)

### comment 5471570336 — 2026-08-30T22:11:24Z — Casey-allard

Activated on `issue-1017-quality-capacity-continuation` from exact live `main` `4b21f9af37bbe4918413cf52b3f9fc905dad58dd`. The immutable #1014 inheritance has been reproduced before implementation: checkpoint `blake3:9c36e109d8dee67deec0362307ba4a471c967ff574835210f87653d628c95c91`, weights `blake3:7d7c26e1a71866dc46973cea3b23b819f4b5060b345d2a0ec1bd067aa493bb7d`, tokenizer `blake3:3f42bcfce7728512076549c63b88387e13c8156fe35c0f91d9b112439f3739cc`, dataset manifest `blake3:3e4d2ddb006771e5be0d4c76580c8971e6c67a23f8e223da8d81668d03bd9a01`, and split policy `blake3:54f0886d3e906a4aeeaa9328ff236440d61d9f16b2f92dcb8c05cac96e54d1aa`. Immediate action: implement the fresh non-replayed train/dev/test tranche and exact inherited-optimizer continuation lifecycle only. No smoke, attention-off rerun, architecture change, sweep, or diagnostic campaign.

### comment 5471784215 — 2026-08-30T22:58:01Z — Casey-allard

## Frozen launch contract — admitted

metric to move: enabled fresh held-out NLL from #1014 2.127407277216677 toward strict < 1.50; autonomous subject/scene retention from 3/5 toward >= 4/5.

reachability ceiling: all 7,324 continuation updates consume the independently constructed fresh train store: 7,324 × 16,384 = 119,996,416 new token presentations; cumulative exposure is exactly 149,995,520. The inherited #1014 development minimum was its final checkpoint, so this continuation begins from a still-improving selected state rather than a demonstrated plateau.

instrument + verdict: PASS. Corrected fresh dataset manifest blake3:5f709a9ef886801c55c799cd6f684774dc87a3e9e192f148d254c3d20a394aec replaces the invalid pre-fix build; predecessor train/dev/test CIDs reconstruct exactly without opening predecessor sealed data; all fresh source ordinals are disjoint; the new confirmation is mode 000 with an actual permission-denied read; 37 Python contract tests pass; focused Rust enabled-only qualifier/parser tests and cargo check pass; two independent launch reviews report PASS. Live admission is Apple MPS, CPU fallback disabled, inherited optimizer step 1,831, 29,999,104 inherited tokens, and all 56 AdamW parameter states present.

exit rule: select only across the 75 predeclared fresh-development candidates. Before reveal, require enabled-only Python/Rust prefix parity with identical top-1, maximum absolute logit delta strictly < 0.005, and exact all-six-layer causal/R4 audits. Reveal once. Quality passes only if enabled fresh held-out NLL is strictly < 1.50 and at least 4/5 fixed Rust generations retain their prompt subject/scene with valid UTF-8 and no period-1..4 loop.

if positive: freeze the checkpoint and release evidence, then advance directly to the next coherent-generation capability rung.

if negative: freeze the result and stop this capacity continuation; do not rerun, tune to the reveal, add an attention-off arm, or launch a sweep. Use the failure mode to choose one architecture change.

cost estimate: approximately 281 minutes from the measured #1014 throughput; hard cumulative MPS wall ceiling 18,900 seconds (5h15m). The lightweight elapsed ledger charges crashes and replay, and partial wall-ceiling output is explicitly uninterpretable.

### comment 5471811213 — 2026-08-30T23:03:57Z — Casey-allard

## Continuation launched

Frozen implementation commit: 29a86b4c (branch issue-1017-quality-capacity-continuation, now on origin).

The sole authorized Apple-MPS run is active with CPU fallback disabled. First durable recovery point completed at continuation step 100: train loss 2.242384, fresh-development loss 2.198040, best fresh-development loss remains the inherited step-0 candidate at 2.148865 during warmup, LR 0.00030000. The live ledger had advanced past step 114 when checked. No attention-off arm, sweep, sealed read, or alternate run has executed.

A task heartbeat is monitoring the same frozen run and will continue directly to enabled-only Rust parity, one-time reveal, five fixed Rust generations/replays, evidence documentation, and protected delivery when training completes.

### comment 5474572519 — 2026-08-31T06:24:10Z — Casey-allard

## Frozen terminal evidence — `FAIL_ENABLED_NLL`

The sole authorized #1017 continuation completed exactly 7,324 optimizer steps and reached `149,995,520` cumulative training tokens. The selected checkpoint is continuation step 7,324 / cumulative step 9,155 at development NLL `1.580241072373312`.

### Enabled-only Rust admission

- Python/Rust top-1: token `448` in both paths
- maximum absolute logit delta: `0.0000057220458984375` against the frozen `0.005` ceiling
- all six learned layers and causal, projection, R4/Spin, and output-policy audits: PASS
- future/provider/Ollama/prior-trace reads: zero
- attention-off executions: zero
- qualification decision: `blake3:5bc79e85fb8427152bcc87a2dd63ff08a0f9a453e5d0ae63161a0b32792312d6`
- admission manifest: `blake3:f0a6a4fc396c919d8f848c9cd7da9cfedecc6ece51b83582fb019f6ab85c8909`

### One-time sealed reveal

Fresh sealed NLL was `1.5727521962806827`, missing the strict `<1.50` gate by `0.0727521962806827`. It nevertheless improves #1014's `2.127407277216677` by `0.5546550809359942` nats/token (`26.071880399960784%`).

- reveal result: `blake3:84eb315adab2dbcc4c8811d4ede96ab531b66f8720b7c299b16bbe9e04bfd0b5`
- reveal manifest: `blake3:599497e087a3662e805498ca5a04085034e25e5a5e26850b2c4e8441a96d75e8`

### Autonomous Rust generation and replay

All five fixed seeds passed the frozen “retain the prompt subject or scene” rubric. Every output was valid UTF-8 and avoided period-1 through period-4 cycles. All five reload replays reproduced token IDs, text, stop state, execution ledgers, and core CIDs exactly after excluding timing only.

- human rubric: `blake3:22de86b5cf04191934df16cd3d5e24f24f3777c3d438bd2e9e9d4722f129c13e`
- final result: `blake3:0dbd94a279b6cd898e0d69667dcc1457c9dddba6061e6ff1d8c222c93793d46c`
- final manifest: `blake3:a53870f17837592cad7ce87d83abe7315c12181bccf0026e8035424fda783bfa`
- final tree: `blake3:19e2abf0ae4448c61ca093de341f6d4f137094491134be957d003f095cb9d448`

The complete Definition of Done fails solely on enabled sealed NLL. This exact 7.15M-parameter campaign closes negative without rerun, reveal-informed tuning, learning-rate changes, or further same-capacity exposure.

#1014's bounded load-bearing ordinary causal-attention conclusion remains established in coherent R4/Spin frames. #1017 adds a positive `5/5` bounded generation result, but does not establish geometry advantage, transformerlessness, multiplication-free/table-native execution, reasoning, chat, WASM, release readiness, or frontier capability.

The next action is one separately frozen parameter-capacity increase using the same qualified attention, split, evaluation, and Rust-generation path. Intrinsic geometry substitution, resonance/softmax replacement, recurrent lowering, exact deployment, and product promotion remain parked.

References #973
References #820

### comment 5474625003 — 2026-08-31T06:30:01Z — Casey-allard

Merged through protected PR #1018 at `967310318bc8cce337dfc1a212a71a8bfb15a2ca`.

Terminal remains `FAIL_ENABLED_NLL`: sealed NLL `1.5727521962806827` missed the strict `<1.50` gate, while enabled Rust parity, all six R4/Spin layers, causal/external audits, subject-or-scene retention `5/5`, and normalized replay `5/5` passed. The final result/manifest/tree are bound in the terminal-evidence comment above and in the merged human- and machine-readable records.

#1017 is closed without rerun, learning-rate tuning, or further 7.15M-parameter exposure.


---
_Comment/review ids in this file: 5471570336, 5471784215, 5471811213, 5474572519, 5474625003_

