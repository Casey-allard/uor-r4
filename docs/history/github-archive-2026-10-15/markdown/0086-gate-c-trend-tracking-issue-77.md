# #86 — Gate C trend tracking (Issue #77)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-23T03:34:54Z
- **Updated:** 2026-10-09T14:17:24Z
- **Closed:** 2026-07-23T04:09:47Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/86
- **Merged:** not merged
- **Merge commit:** aaa760c6587520489ec5d632a99850a42a3c5d60
- **Base / Head:** main / UOR-Foundation:issue-77-ci-gate-c-trend
- **Reviews / inline review comments:** 1 / 3
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Resolves #77 by implementing a CI step to run the Gate C regression check. Adds a Python script to assert against pinned top-1 agreement and bits/token.

## Timeline (6 entries; chronological: conversation comments, reviews, review comments)

### comment 5054012656 — 2026-07-23T03:34:59Z — chatgpt-codex-connector[bot]

You have reached your Codex usage limits for code reviews. You can see your limits in the [Codex usage dashboard](https://chatgpt.com/codex/cloud/settings/usage).

### comment 5054235335 — 2026-07-23T04:09:46Z — Casey-allard

Closing as auser is already handling this.

### review 4760714640 — 2026-07-23T04:10:11Z — copilot-pull-request-reviewer[bot] — COMMENTED

## Pull request overview

Implements Gate C regression tracking in CI by generating a Gate C score report on the fixture corpus and adding an automated regression check against pinned top-1 agreement and bits/token thresholds.

**Changes:**
- Add `scripts/check_gate_c_regression.py` to assert Gate C Rule 1+2 metrics vs pinned thresholds.
- Update CI workflow to run the Gate C scoring command, run the regression checker, and upload a Gate C artifact; also run the `bdd` integration test outside nextest.
- Add `.config/nextest.toml` to exclude the `bdd` test binary from nextest enumeration, and ignore `trend_output/` in git.

### Reviewed changes

Copilot reviewed 3 out of 4 changed files in this pull request and generated 3 comments.

| File | Description |
| ---- | ----------- |
| scripts/check_gate_c_regression.py | New Python regression checker for Gate C Rule 1+2 metrics. |
| .gitignore | Ignore `trend_output/` directory used by Gate C scoring output. |
| .github/workflows/ci.yml | Add Gate C regression step + artifact upload; run `cargo test --test bdd` separately. |
| .config/nextest.toml | Configure nextest to exclude the `bdd` custom-harness test binary. |




<details>
<summary>Comments suppressed due to low confidence (1)</summary>

**.github/workflows/ci.yml:113**
* This adds a second Gate C trend run inside the main `gates` job, but the workflow already has a dedicated `gate-c-trend` job that runs `scripts/gate_c_trend.sh` and uploads a trend artifact with commit metadata. The new steps also upload `score_report.json` (which does not include commit metadata) instead of the trend row required by Issue #77’s acceptance criteria, and calling `./scripts/check_gate_c_regression.py` depends on the file being executable. Consider removing these duplicated steps and relying on the existing `gate-c-trend` job (or, if you want this in `gates`, switch to using the same pinned JSON + trend output format).
```
      - name: Gate C trend tracking (Issue #77)
        run: |
          cargo run --release --bin r4 -- transformerless score \
            --corpus-meta crates/uor-r4-core/tests/fixtures/c_meta.bin \
            --corpus-recs crates/uor-r4-core/tests/fixtures/c_recs.bin \
            --artifacts crates/uor-r4-core/tests/fixtures/tless_artifacts.bin \
            --out trend_output
          ./scripts/check_gate_c_regression.py trend_output/score_report.json

      - name: Upload Gate C trend JSON
        uses: actions/upload-artifact@v4
        if: always()
        with:
          name: gate-c-trend
          path: trend_output/score_report.json
```
</details>



---

💡 <a href="/UOR-Foundation/uor-r4/new/main?filename=.github/instructions/*.instructions.md" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Add Copilot custom instructions</a> for smarter, more guided reviews. <a href="https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Learn how to get started</a>.

### review-comment 3635498914 — 2026-07-23T04:10:11Z — Copilot

_on .github/workflows/ci.yml:18_

The workflow header comment says the pinned record lives in `scripts/check_gate_c_regression.py` and that a `gate-c-trend` artifact is uploaded, but this workflow already uses `docs/transformerless/gate_c_pinned.json` + `scripts/gate_c_trend.sh` (and uploads a per-sha artifact) as the pinned source of truth. This comment should be updated so it documents the actual pinned file and artifact path/name used by CI.

This issue also appears on line 99 of the same file.

### review-comment 3635498929 — 2026-07-23T04:10:11Z — Copilot

_on scripts/check_gate_c_regression.py:12_

The pinned Gate C record and thresholds are duplicated here as hardcoded constants, but the repo already has a single pinned source of truth in `docs/transformerless/gate_c_pinned.json` (used by `scripts/gate_c_trend.sh`). Loading that file when present avoids drift between CI and local checks while still allowing a fallback if the pinned file moves.

### review-comment 3635498948 — 2026-07-23T04:10:11Z — Copilot

_on scripts/check_gate_c_regression.py:29_

Defaulting missing `top1_agreement` / `bits_per_token` to 0.0 will turn a schema/reporting problem into a misleading "regression" (and produces confusing deltas). Since these fields are required for this check to be meaningful, fail with a clear error if either metric is missing or non-numeric.


---
_Comment/review ids in this file: 5054012656, 5054235335, 4760714640, 3635498914, 3635498929, 3635498948_

