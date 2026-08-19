# F rename inventory (#655-F, mechanical half) — site map only, no behavior change

This document is the executable change list for F's mechanical half — "set
`r4` canonical across #654 OpenAI routes / native `/api/chat` / CLI / WS /
WASM; update README / RESEARCH / MODEL_LIFECYCLE / config / threat-model" —
prepared so the flip is a one-day job once F's two preconditions clear
(#741 bundle distribution; the baseline-quality canary). Per
`docs/serving_default_flip_655_f.md`, the flip itself still requires the
maintainer's explicit final go; nothing in this inventory authorizes it.

## The identity model today

Serving reports a **dynamic canonical identity**: the active bundle's
`logical_name` (falling back to the teacher-source basename, then the
literal `"uor-r4"`), with `"uor-r4"` accepted as the one intentional
request-side compatibility alias. F's end state inverts this: **`r4` is the
canonical served identity** (a product name, not a directory name), with
`uor-r4` retained as a deprecated request alias and per-bundle physical
names still visible as metadata, not identity.

Line references at `main` post-#808 (2026-08-19); re-grep before executing —
this file's tables are the checklist, the greps are the source of truth.

## A. Server — request-side alias + reported identity

| Site | Today | F change |
|---|---|---|
| `src/server.rs` `resolve_request_model_name` (`None \| Some("uor-r4") => Ok(active)`) | `uor-r4` = the one alias; `active` = dynamic logical name | Accept `r4` AND `uor-r4` (deprecated); resolve both to canonical `r4` |
| `src/server.rs` `installed_logical_model_name` final fallback `"uor-r4"` | fallback identity | fallback becomes `r4`; function's role narrows to *physical* naming for metadata |
| `active_canonical_model_name` / `active_models` / `/v1/models` id | dynamic logical name (e.g. `teacher`, bundle dir name) | canonical id `r4`; physical root/logical name stay as metadata fields (`physical_root` already exists on `/uor/v1/status`) |
| OpenAI wire ids: `chatcmpl-uor-r4-{ts}`, `resp-uor-r4-{ts}` (×2 sites), `msg-uor-r4-{ts}` (×2 sites), `system_fingerprint = uor-r4-{mode}` | `uor-r4`-prefixed | `r4`-prefixed (`chatcmpl-r4-…` etc.); note these are cosmetic ids — clients must not parse them, but the flip is the moment to change them |
| `owned_by: "uor-foundation"` (`openai_model_object`) | unchanged | unchanged (org, not model, identity) |

## B. Native `/api/chat` + WS

No independent identity constant: the native path reports whatever the
shared serving state reports (section A) plus `generation_mode`. The WS
surface likewise serves through the same state. **F work = verify-only**
(one HTTP-level test asserting the reported name is `r4` on both), no
separate rename site found by grep.

## C. CLI

| Site | Today | F change |
|---|---|---|
| `src/main.rs` `--model` `default_value = "uor-r4"` (client subcommand) + `unwrap_or("uor-r4")` | request default | default `r4` (server accepts both during deprecation) |
| `src/chat.rs` OpenAI-response parse fallbacks (`strip_prefix("uor-r4-")`, `unwrap_or("uor-r4")`) | parses today's fingerprints | accept both prefixes during deprecation |
| `src/main.rs` help text `'./uor-r4-cli'` | binary name in help | product naming decision — rename text only if the shipped binary name changes under #741 |
| `release_bundle_packager` `model_id` default `"r4"` (+ `src/main.rs` flag default) | ALREADY `r4` | no change — packaging was built F-ready |
| Persisted-engine fallback `"r4g1"` (`chat.rs` / `server.rs`) | engine/tier name, NOT model identity | **no change** — `r4g1` is a tier name; F renames the *model*, not the tier |

## D. WASM / frontends

| Site | Today | F change |
|---|---|---|
| `r4_worker.js` comment "Offloads uor-r4 candidate scoring…" | comment only | cosmetic |
| `index.html` | zero `uor-r4` identity literals | verify-only |
| Static-bundle install (#790-5) paths `./graph/score.r4g1`, `./compiled.r4g1`, `./tokenizer.bin` | file names, not identity | no change |

## E. Non-sites (deliberately out of scope)

Crate names (`uor-r4-*`), schema strings (`uor-r4-source-manifest/1`,
stage/seal schemas, cache-digest domains `uor-r4-source-cache-v2`), staging
dir `.uor-r4-source-compile-staging`, temp-dir prefixes in tests, and the
GitHub org/repo name are **infrastructure identifiers, not the served model
identity** — renaming them would churn provenance domains for zero product
effect. F does not touch them.

## F. Docs (the five named surfaces)

| Doc | `uor-r4` mentions | F change |
|---|---|---|
| `README.md` | 26 | rewrite model-identity mentions to `r4`; alias note |
| `docs/RESEARCH.md` | 9 | historical mentions stay (records); forward-looking identity lines change |
| `docs/MODEL_LIFECYCLE.md` | 45 | the big one: lifecycle examples all use `uor-r4` |
| `docs/CONFIGURATION.md` | 2 | alias + default rows |
| threat-model | **no `THREAT_MODEL.md` exists in `docs/`** | F's list references a doc this repo does not have; the flip PR either creates it (product decision) or records the absence — flag for the maintainer at flip time |
| `docs/SERVING_MODEL_DISCOVERY.md` (not on F's list) | 3 | update anyway — it documents the alias |

## Execution shape (when authorized)

One PR: server alias+identity (A) with deprecation-window tests (both
names accepted, `r4` reported), CLI defaults (C), doc rewrites (F), plus
the two verify-only tests (B, D). Estimated ≤ 1 day. Blockers unchanged:
#741, the quality canary, and the maintainer's explicit flip approval.
