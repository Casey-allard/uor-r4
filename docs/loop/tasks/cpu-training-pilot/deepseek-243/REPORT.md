# REPORT: cpu-training-pilot, deepseek-243

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](./ORDER.md). Base: `229739cf9`.

**At the opening, at least one retained position from each value span carries
nonzero weight, and the scalar margin weights do not favor the asked role.**
Across 1,024 first-reply decisions, the chosen token was neither value's first
token in **1,024/1,024** rows. The four type heads gave nonzero weight to at
least one position in both value spans in **873-992/1,024** rows. Their
asked/other totals differed by **-0.62%**; pooling all eight heads gives
**-0.51%**. The copy heads were much more selective: neither copy head 1
weighted both spans in the same row, and depth-6 copy head 1 weighted neither.
This is an attribution result, not a causal selection or capability result.

## What did not work, first

- **Retaining a position is not weighting it.** Of **131,072** kept positions,
  **40,476 (30.89%)** received zero Q31 margin weight.
- **Neither copy head 1 weighted both values in the same row.**
  Depth-2 copy head 1 kept both spans in 906/1,024 rows but weighted both in
  0; it weighted asked and other separately in 48 rows each. Depth-6 copy head
  1 kept both in 973/1,024 and weighted neither from either span. Per seed,
  depth-2 mean distance was 19.53/15.21 bits against margins 14.52/15.51;
  depth-6 was 19.86/14.92 against margins 9.14/9.21.
- **The remaining copy heads were only partly active.** Depth-2 copy head 0
  weighted both spans in 486/897 retained-both rows; depth-6 copy head 0
  weighted both in 480/978. Its per-seed mean distance was 9.61/16.17 bits at
  depth 2 against margins 14.48/16.18, and 6.87/19.19 bits at depth 6 against
  margins 9.18/9.43.
- **Type heads did not distinguish the roles.** Depth-2 type heads weighted both
  spans in 992/1,024 and 928/1,024 rows; depth-6 type heads did so in 929 and
  873 rows. Pooled asked/other weight totals were close, not role-selective:
  1,816,808,827,047 / 1,809,408,881,476 at depth-2 head 2,
  1,256,728,859,470 / 1,287,764,390,380 at depth-2 head 3,
  1,634,401,937,021 / 1,639,533,651,144 at depth-6 head 2, and
  1,509,786,509,955 / 1,520,096,022,295 at depth-6 head 3.
- **The first-token subset shows the same pattern.** Each depth-2 copy head was
  active at the kept asked and other first-token positions in only 16 rows each;
  type heads reached 104/93, 166/219, 411/411 and 60/95. The larger weighted
  support still did not make either first token the final choice.

## What changed

- Added an opt-in trace to the exact CPU Hamming port. It records each kept
  position's distance, its head's Q16 margin, and the exact Q31
  `max(0, margin - distance) / max(margin, 1)` selector weight.
- The trace remains off by default. A regression confirms that tracing changes
  no step output, that CPU query, key and kept positions equal the
  integer-session trace, and that every recorded weight equals the serving
  formula.
- Extended `cpu-first-reply-hamming-trace` to schema 2, preserving the chosen
  token, renderer, artifact, panel, span and executable controls while adding
  per-position metrics and per-head/per-stratum weighted-support totals.
- Changed 4 code files plus task records, a resource ledger and the
  notebook. Policy classifies the 11 changes as 4 code files (+655/-64),
  3 docs files (+232/-0), and 4 other files (+1,561/-0); tests are inline.
  The ORDER gave no line estimate.

## Measurements

Artifacts: Claude-322 seed 1
`86e51d5ee2c00a4041bcde4d2183884e351099f28f155fb1e6a7516f13064dfe`
and seed 2
`8838cd1044ae7152934d8ec09016e268f2dabb0c702dd45e6ab06d7b5198bdae`.
The final executable is
`78ae6595c7407a405592810acbbad9577691be6b66029b17801bd38b1a6ed4f1`.

Each command was the built `cpu-first-reply-hamming-trace` with
`RAYON_NUM_THREADS=32`, the pinned tokenizer and panel files, and all six
expected hashes. Four seed/rendering reports and one seed-1 one-user repeat
were retained.

