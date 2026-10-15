# #789 — serving: EngineProfile completeness — per-tier endpoint bypass, discovery/serving disagreement, decline semantics, HTTP-level tests (#655-E follow-through)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-18T14:51:46Z
- **Updated:** 2026-08-18T22:34:33Z
- **Closed:** 2026-08-18T22:22:31Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/789
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem

#655-E2 landed the `Production`/`Experimental` engine profiles correctly at the cascade entry points, but the 2026-08-18 baseline audit found four gaps between E's acceptance text ("teacher-oracle/attention/geometric/dormant tiers unreachable by default, **proven by tests**") and what actually holds at `aea30bae` (findings AUD-ARCH-003/-004/-008/-009). This is E follow-through, deliberately separate from #655's remaining scope (C1e + F).

1. **Per-tier endpoint bypass.** `POST /api/tless/{predict,index,generate}` reaches the Tier-2 transformerless runtime and `POST /api/r4g1/{predict,generate}` reaches Tier 1 with **no profile check** (`src/server.rs:16249-16719`). Tier 2 is directly HTTP-reachable on a default-profile server.
2. **Discovery contradicts serving.** `active_canonical_model_name` counts **teacher** readiness as model-active (`server.rs:2662-2667`), so a teacher-only install advertises a model on `/v1/models` and `engine_active: true` on `/uor/v1/status` (`:520-534`) that the Production cascade can never serve (every completion 503s `declined_by_all`). Neither surface exposes the active profile.
3. **Decline semantics.** The typed decline echoes the tier constant, not the requested string (`tier_for_engine_name` collapses `transformerless-legacy` → `transformerless`; `production_profile_decline` `server.rs:3910-3928`); an **unrecognized** explicit engine (`"gpt-4"`, `"auto"`, `"ollama"`) is not declined — it silently runs r4g1. On the OpenAI surfaces a decline returns the **native** `declined_by_all` JSON (HTTP 200 when any tier abstained) rather than an OpenAI error envelope (`:15752-15755`, `:15895-15898`), and the OpenAI path mutates router/brain state **before** evaluating the decline while `/api/chat` declines before any side effect (`:14875-14880` vs `:15960-15966`).
4. **No HTTP-level tests.** E0 §5 asked for tier-unreachability "proven by tests" at HTTP level; all E2 coverage is unit-level on pure functions (`server.rs:26489-26682`) — nothing drives `handle_connection` with a profile fixture.

## Scope

