# ORDER: cpu-training-pilot, deepseek-200 (full-sequence reply boundary)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Base: `5ed64d3ca`.

## Goal

Keep deepseek-198's positive full-sequence content/depth arbiter fixed and
replace its inherited reply-only stopping memory with a boundary trained over
the same full prompt-plus-reply history. Compare full-history, reply-only,
shuffled-full, no-boundary and content-oracle stopping paths.

This is D84 CPU development research. It is exact-memory recall evidence, not
fresh-wording generalization, useful chat, serving integration or GPU
qualification.

## Why this is next

- deepseek-198 raised exact replies from 0/0 to 17/17 and body positions from
  578/578 to 847/844 while preserving local copy at ties.
- Its only combined-bar failure was premature stopping: 85/104 against a
  maximum of four.
- The stopping table was trained only on reply suffixes from the old local-copy
  trajectory. It was never conditioned on the prompt history that drives the
  improved content selector.
- A full-sequence terminal/nonterminal suffix table can distinguish the same
  reply tail in different conversation contexts and may separate endpoint from
  internal continuation.

## Fixed construction

Add an isolated full-sequence boundary module.

For each TRAIN conversation:

1. Start history with prompt tokens, then append normalized reply tokens.
2. For every internal generated reply position, count the full-history suffix
   at depths 4 through 32 as nonterminal.
3. At the final reply position, count the same full-history suffixes as
   terminal.
4. At generation, inspect the full prompt-plus-generated stream; the longest
   known suffix with terminal observations and no continuation emits BOS.
5. A shuffled control moves terminal counts across equal-depth suffix keys
   while leaving nonterminal counts fixed.

Content selection remains:

- full prompt-plus-reply suffix table;
- strict-majority confidence with at least two observations;
- global replaces local only when deeper; ties stay local.

## Arms

Per artifact seed:

- `content_full_boundary`: treatment;
- `content_reply_boundary`: deepseek-191's reply-only boundary;
- `content_shuffled_full_boundary`;
- `content_no_boundary`;
- `local_full_boundary`;
- `local_reply_boundary`;
- `writer_full_boundary`;
- `content_reference_oracle`: content body stopped at reference length.

## Data and power

- tables from TRAIN panel `tt-2711` only;
- fresh unopened `cell=dev,dev`, seed 2775, protocol 2, 128 conversations,
  generated only after FIT and GO;
- both public dual-c131 raw artifact seeds;
- 64-token generation cap;
- at least 200 reference-reachable replies per artifact seed;
- at least one full-vs-reply boundary stop change and one full-vs-shuffled
  change per artifact seed;
- fresh-process repeats of both artifact seeds.

## Bar

For both artifact seeds, treatment must:

- keep premature BOS at or below four;
- produce at least `max(5, ceil(content_oracle_exact / 2))` exact replies;
- beat reply-only and shuffled-full boundaries on exact replies by the paired
  two-sigma rule;
- keep correct body positions at or above `content_no_boundary` and
  `local_longest_32_boundary`;
- change at least one boundary decision from reply-only and one from shuffled;
- repeat exactly.

If the content oracle has fewer than ten exact bodies, the exact comparison is
`INSUFFICIENT_POWER`; stop state must still be reported. A combined pass
requires exact, premature-stop and body conditions together.

## Done when

1. The full-sequence boundary, arbitration integration, CLI and focused tests
   are implemented.
2. A FIT-reviewed pre-registration and fresh D67 GO are recorded.
3. TRAIN preflight exercises full, reply, shuffled and no-boundary controls.
4. Seed 2775 and both artifacts run with repeats.
5. REPORT, compact evidence, notebook, item/current state and research program
   record the result.
6. Reviewed merge, protected queue merge, fresh-main verification and claim
   release complete.

## Stop rule

Stop on source, artifact, panel, table, boundary, parity, determinism or
output-root drift; fewer than 200 reachable replies; a vacuous full-boundary
control; any need to edit serving, CUDA, float-training, exporter or
Claude-owned source; wall time above 30 minutes; or peak RSS above 2 GiB.

Lab: deepseek
Order: docs/loop/tasks/cpu-training-pilot/deepseek-200/ORDER.md
