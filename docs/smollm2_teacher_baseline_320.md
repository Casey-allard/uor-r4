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

## P2 — SmolLM2-360M baseline (cloud, 2026-08-08)

The full P2 pipeline was run on the 360M source: observe to a 20,000-record
corpus, cover, score (`relative_tla`), teacher-floor report. Same commands as
the 135M P2 above with the source and output paths swapped to
`smollm2-360m-instruct`.

### Provenance

The descriptor's pinned revision `2366112999e525164f9f74a3fbf50ec19b48b940`
returns 404 on Hugging Face (upstream rewrote it). The run used current `main`,
revision `a10cc1512eabd3dde888204e902eca88bddb4951`; its `model.safetensors` is
byte-identical (723,674,912 bytes) to the copy pinned on the measurement machine
in July, so the weights are the same SmolLM2-360M-Instruct release — the changed
revision is a metadata commit. κ on record: source
`blake3:27ec272b02c2d41c805d8e2e143a9bd43a1c4b8cdee46653ab944f91c5132aa5`,
teacher (compiler layout)
`blake3:eb23c3e8527110b83c091f8660aba676ec4993c9212a9e147503878d6087191f`,
artifacts
`blake3:910b1112537d3b5038cf0e6d7c111391b5e2deab6a0fdda3c8a0ce7d289e4505`. P3's
re-pin is a maintainer step regardless, so measuring the current release is valid
for P2.

### Result (Gate C, 4,098 held-out D3 positions; EXCT-miss 73.1%)

The teacher floor is **3.7908 bits/token**. The TLA table-native bundle reached
5.54% top-1 and 8.61% teacher-argmax agreement at 21.1834 Witten–Bell
bits/token, so the artifact sits **+17.39 bits over the floor** — compiler-bound
on this corpus, not teacher-bound. The scored graph's Rule 1+2 row reached 5.0%
top-1 at 14.78 bits/token; the #399 A-mode forward-anchor row replicated in
direction at 5.7%; the same-corpus TLA3 store baseline was 8.6% / 21.18.

### The migration signal

The 360M teacher floor (3.79 bits) is lower than the 135M cloud floor (4.20) and
the original 135M doc floor (4.95): a measurably more competent teacher with a
higher ceiling. Floors are on different corpora so they are not a direct quality
comparison, but the trend is the point. The artifact top-1 (5.5%) is lower than
the 135M doc's 7.6% — compiler- and corpus-draw dependent, not the ceiling: a
sharper teacher is harder for the lossy TLA compilation to match at 20k
rehearsal scale. The headroom that matters is the 17.4 bits over a 3.79-bit
floor, which is the substrate's to close with more data.

So P2 confirms 360M as a viable, more-competent teacher and sharpens the trade
the 135M P2 recorded — strong-narrow (stories15M, 1.43-bit home floor) versus
competent-broad (SmolLM2-360M, 3.79-bit instruction floor). The ceiling lift is
real but only cashable at broad-corpus scale. **P3 (the baseline re-pin) remains
a maintainer decision** per AGENTS.md, best taken together with committing to the
broad-corpus program; the pinned stories15M fixtures are unchanged.
