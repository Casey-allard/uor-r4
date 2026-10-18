# Core README and geometry front-door corrections — 2026-10-18

Closes #2682 and #2683 (outside-contributor FIX-10 and front-door audit 2.7, part of #2028).

- `crates/uor-r4-core/README.md`: corrected the self-contradiction under "Historical implementation and
  evidence" — "**Active native model**" becomes "**Historical native model** `native_geometric` implemented
  the Rust training/artifact/session path used by `r4 geometric`; Cargo does not select it." Heading, links
  and the rest of the section unchanged.
- `docs/geometry.md`: replaced the 13-line repeated status paragraph with a concise pre-alpha sentence
  (live numbers in `STATUS.md`) plus one D37 scoping sentence; removed the duplicated pod, waiver and
  admission detail. No claims, numbers or mechanism labels changed; the existing body leads.

Checks: `cargo check --locked -p uor-r4-core --lib` **PASS**;
`cargo run -q --locked -p uor-r4-claim-wording -- .` **PASS**; `git diff --check` clean. Docs-only; no
model/compute; $0.