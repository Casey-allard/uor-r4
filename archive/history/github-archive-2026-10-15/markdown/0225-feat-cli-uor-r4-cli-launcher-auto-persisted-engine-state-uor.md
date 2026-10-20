# #225 — feat(cli): uor-r4-cli launcher, auto-persisted engine state, UOR audit inspector, and quickstart docs

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-26T15:38:09Z
- **Updated:** 2026-07-26T15:53:46Z
- **Closed:** 2026-07-26T15:53:46Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/225
- **Merged:** 2026-07-26T15:53:46Z
- **Merge commit:** b11d0957056ec9e1db01115f47b5e7baf16ca6ea
- **Base / Head:** main / UOR-Foundation:fix/r4g1-abstain-fallback-and-routing-speed
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

This PR completes the R⁴ single-command launcher ecosystem, state persistence, interactive menu connections, UOR compliance auditability suite, and quickstart documentation:

1. **Single-Command Launcher Script (`uor-r4-cli` & `r4-app.sh` wrapper)**:
   - Added `./uor-r4-cli` single-command orchestrator with terminal clearing, model/engine state restoration, live progress bars, and automatic 4-stage pipeline execution.
   - Self-resolves repository path (`SCRIPT_PATH` resolution) so `./uor-r4-cli` can be invoked via PATH (`~/.local/bin/uor-r4-cli`) or symlink from any terminal directory.
   - Retains `./r4-app.sh` wrapper for backward compatibility.

2. **Session Model & Engine Persistence**:
   - Auto-restores `.uor-models/last_model_name.txt` and `.uor-models/last_engine.txt` across client restarts.
   - Scaled compilation `--target` token counts dynamically by model capacity (176k for 135M, 500k for 360M, 1.76M for 1.7B).

3. **Fully Connected Interactive Menu Options**:
   - Unified `/` menu selection with main slash command execution handlers (`/models`, `/engine`, `/status`, `/corpus`, `/compile`, `/audit`, `/clear`, `/quit`).

4. **UOR Auditability & Tracing Suite (`/audit`)**:
   - Extended `VendorChatCompletionsResponse` with `uor_audit` payload containing CBOR UOR address (`uor:cbor:blake3:...`), curvature ($\kappa$), deficit angle ($\theta_d$), QIMC control entropy bias (\_bias$), dampening factor ($\gamma$), sampling temperature ($), $\kappa569Xreproduction status (`[✓ PASS]` / `[! DRIFT]`), and token provenance traces.
   - Added `/audit` command, interactive slash menu option `[6] /audit`, standalone CLI command (`r4 audit`), launcher flag (`uor-r4-cli --audit`), and session log export (`.uor-models/audit_log.json`).

5. **Documentation & Quickstart**:
   - Updated `README.md` with an out-of-the-box Quickstart guide covering `uor-r4-cli`, global PATH symlinking, slash commands, and audit log inspection.

## Verification
- `cargo fmt --check`: **PASS (100% clean)**
- `cargo check --workspace`: **PASS (100% clean)**
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`: **PASS (0 warnings)**
- `cargo check -p uor-r4-graph-format --no-default-features`: **PASS**
- `cargo check -p uor-r4-graph-format --no-default-features --features alloc`: **PASS**
- `./uor-r4-cli --audit`: **PASS**

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

