# #654 — api: define and implement a pinned OpenAI-compatible R4 serving profile

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-13T00:28:27Z
- **Updated:** 2026-08-14T03:19:53Z
- **Closed:** 2026-08-14T03:19:53Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/654
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

# Context

The README currently calls the server OpenAI-compatible, but the implementation exposes only a small hand-written subset: `POST /v1/chat/completions` and `GET /v1/models`. The adjacent `/v1/status`, `/v1/reload`, and `/v1/corpus` routes are R4 lifecycle controls, not OpenAI operations.

The requested reference is the official [`openai/openai-openapi` specification](https://raw.githubusercontent.com/openai/openai-openapi/refs/heads/main/openapi.yaml). At inspection it is OpenAPI 3.1.0 / API version 2.3.0 at repository commit `11854aef674352d3f9cd5c0a7038f079a7bbac06` (`openapi.yaml` blob `b4e4080c0baf909bc6bfa293dd0efa553dfb0a29`). It is a broad platform contract, not only a text-generation schema, and it recommends the Responses API for new text-generation integrations.

# Current evidence

- `src/server.rs` defines a local chat DTO with optional `model`, string-only `messages`, `max_tokens`, `temperature`, and the R4-specific `engine` field. Messages are flattened into one prompt.
- The current response always emits one assistant choice with `finish_reason: "stop"`; usage is estimated by whitespace-delimited words, not the selected artifact tokenizer.
- Streaming/SSE, `POST /v1/responses`, `POST /v1/completions`, embeddings, `GET /v1/models/{model}`, standard error objects, model/capability validation, and most request fields are absent.
- Roles/content are narrower than the specification: no developer/tool messages, content-part unions, tool calls, structured output, or multimodal input. Unsupported fields can be ignored by Serde instead of rejected, which makes the compatibility claim ambiguous.
- `/v1/models` returns only `id` and `object`; the pinned `Model` schema also requires `created` and `owned_by`.
- Error bodies are commonly `{ "error": "..." }`, rather than the OpenAI error envelope with `message`, `type`, `param`, and `code`. D4 abstention and runtime-unavailable states also need a defined mapping.
- #194 unified R4's native `InferenceRequest`/`InferenceResponse`; #248 unified the serving cascade. Neither issue defined OpenAI wire conformance.

# Problem

We do not have a pinned definition of what “OpenAI-compatible” means for the R4 model. The official specification currently contains many platform operations that do not apply to a local text model, while even our claimed text routes differ in required fields, validation, errors, usage accounting, streaming, and response shapes. Without a versioned profile and executable conformance matrix, clients cannot know what is supported and spec drift cannot be detected.

# Goal

Define a truthful, pinned OpenAI serving profile for the R4 model and implement it closely enough that official OpenAI SDKs can use the supported text-generation and model-discovery operations through a local `base_url`. Every operation and field outside the profile must be explicitly classified and rejected or marked not applicable—never silently approximated.

# Scope

- Vendor or otherwise pin the exact upstream OpenAPI revision used for conformance. Record repository commit, file digest, OpenAPI/API version, license, retrieval/update procedure, and an intentional spec-bump review process.
- Generate a machine-readable compatibility matrix over the pinned specification. Classify every operation as `supported`, `unsupported`, or `not-applicable`, with R4 capability rationale, owning issue, and planned phase. Do the same at field/variant level for supported operations.
- Define the first R4 text-serving profile around `POST /v1/responses`, `POST /v1/chat/completions`, `GET /v1/models`, and `GET /v1/models/{model}`. Make an explicit, recorded decision on legacy `POST /v1/completions`; do not imply support by omission.
- Introduce spec-derived or mechanically checked OpenAI wire DTOs and adapters over the existing engine/cascade boundary. Keep the native #194 DTOs as the R4 API; do not relabel them as OpenAI schemas.
- Implement non-streaming and SSE streaming forms for the selected generation operations, including correct event/chunk ordering, terminal markers, disconnect handling, and honest partial/failed outcomes.
- Implement required-field and capability validation, model-not-found behavior, supported roles/text content parts, parameter handling, finish reasons/status, and standard error envelope/status mapping. Unsupported parameters and content types fail explicitly with a stable error; they are never accepted and ignored.
- Count prompt/output usage with the selected model tokenizer and state the exact accounting policy. If a count is unavailable, report that state honestly instead of substituting word counts.
- Keep UOR audit/witness/cascade data behind documented namespaced extensions or native `/api/*` routes so the base OpenAI schema remains interoperable.
- Separate lifecycle/control routes from the compatibility surface. Preserve legacy aliases only with an explicit migration/deprecation record; document `/api/*` as R4-native.
- Define the security profile: local loopback behavior, bearer-token handling, CORS, request/body/time limits, and the requirement for explicit authentication before non-loopback exposure.
- Update README/client help/API docs so “OpenAI-compatible” always names the exact pinned R4 profile and unsupported surface.

# Non-goals

- Do not implement the entire OpenAI hosted platform (Assistants, Files, Fine-tuning, Batch, Images, Audio, organization/admin APIs, etc.) merely to make a blanket compatibility claim.
- Do not claim semantic equivalence to OpenAI models or support for tools, modalities, structured output, storage, background execution, or log probabilities until those capabilities exist and pass the profile tests.
- Do not let the wire adapter choose or silently fall back to experimental engines; serving policy remains owned by #248 and #655.
- Do not move OpenAI-specific schemas into the no-std runtime kernel.
- Do not fetch a mutable specification at build or serving time.

# Dependencies and blockers

- Builds on #194's native typed request/response boundary and #248's single cascade/typed abstention semantics.
- Reuse #601 tokenizer identity/parity and #266 witness verification rather than adding competing owners.
- Coordinate model ID, packaged artifact readiness, and the default production engine with #655.
- A spec bump is not dependency drift: it requires a reviewed matrix diff, compatibility decision, generated fixture update, and an era/version note for the R4 profile.

# Proposed implementation shape

Keep a small versioned `r4-openai-profile` artifact containing the upstream pin, selected operations, field policies, and generated golden schemas. Route both Chat Completions and Responses through one internal generation request/result adapter so #248's outcome is mapped once. Maintain official-SDK smoke fixtures beside direct HTTP schema fixtures; keep R4 extensions out of the base response unless explicitly namespaced.

# Acceptance criteria

- [ ] A pinned, versioned compatibility matrix covers every operation in the selected upstream specification and every field/variant of the supported R4 profile; CI detects unreviewed drift.
- [ ] Official Python and JavaScript OpenAI clients work against a local `base_url` for model list/retrieve, Chat Completions, Responses, and both supported streaming modes.
- [ ] Golden/schema tests cover non-stream and SSE request/response/error shapes, developer + user text messages, length and stop terminals, malformed JSON, missing/unknown model, unsupported parameter/content, D4 abstention, runtime unavailable, and client disconnect.
- [ ] `/v1/models` advertises only loadable models with all required fields; `GET /v1/models/{model}` agrees with the list and model capabilities.
- [ ] Token usage comes from the selected artifact tokenizer, and finish reason/response status reflects stop, length, abstention, incomplete, or failure truthfully.
- [ ] Unsupported parameters and endpoints return a standard structured error and are never silently ignored.
- [ ] R4 audit/witness data remains available through a documented namespaced extension or native route without breaking official SDK parsing.
- [ ] Native `/api/*` behavior and #248 cascade semantics do not regress, and lifecycle/control routes are no longer presented as OpenAI-standard operations.
- [ ] README and API documentation name the exact profile/spec pin and replace the blanket compatibility claim.

# Verification

- Contract/golden tests generated from the pinned profile for request, response, model, error, and SSE event schemas.
- End-to-end official Python and JavaScript SDK smoke tests against an ephemeral local server, including streaming cancellation and standard exception parsing.
- Differential tests proving Chat Completions and Responses map the same internal engine outcome without changing cascade or witness semantics.
- Negative tests for every accepted-but-unsupported field family so Serde defaults cannot silently widen support.
- Workspace fmt/clippy/tests, graph-format no-std checks, claim-wording, repo-conformance, and the relevant wasm/integration gates.

# Compatibility and migration

Publish this as an explicitly versioned R4 compatibility profile. Existing native `/api/*` clients remain supported. Legacy `/v1` control aliases receive a documented deprecation or compatibility plan; no route disappears without migration notice. New spec pins create reviewed profile revisions rather than silently changing existing behavior.

# Claim status

Definition and implementation of a bounded wire-compatibility guarantee against a pinned specification profile. Passing the profile proves the tested protocol contract only; it is not full OpenAI-platform parity, model-behavior equivalence, or a quality claim.

## Comments (1)

### comment 5289010056 — 2026-08-14T03:19:52Z — Casey-allard

Closing — the pinned OpenAI-compatible R4 serving profile is complete. All slices landed on main:

- **A** pinned spec + compatibility matrix — #678
- **B** truthful DTOs + error envelope + /v1/models{,/{model}} — #680
- **C** chat conformance: tokenizer usage + truthful finish_reason + roles — #682
- **D** SSE streaming for chat completions — #684
- **E** POST /v1/responses via the shared generation adapter — #686
- **F** official OpenAI Python/JS SDK smoke tests + README — #688
- **G** R4 extended capabilities under /uor/v1 (keeping /v1 OpenAI-pure) — #690
- **+** streaming for /v1/responses (Responses event protocol) — #692

The OpenAI surface (`/v1/chat/completions` and `/v1/responses`, both non-streaming and streaming; `/v1/models`, `/v1/models/{model}`) is verified against the official SDKs, with R4's proprietary capabilities cleanly namespaced under `/uor/v1`. Passing the profile proves the pinned wire-protocol contract only — not platform parity or a quality claim.


---
_Comment/review ids in this file: 5289010056_

