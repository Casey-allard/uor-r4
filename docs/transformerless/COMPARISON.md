# Measured Comparison Against Classical Runtimes

> **Migrated (2026-07-18):** this document moved from the `transformerless`
> repository into uor-r4, where it is integrated as
> `uor_r4_core::transformerless`. Command invocations use the
> `transformerless` binary. Measurements below are the pre-migration record (Linux
> container).

Every number in this document was measured on the same machine (single-core
Linux container, one thread everywhere), against the same κ-pinned source
model (stories15M, `blake3:0ae73395…`), in one session, uncontended. Nothing
is quoted from papers or vendor pages. Reproduction commands are at the end.

## The rule this table obeys

A throughput number never travels without its quality number. The classical
runtimes execute the source model itself, so their agreement with the
teacher is 100% by definition; the transformerless artifact is a *different
member of the source's behavioral equivalence class* whose measured residual
is on every row (see PROOF.md). This is a comparison of runtime
architectures at stated quality, not a claim of quality parity.

## Generation throughput, single thread, same machine, same source

| runtime | version | tok/s | agreement w/ teacher argmax | multiplies/token | weights+state on disk |
|---|---|---|---|---|---|
| **transformerless mul-free (shipped path, decode-on-demand)** | this crate | **77,342** | 30.3% (live, n=4000; full held-out 31.7%) | **0** | **2.17 MB** (0.13 codes + 0.29 books + 0.04 signatures + 1.70 store) |
| transformerless, every op census-counted | this crate | 31,956 | same function (witnessed 3-deep: bundles, codes, predictions) | 0 | same |
| llama.cpp, q8_0 | `e8f19cc`, CPU backend | 344.28 ± 7.32 (tg128) | 100% (runs the source, q8_0-quantized) | ~15M mul-adds | 25.4 MB gguf |
| llama.cpp, f32 | `e8f19cc`, CPU backend | 157.43 ± 0.86 (tg128) | 100% (runs the source) | ~15M mul-adds | 93.8 MB gguf |
| L2E APE build (OpenMP run.c) | `trholding/llama2.c` experimental asset | 103 (self-reported) | 100% | ~15M mul-adds | 56 MB (embedded) |
| in-crate teacher (exactness-witnessed Rust) | this crate | 62 | 100% (it is the reference) | ~15M mul-adds | 60.8 MB checkpoint |
| run.c, gcc -O2 | karpathy master | 48 | 100% | ~15M mul-adds | 60.8 MB checkpoint |

Readings, in order of importance:

1. **The mul-free runtime is 225× the fastest classical runtime measured
   (llama.cpp q8_0) and 491× llama.cpp f32**, at its stated 31.7% teacher
   agreement — because it is not doing the same work. The classical
   runtimes traverse ~15M multiply-adds of superposed weights per token;
   the artifact performs ~1.8 × 10^5 integer operations (measured census:
   ~60k adds, ~37k xors, ~55k table reads, ~23k shifts, ~1.3k compares,
   zero multiplies), decoding its compressed representation on demand and
   resolving the answer from a store. Even with every single operation
   individually dispatched through the counting kernel it holds 31,956
   tok/s — the accounting overhead is ~2.4×, and the claim survives it.
2. **The classical spread itself is instructive**: llama.cpp f32 is 3.3×
   the -O2 C loop and 2.5× the exactness-constrained Rust teacher on
   identical arithmetic content, purely from SIMD-vectorized, reassociated
   kernels — speed bought by abandoning bit-order determinism. q8_0 doubles
   f32 by shrinking the memory stream. The transformerless artifact sits on
   the far end of the same axis: it removed the weight stream entirely, so
   it is bound by a few dozen kilobytes of table reads per token rather
   than tens of megabytes of weights.
