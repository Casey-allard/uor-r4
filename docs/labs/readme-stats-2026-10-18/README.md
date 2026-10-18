# README decision range and statistics corrected 2026-10-18

Phase 1 of the D39 tracker-hygiene queue names README decision-range and statistics drift. This docs-only
correction updates the owner-decision link range from D0-D34 to D0-D39 and replaces the 13 October 2026
snapshot with current verifiable counts (2,130 merged PRs, 1,000 in the last 30 days; about 1.97M Rust
lines; 9,704 test functions; 12 open issues). No code, model, gate or evidence change.

## Verification

- Counts were measured at the current checkout: `git ls-files "*.rs"` line total 1,969,803; test
  attributes 9,704; GitHub merged PRs 2,130 / 1,000 in the last 30 days; open issues 12.
- History was consolidated on 18 October, so the block states that a commit count since 9 June is not
  comparable to the earlier snapshot instead of printing a misleading number.
- Docs-only; no Rust paths touched, no compute, $0.