# Default-flip design (#655-F)

This document scopes #655-F ("flip the public default + docs"), the
final and most sensitive serving sub-track, now that A/B/C (except the
separately-deferred C1e)/D/E are all complete. It is a design and
decision record only — no server/CLI/doc content changes here beyond
this file, matching this project's established convention of scoping a
multi-system, product-facing change before touching it (mirrors
#655-C1b/D0/E0's own shape). **Casey explicitly authorized proceeding
with F and, when asked how, chose "scoping doc first" over an immediate
flip** — this document is that scoping pass, and its central finding is
that F's own written preconditions are not yet met on the merits,
independent of the sign-off process Casey has already granted.

## What F's own text says

The original A-F decomposition on #655 (Casey-allard, 2026-08-14,
issuecomment-5292634081) defines F in one paragraph:

> **F. Flip the public default + docs (final; product-facing → explicit
> Casey sign-off).** Set `r4` canonical across #654 OpenAI routes /
> native `/api/chat` / CLI / WS / WASM; update README / RESEARCH /
> MODEL_LIFECYCLE / config / threat-model. Only after every
> arithmetic-ownership + bundle + P-4 gate is green. I'll bring the
> default flip back to Casey for explicit approval rather than flipping
> autonomously.

The issue body's own **Activation contract** section is more concrete
about the gate ("Only after every ... gate is green" above), and is
just as load-bearing as the decomposition paragraph:

> promotion rule: flip the public default only after every production
> matrix operation is eliminated or uor-matmul-owned, dependency-source
> P-4 audit is green, the bundle validates, both #654 request modes
> work, and the in-domain canary serves from the audited transformerless
> tier.
>
> if positive: publish the bundle/profile, set `r4` as the canonical
> model, and move non-table tiers behind explicit experimental
> selection.

The issue body's **Goal** section states the same requirement from the
other direction:

> The server must be ready without teacher weights, an external
> provider, a manual multi-hour compile, or an engine selector.

And the **Scope** section makes the distribution mechanism explicit:

> Distribute that bundle as part of the supported release/install
> artifact or a packaging-time asset step. First request must not
> silently download teacher weights, contact OpenAI/Ollama/Hugging
> Face, or launch a compile.

And an acceptance-criteria bullet:

> - [ ] At least one pinned in-domain release canary produces a valid
>   assistant completion; Novel/Contradictory/OOD probes retain honest
>   D4 abstention and do not trigger an implicit experimental fallback.

So F, read in full, is not "flip a config value." It is: **(1)** rename
the canonical model identity to `r4` across every serving surface named,
**(2)** update the five named docs, **and** **(3)** only do so once a
real, distributed bundle exists and produces non-degenerate output on a
release canary. (1)+(2) are mechanical; (3) is the actual gate, and this
document's finding is that (3) is not satisfied today.

## What this session's own design docs already say about F

`docs/serving_engine_profiles_655_e.md` (E0), under "Non-goals for
E0-E2":

> Does not touch #655-F (which model/engine is recommended/installed by
> default at the CLI/setup level — a different question from E's
> runtime tier-reachability restriction).

`docs/serving_shared_loader_655_c1.md:122` and
`docs/serving_release_packaging_655_d.md:153` both independently state,
near-identically: "Does not change the default engine (#655-F)." Every
prior sub-track has deliberately routed around F rather than
encroaching on it — this document is the first one to look at F
directly.

## Current state: two separate "default" questions, and where each stands

F's text conflates two axes that #655-E and #655-D actually kept
separate. Worth being precise about both before proposing anything.

### 1. Tier reachability (built by E; already effectively resolved)

