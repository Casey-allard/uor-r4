# ORDER: cpu-training-pilot, deepseek-155 (copy termination-oracle ceiling)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `835cfeb66`.

## Goal

Determine whether perfect termination would make the deepseek-153
longest-first copy arm produce exact complete replies. This separates two
possible next dependencies:

- if any copy body is exact at the reference length, build copy plus
  termination;
- if none is exact, improve the copy/content read before building a terminator.

This is a read-only analysis of sealed deepseek-153 evidence. It creates no new
panels, runs no new model forwards and changes no serving, CUDA, float-training
or Claude-owned source.

## Fixed construction

Read the four sealed attempt-2 reports for artifact seeds 1 and 2 on panels
`train,train` seed 2711 and `dev,dev` seed 2712. Recover each arm's selected
token sequence from its per-decision trace and the reference sequence from the
same panel manifest.

For each reachable row and each arm, form an oracle-stopped reply:

```text
tokens = first reference_len selected tokens
stop   = BOS
```

No model token is generated or changed. This asks only whether the already
selected content would be exact if an external oracle stopped it at the correct
length. It is not a deployable decoder and makes no language claim.

## Measurements

For each artifact/panel cell and arm, report:

- rows and reachable rows reconstructed;
- exact oracle-stopped bodies;
- opening-correct bodies;
- correct body positions within the reference horizon;
- mean and maximum common-prefix length;
- histogram of replies one, two or three trailing tokens from exact;
- reconstructed token count and trace-hash agreement with the sealed report.

Use the paired two-sigma helper on exact oracle-stopped bodies for longest-first
against `k2>k1`, fixed depth 1, random depth and writer, and report body-position
comparisons as support.

## Bar

- **Termination-only blocker** if at least one copy arm has an exact
  oracle-stopped reply and longest-first does not lose its paired body-position
  advantage over `k2>k1`, depth 1 and random depth.
- **Content remains insufficient** if every copy arm has zero exact
  oracle-stopped replies.
- Every other result is mixed and must be reported without selecting a decoder.

The decision unit is the artifact/panel cell. The conclusion is only accepted
if all four cells agree on one of the two primary outcomes.

## Stop rules

Stop on report/schema drift, row-count mismatch, missing or non-monotonic
decision steps, a selected-token trace that cannot reconstruct a reply,
published-summary mismatch, input/hash drift, or a malformed reference.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-155/**`
- `docs/labs/cpu-training-pilot-copy-key-reply-2026-11-05/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`
- `docs/integration/current-state.md`
- `docs/harness/claims.json`

## Done when

1. The sealed-trace reconstruction, horizon truncation and comparisons have
   tests.
2. The analysis reproduces all four existing published cell summaries.
3. Evidence and REPORT state whether perfect stopping alone could rescue the
   copy read.
4. Exact-head review, protected merge and fresh-main verification complete.
