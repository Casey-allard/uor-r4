# uor-r4-lut — integer serving of converted Llama checkpoints

Pre-alpha. This crate serves a Hugging Face Llama checkpoint (SmolLM2-135M/360M-Instruct are the intended
vehicles) with integer arithmetic only, under owner decision
[D10](../../docs/integration/DECISIONS.md). The dense backbone is D10's interim chat vehicle: it reads every
weight on every token, so it is not the sparse end state of D5, and it adds no geometric mechanism by itself.

## What runs at serving

- Every value is an integer with an explicit power-of-two exponent. RMSNorm uses an integer square root and one
  division per vector; exp (softmax), SiLU and RoPE are tables sealed into the artifact by the exporter.
- Learned weight maps (projections, MLP, output head, embedding) are 4-bit offset-binary weights in groups of 32
  with scales `(16 + m) 2^(e - 4)`. A matrix product reads per-activation tables of multiples, sixteen rows per
  vector table instruction (`vpshufb` on AVX2, `tbl` on NEON, through the audited
  [`uor-r4-simd`](../uor-r4-simd/src/lib.rs) crate), and applies scales by shifts and additions. No multiplier
  touches a learned weight or scale: the compiled AVX2 and NEON weight kernels contain no vector multiply, and
  their only multiplies are scalar block-offset computations outside the per-weight loop.
- Products of runtime values (attention scores, value mixing, gating, RoPE rotation, normalization) use the
  integer multiplier, which D10 allows. The key/value cache is 8-bit with one exponent per vector.
- Decoding is one token per step (no batched prompt prefill); a step runs inside a private worker pool, and heads
  are split across threads at long contexts. Every backend and thread count computes the same integers.
- Optionally, a learned Lorentz cache memory (lab M4): the artifact carries 4-bit query, key and gate maps of the
  final normalized state and a sealed arcosh table. Each step stores its key, and reads the entries at least `gap`
  positions back by hyperbolic distance. The integer distance is exact up to the arcosh table, and the weights
  come from the exp table. The next token comes from `(1 - g) p_model + g p_cache`. `lut-chat` then slides a full
  attention window (re-encoding its recent half), and the cache keeps everything older.

## Artifact

