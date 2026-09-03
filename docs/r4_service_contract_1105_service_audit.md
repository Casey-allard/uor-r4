# Service ownership source notes — #1105 under #1084

Source-only engineering audit at `85d9eb8beca2a59ccda47e290afd483f7838982c`; the isolated checkout matched that revision. No implementation, build, executable launch, model/artifact load, qualification call, forward, replay, or sealed input access occurred. This review concerns service ownership and does not reuse the auditor's earlier supervisor authorship as independent evidence.

GitNexus was queried (`uor-r4-07ec3f0d-1094-retained`, indexed September 3 with no last-commit value). Its results were stale retrieval hints and did not locate the newly added learned-reference seam. All conclusions below come from the exact current files. Serena/Rust language-server startup was unnecessary for this bounded read-only audit.

## Existing seams to reuse

| Exact source | Present interface and constraint |
|---|---|
| `crates/uor-r4-api/src/learned_reference.rs:2`; API `Cargo.toml:8` | Feature-gated re-export of `uor_r4_core::learned_reference::*`. This is the existing reusable model boundary; `learned-reference` is not a default feature. No new model implementation or crate family is needed. |
| `crates/uor-r4-core/src/learned_reference/mod.rs:73` | `RawRequest { schema: &str, text: &[u8] }` preserves complete bytes. `ModelToken`/`TextResult` retain the accepted schemas. Host transport decoding is explicitly outside the core. |
| Core `loader.rs:14`; core `mod.rs:149` | `ExpectedBinding`; `LoadedResearchReference::load(Vec<u8>, &ExpectedBinding)` and `load_audited` own immutable artifact bytes and decoded state. Loading performs no source discovery, download, Python call or inference. The host acquires bytes and supplies trust. |
| Core `mod.rs:167,181,225` | `capability()`, `qualify(&mut self, receipt, trusted_digest, runtime)`, `answer(&self, RawRequest)` are the ordinary host seam. Qualification must finish before sharing an engine; `answer` checks qualification before inspecting a request. The successful ordinary path is explicitly NOT_RUN in the #1102 handoff. |
| Core `mod.rs:98,105,232` | `RuntimeIdentity` is caller-supplied. `ComparisonAdmission::from_trusted_release`/`compare` are the separate research path, not a substitute serving authorization. |
| Core `mod.rs:257`; `environment.rs:6` | An atomic busy guard rejects overlapping computation per engine. `answer` is synchronous, with no cancellation or progress parameter. FPCR is checked on the actual arithmetic thread before/after valid computation; the profile presently supports aarch64/macOS and refuses unsupported modes/platforms. |
| `src/server.rs:741,773,1396` | Existing private reservation/loopback/origin helpers provide useful ownership patterns. They are private legacy-server functions, not directly reusable public API exports. |
| `src/server.rs:6192,6284` | Existing private capped no-follow file readers check one opened regular handle, exact length/EOF and metadata identity. Their behavior is a useful artifact-acquisition pattern; the new host must explicitly own any small extraction rather than import the whole serving stack. |

## Why the measured CLI is not a service worker

`crates/uor-r4-api/src/bin/r4-learned-reference-1102.rs:72` supports only `metadata`, `gate`, and `run`. It hashes its own executable, requires the #1102 release schema/issue and four denied-access probes, and uses `ComparisonAdmission` plus `compare()`. Its IPC uses harness-only lowercase hex, emits full tensors/diagnostics, caps 336 requests, and requires exactly 320 valid plus 16 refusal rows at EOF (`:154–213`). It never installs the accepted qualification through `qualify()` and has no ordinary serving mode.

Keeping that executable as evidence is correct. Spawning its consumed `run` mode for user traffic is not authorized; feeding one request and accepting its output would still end in its incomplete-population error. Adding a serving mode changes the binary and creates the same new-host identity/qualification obligation. Preserve the CLI and its receipts unchanged.

A new opt-in Rust host with one internal worker mode of the **same new executable** is the cleaner selected boundary. It is one local HTTP service plus one owned computation process, not a second service or a second numerical implementation. The worker reuses the library's `load`/`qualify`/`answer` path. Treat its service lifecycle and ordinary-path behavior as unverified until a separately admitted implementation/result decision establishes them.

## Cancellation and lifecycle implications

An in-process call to `answer()` cannot be interrupted through today's API. Dropping an HTTP response/future or frontend `AbortController` does not stop its scalar arithmetic. Such a host would have to advertise compute cancellation as unsupported or merely pending until the call finishes.

