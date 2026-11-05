# Pre-registration: exact scale-serving preflight

Run ID: `cpu-training-pilot-scale-serving-2026-11-05`

## Status

COMPLETE - PASS. Run 2 exercised the whole-table-nearest recipe in all 18
cells with zero parity mismatches, three identical decision hashes per cell,
all four negative controls refused, 18.476 s wall time and 76,388 KiB peak
RSS. The exact-head result review found that attempt 1 used the cell-writer
readout; attempt 1 and its committed copy are preserved only as a
configuration/instrument reference and are excluded from this result.

## Question

At Claude's three registered `scale-probe` widths and three representative
contexts, can the current whole-table-nearest `UORLUT01` output path export,
reload and reproduce every CPU and integer-serving decision, and what artifact
size and decode throughput does each shape impose?

This is an instrument and cost preflight. It does not train a corpus model,
qualify GPU training, measure language quality or show that a width fits GPU
memory.

## Why this is new

- `cpu-fbs-forward-bench` hardcodes width 256, MLP 384, six layers and context
  192, and does not call `audit_served_forward`.
- Existing public trained-artifact audit coverage is width 256.
- Claude's scale-probe registers widths 256, 512 and 1,024 on real text. The
  contexts below are this preflight's representative CPU choices, not limits
  registered by Claude.
- No existing CPU benchmark covers all three widths, several contexts, both a
  read-free and a full Hamming/memory artifact, the recipe's whole-table
  nearest readout, exact CPU/serving parity, repeated decision digests,
  artifact bytes and decode throughput.

The result gives the scale run an exact output-path reference. It cannot fill
the scale-probe quality slopes, whose qualification remains GPU research.

## Base artifact

There is no pre-existing trained artifact. Each cell constructs the fixed
synthetic FBS snapshot in the registered source, exports it to `UORLUT01`, and
reloads those bytes. Building the same cell twice must produce byte-identical
artifacts before any timing or parity reading is accepted.

## Data split

No corpus or response mask is read. Training and evaluation data are not used.
Each context length supplies one deterministic input-only token sequence,
`token[i] = (i * 73 + 19) % 4096`, to both arms. The full arm marks row 1 as
the reply start so the exact opening/reply memory lookup path executes.

## Source and binary

- base: `3ac40d6d5`;
- implementation source:
  `ed0d88f3fb5874728bcd087cb53b9b07f9124ab1`;
- binary:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/scale-serving/harness-target/release/cpu-scale-serving-bench`;
- binary SHA-256:
  `ccc730000376be07badab2f01f30686ee9c960473ac0040162bea1c7b325a6ac`;
- host: AMD Ryzen 9 9950X, 16 physical cores and 32 hardware threads;
- execution: one process, platform default worker availability for library
  code, no GPU.

## Fixed grid

Every cell uses vocabulary 4,096, MLP 384, six layers, eight writer cells and a
seed derived as `20_261_105 XOR (width << 17) XOR context`.

| Width | Representative contexts | Arms |
|---:|---|---|
| 256 | 16, 64, 192 | projection, full |
| 512 | 16, 64, 192 | projection, full |
| 1,024 | 16, 64, 192 | projection, full |

Both arms use the recipe's whole-table nearest writer. `projection` has no
Hamming or memory tables. `full` adds Hamming reads at depth 1 and the final
layer plus opening and reply memory. Hamming heads are `width / 64`: 4, 8 and
16 for widths 256, 512 and 1,024.

The token sequence is deterministic:

```text
token[i] = (i * 73 + 19) % 4096
```

The full arm marks a reply start at one-based row 1. The projection arm has no
reply start.

## Steps

Run only after FIT and D67 GO:

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-scale-serving
OUT=/home/callard/projects/codex-playground/uor-r4/local/deepseek/scale-serving/2026-11-05
BIN=/home/callard/projects/codex-playground/uor-r4/local/deepseek/scale-serving/harness-target/release/cpu-scale-serving-bench

test ! -e "$OUT/summary.json"
git -C "$SOURCE" merge-base --is-ancestor ed0d88f3fb5874728bcd087cb53b9b07f9124ab1 HEAD
test "$(sha256sum "$BIN" | cut -d' ' -f1)" = ccc730000376be07badab2f01f30686ee9c960473ac0040162bea1c7b325a6ac
mkdir -p "$OUT"
timeout 1200s "$BIN" --output "$OUT/summary.json"
```