3. **The artifact is an order of magnitude smaller, and its semantics
   differ in kind.** 2.17 MB of compressed artifact (PROOF.md P5: 87.2×
   on the representation at 0.9692 mean cosine, 28.1× end-to-end at the
   certified residual) against 25–94 MB of weights — and the artifact's
   1.7 MB store is
   enumerable entries with counts: every prediction resolves to specific
   keys, ships with a witness any peer can recheck, and any entry can be
   attributed or deleted without retraining. No weight encoding, quantized
   or otherwise, has an entry to point at.
4. **What multiplies buy, priced.** The 68-point agreement gap to the
   teacher ceiling is the current price of zero multiplies at a ~10^5-entry
   store on this distribution. The parent report's scaling measurements
   (§3.10–3.11) show that gap closing with store size with no saturation
   observed — the store converts bytes into resolution at constant
   per-token compute, which is a scaling axis the classical runtimes do
   not possess: their tok/s is fixed by the weight stream regardless of
   how much is known.

## Scenario suite: real-world prompts and diverse input

`transformerless scenarios` runs a fourteen-scenario suite over four input
classes, every stream unseen by the store (train split only), teacher and
artifact fed the identical token stream. The tokenizer is witnessed in-run:
byte-exact round-trip, plus a fluency gate — the teacher must continue a
probe prompt fluently, which a broken encoding cannot pass (it continued
"Once upon a time, there was a little dog." with ". He was very happy and
loved to play. One day he was playing in the park when he"). Prompt
scenarios measure agreement along the teacher's own greedy trajectory (the
deployment question: would the artifact have produced the same
continuation, token by token); real-text scenarios additionally score both
systems against the actual next token of human-written text.

| scenario | class | tokens | agree w/ teacher | tless top1 | teacher top1 |
|---|---|---|---|---|---|
| dog-named | in-domain prompt | 78 | 38.5% | — | — |
| park-ball | in-domain prompt | 79 | 32.9% | — | — |
| sad-bird | in-domain prompt | 75 | 33.3% | — | — |
| red-truck | in-domain prompt | 75 | 26.7% | — | — |
| shiny-key | in-domain prompt | 78 | 23.1% | — | — |
| capital-q | out-of-domain prompt | 71 | 12.7% | — | — |
| explain | out-of-domain prompt | 72 | 22.2% | — | — |
| code | out-of-domain prompt | 73 | 21.9% | — | — |
| business | out-of-domain prompt | 75 | 28.0% | — | — |
| handwritten-story | real text, in-domain style | 108 | 26.9% | 17.6% | 47.2% |
| shakespeare | real text, out-of-domain | 253 | 12.6% | 0.8% | 1.2% |
| repetition | stress | 80 | 32.5% | — | — |
| one-word | stress | 65 | 24.6% | — | — |
| cold-start | stress | 64 | 54.7% | — | — |

Aggregated by class, with per-class throughput measured in the same run:

| class | positions | agree w/ teacher | tless top1 | teacher top1 | tless tok/s | teacher tok/s |
|---|---|---|---|---|---|---|
| in-domain prompt | 385 | 30.9% | — | — | 67,314 | 92 |
| out-of-domain prompt | 291 | 21.3% | — | — | 66,405 | 98 |
| real text, in-domain style | 108 | 26.9% | 17.6% | 47.2% | 75,283 | 101 |
| real text, out-of-domain | 253 | 12.6% | 0.8% | 1.2% | 78,514 | 94 |
| stress | 209 | 36.8% | — | — | 72,730 | 99 |

Readings, including the ones that cut against the artifact:

1. **Generalization to unseen human prompts matches the corpus
   certificate.** In-domain prompt agreement (30.9% over 385 positions)
   reproduces the held-out corpus figure (31.7%) on text no part of the
   pipeline ever saw. The certificate number is not an artifact of
   evaluating on model-sampled text.
2. **Out-of-domain prompts degrade gracefully, not catastrophically**
   (21.3%): the store's coarse levels carry generic English continuation
   even for question and instruction forms absent from the training
   distribution. The worst single scenario is the bare factual question
   (12.7%), the input class furthest from the store's contents.
