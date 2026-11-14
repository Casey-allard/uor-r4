---
name: native-space-cycle
description: Keep UOR-R4 training and serving in one persistent native geometric state. Use before model training, optimizer, carry-state, memory, artifact or parity work.
---

# Native-space cycle

Read `CHARTER.md` §3.7, `D85`, and `docs/harness/native-space.json` before changing a model
path.

<!-- native-space-contract: Native default: one persistent native geometric state uses one declared integer or fixed-point representation for token addresses, vector coordinates, carry, routes, memory keys and reads, learned state, gradients and optimizer updates; training and serving use the same typed operators; route choices, carry state, read choices, memory addresses and token ids are bit-identical on the same prompt and starting state; the artifact losslessly serializes that state instead of an F32 master model or an export gap; floating point remains reference or diagnostic code only. -->

## Invariant

Training and serving are two phases of one state transition:

1. **One persistent state.** Token addresses, vector coordinates, SpiralCore carry, geometric
   route choices, memory keys, memory reads and learned state use one declared integer or
   fixed-point representation from the first training step through serving.
2. **One operator definition.** Training and serving call the same typed native operators.
   Device-specific CUDA, CPU, WASM or browser implementations are allowed only behind one
   source definition and must pass bit-identical differential tests.
3. **One transition.** For the same prompt and starting state, training-mode simulation and
   the served runtime must produce identical route choices, carry state, read choices,
   memory addresses and emitted token ids. A tolerance is an instrument result, not parity.
4. **Training updates that state.** Gradients and optimizer updates land in the same native
   representation the served runtime reads. A float master copy restored after each step is
   not native-space training.
5. **Serialization is not translation.** An artifact may pack, align, compress losslessly or
   reorder bytes. It may not quantize a different master model into the served model. A
   nonzero export gap cannot qualify the default path.

Float may remain in separately scanned reference or diagnostic code. It is not
the default model, carry state, memory state or optimizer state, and it cannot be the source
from which the served state is manufactured.

## Before writing model code

Write down these five items in the ORDER:

- the native state schema and every carried field;
- the shared typed operator paths and their CUDA/CPU/WASM implementations;
- the exact transition replay and its bit-identical oracle;
- how forward, gradient and optimizer update remain in that state;
- the serialization format that preserves the state without numerical conversion.

If one of these is missing, the unit is an inventory or contract unit. Do not start a
training run.

## Required checks

First:

```sh
uor-harness check --only harness.native_space_contract
```

Then the unit's own checks must include a shared-state replay. Compare raw integer words or
bits, never rounded decimal values, for route choices, carry state, read choices, memory
addresses and token ids.

## Reject

- an F32 master model on the default path;
- separate numeric training and serving kernels without a bit-identical differential;
- `--train-as-served` rounding over a float master presented as native training;
- an export gap used as acceptance for a model whose training and serving states differ;
- dense matmul, softmax/squashing, Python model code or transformer fallbacks.

## Report

State the representation, operator source, backend identities, replay fixture and comparison
unit. A test that compares distances or rounded outputs is not a bit-parity result.