The output path must be absent before the run. If the command fails, preserve
the error and do not reuse the root.

## Minimum power and required readings

- 3 widths x 3 contexts x 2 arms = **18 cells**;
- **three** complete exact-parity repeats per cell;
- every artifact rebuilt from the same seed and byte-identical;
- zero query-coordinate, token and kept-cell mismatches;
- CPU and serving decision hashes equal in every repeat;
- every repeat's CPU decision hash is retained in the output;
- complete grid coverage:
  - `positions_checked = context`;
  - `query_coordinates_compared = context * width`;
  - `token_decisions_compared = context`;
  - `kept_cells_compared = context * 8`;
- artifact SHA-256 and bytes;
- reloaded vocabulary, width, context, MLP, layer and kept-cell shape, and the
  `whole_table_nearest_l1` readout mode;
- projection artifacts carry neither Hamming nor memory tables;
- full artifacts retain their expected Hamming heads/depths and nonempty
  opening/reply memory at `key_window == context`;
- serialized table-payload and decoded-resident bytes;
- CPU-oracle and integer-serving tokens/s from the timed final repeat;
- peak RSS and total wall time.

Negative controls:

1. width 2,048 is refused by the writer cap;
2. a truncated artifact is refused;
3. a token outside vocabulary is refused;
4. a token sequence longer than context is refused.

## Bar

The preflight passes only if:

- all 18 cells complete;
- every exact-parity and decision-hash check repeats;
- all four negative controls fail closed;
- peak RSS is below 2 GiB; exactly 2 GiB fails;
- total wall time is at most 20 minutes.

## Outcomes

- **PASS:** all registered widths and contexts use the same exact output path,
  and the measured byte and throughput tables are available to Claude's scale
  run.
- **INSTRUMENT_FAULT:** any parity, repeat, shape, control, timeout or RSS check
  fails. Do not interpret it as a language result.
- **UNSUPPORTED:** a registered width/context cannot be exported or parsed by
  the current path after the instrument class is ruled out.

No width is retuned or removed after the result. A failure closes this exact
preflight configuration and identifies the output-path defect.

## Cost and stop

- local CPU only, $0;
- one run, at most 20 minutes;
- stop at or above 2 GiB peak RSS;
- stop on source, binary, shape, parity, determinism, control or output-root
  drift;
- do not edit Claude-owned float-training, serving or exporter source.

## Claim boundary

The result covers synthetic structural artifacts at the registered widths and
contexts only. It does not establish real-text quality, corpus training,
GPU-memory feasibility, a scale slope or model qualification.

## Attempt 1

The first run exercised the synthetic default cell-writer readout. It passed
its mechanical checks, but exact-head review found that it did not cover the
current recipe's whole-table nearest readout. Its output is preserved outside
the repository and as committed attempt-1 evidence; it is not used for the
current claim.

## Verdict

TEST FITNESS: FIT - M2 #2030
sheet: docs/harness/prep/cpu-training-pilot-scale-serving-2026-11-05.md
reviewer: deepseek fresh-context subagent scale_serving_run2_fit
note: No pre-run blockers at 50f0d7d76. The reviewer verified that run 2
  exercises whole-table nearest, the contexts are representative rather than
  scale-probe registrations, attempt 1 is excluded, the output path is fresh,
  and both binary/source pins match.
date: 2026-11-05

A FIT verdict clears compute for the configuration in this sheet only (D22).

## Pre-run GO

GO - reviewer `deepseek fresh-context subagent scale_serving_run2_fit`,
2026-11-05, at exact head `50f0d7d76`. The reviewer confirmed the nearest
readout, 18-cell controls, shape/repeat checks, fail-closed negative controls
and RSS boundary, pins, fresh output and D84 boundary. No Claude-owned source
is changed.
