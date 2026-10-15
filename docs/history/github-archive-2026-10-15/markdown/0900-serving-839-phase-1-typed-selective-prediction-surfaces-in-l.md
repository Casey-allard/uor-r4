# #900 — serving(#839): phase 1 — typed selective-prediction surfaces in legacy-coverage mode (RF-30)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-21T12:55:54Z
- **Updated:** 2026-10-09T14:16:56Z
- **Closed:** 2026-08-21T14:08:45Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/900
- **Merged:** 2026-08-21T14:08:45Z
- **Merge commit:** c205eb7b7063f4b2097a094c9e28b766da2c2f07
- **Base / Head:** main / UOR-Foundation:issue-839-typed-abstention-surfaces
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

Executes **phase 1** of #839 per the 2026-08-21 maintainer re-scope: the #838 frozen typed selective-prediction contract's **legacy-coverage mode** is lowered onto every production surface. Abstentions are now typed end-to-end (outcome `abstention`, cause `distributionally-novel` — the only legal legacy cause), the OpenAI-compatible surface migrates the generic `engine_declined` envelope to the typed `uor_abstention_*` / `uor_incompatible_artifact` codes with the spec-§5 streaming terminal-event rule, the WASM boundary gains a typed export, and present-but-unexecutable selective-calibration data fails closed as `hard-incompatibility` on every surface.

## Built-capability order (followed)

`model/ids.toml` **RF-30** (scope `normative-runtime`, reachability `deployed-serving`) → `features/suites/selective_prediction_surfaces.feature` (6 tagged scenarios) → marker test `selective_prediction_surfaces_rf_30` + behavior tests → implementation → regenerated `CONFORMANCE.md` (30 ids, CM-01 green). The re-scope's "first non-research-leaf S2 item" judgment call: this DOES create a new served capability, so the order applies.

## Changes

- `src/selective.rs` (new): shared, dependency-free typed vocabulary (canonical kebab labels, snake rewrite, D4→coverage mapping, `selective_calibration.bin` constant) — ungated so WASM and native read identical labels.
- `src/chat.rs`: `ChatAbstention` gains typed `outcome`/`cause`/`coverage` fields; fail-closed calibration seam on both CLI bundle-load paths (typed `ChatError::SelectiveCalibrationPresent`, nonzero exit); abstention stays exit 0.
- `src/main.rs`: CLI renders the typed abstention record.
- `src/server.rs`: native served + declined bodies carry the additive §5 `selective` block (confidence/evidence always `null` in legacy mode — never fabricated; coverage `null` when the D4 signal is absent — absence is absence); `/api/chat` and the shared OpenAI core gain the 409 fail-closed calibration gate; abstained cascades on `/v1/*` return **422** `error.type="uor_selective_prediction"`, `error.code="uor_abstention_distributionally_novel"` (+ coverage/cause); `stream: true` typed declines emit one terminal typed SSE `error` event then `[DONE]` (chat completions) or a terminal typed error event (`/v1/responses`) — never a bare JSON body on a streaming wire, never a silent end; non-abstention declines keep their historical envelopes (`engine_declined` remains the non-abstention terminal).
- `src/tless_uor.rs` + `src/lib.rs`: `typed_r4g1_response` — the typed WASM boundary (served / hard-incompatibility with reason; never a trap; the abstention arm is declared vocabulary, reachable when the policy engine reaches this path — recorded limitation). Legacy `Option<String>` export unchanged.
- Register: `model/ids.toml` RF-30, feature suite, marker test, regenerated `CONFORMANCE.md`.

## Scope and non-goals

No calibrated-mode execution (phase 2, gated on the frozen false-answer UCB95 ≤ 10‰ @ coverage ≥ 20‰ release bar — unchanged); no evidence/confidence fields in legacy mode; no D4 policy-semantics change (served output byte-identical on Serve); `hamming_calibration.json` (compiler-side conversion input) untouched.

## Acceptance-criteria status (phase-1 slice)

- Typed, deterministic, injective per-surface mapping of the three outcomes: DONE (unit + round-trip assertions; legacy envelopes never mistaken for typed ones).
- Fail-closed on present/corrupt calibration: DONE (`selective_calibration.bin` probe on server + both CLI load paths; present never degrades to legacy).
- Abstention never hidden: DONE (never empty-`choices`, never silent stream end; native 200-on-abstain preserved with the typed block).
- P-4 / allocation / no_std / determinism: untouched paths; `status_policy` (14) + `status_policy_census` green.
- Calibrated risk/coverage certification: NOT in this phase (phase 2, gated).

## Tests and verification

fmt; clippy `-D warnings` all-targets; full lib suite (270 passed) incl. 5 new behavior tests + migrated doc on the decline-wrapper test; `r4` bin suite (8) incl. the typed CLI render test; `status_policy` + `status_policy_census`; wasm32 target build; `xtask check-model` (CM-01, 30 ids) + regenerated `CONFORMANCE.md`; repo-conformance R2/R3 + RF-30 marker; `xtask validate` (R1/R4/R5/gnaf); claim-wording gate. The pre-existing wasm-target `tokenizer_path` dead-code warning predates this PR and is unchanged.

## Compatibility and migration

All native additions are additive (`selective` block; existing `outcome`/`r4g1`/`status` fields untouched). The OpenAI abstention envelope intentionally changes from 503 `engine_declined` to the typed 422 — this is the migration the spec's §5 names ("the current `engine_declined` error body is the migration ancestor of the typed `uor_abstention_*` codes; #839 executes the migration"); non-abstention declines are unchanged. Old artifacts keep their documented legacy decision behavior and are explicitly uncalibrated.

References #839


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

