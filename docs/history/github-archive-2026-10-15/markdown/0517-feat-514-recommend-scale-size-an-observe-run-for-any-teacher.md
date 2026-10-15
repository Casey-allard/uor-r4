# #517 — feat(#514): recommend-scale — size an observe run for any teacher weight class

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-08T23:51:48Z
- **Updated:** 2026-10-09T13:08:55Z
- **Closed:** 2026-08-09T00:10:50Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/517
- **Merged:** 2026-08-09T00:10:50Z
- **Merge commit:** 48641daa9d1f9fd5d7a368f57282eac19719454b
- **Base / Head:** main / UOR-Foundation:feat/514-recommend-scale
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

First deliverable of the corpus-scale law (#514): a config-only calculator.

```
r4 transformerless recommend-scale --config <hf dir>
```

Reads d_model / n_layers / vocab and prints recommended observation records + article count + an expected EXCT-miss guide. Closed form: `S = d_model·n_layers·log2(vocab)`, `N = N_REF·(S/S_REF)^BETA`, anchored on the runs (docs/scaling_law.md): N_REF=2M (wiki coverage knee #432), S_REF=S(360M), **BETA=0.5 provisional** (bounded [0.45,1.0]; 0.45 is the measured #460 capacity exponent). Every output is marked provisional; `--beta` overrides.

Sanity (monotone in capacity): 360M→~2.0M records/~9.5k articles, 135M→~1.5M, stories15M→~465k. Unit-tested.

Remaining #514 work: the `subsample-corpus` sweep harness and BETA calibration (needs the big observes on dev hardware). Independent of #513.

Refs #514.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

