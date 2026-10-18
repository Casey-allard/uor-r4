# ROADMAP D37 learning permission and milestone statuses — 2026-10-18

Closes #2680 and #2681 (front-door audit 2.1/2.2, part of #2028).

- Replaces Section 0's superseded total D32 learning ban with the D37 wording: offline training may use a
  float forward pass, gradients and an optimizer in a separately scanned geometric module; matmul/GEMM and
  softmax/squashing stay banned everywhere; the active discrete path stays integer and serving is a
  separately qualified D11 export. Links D37.
- Sets the milestone table statuses to the README wording: M2 **held**, M3 **paused**, M4 **held**. Goal,
  acceptance and latest-result cells untouched; the SVG was not hand-edited.
- No other total float ban remains in ROADMAP; the remaining mentions are serving-scoped D11/R1 and the
  active-path D32 statement.

Checks: `cargo run -q --locked -p uor-r4-claim-wording -- .` PASS; `git diff --check`; only ROADMAP, record
and current-state changed. Docs-only; no code/model/compute; $0.