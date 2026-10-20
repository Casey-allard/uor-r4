# #806 — fix(#789): close the E2 profile gaps — gated tless bypass, honest discovery, complete decline semantics

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-18T22:01:31Z
- **Updated:** 2026-10-09T13:08:58Z
- **Closed:** 2026-08-18T22:22:30Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/806
- **Merged:** 2026-08-18T22:22:30Z
- **Merge commit:** b348cdccf6eb03f93c6c309612040f8e2e208848
- **Base / Head:** main / UOR-Foundation:issue-789-impl
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #789 — implements the maintainer's G1/G2/G3 decisions (decision list + answers on the issue), flips the G4 fixtures from their pinned baselines (PR #798), and updates the two docs rows per the DoD.

- **G1 (decision (c))**: `POST /api/tless/{predict,index,generate}` is profile-gated — declined under `Production` before the body is parsed (`tless_bypass_profile_decline`); `/api/r4g1/*` deliberately stays open (the tier Production serves). Tier 2 is no longer HTTP-reachable on a default-profile server by any route.
- **G2**: `active_canonical_model_name`/`active_models`/`resolve_active_request_model`/`uor_status_json`/`R4g1CompileStatus::json` are profile-aware — under `Production` only a text-ready R4G1 graph counts as active, so teacher-only installs list nothing on `/v1/models`, get `model_not_ready` on completions, and report `engine_active: false`; `/uor/v1/status` gains a `profile` field (round-trips the file's own strings).
- **G3.1**: an explicit engine name outside the closed vocabulary (`r4g1`, four pinnable tiers, `auto`, legacy `ollama`) is a typed decline naming the requested string, on every profile (`engine_name_recognized` + `unknown_engine_decline`); persisted values stay exempt (stale `last_engine.txt` remains inert). **G3.2**: profile declines echo the requested string, not the collapsed tier constant. **G3.3**: on `/v1/*` every serving decline is a 503 OpenAI error envelope (`type: engine_declined`) — never native JSON, never the native 200-on-abstain (that contract stays on `/api/chat`, #223). **G3.4**: `generate_serving_completion` declines before any lock or router/brain mutation, matching `/api/chat`'s ordering.

Tests: the four G4 baselines flipped as deliberate diffs (the #798→here assertion pair is the record); new `g4_openai_decline_under_an_active_model_is_an_engine_declined_envelope` (text-ready fixture graph; pins the audited AUD-ARCH-008 defect end to end), `openai_decline_wrapper_never_forwards_the_native_200` (asserts the native 200 exists and is not forwarded), `unknown_engine_names_are_typed_declines_and_the_vocabulary_is_closed`, and the extended `production_profile_decline` echo case.

Gates: fmt; clippy `-D warnings` (workspace, all targets); lib 253 pass / bin 7 pass; wasm32 lib check (one pre-existing `uor-r4-model-source` warning, untouched here); BDD target compiles under `--all-targets`.

Docs: `CONFIGURATION.md` `engine_profile.txt` row rewritten to the decided semantics (also corrects the now-stale CWD-relative note superseded by #790-6); `SERVING_MODEL_DISCOVERY.md` gains the "#789: profile gaps closed" addendum.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_016X39CfJSLCU4KhjNP2XXTW


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

