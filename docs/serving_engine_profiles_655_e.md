# Engine profiles design (#655-E)

This document scopes #655-E ("Production vs Experimental engine
profiles; migrate `.uor-models/last_engine.txt`"), the last unstarted
serving sub-track once C0-C1d and D0-D3 landed. It is a design and
decision record only — no server/CLI code path changes here, matching
this project's established convention of scoping a multi-system change
before touching it (mirrors #655-C1b/D0's own shape).

## What E's own text says

The original A-F decomposition on #655 defines E in one paragraph:

> **E. Production vs Experimental engine profiles + persisted-preference
> migration.** `Production` profile = only the audited table-native
> transformerless tier; `Experimental` = the current #248 cascade;
> scope/migrate `.uor-models/last_engine.txt`; ordinary requests route
> only through the approved lane; teacher-oracle/attention/geometric/
> dormant tiers unreachable by default (proven by tests).

## Terminology resolved: "Production" = r4g1 (Tier 1)

A research pass into this codebase found a real ambiguity: "the audited
table-native transformerless tier" collides with the literal name of
cascade **Tier 2** (`transformerless`, `src/server.rs:3734`) — an older,
separately-pathed, explicitly-unaudited system per
`docs/SERVING_MODEL_DISCOVERY.md`. Internal evidence pointed the other
way: `docs/matrix_operation_census.md`'s own audit scope
(model-source → compiler → certifier → bundle → graph/TLA runtime → API
adapters) and `docs/inference_contract.md:11` ("the production R⁴/R4G1
**transformerless** graph inference engine") both use "transformerless"
as this project's overall non-neural-mission adjective, applied to
**Tier 1 (`r4g1`)** — not as a pointer to Tier 2's literal name. E's own
unreachable-by-default list (teacher-oracle/attention/geometric/dormant)
conspicuously omits `r4g1`, which only makes sense if `r4g1` is the
surviving "Production" lane.

**Confirmed with Casey (2026-08-18): Production = `r4g1` specifically.**
Tier 2 (`transformerless`), teacher-oracle, `attention`/`r4-attention`,
and `geometric` are all "Experimental" — reachable only when the
Experimental profile is active. This resolves the only genuinely
ambiguous product-facing question in E's own spec; everything below is
implementation-shape design, decided here the same way #655-C1b/D0
decided theirs.

## Current mechanics (what E has to build on top of)

**`.uor-models/last_engine.txt`** (`persisted_engine_preference`,
`src/server.rs:3767-3774`): a plain-text tier-name string
(`r4g1`/`attention`/`r4-attention`/`geometric`/`transformerless`/
`transformerless-legacy`), read fresh from disk on **every** request
that omits an explicit `engine` field — not cached, no admin/reload
gate. Written by the CLI's `/engine` menu and remediation flows
(`src/chat.rs:1370-1378`, `:1712`, `:2049-2052`).

**`resolve_pinned_tier`** (`src/server.rs:3790-3801`): resolves the
request's explicit `engine` field, falling back to the persisted
preference when absent. Returns `Some(tier)` to pin the cascade to
exactly that tier, or `None` to run the full cascade. One deliberate
exception: `"r4g1"` never pins (it's already the cascade's first tier,
and treating it as a pin would silently disable every fallback for
default installs that persisted it). Called from both HTTP entry points
that run the cascade: the native `/api/chat` path
(`src/server.rs:14724`) and the OpenAI-compatible chat-completions path
(`src/server.rs:15789`).

**`run_serving_cascade`** (`src/server.rs:3953-4092`): builds a
`Vec<(tier_name, TierFn)>` gated by an `include(tier)` closure
(`pinned.is_none() || pinned == Some(tier)`) plus the fixed cascade
order r4g1 → transformerless → teacher-oracle → geometric (pinned
`attention`/`r4-attention` substitute for teacher-oracle). `run_cascade`
tries each in order; first success serves. A run where **no** tier
serves already has an honest typed terminal — `declined_by_all`
(`src/server.rs:4142` area) — not a placeholder string. This existing
mechanism is exactly what E needs for "unreachable by default": it
already handles "nothing served" honestly; E only needs to control
which tiers ever enter the `tiers` vec.

## Design decisions for E1+

### 1. A new, separate persisted profile, not a `last_engine.txt` rewrite

"Migrate `.uor-models/last_engine.txt`" does not mean changing that
file's format. `last_engine.txt` answers "which tier, when the client
doesn't say" — a UI/session preference. `EngineProfile` answers a
different, orthogonal question: "which tiers are allowed to exist at
all, regardless of what's requested." Conflating them would make an
operator's existing `last_engine.txt="attention"` file look like a
format error instead of a now-inapplicable preference.

Add `.uor-models/engine_profile.txt`, a plain-text `"production"` or
`"experimental"` string, mirroring `persisted_engine_preference`'s exact
shape and read pattern (`engine_profile_preference() -> Option<String>`
— read fresh per request, no caching, no admin/reload gate, consistent
with how `last_engine.txt` already behaves and for the same reason: an
operator can flip profiles without a server restart). Absent or
unparseable content defaults to `production` (fail-safe: an empty or
corrupt file must never silently grant broader reach than intended).

### 2. Enforcement point: filter the `tiers` vec itself, reuse `declined_by_all`

Under `production`, `run_serving_cascade`'s tier-construction gains one
more condition alongside `include(tier)`: only `TIER_R4G1` may ever
enter the `tiers` vec. Transformerless, teacher-oracle, `attention`,
`r4-attention`, and `geometric` are never attempted — not skipped after
an r4g1 failure, never entered at all. If r4g1 doesn't serve (abstains,
is pathological, fails, or isn't loaded), the cascade already has the
correct honest answer: `declined_by_all`. **No new terminal state is
needed** — this is exactly the existing D4-policy "typed decline over
hard failure" convention this project already applies everywhere else
(#223, #655-C1c/D3's own "advisory, never silently broadens or narrows
what already works" framing).

### 3. An explicit non-r4g1 `engine` request under `production`: typed decline, not silent substitution

E's text draws a line between "ordinary requests" (no explicit `engine`
— ergonomically just get r4g1) and the other tiers being "unreachable."
It does not explicitly say what happens when a client *actively asks*
for `engine: "geometric"` while `production` is active. Two honest
options: silently serve r4g1 anyway (surprising — the client asked for
one thing and got another without being told), or return a clear
declined/error response naming the reason. Recommend the latter: it
matches this project's own stated preference for typed decline over
silent substitution, and it is what lets `production`'s tier-visibility
promise ("unreachable by default") be verified by a real HTTP-level
test rather than only by an internal unit test on `run_serving_cascade`.
The persisted (non-explicit) preference case is different and should
stay silent: an operator's stale `last_engine.txt="attention"` under a
newly-`production` server should not error every "ordinary" request —
it should simply not pin, exactly as `"r4g1"` already doesn't pin today.

### 4. Nothing here changes what happens when `experimental` is active

`experimental` reproduces today's exact behavior: the full #248 cascade,
persisted-preference pinning, pinned `attention`/`r4-attention` — byte
for byte what every existing install already does. `production` is the
new, additive restriction; `experimental` is not a new mode requiring
new code, only the "else" branch of the same `if profile == production`
check.

### 5. Docs to update alongside the code

`docs/CONFIGURATION.md:171` (the `last_engine.txt` table entry) and
`docs/SERVING_MODEL_DISCOVERY.md` need a new row/section for
`engine_profile.txt` once E2 lands, explaining the two files answer
different questions (§1 above) and that a persisted non-r4g1 preference
under `production` is silently inert rather than an error, while an
explicit non-r4g1 request is a typed decline (§3).

## Proposed slice sequence

- **E0** (this document): design/decision record, no code.
- **E1**: additive, dormant infrastructure — an `EngineProfile` enum
  (`Production`/`Experimental`), `engine_profile_preference()` (mirrors
  `persisted_engine_preference`), and unit tests against synthetic
  `.uor-models/engine_profile.txt` fixtures. Nothing calls it from
  `run_serving_cascade` yet — zero behavior change for any existing
  install, matching the additive-and-dormant shape #655-C0/C1c/D1 each
  used before their own wiring slice. Safe to land under the standing
  "same cadence" authorization, no check-in needed.
- **E2**: wire the profile into `run_serving_cascade`'s tier
  construction (§2) and the explicit-request decline path (§3), at both
  call sites (`server.rs:14724`, `:15789`), plus the doc updates (§5)
  and HTTP-level + unit tests proving tier unreachability "by tests" per
  E's own acceptance text. **This is the slice that actually changes
  default runtime behavior for any install without an
  `engine_profile.txt`** (defaulting to `production`, i.e. restricting
  reachable tiers) — even though Casey's own A-F decomposition scoped
  this under E rather than F, its practical effect (most installs'
  request-routing behavior changes silently on upgrade) sits close
  enough to #655-F's "flip default" territory that E2 should get an
  explicit Casey confirmation before merge, not just a design-doc
  mention, out of caution for the standing rule that no default-serving
  behavior change ships without sign-off. E1 carries no such risk (it
  is inert) and needs no check-in.

## Non-goals for E0-E2

Does not touch #655-F (which model/engine is recommended/installed by
default at the CLI/setup level — a different question from E's runtime
tier-reachability restriction). Does not change `resolve_pinned_tier`'s
existing `"r4g1"`-never-pins exception. Does not migrate or rewrite
`last_engine.txt`'s file format. Does not add profile awareness to the
CLI's local (non-HTTP) chat path — scoped to the two HTTP cascade entry
points E's own text names ("ordinary requests"). Does not change
`ModelStore`/CLI `ask`/`chat` behavior (that's #655-C1e's territory, a
separate deferred issue).
