# ORDER: cpu-training-pilot, deepseek-106 (artifact-bound point vote)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `35bfa2708`.

## Goal

Bind deepseek-105's eight-row point-vote screen to the integer artifact and
served session. Preserve every existing `/8` artifact byte for byte at the
default one-row behavior, then prove exact CPU, native-session and
served-forward parity for an artifact that records eight-row voting.

This is a finite schema, kernel and serving build. It runs no new training
capability comparison and makes no language result.

## Files in scope

- `crates/uor-r4-integer/src/stack/{format.rs,mod.rs,point_key.rs,session.rs,tests.rs}`
- `crates/uor-r4-integer/src/bin/fbs-export.rs`
- `crates/uor-r4-cpu-train/src/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-106/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`

## Design

- Add `vote_k` to `PointKeyImage` and `StackPointKey`, defaulting to `1`.
- Omit `vote_k = 1` from JSON and require `1 <= vote_k <= rows` when present.
- Keep `vote_k = 1` exactly on the existing least-distance, commonest-tied
  target, smaller-id rule.
- For `vote_k > 1`, order rows exactly by `(L1 distance, row index)`, count the
  targets in the first `vote_k`, and choose the commonest target, ties to the
  smaller token id.
- Use bounded session scratch. Ranking is per query over the table's declared
  row count; independent examples remain the outer parallel unit, with no
  nested Rayon pool.
- Extend `fbs-export --point-key` with an explicit optional vote width so a
  voted artifact can be built without changing the default output.

## Tests first

1. Existing point artifacts remain byte-identical at `vote_k = 1`.
2. Omitted, zero, oversized and malformed vote widths fail or default exactly
   as specified.
3. A nine-row fixture where the nearest row is wrong and the eight-row vote is
   right names the voted target in the native session.
4. Equal vote counts choose the smaller target id.
5. The independent reference scan and served native session agree for both
   default and voted tables.
6. A synthetic voted artifact reports `vote_k` in the CPU served-forward
   manifest and has exact CPU/native/audit decisions.

## Done when

1. Default artifacts are byte-identical and all existing point tests pass.
2. `vote_k = 8` is artifact-bound, deterministic, allocation-bounded in the
   session and exercised through native serving and the CPU audit.
3. The exporter builds the voted artifact and refuses malformed widths.
4. Integer and CPU package tests, strict `uor-r4-cpu-train` all-target Clippy,
   formatting, report validation, exact-head review, protected merge and
   fresh-main verification pass. The all-target `uor-r4-integer` Clippy
   baseline is recorded separately and is not changed by this unit.

## Cancel probe

- `main` is clean at `35bfa2708`.
- Claude's open #3629 changes float stream decode and does not touch these
  point-key files.
- The existing `/8` point schema is the sole artifact format to extend; no
  newer point-key format exists on `main`.
- The finite nine-row nearest-wrong/vote-right fixture already exists in the
  CPU driver.

## Stop and report

Stop on default artifact drift, nondeterministic ordering, unbounded
per-query allocation, a vote width outside the table, or any need to change
Claude's #3629 files. Do not present the eight-row vote as a capability gain:
deepseek-105 measured only a small offline mechanism delta below its
three-sigma threshold.
