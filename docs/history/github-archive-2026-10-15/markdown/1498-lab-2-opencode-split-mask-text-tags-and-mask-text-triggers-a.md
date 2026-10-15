# #1498 — Lab 2 (OpenCode): split mask_text_tags and mask_text_triggers (Arm C)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-29T08:17:35Z
- **Updated:** 2026-10-09T14:17:22Z
- **Closed:** 2026-09-29T08:25:36Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1498
- **Merged:** 2026-09-29T08:25:36Z
- **Merge commit:** 7cc595495cda7ed9965228bd07fab63f1a67c2b2
- **Base / Head:** main / UOR-Foundation:lab/opencode/arm-c
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Lab 2 (OpenCode). Class A** (option split for the pre-registered Arm C). References #973 and #820.

Pre-registration: [#973 comment 5884957504](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5884957504).

## What

- `AermConfig` now carries **`mask_text_tags`** and **`mask_text_triggers`**, both `#[serde(default)]` false and independent; `train_aerm` reads each for its own text auxiliary loss. The LM loss and the dialogue terms are unchanged.
- `aerm-probe` accepts `mask_text_tags=` and `mask_text_triggers=` (same bool parsing) and records both in the run's `train` config.
- **Compatibility:** the original single-flag behaviour (mask both text tag and trigger losses) is exactly `mask_text_tags=true mask_text_triggers=true` — the configuration Arm A used, and the one to replay it. Arm C is `mask_text_tags=true mask_text_triggers=false`.
- No relationship is implied between the two flags; a reader of a run JSON sees both.

## Checks

- `cargo fmt -p uor-r4-training --check` → pass.
- `cargo check -p uor-r4-training --offline --release` → pass.
- `cargo test -p uor-r4-training --release --offline stack_aerm` → **15 passed, 0 failed** (this branch predates (A)'s new test; the 16th lands with #1497).
- `python3 scripts/check_claim_wording.py` → pass.

A non-author review is requested; Arm C then runs in the slot after Anti-Gravity releases it, per the pre-registration.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5886489172 — 2026-09-29T08:24:41Z — Casey-allard

## Non-author review — **approved**, with one recorded compatibility caveat

Independent pass `uor-evidence-auditor` (session `ses_f13c23ce7ffemkDsx56J9yxon3`), read-only.

1. **Flag independence — confirmed.** `train_aerm` gates the text *tag* loss by `mask_text_tags` and the text *trigger* loss by `mask_text_triggers`; the LM losses and the dialogue auxiliaries are unconditional.
2. **Backwards compatibility — stated correctly, one caveat:** the original `mask_text_tags` gated **both** text auxiliaries, so `mask_text_tags=true mask_text_triggers=true` exactly reproduces the old single-flag behaviour. **Caveat (recorded, not a defect):** an *old serialized config* with `mask_text_tags=true` and no `mask_text_triggers` key now deserializes to `mask_text_triggers=false`, so a replay of arm A/B's saved config would leave the text trigger loss unmasked. Arm C is exactly that combination deliberately; arm A is reproduced by setting both flags. Any future replay of an arm-A/B config must set both flags explicitly.
3. **Probe — confirmed:** both flags parsed with bool validation, both allowed, both recorded in `train`/`train_template`.
4. **Regression risk — none found:** no test or other crate constructs `AermConfig`; only the probe's two constructors exist and both are updated.

Checks on the head (lead-run): fmt pass; `cargo check --release` pass; `stack_aerm` 15/15 (this branch predates the 16th test from #1497); claim wording pass. Class-A conditions hold.



---
_Comment/review ids in this file: 5886489172_