With the proposed one-child design, parent state serializes admission, completion and cancellation. If completion commits first, cancellation reports the existing completed terminal. If cancellation commits first, suppress subsequent worker output, terminate that owned child and wait for confirmed reaping. Keep the single model/job slot occupied while stopping; report `cancelled` only after computation has ceased. A failed or unconfirmed reap remains stopping/error and cannot admit another worker. Specify the bounded termination/escalation policy in the ADR; do not inherit the old server's unrelated port-owner kill behavior.

A killed worker loses its engine. Any replacement must reverify the same configured artifact and host qualification, load a fresh engine and acknowledge readiness before accepting another request. Parent snapshots must distinguish job terminal state from model readiness during this replacement. Give jobs and worker generations stable IDs so stale results cannot complete a later job. Snapshot polling does not require token streaming or a numerical progress hook.

Do not claim a cancelled dispatched job used zero forwards: work may already have begun. Record unknown/partial attempted work honestly, bounded by the one-request operation; cancellation before dispatch is distinguishable. Retained job results serve status polling only, never an answer cache for a new model request.

Progress can name host-observed states and actual artifact bytes read. The loader and scalar call expose no percentage/ETA callback. Loading/qualifying/running remain indeterminate phases; cancellation becomes complete only after reaping. Freeze process-wide one-job admission and bounded request/result framing rather than relying solely on the core's per-engine busy flag.

## Artifact and qualification ownership

The host/operator owns the artifact locator, accepted `ExpectedBinding`, runtime/release records and trusted qualification digest. Public requests cannot choose paths, alternate artifacts, weights, frames, roles, segmentation, runtime flags or qualification. The worker owns the resulting `LoadedResearchReference` for its lifetime; request facts do not become conversation state.

Reuse the accepted artifact bytes and **retain their original export-release binding**. A new host binary does not justify re-exporting or rewriting the manifest to the new host revision. Keep original comparison evidence read-only; any managed installation/copy must bind the exact artifact hash and preserve original evidence. Initial scope needs no download, compile, purge, workspace or model-discovery route.

The current handoff binds artifact SHA `2c209590a64cae16a4140fd43adc1cb1f87b357c02e3d4959f1e37f4ab8cd5ab`, measured CLI SHA `d423d8d3c3acd2d1c6215c21206e1bec7583e4dd37e84f30f70f79e77c40d53f`, runtime receipt SHA `daba1ad4bf60d28def983378a6a856e0990d7eab20d2ec2365552ad07f3d83d2`, and qualification SHA `61d29aa80e6bcd3d163b2ff2a6da4faab04414ea9f4284d80b798c4e46cf5369`.

`qualify()` compares a strict twelve-field receipt against supplied identities; it does not discover the executing image or interpret the referenced acceptance/comparison documents. The service must verify its **actual new executable**, execution mode/runtime and accepted evidence before supplying those trust values. Copying the CLI hash into `RuntimeIdentity` can satisfy caller comparisons while falsely identifying the host; the ADR must explicitly prohibit this. Same source code, same artifact and same executable for parent/worker do not themselves establish the new host path's empirical qualification.

Until a separately accepted new-host binding exists, expose historical #1102 reference evidence but report the new host operation unavailable/unqualified. Its future acceptance must cover the worker's ordinary `qualify`/`answer` path and the selected transport/lifecycle behavior. Keep the core's exact qualification/result schemas intact; outer host discovery can separately identify service/worker runtime, lifecycle and historical evidence. Numerical preservation and HTTP/cancellation acceptance remain distinct claims.

## Exact legacy-server mismatches and missing ADR decisions

`src/main.rs:142,3132` routes `serve` to `server::run_server`. `src/server.rs:1600` initializes the router and discovers prior teacher/compiled state; later startup indexes local reading material and writes the manifold cache. `:2371` offers interactive termination of another process holding the port, and `:2422` spawns one thread per connection. This is not the minimal one-artifact/one-job host.

`handle_connection` (`:16424`) is a hand-written parser with general wildcard OPTIONS, repeated Content-Length assignment and a body allocation bounded only for the two old reference endpoints. The old reference request is `{prompt,max_tokens}`, with method/origin handling and old model-generation semantics. The static fallback (`:18993`) reads a cwd-relative path without the proposed explicit asset root. These are concrete reasons to isolate a small new host boundary; they are not requests to rewrite the legacy server during this ADR.

