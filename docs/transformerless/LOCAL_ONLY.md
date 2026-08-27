# Local-only inference contract

> **Preserved runtime reference:** this contract scopes the existing
> CID-verified TLA/TLS serving lane. It does not establish the unfinished
> route-native geometric decoder or working source-free chat. The current
> architecture and promotion order live in the
> [Geometric Intelligence Programme](../geometric_intelligence_programme.md).

Inference attributed to this preserved transformerless lane is local under the
artifact and execution boundaries below.

## Execution and memory contract

The preserved transformerless inference path is CPU-only. There are no `metal`,
`cuda`, or generic GPU Cargo features and no `--device` option. The portable
invocation is:

```bash
cargo run -- ask "why is the sky blue?"
```

The inference hot path uses fixed-size stack storage or buffers supplied by
the caller. In particular, prompt tokenization, rolling-window generation,
prediction witnesses, history retention, and detokenization use the bounded
`*_into` APIs. Model loading, offline compilation, CID storage, HTTP JSON, and
the final owned response are outside that hot-path guarantee and may retain
owned storage.

## Hard boundary

The `ask`, `chat`, and HTTP synthesis paths may load only CID-verified TLA/TLS
model bundles and execute them through R4's integrated `transformerless` module. They must not:

- call hosted AI APIs;
- connect to an external inference server;
- spawn Ollama, llama.cpp, or another transformer runtime;
- silently fall back to a source transformer when an artifact is missing.

Downloaded open weights are compiler inputs. The teacher is permitted only in
the offline compilation and evaluation workflow; it is not shipped or invoked
by the production answer path.

## Evidence required for claims

Speed and quality are separate gates. A release bundle must include:

1. source-model revision and license;
2. UOR CIDs for source, artifact, store, tokenizer, and evaluation report;
3. same-machine throughput results with warmup and run counts;
4. teacher agreement and grounded-answer evaluation;
5. repetition and refusal rates;
6. proof that the measured answer path used the transformerless runtime.

The CLI rejects continuation-only bundles and instruction bundles without a
CID-addressed passing evaluation report. This prevents throughput results from
being presented as chat-quality results.
