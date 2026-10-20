# #878 — runtime(#836): P-4 hot-path segment ring, reference-equivalent (increment 3a, dormant)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-20T20:55:02Z
- **Updated:** 2026-08-21T01:56:23Z
- **Closed:** 2026-08-20T21:16:39Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/878
- **Merged:** 2026-08-20T21:16:39Z
- **Merge commit:** 5e7fbcf205641f615d37b4d32169d73919734e0b
- **Base / Head:** main / UOR-Foundation:issue-836-pstate-runtime
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

Increment 3a of #836. Lowers the #835 **segment lane** to a hot-path primitive: `SegmentRing<CAP>` in `uor-r4-graph-runtime::runtime_state` — the fixed-capacity, caller-owned, **no_std, allocation-free, P-4 (no multiply/divide/float)** realization of the reference ring. It sits alongside the crate's existing `TokenState`/`ReservedState` fixed-capacity state, filling the reserved "Phase 8 segment update" hook.

Semantics are bit-equivalent to the #835 reference ring (`docs/prompt_state_spec_835.md` §6 / `docs/scoring_semantics.md` §6): halving arithmetic-right-shift decay (division-free), saturating `ScoreQ` accumulation, canonical eviction victim (least weight, ties broken by the larger key so the lower id is retained), and a decode-independent candidate contribution.

## Scope

- `crates/uor-r4-graph-runtime/src/runtime_state.rs` — `SegmentSlot`, `SegmentRing<CAP>` (`with_decay`, `upsert`, `decay`, `contribution`, `capacity`/`len`/`as_slice`/`clear`), and two tests.

## Dormant — no serving change

`SegmentRing` is **not yet wired into the deployed R4Engine scoring path**, so serving behavior is unchanged (no request path reads it). The descriptor it will consume — `ring_capacity`, `decay_shift`, `boost` — rides in the optional PSTATE section (#876 landed the section; #877 completes the descriptor). A later increment activates it on the request path (CLI/API/server/WASM) with witness attribution and the causal-gate re-run.

## Verification (all green)

- `segment_ring_matches_reference_semantics` — **differential** vs a `Vec` oracle mirroring the #835 reference ring: identical contents, eviction returns, length, and per-candidate contributions **at every step** over a 4,000-step deterministic fold/decay stream, for decay shifts {0,1,2}, with key space (16) exceeding capacity (8) so eviction and collisions are exercised (~12k assertion points per shift).
- `segment_ring_zero_capacity_declines_without_panic` — `CAP=0` declines without panic (typed/`None`).
- `cargo test -p uor-r4-graph-runtime`, `clippy -D warnings` clean; `cargo run -p xtask -- validate` — **every gate passed incl. the gnaf forbidden-construct (P-4) scan** (97 modules); wasm target check; workspace check. `forbid(unsafe_code)` preserved.

## Compatibility / conformance

Additive, dormant. `CONFORMANCE.md` unchanged — no serving capability is claimed here (the capability + its `model/ids.toml` row → Gherkin → CONFORMANCE land when the scorer is wired onto the request path). No `no_std`/allocation/P-4 obligation is weakened.

## Closure

**References #836** — increment 3a (hot-path ring primitive). Next: wire it into the normative R4Engine scoring path behind the PSTATE descriptor with witness attribution, then the causal-gate re-run → PROMOTE/REVISE/RETIRE verdict.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