`UORLUT01`, then a little-endian `u64` header length, a JSON header holding integers only (shape, numerics,
matrix and table spans, exporter provenance), and 64-byte-aligned sections. Matrices are stored row-major (two
nibbles per byte, low nibble the even column; one scale byte per group: low nibble `m`, high nibble `de` above
the matrix's `exp_base`). The engine repacks projections into 16-row blocks at load and releases the file
buffer. See [`src/format.rs`](src/format.rs).

## Tools

```text
lut-tool mode=export model=DIR out=FILE.lut [max_positions=2048]
    [calibration=X.u16 [calibration_windows=16] [calibration_time=256] [damp=0.01]]
    [cache=ROOT/models/lorentz-seedS.safetensors]
lut-tool mode=dequantize model=DIR lut=FILE.lut out=NEW_DIR
lut-tool mode=fidelity model=DIR lut=FILE.lut tokens=X.u16 out=NEW_ROOT [windows=8] [time=128] [threads=N]
lut-tool mode=bench lut=FILE.lut [tokens=64] [threads=N] [backend=portable|avx2|neon]
lut-tool mode=cache-fidelity model=DIR lut=FILE.lut cache=MODEL.safetensors tokens=X.u16 out=NEW_ROOT
    [segments=16] [segment=1024] [window=256] [threads=N]
lut-chat lut=FILE.lut tokenizer=DIR/tokenizer.json [prompt=TEXT] [system=TEXT] [tokens=256] [threads=N] [raw=true]
    [temperature=0] [top_k=0] [top_p=1] [presence=0] [seed=1]
```

`lut-tool` is an example of `uor-r4-training` (it needs the float checkpoint); `lut-chat` is this crate's binary.
`export` quantizes by round-to-nearest, or, with `calibration=`, by GPTQ (Frantar et al. 2022): each matrix's
columns are rounded in order and every rounding error is spread over the remaining columns through the inverse
second moment of that matrix's inputs, measured on the calibration tokens by the float model. `dequantize` writes
the float checkpoint the artifact represents, which isolates weight quantization from integer arithmetic.
`lut-chat` decodes greedily by default; a positive `temperature` samples with integer arithmetic only (the sealed
exp table, a Q8 temperature, top-k, top-p, a presence penalty on recent tokens, and a seeded generator, so a seed
reproduces a conversation). `fidelity` compares next-token distributions of the float checkpoint and the integer engine on a token file and
seals a report root. [`scripts/lut-m1-chat.sh`](../../scripts/lut-m1-chat.sh) runs the whole path on an
Apple-silicon Mac (export both ways, fidelity, throughput, a chat turn, and optional joules per token through
[`scripts/energy_per_token.py`](../../scripts/energy_per_token.py) with an optional llama.cpp reference).

## Measured (2026-09-26, 4-vCPU x86-64 container)

Measured behaviour, not a capability claim. The trained model below is a 4.2M-parameter byte-level Llama (6
layers, width 256, 4 heads, 2 key/value heads) trained on WikiText-2 in a scratch workspace for these tests (a
throwaway JAX script, not part of the repository); it scores 1.678 bits per byte in bf16 on the evaluation
windows. Evaluation: 16 windows of 256 validation tokens (4096 positions); GPTQ calibration: 32 windows of 256
training tokens.

| Integer engine versus | NLL change (nats/token) | KL | top-1 agreement |
| --- | ---: | ---: | ---: |
| original bf16, round-to-nearest export | +0.0067 | 4.6e-3 | 95.95% |
| original bf16, GPTQ export | +0.0034 | 1.8e-3 | 97.95% |
| its own dequantized float model (round-to-nearest) | +0.0007 | 2.6e-4 | 99.32% |
| its own dequantized float model (GPTQ) | +0.0009 | 2.7e-4 | 99.41% |

Greedy 110-token continuations of the integer engine match those of its dequantized float model token for token
(both exports), so on this model the remaining gap to the original is the 4-bit weights, which GPTQ halves.

Decoding throughput on a random artifact with SmolLM2-135M's shape (86.8 MB; AVX2; other work paused):

| threads | 64 tokens | 1024 tokens |
| ---: | ---: | ---: |
| 1 | 36.5 tok/s | 30.4 tok/s |
| 2 | 47.4 tok/s | 39.7 tok/s |
| 4 | 54.8 tok/s | 42.0 tok/s |

The scalar table kernel it replaced ran at 5.9 tokens/s on one thread. The weight kernel alone takes about 0.10 ns
per weight with matrices in cache and about 0.17 ns streaming the model; at four threads the container's memory
bandwidth limits throughput.

The cache is trained offline by the `cache-memory` example of `uor-r4-training` (`save=true`), exported with
`cache=`, and checked by `cache-fidelity`. That mode runs the float cache over the float backbone and the integer
engine with its cache on the same held-out positions. On the small Llama the integer cache kept 99.3% of the float
cache's gain: −0.01828 against −0.01840 nats per token on 49,152 positions beyond the 256-token window.

## Limitations

- No Apple-silicon measurement yet: the NEON kernels are checked bit for bit under user-mode emulation only, so
  M1 speed and energy are unmeasured. No energy figure exists for this engine.
- The per-position parts of attention (score scaling, exp lookup) are scalar; prompts are fed one token per
  step.
- The portable fallback (CPUs without AVX2 or NEON) is slower than the old scalar kernel.
- Fidelity on the 135M/360M checkpoints is unmeasured here (their weights are not reachable from this
  environment); the numbers above come from a small model trained for the purpose.
