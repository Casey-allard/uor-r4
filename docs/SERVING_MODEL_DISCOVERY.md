# Serving-time model discovery (#655)

> **Status:** historical point-in-time audit of the legacy loader and fallback
> surfaces. It is preserved to prevent regressions while those surfaces remain,
> but it is not current architecture authority. In particular, the listed
> cascade must not be read as a route-native, provider-free chat implementation.
> See the [Geometric Intelligence Programme](geometric_intelligence_programme.md)
> for the required source-free serving sequence.

This document maps how a running process decides *which* model/engine to
load and serve, as of `origin/main` at `146a976e` (2026-08-16). It exists
because this area had no documentation at all before now — `docs/MODEL_LIFECYCLE.md`
covers download/compile/`ask` thoroughly through its own "7. Ask or chat with
an imported manifest" section, but stops before the HTTP server's own,
separate discovery path. This gap is exactly what #655 ("ship a
ready-by-default R4 model") needs closed before a shared startup loader
(#655-C1) or a default-engine flip (#655-F) can be attempted safely.

**Headline finding: there are three independent, currently-disconnected
model-loading systems in this codebase.** A change to one does not, by
itself, change what the others do. Anyone editing "how the model loads"
needs to know which of the three they are actually touching.

1. `src/model.rs`'s `ModelStore`/`ModelManifest` — used by the CLI
   `ask`/`chat` commands.
2. `src/server.rs`'s ~27,000-line HTTP server — its own ad hoc per-engine
   "#248 cascade", used by every HTTP-serving code path.
3. `crates/uor-r4-api::release_bundle::ReleaseBundleManifest` (#655-C0,
   PR #731) — a schema-only type, not yet read or written by either of
   the above.

## 1. The `#248` cascade in `src/server.rs`

`run_serving_cascade` (`src/server.rs:3872-4007`) builds an ordered list of
tiers and calls `uor_r4_router::fallback::run_cascade(tiers)`:

```rust
const TIER_R4G1: &str = "r4g1";
const TIER_TRANSFORMERLESS: &str = "transformerless";
const TIER_TEACHER_ORACLE: &str = "teacher-oracle";
const TIER_GEOMETRIC: &str = "geometric";
```

Each tier is attempted in order; a `Failed`/`Pathological`/`Abstained`
outcome falls through to the next. The four tiers, briefly (full detail
below): **r4g1** (the packed-graph, table-native production candidate),
**transformerless** (an older, separately-pathed artifact+store+tokenizer
trio), **teacher-oracle** (the raw HF teacher model, used as a fallback
oracle), and **geometric** (an always-degrades-gracefully manifold router,
functionally the cascade's terminal "always something" tier).

### Tier 1 — `r4g1`

- Entry points: `run_server`'s startup load loop (`src/server.rs:888-1249`),
  `resolve_loadable_compiled_bundle_with_authority` (`2166-2227`); per-request
  `r4g1_tier` (`3729-3768`) → `generate_r4g1_text` (`3014-3086`).
- Discovery: explicit `--r4g1-artifact` flag, else
  `.uor-models/compiled/<logical_name>/graph/score.r4g1` (fallback
  `compiled.r4g1`) **plus** a required sibling `tless_artifacts.bin` in the
  same directory — the graph and its teacher-companion artifact must both be
  present or the whole bundle is refused (`2200-2213`).
- State type: `r4g1::R4g1State`, held in `ServingModelState.r4g1`.
- Availability: `installed.r4g1.is_some()` after startup load succeeds.
- Decode (#655 decode-default decision, 2026-08-19): **seeded weighted
  sampling is the default** on every serving surface (native `/api/chat`,
  OpenAI chat-completions and responses, and the CLI `ask`/`chat`),
  weighting the deployed step scorer's own top-K candidates through the
  identical D4 policy path — abstention behavior is decode-mode
  independent by construction, and the seed is pinned
  (`chat::DEFAULT_SAMPLE_SEED`) so identical requests reproduce
  identical completions. `temperature: 0` on the wire (or `--greedy` on
  the CLI) is the deterministic opt-out; a request `seed` field
  overrides the pinned seed; the opt-in witness envelope decodes greedy
  (witness claims bind the greedy selection). See
  `r4g1_decode_from_request` (`src/server.rs`) and
  `R4Engine::generate_sampled_into` (`crates/uor-r4-api/src/engine.rs`).

### Tier 2 — `transformerless`

- Entry point: `with_tless_server_state` (`1430-1443`) →
  `tless_uor::load_tless_state()` (`src/tless_uor.rs`).
- Discovery (`src/tless_uor.rs:424-467`): `TLESS_ARTIFACTS` env var / CLI flag,
  default `/tmp/tless_artifacts.bin`; `TLESS_STORE`/CLI, default
  `/tmp/tless_store.bin`; tokenizer via `TLESS_TOKENIZER`/CLI, else a
  hardcoded probe of
  `.uor-models/compiled/{smollm2-135m,smollm2-360m}-instruct/tokenizer.bin`,
  else `/tmp/ref/tokenizer.bin`.
- **Naming collision to be aware of:** the *filename* `tless_artifacts.bin`
  is reused by both Tier 1 (as the R4G1 bundle's required teacher-companion
  file, under `.uor-models/compiled/<name>/`) and Tier 2 (as its own primary
  artifact, default location `/tmp/`) — same basename, different directory,
  different role. Do not assume one `tless_artifacts.bin` on disk implies
  the other tier is also satisfied.
- State type: `tless_uor::TlessState { art, store, artifact_kappa,
  artifact_address, store_kappa }`.

### Tier 3 — `teacher-oracle`

- Entry points: standalone startup resolution of `startup_source_candidates`
  (`547-627`) → `prepare_optional_teacher_source_for_identity` (`3149-3197`)
  → `Teacher::load(source)`; or riding along with a successful Tier-1 load
  via the same bundle's `.uor-models/sources/<logical_name>` directory.
  Per-request: `attention_tier` (`3799-3822`) → `generate_attention_text`.
- Discovery: `.uor-models/sources/<last_model_name.txt content>`, else
  hardcoded fallbacks for the three pinned SmolLM2 sizes; expects an
  HF-style weights directory (safetensors, optionally sharded).
- State type: `uor_r4_model_source::Teacher`, held in
  `ServingModelState.oracle`.
- Availability: `oracle.as_mut().is_some()`; only attempted when the R4G1
  host encoder wasn't tagged-but-missing (`3934-3946`).

### Tier 4 — `geometric`

- Entry point: `geometric_tier` (`3830-3852`) →
  `router.generate_geometric_response_native(...)`.
- The router (`UorR4Router`) is constructed unconditionally at startup
  (`let router = Arc::new(Mutex::new(UorR4Router::new(0.85)))`, `559`) and
  optionally hydrated from `cli.manifold_cache` if that file exists — a
  missing cache file just means an empty-but-constructed router, never a
  hard "unavailable" state.
- This tier has **no file-presence gate** the way Tiers 1-3 do; it always
  runs, degrading to a typed `Abstained` outcome on sparse resonance rather
  than failing outright. It functions as the cascade's terminal
  "always-something" fallback, which is presumably why it is last.

## 2. `.uor-models/last_engine.txt`

Read in `src/server.rs` only as a fallback when an HTTP request carries no
`engine` field (`persisted_engine_preference`, `3686-3695`; consulted inside
`resolve_pinned_tier`, `3700-3721`). It stores a **plain tier-name string**
(`"r4g1"`, `"attention"`, `"geometric"`, `"transformerless-legacy"`, ...),
never a path.

Written and read primarily by `src/chat.rs` (the CLI/terminal client): read
at startup to restore the last active engine (`chat.rs:1265`, default
`"r4g1"`), written on `/engine` menu selection (`chat.rs:1550`) and
remediation-flow switches (`1208-1217`, `1900-1908`). `chat.rs` always sends
`engine` explicitly in every request body (`2168`), so for the CLI client
itself the server-side file read is redundant — it is a genuine fallback
only for other HTTP clients that omit `engine`.

**Deliberate design decision** (`src/server.rs:3699-3708`): a persisted (or
request-supplied) preference of anything other than `"r4g1"`/`"auto"`/unknown
**pins the cascade to that single tier, no fallback**. `"r4g1"` (or its
absence) never pins — it is the full cascade's own first tier, so pinning it
would lose nothing while silently disabling every fallback tier for the
common default-install case.

## 3. `ModelStore`/`ModelManifest` (`src/model.rs`) vs. the cascade

`ModelStore::from_env()` roots at the same `.uor-models` directory (env var
`UOR_MODEL_STORE`) and even shares Tier 1's `compiled/<name>/` subdirectory
naming convention — but its required-file set is a third, incompatible
contract:

```rust
// src/model.rs:430-434
fn is_compiled_bundle(path: &Path) -> bool {
    path.is_dir()
        && ["tless_artifacts.bin", "tless_store.bin", "tokenizer.bin"]
            .iter()
            .all(|name| path.join(name).is_file())
}
```

This requires `tless_artifacts.bin` + `tless_store.bin` + `tokenizer.bin`
together — the pre-R4G1 bundle shape. It never checks for
`compiled.r4g1`/`graph/score.r4g1` (Tier 1's required file), so a directory
satisfying Tier 1 does not satisfy `ModelStore`, and vice versa. `ask`/`chat`
go through `ModelStore` exclusively (`chat.rs:133-146`) and never touch
`ServingModelState`, `R4g1State`, `TlessState`, or `Teacher` directly.

## 4. `ReleaseBundleManifest` (#655-C0) fit against the four tiers

`crates/uor-r4-api/src/release_bundle.rs` models **one bundle serving one
engine kind** (`schema`, `model_id`, `capability`, `abi`, `uor_matmul`
provenance, one `components` block of `{graph, signature_artifact,
tokenizer, score_report, compile_report}` digests, `tokenizer_adapter`) — not
"a release that offers up to four alternative engines with cascade
fallback." Per tier:

| Tier | Fit today |
|---|---|
| **r4g1** | Closest fit. `components.graph` ↔ `score.r4g1`/`compiled.r4g1`; `components.signature_artifact` plausibly ↔ the teacher-companion `tless_artifacts.bin`; `score_report`/`compile_report` ↔ the R4G1 compile pipeline's own outputs. This is, not coincidentally, exactly what `crate::compile::CompiledModel` (the in-process compile pipeline's output type) produces — see §5. |
| **transformerless** | Ambiguous. No distinct field for `tless_store.bin` (the graded-code store); the schema's single graph+signature_artifact pair doesn't obviously map onto this tier's artifact+store+tokenizer trio. |
| **teacher-oracle** | Not represented. No field for an HF-style multi-file weights directory. |
| **geometric** | Not represented, and arguably shouldn't need to be — this tier is always-available and degrades gracefully rather than being an "artifact" in the same sense as the other three. |

**Conclusion carried forward from #655-C1's own prior research comment,
confirmed by this pass:** `ReleaseBundleManifest` should **not** be
generalized into a multi-tier/multi-engine schema. #655-D's own text asks
for "one minimal ... instruction-chat bundle," not a description of the
whole four-tier cascade. The schema's current one-bundle-one-engine shape is
the right shape for what D actually wants to produce and ship (almost
certainly the R4G1 tier, since that is the table-native, non-neural-hot-path
candidate this project's whole mission is built around). `ModelStore` should
not be replaced wholesale either — it is a live, working system with a
distinct purpose (content-addressed store + `QualityAttestation` gate for
`ask`/`chat`) serving a real, different bundle shape. Reconciling
`ModelStore` and `ReleaseBundleManifest` into one loader remains an open
question explicitly deferred to #655-C1/E/F, not resolved by this document.

## 5. This slice: `ReleaseBundleManifest::from_compiled_model`

`crate::compile::CompiledModel` — the in-process compile pipeline's own
output type (`crates/uor-r4-api/src/compile.rs`) — already carries exactly
the digest and ABI/contract shape `ReleaseBundleManifest.components`/`.abi`
need for the R4G1 tier: `graph`, `signature_artifact`, `tokenizer`,
`score_report`, `compile_report` digests plus `format_version`/
`contract_version`/`tokenizer_adapter`. `ReleaseBundleManifest::
from_compiled_model` is a small, pure, in-memory constructor added in this
slice that copies those fields across, so a future #655-D packaging step (or
this crate's own tests) has a checked bridge from "a completed compile" to
"a release-bundle manifest describing it," instead of hand-assembling one
field by field each time. `model_id`, `capability`, and `uor_matmul`
provenance stay caller-supplied, since `CompiledModel` carries none of them
and this schema crate should not invent application policy.

This function performs no filesystem I/O, discovers no bundle directory, and
is not wired into any server/CLI/client code path — it is additive and
dormant, per #515's convention, exactly like #655-C0 itself.

## 6. Other discovery/readiness consumers (flagged, not deep-dived)

- `/v1/models`, `/uor/v1/status` — reflect only Tier 1's installed state.
- `/api/tags`, `/api/sysinfo` — compute their own independent readiness
  booleans from Tier 1 + Tier 2 file presence, a third readiness
  computation distinct from the cascade's own tier-by-tier attempt/fallback.
- `/api/r4g1/*` and `/api/tless/*` — parallel per-tier HTTP surfaces that
  bypass the OpenAI-compatible cascade entirely.
- `select_synthesis_engine` (`3605-3616`) — a legacy single-value resolver,
  apparently superseded by `resolve_pinned_tier` but still present; worth
  checking whether anything still calls it before #655-E/F touches routing.

## Next open questions for #655-C1

1. Does the shared startup loader read `ReleaseBundleManifest` for the R4G1
   tier only, leaving Tiers 2-4 on their current ad hoc discovery? (This
   document's answer leans yes, given §4's conclusion.)
2. Does `ask`/`chat`'s `ModelStore` get pointed at the same manifest-and-
   digest-described bundle `ReleaseBundleManifest` names, or does it stay a
   fully separate mechanism? A migration path for existing `.uor-models`
   stores is required either way per #655's own compatibility requirement.
3. `#655-D` (packaging) needs to actually write a bundle directory somewhere
   before a real loader can be tested end to end — that dependency should
   probably be sequenced before the loader's file-discovery half is built,
   even though the loader's manifest-parsing half (this slice feeds that)
   can be developed and tested independently now.

## Addendum 2026-08-18 (post-E2; baseline audit)

This document's cascade description above is the **Experimental**-profile
behavior as of `146a976e`. Since #655-E2 (PR #781, merged `cd6bce6d`):

- `.uor-models/engine_profile.txt` selects `production` | `experimental`;
  absent/empty/unparseable ⇒ **`production`** (fail-safe). Under `production`,
  `run_serving_cascade` admits **only** `TIER_R4G1`; an explicit non-r4g1
  `engine` request returns a typed `declined_by_all` response, and a persisted
  non-r4g1 `last_engine.txt` preference is silently inert.
- The per-tier HTTP surfaces `POST /api/tless/*` and `POST /api/r4g1/*`
  (§6 above) are **not** filtered by the profile — Tier 2 remains directly
  reachable on a `production` server through them.
- Correction to §6: `/v1/models` and `/uor/v1/status` reflect Tier-1 **or
  Tier-3 (teacher)** readiness (`active_canonical_model_name` requires either),
  so a teacher-only install advertises an active model that the `production`
  cascade cannot serve; neither surface exposes the active profile.
- §6's `select_synthesis_engine` question is answered: its only remaining
  caller is the BDD suite (`tests/bdd.rs`) — no serving path uses it.

Full evidence: `docs/project_baseline_audit_2026_08_18.md` §8 (reachability
matrix), findings AUD-ARCH-002/-003/-004/-005.

## Addendum 2026-08-18 (#789: profile gaps closed)

The two audit gaps the previous addendum records are closed by #789, per
maintainer decisions on that issue. As of the #789 implementation PR:

- **Bypass endpoints (G1, decision (c)):** `POST /api/tless/{predict,index,
  generate}` is profile-gated — under `production` (including the fail-safe
  default) each returns the typed 503 `declined_by_all` decline *before*
  parsing the request body. `POST /api/r4g1/*` deliberately stays open: it
  reaches the exact tier `production` serves. Tier 2 is therefore no longer
  HTTP-reachable on a default-profile server by any route.
- **Discovery agrees with serving (G2):** `active_canonical_model_name` is
  profile-aware — under `production` only a **text-ready R4G1 graph** makes
  a model active, so a teacher-only install lists nothing on `/v1/models`,
  404s on `/v1/models/{id}`, answers OpenAI completions with the
  `model_not_ready` envelope, and reports `engine_active: false`. Under
  `experimental` the teacher lane still counts, exactly as before.
  `/uor/v1/status` now carries a `"profile"` field (`"production"` |
  `"experimental"`, the same strings the file parses); `teacher_ready`
  stays reported as the pipeline fact it is.
- **Decline semantics (G3):** the typed decline echoes the *requested*
  engine string (`transformerless-legacy` is answered as
  `transformerless-legacy`, G3.2); an explicit engine name outside the
  recognized vocabulary (`r4g1`, the four pinnable tier names, `auto`,
  legacy `ollama`) is a typed decline on **every** profile — never a silent
  full-cascade run (G3.1); on the OpenAI surfaces (`/v1/*`) every serving
  decline is a **503 OpenAI error envelope** (`"type": "engine_declined"`),
  never the native `declined_by_all` JSON and never the native
  200-on-abstain (G3.3); and both entry paths decline before any
  router/brain mutation (G3.4). The native `/api/chat` surface keeps its
  documented `declined_by_all` contract (200 when a tier abstained,
  #223 semantics) — the envelope rule is OpenAI-surface-only.

Proven at HTTP level by the `g4_*` tests in `src/server.rs` (they first
landed pinning the pre-decision behavior in PR #798; the flipped
assertions are the before/after record).