`docs/integration/frontend-port-plan.md` already proposes one Rust owner, same-origin discovery, explicit execution identity, truthful lifecycle and an explicit static asset root. Its first-flow reference to #1017 raw continuation is historical: this ADR's selected operation is only `answer_four_fact_raw_text/v1`, with the original known vocabulary/query/four-fact scope. Do not advertise `/v1/chat/completions`, generation, a model token stream, or donor browser-teacher fallback as this operation.

The ADR still needs exact proposed module/binary and route ownership; lossless raw-byte HTTP/IPC encoding and strict limits; admission/refusal/transport error precedence; public discovery/result/job snapshot schemas; one worker/job slot; deterministic completion-versus-cancel/restart races; finite retention of job snapshots; read-only artifact acquisition and launch trust; and a separate host-bound qualification/acceptance successor. These are specification choices, not authority to implement or run them now.

## Exact inspected source hashes

- `src/server.rs` — SHA256 `86b66222d75ba8320af782b6aa46001052b7dd511d2fb3bdc572a6f7c23f05ea` (1295150 bytes).
- `src/main.rs` — SHA256 `3cfdff7c7bcad4c578db198f16ab1d4914df8723211314d1d71ff83be8f67577` (221274 bytes).
- `crates/uor-r4-api/Cargo.toml` — SHA256 `bf5061338e5cfbe83231111929bd12ad8ee1b8d179e3812cab99ca32e0f151a5` (1986 bytes).
- `crates/uor-r4-api/src/learned_reference.rs` — SHA256 `87926c7df0f5493221385377282586f17eeae59cd8db5f4d92ccb12181b57d98` (117 bytes).
- `crates/uor-r4-api/src/bin/r4-learned-reference-1102.rs` — SHA256 `ba10ca22b6124ce5a551424e4793cb1a3f324b2857ed20c58542d4e01aa43457` (8983 bytes).
- `crates/uor-r4-core/src/learned_reference/mod.rs` — SHA256 `2ead8e40b9f3f095a057c5dec0c1d36bb38f0dd7ea61fc7ca7b5da7924401cec` (13528 bytes).
- `crates/uor-r4-core/src/learned_reference/loader.rs` — SHA256 `208fbe0a2447a650063909cc623e9c9c3804a839a4c0ba99ab39c79e8b26ec07` (22959 bytes).
- `crates/uor-r4-core/src/learned_reference/environment.rs` — SHA256 `307341a98cb20719604681e528966c20d8b622f107d29939fae6f4f02ff07bf0` (1477 bytes).
- `docs/integration/frontend-port-plan.md` — SHA256 `6cecd10abb9f41b892e1eb083174546cb85a8a5be014995d1d5c2ab0ed6309f6` (11483 bytes).
- `docs/r4_native_bridge_1102_evidence/qualification-handoff.json` — SHA256 `efc02551d493e255f12680ccf2e4ee99cca5f645e0ca3d7fcd6445419e963426` (5001 bytes).
- `docs/r4_native_bridge_1102_evidence/qualification.json` — SHA256 `61d29aa80e6bcd3d163b2ff2a6da4faab04414ea9f4284d80b798c4e46cf5369` (857 bytes).

## Export provenance versus fresh host execution permission

Core `compare()` (`mod.rs:232`) requires `admission.release_sha256` to equal the artifact manifest's immutable `export_provenance.release_sha256`. `ComparisonAdmission::from_trusted_release` verifies the supplied bytes/hash and identity syntax; it does not independently establish a newly permitted host experiment. These are distinct obligations.

A future private candidate-comparison mode in the **new host executable** must first validate a fresh, separately accepted host execution release naming that executable/runtime, exact artifact and inputs, bounds and outputs. It may then verify the retained original export-release bytes solely to satisfy the existing artifact-provenance comparison seam. The old export SHA stays an identity anchor and provides no renewed execution authority. Keep both identities explicit: original export provenance and fresh host-execution permission. Do not invoke the consumed #1102 CLI/coordinator or rewrite the artifact manifest to make the new permission digest fit.

Private candidate verification calls `compare()` only under that new independent gate. Public serving calls only the ordinary `qualify()` then `answer()` path. No provisional or fabricated qualification receipt may expose an unqualified candidate. After a successful candidate result is independently accepted, an exact new-host qualification can be constructed and used for separately declared ordinary-path/HTTP/lifecycle acceptance on the unchanged binary. Numerical preservation and ordinary HTTP/cancellation acceptance are separate gates; neither can borrow the other's status. This is a required future contract distinction, not an execution release or a new API implementation in #1105.
