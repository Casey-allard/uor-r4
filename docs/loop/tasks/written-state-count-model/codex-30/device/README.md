# D30 earlier-context CUDA replay

This unit loads the frozen D26 recurrence bank. It does not fit held rows.
The CUDA context projector, local projector and signed exemplar scorer use
the D25 and D24 source paths. The installed runtime is the oracle for
recurrence words and nearest-class distances. The held CUDA arms are freshly
launched twice from the same binaries and byte-identical inputs.

The D30 helper creates one cumulative prefix record per original token,
including BOS, for each member of a reciprocal donor pair. The paired
suffix must be identical from the original factual mention through the
end of the prompt. In the trajectory index, offset zero ends immediately
before that mention. Positive offsets count the number of shared suffix
tokens consumed. Initial differences use equal BOS-inclusive prefix lengths.
Later retention or collapse uses equal suffix offsets. An equal state
at one offset can diverge again at another offset.

The wide local and wide recurrent scorer results are arms 1 and 3. The
nominal packet takes its 32 recurrence coordinates from the GPU and is
checked by the D26 view pack against the original D11 model and complete
Rust recurrence oracle. The reciprocal packet copies the corresponding
donor's 32 coordinates and the zero packet zeros those columns. The first
37 columns, including token address, mask and local coordinates, remain
fixed. The local projector output must match the original panel's expected
coordinates byte for byte.

## Reproduction

Build the standalone helper with Cargo and use the existing D25
`d25-context-pack` and D26 `d26-view-pack` to construct the device packets.
Set `BASE` to the original 19-table seed1 D11, `BANK` to the D26 recurrence
bank, `PANEL` to the panel directory and `OUT` to a new owned directory.

```sh
d30-context-swap prepare "$PANEL/held.grade.json" "$PANEL/held.donors.tsv" "$OUT/trajectory"
d25-context-pack "$BASE" "$PANEL/held.context-input.txt" "$OUT/project-in/held.packet"
d25-context-pack "$BASE" "$OUT/trajectory/trajectory-input.tsv" "$OUT/project-in/trajectory.packet"
```

On a leased pod, stage `dispatch.sh`, `context.cu` from D25, `discrete.cu`
from the frozen discrete source, the D15 and D24 projector and D25 scorer
sources in their original relative directories, D30's capacity source
and root words, and the finite D26 switch fixture with its zero arm. Seal
the relative paths in `build-input.sha256`. The on-pod script uses nvcc
with the actual device compute capability and records the cubins, imports
and executable hashes. Run the finite job first and compare all three
fixture traces and results with the helper's `parity` command. The frozen
D25 standalone finite cases include a positive tie and an empty group.

The project job reads `project-in` with both packets plus the local
projector table and the held local input and expected output. Authenticate
them using `project-input.sha256`. After the CUDA projection, run the
native differential and construct the intervention packets.

```sh
d30-context-swap trajectory "$BASE" "$OUT/trajectory/trajectory-input.tsv" \
  "$OUT/projection/trajectory.pre" "$OUT/trajectory"
d26-view-pack "$BASE" "$PANEL/held.context-input.txt" "$PANEL/held.rows" \
  "$OUT/projection/held.post" "$OUT/projection/held.pre" "$OUT/views"
cp "$OUT/views/recurrence.rows" "$OUT/score-in/nominal.rows"
d26-view-pack donors "$OUT/score-in/nominal.rows" "$PANEL/held.donors.tsv" "$OUT/score-in/swap.rows"
d30-context-swap zero "$OUT/score-in/nominal.rows" "$OUT/score-in/zero.rows"
```

Stage the score packets and frozen bank with `score-input.sha256`. The
score job emits the original D25 trace and decisions for each arm and the
exhaustive native-code capacity map. The helper compares the entire score
trace and all source decisions back to the Rust `SourceBank` in each arm.

```sh
for ctl in nominal swap zero; do
  d30-context-swap parity "$OUT/score-in/$ctl.rows" "$BANK" \
    "$OUT/score/$ctl.trace" "$OUT/score/$ctl.results"
done
```