As of #655-E2 (PR #781, merged `cd6bce6d`), a fresh install with no
`.uor-models/engine_profile.txt` already fails safe to the `Production`
profile, under which `run_serving_cascade` only ever admits `TIER_R4G1`
(`EngineProfile::from_persisted`, `profile_restricts_to_r4g1`,
`tier_admitted` — `src/server.rs`). In that narrow sense, "only the
audited table-native tier is reachable by default" — part of what F's
activation contract asks for ("move non-table tiers behind explicit
experimental selection") — is **already true today**, independent of
F itself. E deliberately built this without waiting for F, per E0's own
non-goals note.

### 2. Canonical model identity (F's actual remaining question)

This is the part E and D explicitly left alone. Today:

- The OpenAI-compatible surface's one intentional compatibility alias is
  the literal string `"uor-r4"` (`src/server.rs:2671-2678`, comment:
  "`uor-r4` is the one intentional compatibility alias").
- The release-bundle packaging CLI already defaults to `model_id: "r4"`
  (`src/main.rs:345`, `src/release_bundle_packager.rs:210`) — but that's
  a packaging-time manifest field, not a change to what any serving
  surface reports as canonical.
- Nothing writes a default `.uor-models/last_engine.txt` or
  `engine_profile.txt` on a fresh install; the CLI's in-memory fallback
  when the file is absent is the literal string `"r4g1"`
  (`src/chat.rs:1427-1435`), mirroring the server's own absent-file
  behavior.

F's "set `r4` canonical" will need to reconcile the existing `uor-r4`
alias with the `r4` identity already used elsewhere, across every
surface it names (OpenAI routes, native `/api/chat`, CLI, WS, WASM).
This is mechanical once undertaken, but has not been scoped in detail
here, because of the two blockers below — there is no point precisely
scoping a rename that can't ship yet.

## The two preconditions F's own text requires, and why neither is met today

### Precondition 1 — a distributed default bundle (unmet; explicitly deferred as #741)

F's Scope section requires the bundle to be "part of the supported
release/install artifact" so "first request must not silently ...
launch a compile." Today, `.uor-models/` is entirely gitignored
(`.gitignore:18,29`) except a corpora manifest; no compiled `.r4g1`
graph, store, or tokenizer artifact ships with the repo or a release.
#655-D (fully complete, D0-D3) built the **packaging mechanism** —
`package_release_bundle`, the CLI subcommand, the sidecar manifest and
its verifier — for a bundle that already exists on a local disk. It
deliberately did not solve **distribution**: how those bytes reach a
fresh clone or release artifact in the first place. That question was
spun out as its own issue and explicitly deferred:

> #741 — the "what does shipping a product actually look like" question
> (CLI vs WASM frontend vs both, bundle distribution mechanism) that
> #655-D's orientation surfaced. Deferred, explicitly left open/unworked
> until the model-quality work lands.
>
> — Casey-allard, 2026-08-16T20:24:03Z

#741 (https://github.com/UOR-Foundation/uor-r4/issues/741) remains open
and is itself marked "deferred" in its own title. It has not been
touched this session and is explicitly out of scope for this document
per the standing instruction not to work Casey-deferred issues without
being asked.

### Precondition 2 — the in-domain quality canary (unmet at baseline, per the most recent recorded assessment)

F's activation contract requires "the in-domain canary serves from the
audited transformerless tier" and produces "a valid assistant
completion." The most recent status consolidation on #655 itself
assessed this directly:

> F (default-engine flip): explicitly gated on maintainer sign-off
> regardless of the other slices' status ... Also, substantively: given
> #762's own measurement still shows word-salad/degenerate output on
> the best locally available bundle at baseline, flipping the
> production default now would be premature on the merits, independent
> of the process gate.
>
> — Casey-allard, 2026-08-17T14:02:23Z

Note the nuance: issues #759 ("distinct prompts converge onto the same
high-confidence n-gram trajectory") and #762 ("measure whether
context-window resolution or sampling decoding reduce the #759
attractor-basin collapse") both closed on 2026-08-17, with #762 shipping
an **opt-in** weighted-sampling mitigation that took a 15-prompt
collapse measurement from 13/15 to 15/15 distinct outputs. That is real
progress, but it is opt-in, not the default. The quoted assessment
above — "word-salad/degenerate output ... at baseline" — was written
*after* #762 closed, and evaluates default (not opt-in-flagged)
behavior. No comment on #655 since then (through E0/E1/E2 landing,
2026-08-18) has revisited this assessment. Absent a fresh measurement
showing baseline quality now clears the bar (or a decision to make
#762's mitigation the default before or as part of F), this document
treats precondition 2 as still open.

## Decision: F's literal flip does not proceed from this document

Casey's "proceed with F" authorized starting F's own process (this
scoping pass). It is not read here as also having resolved, on the
merits, whether bundle distribution and generation quality are ready —
those are separate, substantive engineering questions this document
surfaces but does not have standing to answer by itself, since F's own
text ties the flip to gates ("only after every ... gate is green") that
are independent of the sign-off process. Doing the actual rename now,
against F's own unmet preconditions, would produce a canonical `r4`
identity with nothing real behind it for a fresh install (no bundle) and
degraded output at baseline even when a bundle is compiled locally
(quality). That is a worse outcome than not flipping, and is exactly
what #655's own title ("ship a ready-by-default R4 model") is trying to
avoid.

## Proposed sequence

- **F0** (this document): scoping/decision record. No behavior change,
  no doc rewrites beyond this file.
- **Blocked on precondition 1**: #741 (bundle distribution) needs its
  own resolution — out of scope here, explicitly Casey-deferred, not
  restarted by this document.
- **Blocked on precondition 2**: a fresh, explicit quality measurement
  (or a decision to default-enable #762's opt-in mitigation) showing the
  in-domain canary clears F's "valid assistant completion" bar at
  baseline, not just under an opt-in flag.
- **Once both preconditions clear**: the actual identity-rename slice
  (reconciling the `uor-r4` alias with `r4` across OpenAI routes, native
  `/api/chat`, CLI, WS, WASM, plus the five named docs) still needs its
  own explicit Casey confirmation before implementation — the same
  "bring it back to Casey for explicit approval rather than flipping
  autonomously" language from F's own decomposition text applies
  in full at that point, regardless of this document or today's
  "proceed with F" authorization. Preconditions clearing does not by
  itself re-authorize the flip; that requires Casey looking at the
  cleared preconditions directly and saying so.

## Non-goals for F0

Does not rename or alias any model identity. Does not modify README,
RESEARCH, MODEL_LIFECYCLE, config docs, or the threat-model doc — doing
so now would describe an end state ("r4 is canonical") that isn't true
yet. Does not touch #741 or the quality/collapse issues (#759/#762 and
siblings) — those are their own already-tracked, already-decided work.
Does not change `run_serving_cascade`, `resolve_pinned_tier`, or any
`EngineProfile` mechanics landed in E — F operates one layer up, on
canonical identity, not tier reachability, and E's own work already
stands on its own regardless of F's timeline.

## Addendum 2026-08-19 — the flip executed

Both preconditions cleared and the maintainer gave the fresh explicit
approval this document required, so the flip shipped:

- **Precondition 1 (distributed bundle): met.** Release v0.1 is
  published with the packaged `smollm2-360m-broad` bundle and attested
  manifest as GitHub Release assets; `r4 install-release --tag v0.1` is
  the explicit hard-verified fetch, proven end to end against the live
  release (#741 closed; `docs/RELEASE_PIPELINE.md`).
- **Precondition 2 (baseline-quality canary): met.** The sampled decode
  became the serving/CLI default (#813/#814) and the default path is
  byte-identical to the measured 15/15-valid canary arm; the F-p2
  acceptance probes now pass through the same deployed D4 policy on the
  CLI as on the server (#811/PR #816). Distinctness remains open as
  #784 and is disclosed on the release.
- **Authorization**: maintainer decisions of 2026-08-19 (~01:50Z,
  recorded #655 issuecomment-5336515423) — flip authorized, sequenced
  after the release publish and #811; both held before this change.

What shipped (per `docs/serving_rename_inventory_655_f.md`, section by
section): `CANONICAL_MODEL_ID = "r4"` served on every surface with
`uor-r4` retained as the deprecated request alias;
`active_canonical_model_name` reports the canonical id (physical names
stay metadata); OpenAI wire ids and `system_fingerprint` are
`r4`-prefixed; CLI `client --model` defaults to `r4` and the remote
client parses both fingerprint prefixes for the deprecation window;
README/CONFIGURATION carry the identity contract. Deliberate
exclusions, unchanged from the inventory: crate names, schema strings,
staging dirs, the `uor-r4-cli` orchestrator script file name, and the
four source-verification logical-name fallbacks (infrastructure
identifiers, not served identity). **The threat-model doc F's list
names does not exist in this repository**; creating one remains a
separate product decision, recorded here rather than improvised at
flip time.
