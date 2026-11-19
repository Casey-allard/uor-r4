# D25 supplied-assertion source audit

The four arms selected **zero** sources. Each scored 0/6 primary, 0/2 secondary and 0/4 exposed exact source addresses. Each abstained on ambiguity row 209 with eligible candidates and returned an empty group on 210 and 211, which were supplied no anchor. There were no rescues, losses or false source writes between arms. Ordinary binding and emitted IDs are evaluated separately.

The CUDA prefix read returned **one identical vector across 21 anchors**: 8 TRAIN, 9 held, 4 exposed. The 32-coordinate vector was `[0,0,-2,0,-2,0,0,0,2,2,0,0,0,0,0,0,2,2,0,0,0,0,0,0,-2,2,0,0,0,0,0,0]`. This read is the first observed point where the intended contextual contrast is absent. No inference about where earlier trunk intermediates lost the contrast follows from this observation. Both directions of all four held pairs passed exact donor alignment for the packet, contextual traces and decisions. The local traces and decisions stayed with their recipients. All eight swaps transferred equal vectors, so the control had no contextual leverage. Corresponding local vectors were also identical within every held pair. With a common TRAIN and prediction context, the context term added zero to every distance.

All 13 nonempty held/exposed groups had a maximum of exactly zero in every arm. The device recorded `nonpositive` before the positive-tie decision. The table gives actual positive/negative distances at the early and late onsets and numbers of legal candidates with equal nearest-positive and nearest-negative distances. The latter all tied for the zero maximum. Counts and distances are unchanged by adding context. Offsets are relative to the last factual-name token.

| Rows | Early, late offsets | Narrow and legacy offsets | Wide offsets | Early positive/negative | Late positive/negative | Narrow / wide at zero |
|---|---|---|---|---|---|---|
| 201/202 | +3, +5 | +2, +3 | +1, +2, +3, +5 | 40/40 | 28/28 | 1 / 2 |
| 203/204 (secondary) | +3, +6 | +2, +3 | +1, +2, +3, +6 | 52/52 | 72/72 | 2 / 4 |
| 205/206 | +3, +4 | +2, +3, +4 | +1, +2, +3, +4 | 44/44 | 44/44 | 2 / 2 |
| 207/208 | +3, +4 | +2, +3, +4 | +1, +2, +3, +4 | 56/56 | 52/52 | 3 / 3 |
| 209 (ambiguity) | +3, +5 | +2, +3 | +1, +2, +3, +5 | 40/40 | 28/28 | 1 / 2 |
| 210/211 (empty) | — | — | — | — | — | 0 / 0 |
| 301/302 (exposed) | +3, +4 | +2, +3, +4 | +1, +2, +3, +4 | 48/48 | 60/60 | 2 / 3 |
| 303/304 (exposed) | +3, +5 | +2, +3 | +1, +2, +3, +5 | 36/36 | 60/56 | 2 / 2 |

The narrow mask matched the legacy supplied-assertion eligibility after its whole-word and completion checks on every anchored row. Gold reachability went from 5/6 to 6/6 primary (201), 1/2 to 2/2 secondary (204), and 3/4 to 4/4 exposed (304). The legacy supplied-assertion reader picked +3 on 201–204, 209 and 301–304, and +4 on 205–208. That gives 3/6 primary, 1/2 secondary, and 2/4 exposed exact and a false source on 209. The three widened golds reached the ranker. Each failed its strict positive margin gate. Primary 201 already failed reachability under the narrow and legacy masks. This closes the tested contextual configuration on these fixtures and preserves the distinct mask reachability result.

## Reproduce

Run from the repository root. Set `q` to the retained run root containing `final-projection` and `outcome-01`. The audit emits original reason codes and assertion-relative addresses, all eligible per-arm distances and margins, and the donor checks.

```sh
p="$q/final-projection"
o="$q/outcome-01"
(cd "$o" && shasum -a 256 -c observations.sha256)
for file in bank.bin held.results held.trace held-swapped-context.results held-swapped-context.trace exposed.results exposed.trace; do
    cmp "$o/primary/$file" "$o/cold/$file" || exit 1
done
jq -n \
    --slurpfile held_key "$p/held.grade.json" --slurpfile exposed_key "$p/exposed.grade.json" \
    --rawfile held_rows "$p/held.rows" --rawfile swapped_rows "$p/held-swapped-context.rows" \
    --rawfile held_trace "$o/primary/held.trace" --rawfile exposed_trace "$o/primary/exposed.trace" \
    --rawfile swapped_trace "$o/primary/held-swapped-context.trace" \
    --rawfile held_results "$o/primary/held.results" --rawfile exposed_results "$o/primary/exposed.results" \
    --rawfile swapped_results "$o/primary/held-swapped-context.results" \
    --rawfile train_context "$o/train.context.tsv" --rawfile held_context "$o/held.context.tsv" \
    --rawfile exposed_context "$o/exposed.context.tsv" \
    -f docs/loop/tasks/written-state-count-model/codex-25/analysis/source.jq
```

The [source receipt](source.sha256) hashes the inspected inputs and audit output paths relative to the retained run root. No fitting or inference was run by the reviewer.
