# #1402 — Establish fourth research lab and align dialogue token contracts

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-27T00:49:25Z
- **Updated:** 2026-10-09T13:17:15Z
- **Closed:** 2026-09-27T00:50:51Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1402
- **Merged:** 2026-09-27T00:50:51Z
- **Merge commit:** 06850e05c9cd30403701170a6cbe5e16d5b9bdf0
- **Base / Head:** main / UOR-Foundation:codex/fourth-research-lab-20260926
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Concurrent dialogue training and chatbot work currently use incompatible role serialization, and even identical visible role text can yield different BPE tokens because training encodes marker/content segments independently. Add a versioned tokenizer-bound Rust dialogue encoder and explicit validation so both owners can adopt the same token/mask contract without altering existing bundles or their active adapters.

Establish the owner-requested fourth Codex lab alongside Google, OpenCode/DeepSeek/Kimi and Claude. The canonical roadmap now records attention → inference → prose → chat → reasoning priorities, broad specialist responsibilities, recursive source/evidence recovery, shared GitHub/worktree coordination, and decision-driven discovery. The source-reviewed geometric attention blueprint distinguishes a proposed nonlinear finite relative-group reader from ordinary dot/bilinear attention; all model outcomes for that proposal remain NOT_RUN. The integration review records PR #1400's synthetic evidence limits and preserves neighboring scan, dialogue, session and emission tracks.

Validation:
- `cargo test -p uor-r4-tokenizer --offline`: 7 passed (4 new protocol checks and 3 retained); 13.15s compilation, with the shared Cargo slot free at launch.
- Real retained-tokenizer Rust witness confirms both segmented-versus-concatenated BPE and literal-role-versus-BOS/UNK mismatches; no model or ChatSession execution.
- Touched Rust formatting, shell syntax, offline/live context-helper runs, new-document local links, claim wording and staged whitespace checks passed.
- Independent source review matched corpus formatting/masks and reviewed the geometry proposal; corrected the guide to permit planned independent seeds/replications.

Scope: no training, model promotion, changed accepted artifact, paid compute, or modification to another lab's active worktree. Shared encoder adoption by the training/panel/session adapters is the next integration step; tests do not qualify learned dialogue. The resource receipt binds source/witness identities and cumulative accounting; the previous unexplained global allowance is preserved rather than adopted.

References #973
References #820


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

