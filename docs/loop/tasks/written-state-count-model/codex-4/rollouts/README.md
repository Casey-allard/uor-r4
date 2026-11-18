# [codex] Free-rollout case tables

Each TSV has one row per prompt and arm. The generated IDs include the continuation after the copy cursor releases. The first wrong index is zero-based against the entire reference, including EOS. An exact reply has no wrong index. `NA` marks an unavailable field; the separate strict-reply flag distinguishes an exact sequence from a missing expected or emitted ID. The error-rank difference uses the fixed-offset arm on the same prompt as its baseline. Positive values mean the first error occurred later.

Each JSON summary counts matched pairs within a seed. A pair succeeds only if both question directions selected their own requested source occurrence. Individual source, opening, whole-value and strict-reply counts use prompts as their denominator. The held-out evidence uses registered BPE anchors. For the original regression only, the summarizer reconstructs the name anchors **post hoc** from the pinned original prefix. Ann and Ben are IDs 2241 and 546. Each occurs exactly twice in each original prompt, first in the facts and again in the question or exclusion. That reconstruction produced nine correct query/fact bindings with the wrong value start and seven wrong query selections at each seed. It affects no registered bar.

To reproduce from `codex-4/`, supply a primary writer JSON from the evidence bundle. Verify the original panel SHA-256 against `PANEL-SHA256SUMS`. For each of the four primary files the independent process replay matched byte-for-byte.

```sh
jq -r --slurpfile original ../codex-1/panel.json --arg format tsv -f summarize.jq "$ROLLOUT" > cases.tsv
jq --slurpfile original ../codex-1/panel.json --arg format summary -f summarize.jq "$ROLLOUT" > summary.json
```
