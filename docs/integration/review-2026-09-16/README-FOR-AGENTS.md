# For any agent working in this repo (Antigravity, Codex/Astra, Claude, Gemini)

Before proposing or executing work in uor-r4, read in this order:
1. `00-project-brain-index.md` (2 min) — what is here.
2. `08-outlook-and-roadmap-critique-2026-09-16.md` (15 min) — where the project actually stands, root causes R1–R5, paths P1–P6.
3. `10-execution-plan-agent-briefing.md` (10 min) — ground rules, decision D0, pre-registered experiment cards, multi-agent protocol.
Then `02` and `04` before touching `crates/uor-r4-core/src/native_geometric/`.

Rules that apply immediately:
- Do not start compute on any experiment without a signed card (`10-…` §5 template) in `docs/integration/cards/`. Casey signs; agents draft.
- Every result needs a matched non-geometric control on identical data and goes into `docs/integration/EVIDENCE.md`.
- Template-generated panels are diagnostics, not headline results.
- Measure energy/latency; never hard-code them.
- Decision D0 (what "no matmul" means) is Casey's and blocks P3/P2 model choice; ask for it, do not assume it.

These files were produced by a Claude research session on 2026-09-16 from GitHub HEAD a5655e93 and the local Codex context; they are untracked here until Casey commits them.