Maintainer decisions first, then tests-first implementation:
- **G1 (decision):** is the per-tier endpoint bypass intended (documented escape hatch) or a gap to close under Production? Either way, document + test the answer.
- **G2:** make discovery agree with serving under Production (hide or mark teacher-only models; add a `profile` field to `/uor/v1/status`).
- **G3:** decline completeness — echo the requested engine string; decide the unknown-name policy (decline vs. run-cascade, but explicitly); OpenAI-envelope conformance for declines; align side-effect ordering across the two entry paths.
- **G4:** HTTP-level tests driving `handle_connection` with profile fixtures for: production default, explicit non-r4g1 decline, persisted-preference inertness, bypass endpoints (per G1's decision), teacher-only discovery (per G2).

## DoD

G1 decision recorded; G2–G4 landed with the HTTP-level tests E0 asked for; docs rows (`CONFIGURATION.md` engine_profile entry, `SERVING_MODEL_DISCOVERY.md` addendum) updated to match the decided semantics.

**Next action:** post the G1/G3 decision list for the maintainer, then write the G4 tests against current behavior as the baseline.


## Comments (2)

### comment 5332610973 — 2026-08-18T18:50:09Z — Casey-allard

## G1/G3 decision list (maintainer input needed before the semantics change; G4 baseline tests land first against current behavior)

Per this issue's next-action: the decisions, each with the honest options and what each implies. Nothing below is implemented until called; G4's baseline tests will pin **current** behavior so the eventual change is a deliberate, reviewed diff.

**G1 — per-tier endpoint bypass under `Production`** (`/api/tless/{predict,index,generate}` reaches Tier 2, `/api/r4g1/*` reaches Tier 1, no profile check):
- (a) **Close the bypass**: profile-gate the per-tier endpoints; `Production` 403/typed-declines `/api/tless/*` (Tier 2 is an Experimental tier by the #655-E terminology decision). Strongest reading of "unreachable by default, proven by tests".
- (b) **Documented escape hatch**: keep them open, document them as operator/diagnostic surfaces exempt from profile routing, and say so in `SERVING_MODEL_DISCOVERY.md` + the threat-model row. Weakens the E guarantee to "the *cascade* is profile-restricted".
- (c) Middle: gate `/api/tless/*` (cross-tier reach) but leave `/api/r4g1/*` (same tier Production already serves).
- My read: (a) or (c); (b) contradicts E's own acceptance text unless the docs explicitly re-scope it.

**G3.1 — unknown explicit engine name** (`"engine":"gpt-4"` today silently runs r4g1):
- (a) **Typed decline naming the unknown string** (matches the project's decline-over-silent-substitution convention and the #790-2 CLI precedent just landed).
- (b) Treat unknown as "no pin" (today's behavior), documented.
- My read: (a); today's behavior is exactly the silent substitution the E design doc §3 argued against.

**G3.2 — decline echo**: echo the *requested* string (e.g. `transformerless-legacy`), not the collapsed tier constant. Reads as a bug fix, not a decision — will implement with G3 unless objected.

**G3.3 — OpenAI envelope for declines**: return an OpenAI error object (with an appropriate 4xx/5xx) instead of native `declined_by_all` JSON at HTTP 200 on `/v1/*`. Conformance fix — will implement with G3 unless objected; the only choice is the status code (proposal: 503 with `"type":"engine_declined"`, mirroring the native path's 503).

**G3.4 — side-effect ordering**: align the OpenAI path with `/api/chat` (decline **before** router/brain mutation). Correctness fix — will implement with G3 unless objected.

**G2 shape** (discovery vs serving): adding a `profile` field to `/uor/v1/status` is additive and will land with G4. For teacher-only models on `/v1/models` under Production: (a) hide, (b) keep listed but mark (e.g. `"servable": false`), (c) leave the list as-is and rely on the status field. (b) preserves information without advertising an unservable model as plainly available; picking between (a)/(b) changes client-visible listings, so it waits for this list's answer.

**Sequence**: G4 baseline tests (current behavior, tests-only PR) → decisions above → G2/G3 implementation PRs (individual, serving semantics) → G4 fixtures updated to the decided semantics → docs rows per DoD.


### comment 5334920198 — 2026-08-18T22:34:32Z — Casey-allard

**Resolution — closing as completed.** All four gaps closed by PR #806 (merged to `main`), implementing the maintainer decisions recorded above; the G4 tests flipped from their #798-pinned baselines as deliberate diffs (the before/after assertion pair is the record).

- **G1 — decision (c), implemented.** `POST /api/tless/{predict,index,generate}` is profile-gated: under `Production` (including the fail-safe default) each declines with the typed 503 `declined_by_all` **before parsing the request body**; `/api/r4g1/*` deliberately stays open (the exact tier Production serves). Tier 2 is no longer HTTP-reachable on a default-profile server by any route — the E-acceptance "unreachable by default, proven by tests" now holds for the bypass surface, at HTTP level (`g4_tless_bypass_is_profile_gated_and_r4g1_stays_open`).
- **G2 — implemented.** Discovery agrees with serving: `active_canonical_model_name` (and everything downstream — `/v1/models`, `/v1/models/{id}`, the OpenAI model guard, `engine_active`) is profile-aware; under `Production` only a **text-ready R4G1 graph** counts, so a teacher-only install lists nothing, 404s on the id lookup, gets the `model_not_ready` envelope on completions, and reports `engine_active: false`. `/uor/v1/status` carries the new `"profile"` field (`"production"`/`"experimental"`, round-tripping the file's own strings); `teacher_ready` stays reported as the pipeline fact it is (`g4_status_reports_the_active_profile`).
- **G3 — implemented in full.**
  - **G3.1** (decision (a)): an explicit engine name outside the closed vocabulary (`r4g1`, the four pinnable tier names, `auto`, legacy `ollama`) is a typed decline naming the requested string, on **every** profile — the cascade never runs (empty trail). Persisted `last_engine.txt` values stay exempt (stale preferences remain silently inert, #655-E0 §3). (`g4_unknown_engine_name_is_a_typed_decline`, `unknown_engine_names_are_typed_declines_and_the_vocabulary_is_closed`.)
  - **G3.2**: declines echo the *requested* string — `transformerless-legacy` is answered as `transformerless-legacy`, never the collapsed tier constant (`g4_explicit_experimental_engine_is_profile_declined_echoing_the_request`).
  - **G3.3**: on `/v1/*` every serving decline is a **503 OpenAI error envelope** (`"type": "engine_declined"`) — never the native JSON and never the native 200-on-abstain (that #223 contract stays on `/api/chat`, where it belongs). The audited AUD-ARCH-008 defect is pinned end to end by a text-ready fixture-graph test (`g4_openai_decline_under_an_active_model_is_an_engine_declined_envelope`) plus a unit witness that the native 200 exists and is not forwarded (`openai_decline_wrapper_never_forwards_the_native_200`).
  - **G3.4**: `generate_serving_completion` now declines **before any lock or router/brain mutation**, matching `/api/chat`'s decline-first ordering (previously it evolved brain state and ran routing first).
- **G4 — the HTTP-level tests E0 asked for.** Landed first as baselines pinning pre-decision behavior (PR #798, including the merge-queue recovery), then flipped alongside the implementation. Six `g4_*` tests drive `handle_connection` over a real socket pair.

Docs per the DoD: `CONFIGURATION.md`'s `engine_profile.txt` row rewritten to the decided semantics (also correcting the CWD-relative note superseded by #790-6), and `SERVING_MODEL_DISCOVERY.md` gained the "#789: profile gaps closed" addendum.

Audit findings AUD-ARCH-003/-004/-008/-009 are resolved.



---
_Comment/review ids in this file: 5332610973, 5334920198_