| Read | Head | Kind | Margin bits s1 / s2 | Mean dist. bits s1 / s2 | Active positions / kept | Active rows asked / other | Asked weight | Other weight |
|---|---:|---|---:|---:|---:|---:|---:|---:|
| 2 | 0 | copy | 14.48 / 16.18 | 9.61 / 16.17 | 11,360 / 16,384 | 546 / 548 | 736,407,628,496 | 736,336,212,095 |
| 2 | 1 | copy | 14.52 / 15.51 | 19.53 / 15.21 | 3,592 / 16,384 | 48 / 48 | 5,621,811,920 | 5,621,811,920 |
| 2 | 2 | type | 36.65 / 36.87 | 28.56 / 27.20 | 16,384 / 16,384 | 1,008 / 1,008 | 1,816,808,827,047 | 1,809,408,881,476 |
| 2 | 3 | type | 36.67 / 36.51 | 28.64 / 27.79 | 16,384 / 16,384 | 972 / 980 | 1,256,728,859,470 | 1,287,764,390,380 |
| 6 | 0 | copy | 9.18 / 9.43 | 6.87 / 19.19 | 8,060 / 16,384 | 496 / 496 | 629,008,514,168 | 628,883,551,244 |
| 6 | 1 | copy | 9.14 / 9.21 | 19.86 / 14.92 | 2,048 / 16,384 | 0 / 0 | 0 | 0 |
| 6 | 2 | type | 38.09 / 37.36 | 27.34 / 27.48 | 16,384 / 16,384 | 973 / 980 | 1,634,401,937,021 | 1,639,533,651,144 |
| 6 | 3 | type | 38.00 / 36.99 | 25.91 / 27.91 | 16,384 / 16,384 | 946 / 943 | 1,509,786,509,955 | 1,520,096,022,295 |

Pooled active support is 90,596/131,072 positions (69.11%); zero-weight support
is 40,476. Asked and other total weights are 7,588,764,088,077 and
7,627,644,520,554, a **-0.51%** asked-minus-other difference. Exact per-stratum
counts are in [`per-stratum.json`](./evidence/per-stratum.json); run hashes,
semantics and totals are in [`summary.json`](./evidence/summary.json).

The final four seed/rendering reports took **3.567 s**; the separate repeat
took 0.988 s. All 25 retained reports consumed 22.752 s of measured probe time,
with 157,560 KiB maximum peak RSS. The trace adds a second CPU Hamming selection
pass, so it is slower than deepseek-242's position-only trace but remains
sub-second per full panel.

Resources are recorded in [`resources.json`](./evidence/resources.json) and the
[D84 ledger](../../../../labs/cpu-training-pilot-first-reply-hamming-weight-2026-11-08/resource-ledger.json).
The previous admitted total 100,367.677487699 seconds plus this unit's
1,800-second projection gives 102,167.677487699 seconds.

## Checks run at the head

- `cargo test --offline --locked -p uor-r4-cpu-train --bin
  cpu-first-reply-hamming-trace`: 8 passed.
- The binary tests cover schema-2 tallies, exact renderer parity, ambiguity
  refusal, EOS exclusion, trace isolation and fail-closed metadata hashes.
- `cargo test --offline --locked -p uor-r4-cpu-train --lib
  fbs::tests::hamming_trace_is_observation_only_and_accounts_exact_weights`:
  passed; the complete library run has 456 passed and 4 ignored.
- `cargo test --release --offline --locked --features cpu-parity --bin
  served-reply-panel`: 15 passed.
- `cargo check`, targeted `rustfmt --check`, driver Clippy with no
  changed-code warnings, `git diff --check` and report check pass.
- Local policy passes with advisory commit-order/filemap warnings; all 11 full
  loop gates are green.

## Skills used

No system-provided Codex skill applied. The repository's CPU-research and
delivery rules were followed directly.

## Still open

- This reports selector weight, not the signed downstream residual contribution
  after payload means, channel weights, cancellation and readout. The next
  measurement should retain the per-channel contribution if that distinction
  remains decision-relevant.
- The trace still cannot show that a weighted position caused the final token.
  Claude-328's trained query-recall probe remains the causal test.
- Run the same schema-2 trace on Claude-323 and Claude-325 integer artifacts
  when their local served files are available.
- The free-generation float/integer whole-reply gap remains separate.

## Plain summary

The model's Hamming reads contain a short list of earlier token positions that
look geometrically similar to the current query. "Retained" previously meant
only that a position made that short list. This unit also records how far each
position was from its head's query, the head's cutoff threshold or margin, and
the resulting integer weight. Some retained positions have zero weight and
therefore contribute nothing through that head.

The result is mixed but clear. Two copy heads barely clear their thresholds,
and neither depth-level copy head 1 weights both value spans together. The deep
copy head contributes nothing from either value; the depth-2 head contributes
only sparsely and never to both together. The type heads have at least one
nonzero-weight position in both the asked and other spans almost everywhere,
with nearly equal total weights. So the model is not failing merely because
value positions are absent or unweighted. It still fails to turn that balanced
positional evidence into a role-correct first token.
