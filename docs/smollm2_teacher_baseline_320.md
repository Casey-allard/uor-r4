# SmolLM2 teacher baseline — issue #320 P1

This records the P1 rehearsal for the teacher-upgrade issue. It does not
replace the pinned stories15M fixtures or make a baseline-migration decision.

## Pinned source

| field | value |
|---|---|
| repository | `HuggingFaceTB/SmolLM2-135M-Instruct` |
| revision | `7e27bd9f95328f0f3b08261d1252705110c806f8` |
| source κ | `blake3:12d2cd8a877ef2cdcf785b3d4d1f373e0419074cc884aeaff06fc059686a5ba5` |
| source κ scope | `model.safetensors` |
| source bytes | `269060552` |
| license | Apache-2.0 |

The source κ is emitted by the HF teacher loader after reading the pinned
weights. The revision and κ are both recorded in
`models/smollm2-135m-instruct.json`.

## Rehearsal

The source was downloaded with the existing pinned HF path and compiled in an
isolated worktree:

```bash
hf download HuggingFaceTB/SmolLM2-135M-Instruct \
  --revision 7e27bd9f95328f0f3b08261d1252705110c806f8 \
  --local-dir .uor-models/sources/smollm2-135m-instruct \
  --include '*.safetensors' --include '*.json' --include '*.model' \
  --include 'merges.txt' --include 'LICENSE*' --include 'README.md'

cargo run --release --offline --bin r4 -- transformerless compile \
  --source .uor-models/sources/smollm2-135m-instruct \
  --output .uor-models/compiled/smollm2-135m-instruct \
  --seconds 60 --target 1000 --sequence-length 128
```

Observed rehearsal result:

- source directory: 260 MB;
- corpus: 1,000 teacher-labeled tokens across 11 stories;
- teacher generation: 1,000 tokens in 8.77 s (114.1 tokens/s);
- table-native compilation completed, including artifacts, store, tokenizer,
  calibration, hierarchical codes, and manifest.

This is a pipeline smoke/rehearsal, not a quotable quality baseline: the
1,000-token corpus is not the D3 held-out distribution and no comparison with
the stories15M teacher floor was made. P2 should run the declared row matrix
on a complete corpus before any migration decision.

## P2 — complete 135M rehearsal and graph row

The resumable compile was continued with the same pinned source and a
20,000-token target. It completed with 20,000 records across 199 stories
(19,000 new records in 148 seconds; 128.4 tokens/s), preserving the teacher
κ above. The resulting TLA5/TLS1 bundle was measured against the compiler's
80/20 story split and the retained corpus was used to induce and score the
R4G1 graph.

Commands:

\`\`\`bash
cargo run --release --offline --bin r4 -- compile \
  --source .uor-models/sources/smollm2-135m-instruct \
  --output .uor-models/compiled/smollm2-135m-instruct \
  --seconds 300 --target 20000 --sequence-length 128

cargo run --release --offline --bin r4 -- transformerless cover \
  --corpus-meta .uor-models/compiled/smollm2-135m-instruct/corpus.meta \
  --corpus-recs .uor-models/compiled/smollm2-135m-instruct/corpus.records \
  --artifacts .uor-models/compiled/smollm2-135m-instruct/tless_artifacts.bin \
  --out .uor-models/compiled/smollm2-135m-instruct/graph-cover

cargo run --release --offline --bin r4 -- transformerless score \
  --corpus-meta .uor-models/compiled/smollm2-135m-instruct/corpus.meta \
  --corpus-recs .uor-models/compiled/smollm2-135m-instruct/corpus.records \
  --artifacts .uor-models/compiled/smollm2-135m-instruct/tless_artifacts.bin \
  --cover .uor-models/compiled/smollm2-135m-instruct/graph-cover/cover.r4g1 \
  --quality-profile relative_tla \
  --out .uor-models/compiled/smollm2-135m-instruct/graph

cargo run --release --offline --bin r4 -- evaluate-report \
  --source .uor-models/sources/smollm2-135m-instruct \
  --compiled .uor-models/compiled/smollm2-135m-instruct \
  --report .uor-models/compiled/smollm2-135m-instruct/instruction-eval.json \
  --sequence-length 128
\`\`\`

The teacher-floor report covered 4,419 held-out positions. The table-native
bundle reached 7.6% top-1 accuracy and 13.0% teacher-argmax agreement at
20.6962 Witten–Bell bits/token, against a 4.9482-bit teacher floor. The scored
graph's Rule 1+2 row reached 5.6% top-1 agreement and 48.1532 bits/token;
cloud-size-normalized and margin-weighted variants reached 9.3% / 13.5412
bits/token and 9.1% / 13.5709 bits/token respectively. The same-corpus TLA
baseline was 13.0% / 20.6962 bits/token. Status counts were 1,190 exact
context, 3,170 graph, and 59 novel positions; witness replay was 64/64.

For scale, the existing stories15M fixture reference is 31.7% teacher-argmax
agreement and 9.86 bits/token on 30,036 held-out positions (see
\`docs/transformerless/BASELINE.md\`). Those figures are retained as historical
reference only: the checkpoints, corpora, and teacher floors differ, so this
P2 row is not a cross-teacher quality comparison.

This is a valid P2 pipeline and row measurement, but it is not a migration
candidate: the generated 20k-token rehearsal corpus has a weak teacher ceiling
and the graph does not improve on the same-corpus TLA baseline. The declared
next step remains P3 only as a maintainer decision after a larger/stronger
teacher corpus or the 360M rehearsal; the pinned stories15M fixtures remain
unchanged.

## CPU teacher path — issue #320 follow-up

The Hugging Face teacher compiler remains CPU-only. The native macOS path uses
Accelerate's CPU matrix-vector and vForce math kernels; it does not select
CUDA, Metal, OpenCL, or any GPU backend. The deployed transformerless runtime
is unaffected and remains multiplication-free.

Two CPU-side costs were reduced:

- RoPE `powf`/`sin`/`cos` values are precomputed once per position and head
  dimension instead of once per layer and token.
- macOS vocabulary exponentiation uses CPU vForce by default in noncanonical
  mode. Set `TLESS_TEACHER_VFORCE_EXP=0` for the scalar comparison path;
  `TLESS_CANONICAL_DETERMINISTIC=1` always disables the fast math backend.

On the local 360M source, a fresh 5,000-token compile measured 74.5 tokens/s
with both changes enabled, versus 54.5 tokens/s in the earlier scalar-exp /
uncached-RoPE run. This is a throughput measurement, not a quality or
determinism claim; canonical mode remains the reproducibility path.
