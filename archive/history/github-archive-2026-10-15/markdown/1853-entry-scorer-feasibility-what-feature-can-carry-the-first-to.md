# #1853 — Entry-scorer feasibility: what feature can carry the first token, and what cannot (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-07T18:45:55Z
- **Updated:** 2026-10-07T18:47:42Z
- **Closed:** 2026-10-07T18:47:42Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1853
- **Merged:** 2026-10-07T18:47:42Z
- **Merge commit:** 761987e6b9b69cb8b56fa340c1f9db71ed451077
- **Base / Head:** main / UOR-Foundation:deepseek/entry-scorer
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What this is

A data-side feasibility read for a dedicated entry scorer, run against the retained construction panel. No model, no GPU, no fit: it answers whether the first answer token is determined by features the native path exposes, and which features survive a query the scorer has never seen.

This is the gate the previous comment on #820 named, and it decides what feature the scorer must have.

## Result 1 — a random held-out split is 100 %, and a row-keyed table cannot work

512 rows, **2 distinct first answer tokens**, and the answer token is a source surface token in **0 of 512** rows. Split by a hash of the case id (256 fit / 256 test):

| signature | distinct | seen at test | correct |
|---|---:|---:|---:|
| query only | 12 | 256 / 256 | **256 / 256** |
| query + source identity | 72 | 240 / 256 | **240 / 240** |
| all boundary features (row-unique) | 512 | **0 / 256** | — |

The task is learnable and a feature-keyed table generalises within the panel. A table keyed on the whole row does not: every row is its own signature, so it covers nothing.

## Result 2 — hold out a whole query and every cheap feature collapses to 0 %

Fit on every query signature but one, predict the held-out query's rows:

| signature | test rows | seen | correct |
|---|---:|---:|---:|
| query only | 43 | 0 | 0 |
| query + source identity | 43 | 0 | 0 |
| record metadata only (relation, view, entity) | 512 | 512 | **0** |
| record metadata + context shape | 512 | 512 | **0** |
| context shape only | 512 | 512 | **0** |

The metadata signatures *are* seen — the same record shapes appear under other queries — and they still predict nothing. What the query asks is carried by the query's wording, and nothing in the record metadata or the context shape substitutes for it.

## Why this matters

- A scorer keyed on the prompt reaches 100 % in-panel and **cannot answer a wording it has not seen**. That is what the query-only row shows.
- Every structural shortcut is now measured dead at 0 %, with full coverage. There is no "read the relation and emit the template" path.
- So the entry decision requires relating the query's tokens to the record's tokens and to the answer form. That is a comparison mechanism, not a lookup, and it is the same conclusion the entry-isolation pair and the emission-rank read reached from the other side: weighting is not it, Copy suppression is not it, component calibration is not it, and prompt or metadata lookup is not it.

## Also in this PR

The label-independent split. The panel alternates its two answers with index parity — rows 0, 2, 4 … are `Your job is …`, rows 1, 3, 5 … are `You live in …` — so a train/test split on position is a split on the label. The first run of this tool reported 0 % everywhere for exactly that reason; the shipped split hashes the case id.

## Verification

`cargo check -p uor-r4-training --example entry-scorer-feasibility` clean, `rustfmt --check` clean, and the run is reproducible with one command from the two panel files plus the checkpoint tokenizer (panel SHA256 `b9661606b280…` / `84991e0657b5…`, both re-verified on fetch). No model or GPU is touched by this tool.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

