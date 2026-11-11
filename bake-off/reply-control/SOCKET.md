# Reply-control bake-off socket

This bake-off chooses only whether to keep the product's served token or emit
EOS at the current online reply position. It does not choose a different token,
rewrite history, inspect the reference, or run a model step.

## Input

One case is the ordered subset of `raw-id-replies` control rows with the same
`case` number. Each row supplies:

- `case`, `reply_position`, `token`;
- `nearest.margin`;
- `eos.rank`;
- `memory.point_key`, `memory.movement`;
- `reads.copy_hits`, `reads.type_hits`;
- `short_cycle_period`;
- `class = opaque | reflective | permeable`.

The reference file is the panel's original sample array. Its `reference` ids
are used by the scorer after the controller returns; the controller never
receives them.

## Output

The controller returns one reply id sequence:

- it appends the current row's `token` when it keeps the token;
- it appends EOS 1 when it stops early;
- it stops after a served EOS, a short cycle under the current rule, its own
  stop decision, or 48 emitted tokens;
- it emits no other token and never emits after a stop.

## Controller interface

The online decision is:

```text
Decision::Keep | Decision::EmitEos
```

`Keep` appends `row.token`. `EmitEos` appends EOS and terminates. A served EOS
is always terminal. A short-cycle terminal keeps the token that completes the
third repetition, matching `StackChat`.

## Scoring

For each case:

- **exact complete** when the generated ids equal `reference`;
- **truncated** when the generated reply stops on an injected EOS before the
  reference's own EOS;
- **EOS position error** when the generated EOS index differs from the
  reference's EOS index;
- **kept served prefix** is the count of emitted non-EOS ids equal to the served
  prefix before the first changed decision.

The primary metric is exact complete replies. Secondary metrics are truncation
rate and EOS position error. A controller must not read the reference during
decision.