3. **The honest gap is real human-written text.** On in-domain-style
   human prose the teacher predicts the actual next token at 47.2% while
   the artifact manages 17.6%. The store was distilled from teacher
   SAMPLES, and the sample→human distribution shift costs the artifact
   more than it costs the teacher. At this store scale, trajectory
   agreement transfers to human text substantially diluted; this row, not
   the throughput rows, is the deployment-relevant residual.
4. **Cross-compilation preserves ignorance faithfully.** On Shakespeare
   both systems collapse (teacher 1.2% top-1, artifact 0.8%): the artifact
   cannot know what its source does not, and does not pretend to. The
   equivalence claim is to the source's behavior, including its limits.
5. **Structure helps.** The highest single-scenario agreement is the cold
   start (54.7%): from BOS the teacher's trajectory funnels through
   high-probability openings the store knows densely. Stress inputs as a
   class (36.8%) sit above every prompt class.
6. **Throughput is scenario-invariant** (66–79k tok/s across all classes;
   same-run teacher 92–101 tok/s, ratio ≈700×): the artifact's cost is a
   fixed number of table reads regardless of input difficulty, where the
   teacher's cost is fixed FLOPs. Note teacher throughput varies 60–100
   tok/s across sessions on this shared container; ratios should be read
   within one run, and the main table above keeps its own session's pairs.

## Conditions and caveats, stated

- Single hardware thread throughout (`-t 1` for llama.cpp; OpenMP on one
  core for the APE build). llama.cpp `tg128`, `-r 3`, mean ± σ reported.
- The gguf converter unties the classifier, so llama.cpp reports 24.41M
  parameters for the 15M shared-weight checkpoint and the f32 gguf is
  93.8 MB; arithmetic per token is the same order either way.
- The transformerless timing covers the full per-token path — compressed
  row decode, bundle, signature, Hamming assignment, store probe, argmax —
  over held-out corpus positions; its live agreement on the timed sample
  (30.3%, n=4000) sits below the full held-out figure (31.7%, n=30,192) by
  sampling variation.
- llama.cpp numbers are for this build (`e8f19cc`, CPU backend, native
  ISA detection off) on this container's core; faster hardware moves every
  row, not the ratios' character. On macOS, Metal is enabled by default in
  llama.cpp, so reproducing the CPU row requires disabling Metal at build
  time or passing `--device none` at run time.

## Cross-system reproduction

Use one path as the **portable baseline**, then add accelerated variants as
separate rows. Do not merge CPU, Metal, CUDA, Vulkan, or other backends into
one headline number: backend and machine class are part of the condition.

The active UOR architecture/runtime direction is CPU-native. Apple
Accelerate/BLAS and Metal/MPS may be used only for local offline compilation,
training, or bounded comparison tests. The non-Apple accelerator recipes below
are retained solely to reproduce historical external comparison rows; CUDA and
external GPU execution are out of scope for current UOR development and are not
an execution path for #1019.

Before any `llama.cpp` conversion or benchmark command below, fetch the source
checkpoint required by this repo's teacher path:

    # prerequisite: source checkpoint used throughout this repo
    curl -sL -o /tmp/run.com https://github.com/trholding/llama2.c/releases/download/experimental/run.com
    cd /tmp && unzip -o run.com out/model.bin tokenizer.bin -d ref

The conversion commands below expect the checkpoint at
`/tmp/ref/out/model.bin`. If you have the checkpoint in another location,
either move it there or update the `--llama2c-model` argument consistently.

### Portable baseline: macOS, x86/Linux, any system with a C++ toolchain

