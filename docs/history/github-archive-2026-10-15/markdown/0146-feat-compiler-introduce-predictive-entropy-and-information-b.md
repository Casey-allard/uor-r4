# #146 — feat(compiler): introduce predictive entropy and Information-Bottleneck objectives (#127)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-24T14:57:29Z
- **Updated:** 2026-07-24T14:58:55Z
- **Closed:** 2026-07-24T14:58:31Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/146
- **Merged:** not merged
- **Merge commit:** 58c427e82fea34a22accd2ad858398d471ba5b46
- **Base / Head:** main / UOR-Foundation:issue-127-information-bottleneck
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Overview & Purpose
Resolves #127. Implementation follows PDF §§6, 8 and `docs/formal_vocabulary.md` §4.

## Key Changes
1. **Information-Bottleneck Objective**: Implemented $J_{\text{IB}} = I(Z; X) - \beta \cdot I(Z; Y_{\text{future}})$ estimator measuring compression of surface detail $I(Z;X)$ against predictive future utility $I(Z; Y_{\text{future}})$.
2. **Versioned Composite Cost Model**: Implemented versioned $J = L_{\text{teacher}} + \lambda_{\text{runtime}} C_{\text{runtime}} + \mu_{\text{artifact}} C_{\text{artifact}} + \alpha_{\text{IB}} J_{\text{IB}}$ objective evaluation.
3. **Split-Safe Region Decision Auditing**: Implemented `RegionDecisionReport` evaluating region topological decisions (Split, Merge, Retain, Remove) based on held-out evaluation set objective deltas ($\Delta J$).
4. **Cucumber BDD Suite**: Added `features/suites/information_bottleneck.feature` with 3 scenarios and step definitions in `tests/bdd.rs`.

## Verification
- `cargo fmt --check`: Clean
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`: Clean
- `cargo test --workspace --offline`: All unit tests passed
- `cargo test --test bdd --offline`: All 28 BDD scenarios passed (91 steps)
- `cargo check -p uor-r4-graph-format --no-default-features`: Clean
- `python3 scripts/check_claim_wording.py`: Wording gate passed

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5071239027 — 2026-07-24T14:57:33Z — chatgpt-codex-connector[bot]

You have reached your Codex usage limits for code reviews. You can see your limits in the [Codex usage dashboard](https://chatgpt.com/codex/cloud/settings/usage).


---
_Comment/review ids in this file: 5071239027_

