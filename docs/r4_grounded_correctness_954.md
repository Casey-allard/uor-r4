# C1-SB0: source-backed grounded answer prototype (#954)

Status: **ACTIVE / PRODUCT PROTOTYPE**
Parent programme: #820
Working model: #1017 six-layer R4/Spin causal-softmax checkpoint
Final source-free correctness terminal: still blocked by parked #973

## Why this is the next build

PR #1022 made the #1017 checkpoint directly usable through `r4 generate` and
added an opt-in Apple Accelerate CPU-BLAS build for local Apple Silicon use.
On the project M1, a 64-token continuation completed in `0.220401125` seconds
of recorded generation time (`0.67` seconds process wall time) and remained a
coherent child/mother/oak-tree scene. All six R4/Spin attention layers ran, the
causal and R4 audits matched, and future reads were zero.

The same checkpoint did not yet behave as an answer model:

- `Context: The sky is blue. Question: What color is the sky? Answer:` did not
  produce the supplied answer; and
- a natural cloze ending in `the sky was` generated `dark`, contradicting the
  supplied sentence.

Those observations locate the product seam. Attention and fluent generation
work; grounded answer formatting and abstention have not been taught. No new
attention mechanism, capacity run, or backend comparison is needed first.

## Prototype

`C1-SB0` adds two deliberately small pieces:

1. `r4 answer --source-file <path> --question <text>` binds exact source bytes,
   runs the existing #1017 R4/Spin generator once, and serves only an exact
   contiguous source span. `ABSTAIN`, `CONTRADICTION`, empty output, and
   unsupported generated text remain typed outcomes. Raw model text is retained
   only in the audit report.
2. One fixed MPS supervised fine-tune starts from a copy of #1017 and teaches
   the exact answer interface using deterministic supported, unsupported, and
   conflicting-context examples. It does not alter the architecture, tokenizer,
   R4/Spin attention path, or Rust loader.

The training budget is fixed at 384 optimizer steps with no sweep. At the
measured conservative #1019 rate of `3.491307 s/step`, the upper estimate is
about 22 minutes 21 seconds. Apple Accelerate is used for the subsequent local
Rust behavior run; MPS is used only for this offline fine-tune.

## Decision

All three frozen unseen product probes must pass in the same run: the supported
question must return its exact source span, the unsupported question must return
typed `ABSTAIN`, and the conflicting source must return typed `CONTRADICTION`.
Failure keeps work at this answer-format/data seam; it does not trigger another
attention, geometry, capacity, or hardware campaign.

## Scope boundary

A positive result establishes only a source-backed prototype answer/abstention
surface for this small model and prompt policy. It does not establish semantic
entailment, general factual correctness, reasoning, intrinsic-geometric
advantage, source-free/table-native inference, transformerlessness, browser or
WASM readiness, frontier quality, or release readiness.

## Observed result — 2026-08-31

The fixed run completed on the project Apple M1 through PyTorch MPS without CPU
fallback or CUDA. All 384 optimizer steps finished in `883.7735486670863`
seconds (14 minutes 44 seconds), versus the conservative 22-minute estimate.
Answer-token development loss was `0.0013607110659437775` at step 192 and
`0.00014458666336423967` at step 384. The export is bound by:

- weights CID `blake3:0949d925c1189cdc784a4b93bbb9119da5c1a573fe3385c3b3c8f82a299c9439`;
- final manifest CID `blake3:c2c5b163485e39f14ba0a7869d83fab8c641ed061b5c0e8e5783924aff187b2f`;
- run-contract CID `blake3:31b6d7776a42779d0b8daddfc8034f9984dc9f85476eb0bd5df713cd7624f589`; and
- training-result CID `blake3:f957c11f668a9a81098f4b2ad9cac9af75d3636b5a2c6cc1a796c5817be10ac9`.

The Rust product command then loaded that exact weight CID through Apple
Accelerate CPU BLAS and ran all three frozen prompts. It emitted exact
`ABSTAIN` plus EOS for every prompt:

| Frozen prompt | Required outcome | Observed outcome | Verdict | Decision CID |
| --- | --- | --- | --- | --- |
| supported amber-coin source | exact supported sentence | typed `ABSTAIN` | FAIL | `blake3:63024db98ceb1cfdae5b106ddae0d611a027cc160578452c78ef6f4fda18bfe7` |
| unsupported velvet-ribbon question | typed `ABSTAIN` | typed `ABSTAIN` | PASS | `blake3:de1baea232dcb93bbae199e48ea3cdddc5c10de7da7c59e0b416dd0454eae35a` |
| conflicting amber-coin source | typed `CONTRADICTION` | typed `ABSTAIN` | FAIL | `blake3:b89b10fcff8c88cc7ed0d39c3d9c3293e0aa619d2fe34d7d044e6f00dfec4383` |

Terminal: **`FAIL_GROUNDING_PRODUCT_TRANSFER_ABSTENTION_COLLAPSE`** (`1/3`).
The low same-vocabulary teacher-forced development loss did not transfer to the
reserved lexical items under autonomous decoding. This is a copying/generalized
span-selection failure at the answer interface, not an attention-backend or
hardware failure. The run is not retuned or repeated.

The next successor should replace free-form answer generation with a learned
source-span pointer/copy head plus explicit abstention and conflict scores over
the already established causal R4/Spin states. That mechanism must be frozen on
new lexical families before a newly reserved product population is revealed.
Do not return to #1019 capacity, resonance, intrinsic-attention substitution, or
backend comparison for this failure.
