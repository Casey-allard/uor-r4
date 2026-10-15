# #450 — Hazard: score/cover silently default --artifacts to a shared mutable /tmp path; log the artifact κ

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-06T04:13:07Z
- **Updated:** 2026-08-06T13:37:42Z
- **Closed:** 2026-08-06T13:37:42Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/450
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Root cause of the #448 false alarm. `--artifacts` defaults to `compiler::ART_PATH` = `/tmp/tless_artifacts.bin` (crates/uor-r4-core/src/transformerless/compiler.rs:1071) — a world-writable path that several of our scripts stage into and restore. Any concurrent run silently reads whatever container is there, and nothing in the output reveals it.

Two cheap fixes: (1) print the artifact κ on the induction line — it is already computed as `artifact_kappa`, and would have made this self-diagnosing at a glance; (2) reconsider the default: either require --artifacts explicitly, or default to a per-run path. Also worth auditing our helper scripts to stop the cp-in/cp-out staging pattern in favour of passing --artifacts explicitly.

## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

