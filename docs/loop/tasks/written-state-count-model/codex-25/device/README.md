# D25 contextual source CUDA fit and scorer

Build for the qualification GPU's SM. The SM89 artifact is built ahead of time with
CUDA 12.8.1 and contains native cubins.

```sh
nvcc -std=c++17 -O2 -gencode arch=compute_89,code=sm_89 device/source.cu -o d25-source
./d25-source finite finite.bank
./d25-source fit train.rows train.bank
./d25-source predict held.rows train.bank held.trace.tsv held.results.tsv
```

Each output requires a fresh path. The row ABI starts with the line `D25ROWS1`.
Each following line contains one candidate's unsigned 32-bit row identifier and
original token address, two binary eligibility bits (narrow, wide), its signed
class, 32 signed 16-bit local coordinates in [−48,48], and 32 signed 32-bit
contextual coordinates. Integer fields are whitespace-delimited. Row identifiers
are increasing across groups and addresses are strictly increasing within each
group. Narrow eligibility implies wide eligibility. A group with no candidates
uses one masked, zero-feature record at address zero. Wide-eligible TRAIN rows
carry −1 (negative) or +1 (positive); remaining TRAIN rows and every prediction
row carry zero. The panel supplies the two masks and preserves original addresses.

The device fit copies wide-eligible examples into a single immutable bank in
first-witness packet order, deduplicating exact matches of the complete
coordinate tuple and class. Opposite labels at the same coordinates remain
distinct. There is no gradient or optimizer state. The complete persisted bank
is eight ASCII bytes `D25BANK1`, a little-endian unsigned 32-bit record count
(2–8192), then exactly that many 193-byte records. Each record contains a signed
8-bit class, 32 little-endian signed 16-bit local coordinates, and 32
little-endian signed 32-bit contextual coordinates. Both labels are required.
The host writes the GPU-fitted integer values directly and checks the packet
before uploading it in a separate prediction process.

The device computes both nearest-class L1 distances with 64-bit integer
arithmetic. The local metric sums the 32 local differences. The contextual metric
adds the 32 contextual differences to that sum. The margin is the distance to
the nearest negative minus the distance to the nearest positive. Among eligible
candidates, exactly one must attain a strictly positive maximum for its address
to be selected. The four arms share the learned bank, candidate rows and
distance arithmetic.

Arm 0 is narrow/local, 1 is wide/local, 2 is narrow/contextual and 3 is
wide/contextual. Trace TSV has no header and records row identifier, original
token address, arm, eligibility bit, nearest positive distance, nearest negative
distance, and margin in candidate and arm order. Distances are present for
masked candidates as well. `NA` denotes an absent distance or a margin with an
absent operand. Result TSV has no header and records row identifier, arm,
decision (`pick`, `empty`, `missing`, `nonpositive`, or `tie`) and selected address
(`NA` for abstentions). The host prints device results and does no panel scoring.

The finite fixture checks ordered deduplication, byte-preserving reload, all four
arms, extreme 32-bit context coordinates, a +5 gold at the third candidate,
an empty group, a positive tie, a missing class, label swap, and malformed-bank
rejection. The finite command alone also runs the typed scorer on the host to
compare the raw distances and selections.