This is the path the table above uses: pinned `llama.cpp`, single thread,
CPU backend, native ISA detection off.

    # llama.cpp at e8f19cc, CPU backend
    git clone https://github.com/ggml-org/llama.cpp && cd llama.cpp
    git checkout e8f19cc
    cmake -B build -DCMAKE_BUILD_TYPE=Release -DGGML_NATIVE=OFF -DLLAMA_CURL=OFF -DGGML_METAL=OFF
    cmake --build build -t llama-cli llama-bench llama-quantize llama-convert-llama2c-to-ggml -j 1
    ./build/bin/llama-convert-llama2c-to-ggml \
        --copy-vocab-from-model models/ggml-vocab-llama-spm.gguf \
        --llama2c-model /tmp/ref/out/model.bin \
        --llama2c-output-model /tmp/stories15M-f32.gguf
    ./build/bin/llama-quantize /tmp/stories15M-f32.gguf /tmp/stories15M-q8_0.gguf q8_0
    ./build/bin/llama-bench -m /tmp/stories15M-f32.gguf  -t 1 -p 0 -n 128 -r 3 --device none
    ./build/bin/llama-bench -m /tmp/stories15M-q8_0.gguf -t 1 -p 0 -n 128 -r 3 --device none

### macOS accelerated variant: Metal

On macOS, Metal is enabled by default. Keep the same pinned commit and the
same benchmark arguments, but allow the Metal backend:

    git clone https://github.com/ggml-org/llama.cpp && cd llama.cpp
    git checkout e8f19cc
    cmake -B build -DCMAKE_BUILD_TYPE=Release -DGGML_NATIVE=OFF -DLLAMA_CURL=OFF
    cmake --build build -t llama-cli llama-bench llama-quantize llama-convert-llama2c-to-ggml -j 1
    ./build/bin/llama-convert-llama2c-to-ggml \
        --copy-vocab-from-model models/ggml-vocab-llama-spm.gguf \
        --llama2c-model /tmp/ref/out/model.bin \
        --llama2c-output-model /tmp/stories15M-f32.gguf
    ./build/bin/llama-quantize /tmp/stories15M-f32.gguf /tmp/stories15M-q8_0.gguf q8_0
    ./build/bin/llama-bench -m /tmp/stories15M-f32.gguf  -t 1 -p 0 -n 128 -r 3
    ./build/bin/llama-bench -m /tmp/stories15M-q8_0.gguf -t 1 -p 0 -n 128 -r 3

Report these rows as `Metal` (or whatever backend string `llama-bench`
prints), not as CPU rows.

### Historical external comparison only: CUDA (out of current UOR scope)

The commands below reproduce the historical NVIDIA comparison condition. They
are not an active UOR implementation, training, runtime, or #1019 path:

    git clone https://github.com/ggml-org/llama.cpp && cd llama.cpp
    git checkout e8f19cc
    cmake -B build -DCMAKE_BUILD_TYPE=Release -DGGML_NATIVE=OFF -DLLAMA_CURL=OFF -DGGML_CUDA=ON
    cmake --build build -t llama-cli llama-bench llama-quantize llama-convert-llama2c-to-ggml -j 1
    ./build/bin/llama-convert-llama2c-to-ggml \
        --copy-vocab-from-model models/ggml-vocab-llama-spm.gguf \
        --llama2c-model /tmp/ref/out/model.bin \
        --llama2c-output-model /tmp/stories15M-f32.gguf
    ./build/bin/llama-quantize /tmp/stories15M-f32.gguf /tmp/stories15M-q8_0.gguf q8_0
    ./build/bin/llama-bench -m /tmp/stories15M-f32.gguf  -t 1 -p 0 -n 128 -r 3 -ngl 99
    ./build/bin/llama-bench -m /tmp/stories15M-q8_0.gguf -t 1 -p 0 -n 128 -r 3 -ngl 99

Report these rows as `CUDA`; they are not comparable as CPU rows.

### Other backends

The same pattern applies to Vulkan, OpenCL, SYCL, or other `llama.cpp`
backends: pin the same `e8f19cc` commit, keep `-DGGML_NATIVE=OFF`, use the
same f32/q8_0 gguf artifacts, and report the backend and hardware alongside
the throughput. If you change backend, device, or offload depth, you have a
new row, not a re-measurement of the CPU row.

## In-crate reproduction

    # this crate (after `gen` and `compile`; see README)
    transformerless compare
    transformerless scenarios
