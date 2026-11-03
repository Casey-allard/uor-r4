# DESIGN: Native OpenAI-compatible loopback service

## Boundary

The unit is transport, not a new model. `OpenAiServing` owns one loaded `IntegerStackModel`, tokenizer bytes, literal-role protocol version and immutable artifact identity. Each request constructs a fresh `StackChat`, encodes the complete dialogue history, preflights context and decodes greedily. The retired router is not linked and the CLI is not spawned.

## Shared generation

`StackChat::reply_history_with(history, max_new_tokens, emit)` factors the existing loop. The callback receives `(id, tokenizer_bytes)` after the ID is appended to history and before the next step. `reply_history` delegates with a no-op, preserving IDs, decoded text, state and stop behavior. EOS is reported to the callback so transport can account for it, but its text is never emitted as reply content.

The HTTP adapter owns UTF-8 buffering and emits only valid boundaries. JSON, SSE and CLI all use the same reply result; token usage counts IDs, never character estimates.

## Protocol surface

- `GET /healthz`: process/artifact readiness and sequential availability.
- `GET /v1/models` and `/v1/models/{id}`: one loaded model identity.
- `POST /v1/chat/completions`: string or text-part messages, multi-turn history, JSON or SSE.
- `POST /v1/completions`: one text prompt mapped to a user turn, JSON or SSE.
- `POST /v1/responses`: text input/instructions mapped to the same dialogue path, JSON or SSE.
- `GET /v1/capabilities`: exact routes, accepted fields, artifact hash/schema/context, greedy-only decoding, sequential limit, body limits and `vision: false`.
- Other OpenAI families and `image_url`/unknown content parts: explicit OpenAI-shaped 501. Unknown models are 404, malformed requests 400, context/body overflow 413, and concurrent generation 429.

Accepted generation fields are limited to the model identity, one choice, `max_tokens`/`max_completion_tokens`, `stream`, `stream_options.include_usage`, and `temperature: 0`. Unknown or unsupported fields fail instead of being ignored. Usage is exact prompt/completion/total IDs. Additive `uor` metrics contain integer `elapsed_ns`, `first_token_ns`, `completion_tokens`, `tokens_per_second_milli`, and the precise stop reason. No floating-point timing is fabricated.

## Loopback admission and streaming

`LoopbackConfig` accepts only `127.0.0.1`. A per-launch random token is required by bearer or `X-UOR-Studio-Token` and compared in constant time without logging. Host must equal the bound authority; Origin is absent by default or exactly the configured origin. JSON content type and one Content-Length are required for POST. Headers and bodies are bounded, chunked/ambiguous bodies refused, and generation is sequential with an atomic busy guard.

SSE writes valid `data: <json>\r\n\r\n` frames, final usage according to `stream_options`, and `data: [DONE]`. Disconnect/write failure stops the shared callback and therefore further model steps. JSON responses are non-cacheable and close the connection. Optional static serving canonicalizes paths, refuses traversal/symlinks and injects an ephemeral native-source bootstrap without persisting the launch token in Studio storage.

## Verification

Synthetic artifacts cover parity, routes, JSON/SSE identity, history, exact usage/timing, stops, malformed inputs, overflow, unsupported families, admission failures, oversized bodies, and disconnects. The existing integer/CLI/discrete/WASM suites and serving audits remain the regression boundary. A labeled local non-discrete artifact smoke run proves the shipped CLI-to-HTTP path only; it makes no useful-chat, M4/M7, production or visual-capability claim.
