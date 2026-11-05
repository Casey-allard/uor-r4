# ORDER: cpu-training-pilot, deepseek-181 (writer-anchor export audit)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `e94afaae8`.

## Goal

Add a read-only, fail-closed audit for Claude's anchored-writer artifacts. It
must prove that Q12 export/read-back preserves every token's two 8-coordinate
port blocks at its own geometric cell roots before a writer-admission artifact
is served or compared.

## Why this is next

Claude-290 merged `train-in-space --writer-anchor 1`. Its REPORT records a gap:
the float roots are expected to survive the exporter's Q12 rounding, but no
test checks the rounded/exported table's ports. The CPU lane owns exact export
and serving parity controls. This unit closes that evidence gap without
touching Claude's float training or serving source.

## Fixed construction

1. Load a `UORLUT01` FBS artifact through the existing exact CPU parser.
2. For every vocabulary token `t`:
   - derive the writer cell `t mod 256`;
   - derive port roots `cell / 16` and `cell mod 16` from the artifact's 16
     parsed SpiralCore roots;
   - compare Q12 table columns `0..8` with the first root, and columns
     `W-8..W` with the second root.
3. Report the artifact hash, vocabulary, width, tokens checked, 16 port
   coordinates per token, mismatch count, affected tokens, affected rows and
   distinct cells represented.
4. Fail the CLI when any port mismatch exists or the artifact cannot be parsed.
5. Add `cpu-writer-anchor-audit --artifact PATH
   [--expected-artifact-sha256 HEX] [--output PATH]`.

The audit is read-only. It does not alter the artifact, writer, exporter,
serving path or training code.

## Tests

- Positive finite control: build a synthetic FBS snapshot, set both port blocks
  to the parsed roots at float integers, export it, reload it, and require zero
  mismatches.
- Negative controls: move one port coordinate, assign one token to another
  cell's roots, and require exact mismatch accounting.
- Malformed artifacts, short widths and hash drift fail closed.
- A synthetic Q12 boundary test proves `-2`, `0`, and `2` survive export
  exactly.

## Done when

1. The library audit and CLI are implemented with focused tests.
2. Both unanchored public artifacts are measured as negative controls and
   affirm that the gate rejects them; these are not model results.
3. CPU tests, strict Clippy, rustfmt, diff checks, report and loop gates pass.
4. A REPORT records the executable and its use for Claude's next anchored
   artifact; no writer-admission capability is claimed.
5. Reviewed merge, fresh-main verification and claim release complete.

## Stop rule

Stop on artifact, root, width, source, report or lane drift. Do not edit
Claude-owned float-training or serving source. Public artifacts that predate
writer anchoring must fail this new gate.
