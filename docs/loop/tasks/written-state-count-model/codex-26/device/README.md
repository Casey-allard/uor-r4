# D26 pre-gate source view packing

The two CUDA executables come from D25. The forensic context executable
replays the original BOS-inclusive token prefix ending before the supplied
assertion. It writes the post-gate vector and a second raw trace with the
pre-gate Relation coordinates and packed recurrence roots. The source
executable fits and scores the same D25 signed L1 instrument. D26 selects
arm 3 for each wide contextual view and arm 1 for the wide local reference.
Each contextual view uses its own TRAIN exemplar bank. The local arm derives
its nearest distances solely from the original signed local coordinates.

## Build and reproduce

Use the D25 [context](../../codex-25/context/README.md) and
[source](../../codex-25/device/README.md) build commands for the qualification
GPU or verify the D25 executables against their receipts. For the Wick RTX
4090 readback, the forensic SM89 context executable SHA-256 is
`7b83cc81c246e724d054e5ec525775154cb4a98f43c67070e204af526015784a`. The
source executable SHA-256 is
`57608d548642b8cca0005678e6dff0b9190781c2012882e84ed7184292e5f7dc`.
The original 19-table seed1 D11 SHA-256 is
`04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d`.

From the repository root, set `Q` to the owned scratch directory, `BASE` to the
original 19-table D11, `PANEL` to the panel output directory and `split` to the
name of a split:

```sh
DEVICE=docs/loop/tasks/written-state-count-model/codex-26/device
CARGO_TARGET_DIR="$Q/target-views" cargo build --locked --release --manifest-path "$DEVICE/Cargo.toml"
PACK="$Q/target-views/release/d26-view-pack"
d25-context-pack "$BASE" "$PANEL/$split.context-input.txt" "$Q/$split.ctx"
# Run the device command on the GPU host and retain both outputs.
d25-context "$Q/$split.ctx" "$Q/$split.post.tsv" "$Q/$split.trace.tsv"
"$PACK" "$BASE" "$PANEL/$split.context-input.txt" "$PANEL/$split.rows" \
  "$Q/$split.post.tsv" "$Q/$split.trace.tsv" "$Q/$split.views"
```

The packer checks exact original prefix sequence metadata, row coverage and
all 32 post-gate words against the original panel packet. It checks the 32
GPU Relation words against both the packed Relation roots and the ordinary
integer Rust Relation read over the same original codebook, writer cells,
routes, gates and prefix. For this oracle it loads the four base tables
into the existing four-table D11 schema, so the ordinary Relation reader
returns the vector before the optional trunk gate. Separately the existing
Rust discrete recurrence computes both the normalized 32-coordinate state and
its packed code from the original IDs, codebook roots, step actions and
normalization frames. Both must equal the GPU observations bit for bit. All
host replay is differential validation. Fit, score and nearest-class
selection execute on the device.

`post.rows`, `relation.rows` and `recurrence.rows` preserve each candidate's
address, masks, label and local coordinates. The only substituted fields
are the shared 32 contextual coordinates of anchored rows. A packed
recurrence code expands through the native 16-root orbit table in
least-significant-nibble-first order, four signed 8D blocks at the original
integer scale. The packed code never enters the metric as a scalar.
Perform the separate D24 local-coordinate check with the D25 projector and
compare all projected integers to the panel's original expected packet.

For the wrong-donor arm, use an explicit no-header TSV with a target row ID
and donor row ID on each line. The mapping must be a permutation of its
participating prediction rows. Unlisted rows, including empty controls,
retain their existing features. Score this packet with the original bank
from the same view.

```sh
"$PACK" donors "$Q/held.views/relation.rows" "$PANEL/held.donors.tsv" "$Q/held.relation-donors.rows"
"$PACK" donors "$Q/held.views/recurrence.rows" "$PANEL/held.donors.tsv" "$Q/held.recurrence-donors.rows"
```

## Finite control

The additional fixture has two candidate positions in each of a pair of
rows. Margins at both positions are zero with the local metric. In context
the two rows pick opposite positions. A wrong donor changes each selection
to the other's address while preserving the local abstentions. Use a fresh
directory. Only the scorer runs on the GPU:

```sh
"$PACK" finite "$Q/finite"
cd "$Q/finite"
d25-source fit train.rows finite.bank
d25-source predict held.rows finite.bank native.trace.tsv native.results.tsv
d25-source predict held-donors.rows finite.bank donors.trace.tsv donors.results.tsv
cmp native.expected.tsv native.results.tsv
cmp donors.expected.tsv donors.results.tsv
```

Also run D25's own `d25-source finite` control for ties, malformed banks,
label swaps, missing classes and original device/host scorer comparisons.
