# Frozen R4/Spin quality-capacity continuation (#1017)

- **Status:** `FULL_QUALITY_DOD_FAILED_NLL_ONLY / ATTENTION_RETAINED`.
- **Owner:** #1017 under attention issue #973 and programme root #820.
- **Predecessor:** [#1014](r4_softmax_end_to_end_attention_1014.md).
- **Machine-readable aggregate:**
  [`r4_softmax_quality_capacity_continuation_1017_raw.json`](r4_softmax_quality_capacity_continuation_1017_raw.json).
- **Binding local evidence root:** `.uor-models/research/issue-1017/`
  (ignored bulk population, checkpoints, parity, one-time reveal, generation,
  and replay reports).

## Result and decision

The one frozen #1017 continuation completed its exact 7,324 optimizer steps and
reached `149,995,520` cumulative training tokens without changing #1014's
7,155,360-parameter architecture, tokenizer, causal-softmax mechanism, Rust
execution path, sampler, or split discipline. Selection remained sealed and
chose continuation step 7,324 at development NLL
`1.580241072373312`.

The enabled-only Rust admission gate passed before reveal. Python and Rust
selected the same top-1 token, maximum absolute logit delta was
`0.0000057220458984375` against the frozen `0.005` ceiling, all six learned
layers executed through coherent R4/Spin transport, and the audit recorded zero
future, provider, Ollama, or prior-trace reads. The Rust qualification decision
CID is
`blake3:5bc79e85fb8427152bcc87a2dd63ff08a0f9a453e5d0ae63161a0b32792312d6`;
the bound admission-manifest CID is
`blake3:f0a6a4fc396c919d8f848c9cd7da9cfedecc6ece51b83582fb019f6ab85c8909`.

The fresh sealed confirmation tranche was then opened exactly once. Its NLL was
`1.5727521962806827`, which fails the strict `< 1.50` quality criterion. The
reveal manifest CID is
`blake3:599497e087a3662e805498ca5a04085034e25e5a5e26850b2c4e8441a96d75e8`.
This is an absolute improvement of `0.5546550809359942` nats/token, or
`26.071880399960784%`, from #1014's enabled NLL `2.127407277216677`.
All five fixed Rust continuations passed the frozen subject-or-scene rubric,
all five normalized reload replays were exact, and every remaining mechanical,
causal, R4/Spin, decode-integrity, and external-closure gate passed.

The create-once final result CID is
`blake3:0dbd94a279b6cd898e0d69667dcc1457c9dddba6061e6ff1d8c222c93793d46c`;
the final manifest CID is
`blake3:a53870f17837592cad7ce87d83abe7315c12181bccf0026e8035424fda783bfa`.

The overall result is therefore negative solely because sealed-confirmation NLL
missed the frozen ceiling. There is no rerun, learning-rate adjustment, seed
change, additional 7.15M-parameter exposure extension, or reinterpretation of
the five-generation positive. #1014 already established load-bearing ordinary
causal attention at this learned R4/Spin scope; #1017 preserves that conclusion
and supplies a strong bounded language-quality result, but it does not satisfy
the complete quality Definition of Done.

The next action is one new, predeclared parameter-capacity issue over the same
qualified attention and Rust evidence path. It must change model capacity rather
than continue training this 7,155,360-parameter checkpoint or tune its learning
rate. External training hardware is permitted only if the predeclared model and
wall-clock budget require it. Intrinsic/readout substitutions, resonance-based
softmax replacement, recurrent lowering, exact deployment, product promotion,
correctness, and reasoning remain parked.

## Frozen inheritance and continuation

The continuation inherited the exact #1014 selected checkpoint and AdamW state:

- vocabulary 4,096; width 288; six layers; six Q heads and six KV heads;
- head width 48, exactly twelve R4 blocks per head;
- FFN width 768; context 256; tied embedding and language-model head;
- RMSNorm, RoPE, SwiGLU, learned Q/K/V/O, complete-prefix scaled dot product,
  stable causal softmax, and weighted-value aggregation;
- batch 16, gradient accumulation 4, inherited AdamW betas, weight decay,
  clipping, tokenizer, story split, Rust sampler, and all-layer R4/Spin path.

The fresh deterministic population added `119,996,416` training tokens after
the original 30M cap, used the next disjoint 250,000 development tokens, and
sealed the next disjoint 250,000 confirmation token IDs before training. The
continuation finished inside the frozen 5 h 15 min MPS ceiling. No sweep,
alternate schedule, architecture change, CPU fallback, attention-off arm,
comparison model, or prior sealed-test reuse was permitted.

## Frozen gate ledger

| Criterion | Frozen rule | Observed | Verdict |
|---|---:|---:|---|
| Cumulative training exposure | exactly `149,995,520` tokens | `149,995,520` | **PASS** |
| Development selection | minimum complete-development NLL before reveal | step 7,324; `1.580241072373312` | **PASS** |
| Enabled Python/Rust top-1 | identical | identical | **PASS** |
| Enabled Python/Rust maximum logit delta | `< 0.005` | `0.0000057220458984375` | **PASS** |
| Learned R4/Spin layers | all six | six | **PASS** |
| Fresh sealed-confirmation NLL | `< 1.50` | `1.5727521962806827` | **FAIL** |
| Prompt subject or scene | at least `4/5` | `5/5` | **PASS** |
| Decode integrity | valid UTF-8; no period-1..4 loop | all five | **PASS** |
| Normalized reload replay | exact `5/5` | exact `5/5` | **PASS** |
| Causal and external closure | zero future/provider/Ollama/prior-trace reads | zero | **PASS** |

The attention-off policy was intentionally not executed: #1017 was a
quality-capacity continuation, not another attention intervention. The sole
failed row is sealed NLL.

## Claim boundary

Training and qualification used floating point, multiplication, allocation,
autograd, Apple Metal, dense full-prefix dot products, and ordinary softmax.
This result does not establish advantage for geometry over ordinary
coordinates, transformerlessness, multiplication-free or table-native
execution, dependable general-purpose generation, inference, correctness,
reasoning, chat, browser-WASM operation, release readiness, or frontier
capability. Five bounded rubric passes are evidence for those five frozen
prompts only.

## Owner direction and local M1 inference update — 2026-08-31

The historical next-action paragraph above is superseded. #1019 is optional
and paused after its MPS training projection and one slower fused-optimizer
probe; it does not block use of this #1017 checkpoint. The active product path
is `r4 generate --prompt "..."`. Broad qualification remains deferred until
the prototype delivers useful behavior.

The repository's existing Apple Accelerate `cblas_sgemv`/`cblas_sgemm` path
was exercised on the project M1 rather than rejected by static analysis. For
the same prompt `Once upon a time`, greedy four-token exact and Accelerate runs
both produced IDs `[14, 403, 285, 261]`, decoded `, there was a`, output CID
`blake3:ad043d419e9a3f30cc9be75d6a84f519d988e370a7288f6455763afe6257818e`,
and attention-audit CID
`blake3:1552cef6effdb28b3a4b5e1a29313a90beef7df0494ff7230040389c01d4fd78`.
Exact `uor-matmul` required `3.060506042 s` of recorded generation and `3.41 s`
wall time; Apple Accelerate required `0.116236875 s` and `0.52 s`, respectively:
`26.33x` faster inside generation and `6.56x` end to end. Decision and
persistent-state CIDs differ intentionally because they bind truthful backend
and execution provenance.

The local CPU-BLAS build is:

```bash
cargo build --release --offline --features local-inference-accelerate --bin r4
target/release/r4 generate --prompt "Once upon a time"
```

This keeps exact `uor-matmul` as the portable default and uses Apple
Accelerate only when explicitly requested for local source-backed inference.

## #1041 normal-use product-boundary follow-up — 2026-09-01

The deferred behavior decision above is complete. #1039 exposed this exact
checkpoint through a disabled-by-default loopback raw-completion surface, and
#1041 exercised the native dashboard plus seven frozen direct requests. All
mechanical conditions passed. Fresh narrative continuation passed `2/3`, while
neither supplied-history binding comparison passed. Terminal
`KEEP_RAW_CONTINUATION_ONLY` retains the checkpoint as a bounded source-backed
single-turn story-continuation reference; it does not authorize a history
serializer, multi-turn/chat adapter, retraining, or prompt widening. See the
[#1041 record](r4_softmax_local_normal_use_1041.md).
