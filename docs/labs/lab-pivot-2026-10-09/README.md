# Lab review and pivot (9 October 2026, claude, owner decision D21)

The owner asked whether the DeepSeek and Codex labs had gone down rabbit holes. This is the review behind [D21](../../integration/DECISIONS.md#d21--three-negatives-on-one-line-force-a-pivot-deepseek-trains-the-pointer-fix-codex-stops-the-constraint-line). Source: the merged PRs from 19:00 UTC on 9 October to 03:00 UTC on 10 October, and their result sections.

## DeepSeek, M1 (#2029): 33 PRs, headline unchanged
- **Open reply panel:** 43/232 reproduced (#2089).
- **Diagnosis:**
  - mid-clause failures are the 64-token budget (#2092);
  - a 96-token cap does not raise acceptance (#2103);
  - failures are short wrong answers (#2105);
  - deterministic sub-readings were added (#2108, #2110, #2111).
- **v5 memory panel:** frozen (#2080); the declared acceptance run scored **10/40** against ≥ 34 (#2112).
- **Numeric-row chain:**
  - diagnosis, plan, probe, reader-vs-emitter split, pointer trace, reproduction (#2114 to #2124);
  - one inference-path change, 0 → 3 of 13, which failed its bar (#2126);
  - positional read and decode: **the pointer attends the sentence frame, never the value's slot** (#2127, #2128);
  - plan for the target change (#2129), access attempt (#2130, #2131), copy-mass read (#2133).
- **Reading:** a real mechanism defect was found, and a concrete fix exists in the training path. The line then kept refining the reading instead of training the fix. No PR changed the model.

## Codex, M2 (#2030): 15 PRs, headline unchanged
- **Accepted:** 8/512 complete replies, unchanged since 7 October.
- **Protected / discrete-constructor line:**
  - Prefix transactions (#2079, negative #2084);
  - margin credit (#2088);
  - protected joint learning with a negative quantization screen (#2101);
  - quantized-protection attribution (#2109);
  - discrete feedback: 32 rounds, 0 eligible displacements (#2117);
  - attribution (#2121);
  - direct legal construction: `Singular matrix` (#2125);
  - branch-basis small pivots in the constraint solver (#2132).
- **Also:** reviews and traces (#2085, #2087, #2091, #2093, #2097).
- **Reading:** each step was carefully reviewed and archived, and each ended "not yet promoted, 8/512 unchanged, next is another constructor". The work moved from reply quality to solver numerics.

## Decision (owner, 9 October)
- **DeepSeek:** train the pointer fix (supervised `bound` / `competing` labels, `gate_supervised_loss`) on generated training dialogues. Score it on the frozen v5 and reply panels with a pre-registered bar, in one PR.
- **Codex:** stop and archive the constraint line. The next M2 piece aims directly at 8/512.
- **All labs:** the three-negatives rule (D21 §1, AGENTS.md "Progress control").
