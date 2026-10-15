# #807 — feat(#804-S1-0): traced teacher-forced text observation — #603 capture reaches observe-text

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-18T22:39:55Z
- **Updated:** 2026-10-09T13:08:53Z
- **Closed:** 2026-08-18T23:05:38Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/807
- **Merged:** 2026-08-18T23:05:38Z
- **Merge commit:** c2b49ece5382f1c56e9febe6a71758887ccd070e
- **Base / Head:** main / UOR-Foundation:issue-804-s1-wiring
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #804 (S1-0, per the decided plan and profile: `full/1`, declared layers `[0,4,9,13,18,22,27,31]`, S=8, on the `smollm2-360m-broad` corpus) and #605 (whose real-corpus arm consumes the output through `load_route_trace_corpus`).

The #603 trace stack was complete at library level for the **generation** driver (`observe_sharded_traced`, the oracle's exact-executor taps, the sidecar writer, the route-fit loader) — but the **teacher-forced text** pipeline, which is how the broad corpus is actually observed, had no trace plumbing, and `observe-text` exposed no flag. This PR wires it end to end:

- **`observation.rs`** — the #603 profile-pinning block is factored into one shared preflight helper (`preflight_trace_profile_pin` / `pin_writer_trace_profile`) used by both drivers, so they cannot drift; `ObservationShardWriter` gains the record+probability+trace+partition combined append; `TraceCapture` becomes crate-visible for the sibling driver.
- **`observation_text.rs`** — `observe_text_corpus_traced[_in_session]`: one capture assembler per worker oracle (multi-worker traced passes stay byte-deterministic), produce/commit thread sidecar rows index-aligned with records (misalignment refuses before writing), resume trims sidecars to the committed whole-article checkpoint (truncate-only; a shortfall or missing sidecar under committed records refuses by name), and minimal resumes / profile changes of a traced corpus refuse via the shared pin. The batched driver has no per-head taps and is refused for traced passes.
- **`graph-cli`** — `observe-text --trace-profile <id>/<version> --trace-layers i,j,... [--trace-support S]`, resolved against the immutable registry before any teacher load; incoherent combinations (`--batch`, legacy `--checkpoint`, orphan bounds flags) refuse at parse with typed errors. Help text updated.

Tests (all falsifier-verified red where a refusal is claimed): traced sidecar alignment + manifest pinning; fresh-pass and worker-count byte-determinism; pin refusals (minimal resume, bounds-only profile change); trim exactness incl. torn-tail bytes + shortfall/missing refusals; commit misalignment falsifier; the CLI flag matrix.

Gates: fmt; clippy `-D warnings` (workspace, all targets); graph-compiler lib 203 pass; graph-cli lib 129 pass; wasm32 graph-compiler check (the 1 pre-existing warning is unchanged on main, verified by stash A/B).

Next (on #804's thread, per the posted plan): the 1-article smoke run for measured capture-rate numbers, the contract amendments on #605/#643, then the overnight corpus pass on the maintainer's confirmed window.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_016X39CfJSLCU4KhjNP2XXTW


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

