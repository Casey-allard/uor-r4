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

---

# P3 — Broad-corpus program (#509): SmolLM2-360M observed on Simple-Wiki

P2 above ended on the open trade: strong-narrow (stories15M, 1.43-bit home
floor) versus competent-broad (SmolLM2-360M, 3.79-bit instruction floor), with
the ceiling lift "only cashable at broad-corpus scale." Issue #509 is the P3
decider for exactly that claim. It asks a single causal question: **if the only
thing that changes is teacher breadth — a competent-broad teacher observed on
broad text instead of a narrow one — does the substrate's broad-text accuracy
move off the floor?**

## Why the question is live

The narrow teacher is near-degenerate off its home distribution. Measured, and
already recorded in `RESEARCH.md` (#320): the stories15M argmax is
`6.4%` next==argmax on wiki10k versus `70.2%` on its home corpus, and the legacy
teacher-hash store carrying that evidence serves at `~0.1%` off-distribution —
below the unigram null. Every prior broad-text substrate attempt inherited that
degeneracy from the teacher, not from the geometry. #509 replaces the teacher
and holds the corpus family fixed.

## What was run

A single end-to-end broad-text pass, teacher → substrate → Gate C, all native
(no GPU, AVX2/FMA CPU matmul in the teacher loader):

| stage | command | result |
|---|---|---|
| observe | `r4 transformerless observe-text` over Simple-Wiki (3,000 articles) | 21,235 observation records |
| convert | `obs_bundle_to_corpus.py` | `corpus.meta` / `corpus.records` |
| compile | `r4 compile` (recorded, `--vocab-size 49152`) | `tless_artifacts.bin`, `tless_store.bin` |
| cover | `r4 transformerless cover` | 46 regions across 3 depths |
| score | `r4 transformerless score --quality-profile relative_tla` | Gate C below |
| floor | `r4 evaluate-report --sequence-length 128` | teacher floor below |

Corpus: `simple-wiki-20231101-sample` — `wikimedia/wikipedia`, config
`20231101.simple`, rows `0..2999`, the D3 natural partition
(`docs/transformerless/BASELINE.md §2`), CC-BY-SA-4.0. Teacher:
`HuggingFaceTB/SmolLM2-360M-Instruct` rev `2366112999e525164f9f74a3fbf50ec19b48b940`.

## Result (Gate C, 4,358 held-out D3 positions; EXCT-miss 62.5%)

The teacher floor on this Simple-Wiki partition is **3.6015 bits/token**
(`instruction-eval.json`; 21,235 replayed positions, story-contiguous — lower
than the 3.79-bit instruction floor, i.e. the 360M is a touch sharper on
narrative wiki prose than on its instruction corpus). The evaluate-report
table-native artifact itself reached 17.3% top-1 / 25.4% teacher-argmax
agreement at 14.6954 Witten–Bell bits/token — **+11.09 bits over the floor**,
the same compiler-bound signature P2 recorded. The scored-graph substrate rows
(held-out, full population unless a live slice is named):

| scorer | top-1 | bits/token | note |
|---|---|---|---|
| legacy teacher-hash sum | 0.07% | 72.46 | the off-distribution baseline this program had to beat |
| Rule 1 (graph chain) | 0.32% | 13.51 | exact-context only |
| **Rule 1+2 (D4 precedence)** | **10.19%** | **12.72** | ±0.46pp — the causal, quotable generation number |
| Rule 1+2, cloud-size-normalized | 13.47% | 12.08 | best plain full-population variant |
| TLA3 store baseline | 25.70% | 14.70 | same-corpus table-native reference |
| **Rule 1+2 + LATENT-MIX (#446 M2)** | **29.0%** | **11.71** | left-key-only at serving — causally legitimate |
| Rule 1+2 + latent ORACLE-RIGHT | 15.9% | 12.39 | upper bound only, NOT causal |
| Rule 1+2 + latent SHUF-CLASS | 21.5% | 12.12 | null control the exit rule must beat |

Cover held-out routing recall: depth-1 reference top-1 `85.5%` / top-M `95.2%`.
Latent headroom: baseline `10.2%` → latent-mix `29.0%` → oracle-right `15.9%` =
`331.2%` of available top-1 headroom; the #446 exit rule (≥ 2.0pp over baseline
AND beats shuffled-class) is **MET (positive)**.

## Decision — POSITIVE

Holding the corpus fixed and swapping only the teacher moves broad-text held-out
top-1 from the documented `~0.1%` off-distribution floor of the narrow teacher to
**10.2% (Rule 1+2, causal) and 29.0% (latent-mix, causal)**. Teacher breadth was
the binding constraint on broad-text substrate accuracy, exactly as #320
predicted; a competent-broad teacher lifts the substrate off the floor. This is a
`~100–290×` movement on the causal rows and clears the unigram null and the
shuffled-class control.

Scope and honesty bounds, so the number is not over-read:

- This is not a byte-identical A/B against a stories15M-on-Simple-Wiki run; the
  narrow-teacher figure is the recorded off-distribution serving number, not a
  re-run under this exact harness. The attribution rests on the teacher being the
  only changed input plus the measured narrow-teacher degeneracy above.
- `10.2%`/`29.0%` are absolute broad-text held-out accuracies, still far below
  the teacher floor — the substrate is compiler- and data-bound, not saturated.
  The result licenses the broad-corpus program; it does not claim the substrate
  matches the teacher.
- The `29.0%` latent-mix row is quotable as a generation number because it reads
  the left key only at serving (#446 M2); ORACLE-RIGHT (`15.9%`) is an upper
  bound and is not.

P3 therefore returns a positive verdict on the broad-corpus direction: it is
warranted, and the #320 baseline re-pin — still a maintainer decision per
AGENTS.md — is now backed by a measured broad-text lift rather than only a
ceiling argument.

## Provenance (κ)

| artifact | blake3 (24) | bytes |
|---|---|---|
| teacher `model.safetensors` | `eb23c3e8527110b83c091f86` | 723,674,912 |
| `tless_artifacts.bin` | `9710cd9959c2e9e8d5bd6df8` | 1,415,444 |
| `tless_store.bin` | `27f856de73c4a4ed5cecef57` | 2,891,964 |
| `corpus.records` | `41fa6286a869d41202197482` | 1,868,680 |
| `graph/score.r4g1` | `7dab1537ab1e34e76f50505b` | 3,317,436 |
| `graph-cover/cover.r4g1` | `c04bf3901b0d730a1959c324` | 75,196 |

Compiled bundle at `.uor-models/compiled/wiki-360m/` (untracked, regenerable
from the commands above).
