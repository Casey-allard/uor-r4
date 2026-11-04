# ORDER: cpu-training-pilot, deepseek-133 (generated-prefix writer)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `75112a053`.

## Goal

Test whether lawful writer-table rank-margin updates on the model's own
generated prefixes delay trajectory divergence better than reference-prefix
training. Feed every generated token back through the exact integer/FBS path,
including opening and reply memory, and align its writer query with the
reference content token at the same reply offset.

This is D84 CPU development research. It does not change integer serving,
claim useful conversation or replace GPU qualification.

## Cancel probe

Deepseek-95/97/131 train reference-prefix memory or gain geometry;
deepseek-132 classifies terminal rows after generation. None trains writer
ranking on generated prefixes. Claude PR #3664 touches only its GPU P3(a)
scoring sheet and ORDER. The public split and artifacts remain pinned.

## Fixed construction

Use 16 repeated keys, even documents train and odd documents evaluate, up to
eight examples per key, and training-derived memory. Greedily generate at most
16 content decisions. For each decision pair the live query with the reference
content token at the same offset; strip the trailing corpus EOS/BOS. Missing
horizon decisions after an early BOS count as wrong.

Reuse the existing K=1 exact BVG-K target/rival writer update, rebuilt from
each patched artifact. Dynamic arms recapture generated trajectories after an
accepted update; static arms reuse frozen queries. Compare frozen,
reference-prefix true, static generated-prefix true and same-cell shuffled,
and dynamic generated-prefix true and same-cell shuffled. Six arms use the same
80-attempt budget and geometry. Shuffles preserve target cells and counts and
must change targets.

## Eligibility preflight

Before training, require on each seed at least 640 admitted generated training
decisions with nonzero margin loss, spanning at least 16 prompts and 8 target
cells, with at least 64 same-cell shuffle pairs. Otherwise report an
insufficiency result and make no capability claim.

## Bar

Both seeds must show on 103 held-out prompts:

- dynamic true improves fixed-horizon summed matching prefix depth over frozen
  and dynamic shuffled, with paired threshold
  `max(2, ceil(3 * sqrt(b + c)))` and positive total depth;
- dynamic true aligned generated top-1 exceeds both controls;
- full teacher-forced content top-1 does not fall;
- all six arms retain exact CPU/integer query, token and kept-cell parity;
  and
- seed-1 repeat matches every non-timing field and retained artifact.

Report exact replies, stops, opening accuracy, first divergence, admission,
distinct/repeated replies, workers, wall and RSS.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `docs/harness/claims.json`
- `docs/harness/prep/cpu-training-pilot-generated-prefix-2026-11-04.md`
- `docs/labs/cpu-training-pilot-generated-prefix-2026-11-04/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-133/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`

No serving, integer-session, CUDA, Claude-owned or float-training source
changes.

## Done when

1. A CPU-only scanner/trainer exposes live-prefix rows, fixed horizons,
   same-cell shuffles, static/dynamic modes and reused BVG-K.
2. Tests cover memory semantics, query divergence, premature BOS, shuffle
   validity, refusals, reparse and exact parity.
3. FIT and a fresh non-author GO precede two seeds and a seed-1 repeat.
4. REPORT, compact hashed evidence, notebook, roadmap and all loop gates pass;
   exact-head review, protected merge and fresh-main verification complete.

## Stop and ask if

- The preflight cannot exercise the mechanism or a shuffle is vacuous.
- A candidate loses exact parity or the teacher-forced non-loss guard.
- The repeat differs or source/input identity drifts.
- The work requires serving, integer-session, CUDA or Claude-owned files.
