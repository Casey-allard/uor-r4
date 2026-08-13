# `r4-openai-profile` — pinned OpenAI wire-compatibility profile (#654 phase A)

This directory pins the exact OpenAI REST specification the R4 server claims
wire-compatibility against, and a machine-readable compatibility matrix that
classifies **every** operation in that specification. It is the anchor for the
phased #654 work: no wire DTO, route, or "OpenAI-compatible" claim elsewhere in
the repo may reference anything but this pin.

Phase A (this slice) is **definition only** — it vendors the spec, records the
pin, generates the matrix, and adds a CI drift gate. It changes no server route
and adds no wire DTO; those are later phases (B: truthful DTOs + error
envelope; C: chat-completions conformance + usage; D: SSE; E: `/v1/responses`;
F: official-SDK smoke tests; G: control-route separation).

## The pin

| Field | Value |
| --- | --- |
| Upstream repo | `openai/openai-openapi` |
| Commit | `11854aef674352d3f9cd5c0a7038f079a7bbac06` |
| `openapi.yaml` git blob sha1 | `b4e4080c0baf909bc6bfa293dd0efa553dfb0a29` |
| `openapi.yaml` blake3 | `blake3:396df55705eaca49b0f87c606a150443c4c0bd291efc3347cb8497f11d6e60f6` |
| OpenAPI version | 3.1.0 |
| API version | 2.3.0 |
| License | MIT |

`openapi.yaml` is vendored here byte-for-byte (do **not** fetch a mutable
specification at build or serving time). The same values are recorded
machine-readably in `compatibility_matrix.json` under `spec_pin`.

## The compatibility matrix

`compatibility_matrix.json` classifies each of the specification's 288
operations as one of:

- **`supported`** — implemented (or committed to be implemented) by the R4
  profile. Phase 1 is exactly four text-serving operations:
  `createChatCompletion` (`POST /chat/completions`), `createResponse`
  (`POST /responses`), `listModels` (`GET /models`), and `retrieveModel`
  (`GET /models/{model}`).
- **`unsupported`** — a text/inference-adjacent operation deliberately **out**
  of the phase-1 profile (legacy `POST /completions`, embeddings, the stored /
  stateful chat-completion and response management operations, and their
  `?beta=true` variants). Recorded so support is never implied by omission; the
  server must reject these explicitly, never accept-and-ignore.
- **`not-applicable`** — an OpenAI platform operation outside a local R4 text
  model (assistants, files, fine-tuning, batch, images, audio, moderations,
  vector stores, realtime, admin, …). See the #654 non-goals.

Current counts: **4 supported, 20 unsupported, 264 not-applicable, 288 total.**

## Regenerating (reviewed spec bump only)

A spec bump is **not** dependency drift. To move the pin:

1. Fetch the new revision and re-vendor it:
   `curl -sSL https://raw.githubusercontent.com/openai/openai-openapi/<commit>/openapi.yaml -o profiles/openai/openapi.yaml`
2. Update `SPEC_PIN` in `scripts/gen_openai_compat_matrix.py` (commit, git blob
   sha1, blake3, OpenAPI/API versions, license) and regenerate:
   `python3 scripts/gen_openai_compat_matrix.py`
3. **Review the matrix diff.** Any newly-added operation defaults to
   `not-applicable`; promote it to `supported`/`unsupported` only with a
   rationale and an owning phase. Removed operations must disappear from the
   matrix.
4. Record the change as a reviewed profile revision (bump `profile_version`)
   with an era/version note — never a silent behavior change.

The drift gate `tests/openai_profile_pin.rs` fails CI if the vendored spec and
the matrix disagree, if the vendored bytes do not match the pinned blake3, or if
the `supported` set changes without updating the test — so drift cannot land
unreviewed.

## Claim status

Passing the profile proves the tested wire-protocol contract against this
pinned specification only. It is **not** OpenAI-platform parity, model-behavior
equivalence, or a quality claim.
