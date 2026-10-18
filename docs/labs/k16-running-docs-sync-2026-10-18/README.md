# K16 running-state docs sync — 2026-10-18

#2698 merged at `b0c0f56a` while recording the pre-start state, so NOW.md, ROADMAP.md, STATUS.md,
README.md and docs/labs/README.md still said training was NOT_RUN after the first native role had started.
This docs-only correction records the observed state:

- four owner/admission pairs were produced from the genuine Kimi records;
- **K16-A/seed1 has been RUNNING since 20:24:29Z** (PID 173683, GPU0, position cursor 1,360 / 136 accepted
  updates at the 20:27Z observation);
- R-seed1 / A-seed2 / R-seed2 are **NOT_STARTED** and run serially;
- no result, capability, KEEP or mechanism claim follows.

Sources: Codex start receipt/comment, `progress-observation-2027z.txt`, `uor-pod status`. Docs-only; no
compute, $0. Claim-wording PASS; diff-check clean.