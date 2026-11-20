# [codex] Non-author FIT for the finite resident count batch

TEST FITNESS: FIT

The finite readback establishes that a separately pinned host batch adapter can
feed original-ID causal windows to the already qualified compact CUDA lookup and
write lossless target features. This is a prerequisite for a later R-only
mixture fit against the deployed pruned representation. This review admits only
the four fresh C128 fixture processes in the [finite sheet](SHEET.md), after a
separate top-session GO and Wick GPU lock admission.

I checked the exact source manifest
`98b9ed244394c45a90def5a59e265b95b1695e2b7ac0afd64dddee410cab979f`
from the Wick source worktree and the exact build/input manifest
`639e35806533015e5ef30e654c942145e6daa4f54bc668f2731dc60120302874`
from the Wick stage. The launcher digest is
`d7aeeaf1859e5853bf961ab1bde8f64ba37d61ff4e6923a241518281dc2f9fd3`.
The exact sm_89 cubin
`a29613d9a0c6e3a6fcda13e09c489685b4e70c4e9a11c190f4b8ca389443f6b4`
contains the same compact lookup entrypoint used in the prior physical C128
replay. Independent disassembly showed integer and control opcodes only.
The runner links libcuda with no NVRTC or BLAS library. The loaded cubin is
digest-checked, and the CUDA path transfers the actual compact file once.
The same streamed bytes feed the artifact digest and header-fragment checks
before the first lookup launch. The GPU invocation carries no oracle
features.

The 30-ID UORT fixture contains 26 non-BOS targets across four records.
I reran the native oracle independently on Wick. Its complete 3,752-byte
tape matched the planned digest
`5ab0805021b32409e5232420b2b9b2ab219b69c455115990ee3b2b5427595a1e`
and the staged tape byte for byte. The dispatch's discriminator is exact
agreement across all 26 rows and all 23 words per row at batch widths 1, 3,
10 and 26. The expected launch counts are respectively 26, 9, 3 and 1,
with one 4,517,125-byte upload per fresh process. A mismatch is a batch
instrument failure and blocks review of R-feature generation.

I rebuilt the reviewed source on Wick and all five host tests passed. I
then rebuilt two separate planted source variants with the unchanged
tests. The variants give direct probe-word sensitivity:

| Planted error | Exact observed rejection |
|---|---|
| Ignore the record root when clipping to ten preceding IDs | The first token after EOS leaked `[0,10,11,1,0,13]` in place of `[0,13]`. The record seam test failed. |
| Reset causal history at every batch fill | The first target after an interior cut lost IDs 2–11 and emitted target 12 with zero padding. The ten-history seam test failed. |

The direct word tests matter because a longer history containing a prior
record can still fall back to the same numeric Q48 mass. The source tests
also pin the empty BOS/EOS record, truncation to exactly ten previous IDs,
padding, malformed framing, digest and uploaded-header rejection. The
review captures and isolated mutant diffs remain on Wick at
`/mnt/plexpool/uor-r4-corpus/runs/20261120-whole-context-36/batch/review`.
Their checked manifest digest is
`e70062d2974b83078c38d598a24e908e9ec296069863b8e92ebfae226715e9ef`.

Wick RTX 4090 admission must bind UUID
`GPU-7c813ad0-2151-bb1d-5efb-a3340b9a6a41`, reject active compute under
the common lock and set `CUDA_DISABLE_PTX_JIT=1`. This FIT is for finite
feature serialization and lookup parity. Full-size R input, fitting,
language scoring and held data require separate review and GO.
