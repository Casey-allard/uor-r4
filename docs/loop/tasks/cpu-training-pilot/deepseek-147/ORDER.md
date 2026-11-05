# ORDER: cpu-training-pilot, deepseek-147 (trajectory-density termination)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `6c21d3133`.

## Goal

Test whether the current modulo-256 content code plus the non-vacuous
phase-aligned transition separates generated reply endpoints from ordinary
continuation under the unchanged termination-density rule.

This is D84 CPU development research. It does not change integer serving,
`UORLUT01`, CUDA, float training or Claude-owned source.

## Fixed construction

At generated offset `t`, retain the current 2,048-bit modulo-256 code and the
2,048-bit phase-aligned relative transition:

```text
phase(q_t) = popcount_mod_64(previous_code(q_(t-1)))
transition_t = rotate_left_64(current_code(q_t), phase(q_t)) XOR previous_code(q_(t-1))
termination_t = current_code(q_t) || transition_t
```

Offset zero has no predecessor and uses an all-zero transition. Content
selection continues to use only the current code. Fit the existing generated-row
density classifier with the unchanged maximum neighbour set `{1,2,4,8}` and
offset radius `2`. Use a class-preserving shuffled terminal-label control.

## Cancel probe

Deepseek-142 closed current-code density at 19/84 terminal recall against a
26/84 floor. Deepseek-146 closed phase-transition density at 14/84. Deepseek-144
and deepseek-145 closed current-plus-XOR content readout, not termination.
Neither termination arm combined the surviving current state with the
phase-aligned transition.

## Preflight

Run seed 1 first with `--stop-trajectory-neighbors 8`. Stop before seed 2 if:

- generated terminal rows are below 84 or nonterminal rows below 1,484;
- true combined density recalls fewer than 26/84 terminal rows;
- true combined density emits more than 14/1,484 false stops;
- the shuffled-label gate produces no changed leave-one-out firing decision;
- the ungated content-prefix audit has a mismatch; or
- source, public inputs, split, combined-signature width or history differ.

If seed 1 passes, run seed 2 and the complete public generation comparison.

## Generation bar

Run code, combined-density and combined-density-shuffled arms over complete
16-token replies on both seeds. A pass requires:

- combined density stops at least 26/84 training endpoints with at most
  14/1,484 false stops on each seed;
- combined density beats its shuffled terminal-label control by the prompt-level
  paired two-sigma rule;
- combined-density BOS completions beat code by the same paired two-sigma rule;
- ungated content decisions remain byte-identical to code;
- combined stopping yields at least 9 exact references and 9 BOS-complete
  replies, with at least 8 stops within the reference length;
- generated top-1 and matching depth are not below code; and
- a one-worker seed-1 repeat matches every non-timing field.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `.gitattributes`
- `docs/harness/prep/cpu-training-pilot-trajectory-density-2026-11-04.md`
- `docs/labs/cpu-training-pilot-trajectory-density-2026-11-04/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-147/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`
- `docs/integration/current-state.md`

## Done when

1. Combined-signature packing, history, width, deterministic generation,
   label-control and content-prefix tests pass.
2. FIT and a fresh GO precede the registered preflight.
3. Evidence preserves the preflight or complete result; seed 2 and generation
   remain explicitly `NOT_RUN` on a preflight stop.
4. REPORT, notebook and roadmap classify only the tested configuration.
5. Exact-head review, protected merge and fresh-main verification complete.

## Stop rules

Stop on source/input/split drift, a vacuous transition or shuffle, retuning
outside the fixed neighbour/radius contract, worker nondeterminism, resource
breach, or any need to change integer serving, CUDA, float training or
Claude-owned source.
