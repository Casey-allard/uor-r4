# [codex] Finite FIT sheet for resident compact count feature batches

The next R-only fit requires exact target features from the selected physical
compact artifact. This unit qualifies one resident AOT lookup upload and
lossless feature serialization on the planted finite fixture. Run the CUDA
cases only after a non-author FIT and a separate top-session GO. The fixture
contains 30 original IDs in four BOS/EOS records and 26 scored targets.
The 124-byte UORT digest is
`01340c7ccafc474d0651f20c5158e72d239db890d1124fe7b76eea0dd8f5f333`.
The fully scored R stream is outside this finite dispatch.

## Bound inputs and outcome

Use the 4,517,125-byte C128 threshold-two compact artifact
`b50e64c74be682f04ef1661b5599c6729ede643b005e062ffa71c0799b640f71`
and C128 TRAIN binding
`1466d3790e1c53bc4ac3982712f669967cce7b3ad1d7e0e34259568154d19cb6`.
The launcher accepts the original-ID UORT, compact bytes and external
digests. Its CUDA arguments carry the raw compact bytes, bounded section
descriptors and actual probe IDs. Expected result records are absent from
the CUDA invocation. Lookup uses the exact sm_89 image
`a29613d9a0c6e3a6fcda13e09c489685b4e70c4e9a11c190f4b8ca389443f6b4`
extracted from the previously FIT-bound D36 CUDA executable.

Use the same fixture in separate cold processes at batch widths 1, 3, 10
and 26. Require all 26 result rows and all 23 words per row to match
the [ordinary compact CPU oracle](../verify/README.md) exactly. Compare
byte-for-byte complete feature tape with SHA-256
`5ab0805021b32409e5232420b2b9b2ab219b69c455115990ee3b2b5427595a1e`.
The CUDA receipt must show exactly one artifact upload per process, each
upload totaling 4,517,125 bytes, and respectively 26, 9, 3 and 1 kernel
launches. For the planted framing controls compare the source-origin
sequence directly, including the empty BOS/EOS frame, history after EOS,
the interior long-context batch seam and unused zero padding. Reject
malformed framing, truncated or stale UORT identity, batch size zero,
altered header words during upload and an existing output. Archive the
first divergent ordinal and the partial file if a dispatched case fails.

A pass admits review for R-feature generation once the full-pruned file
is sealed and a new FIT/GO binds that exact artifact and R input. A
failure is an instrument discrepancy and requires repair before that
next review. Capture wall time and device admission for the finite case.
Full-R throughput and the fitted model require their own evidence.

## Wire and resource bounds

The producer emits the 112-byte header `UORCBF01`, little-endian u64
row count, TRAIN SHA-256, compact SHA-256 and UORT SHA-256. Each row is
140 bytes with eleven u32 raw target counts, eleven u64 raw context totals
and one u64 final KN Q48 mass. All words are little-endian in ascending
order, with absent orders zero. File order defines the global non-BOS
ordinal. Each output first writes a private exclusive `.partial`, syncs
its exact length, and publishes using a no-clobber link.

A ≤64 MiB verified UORT is decoded in host memory. The compact upload
streams in 8 MiB chunks, rechecks header records against the same bytes
and hashes the uploaded bytes before the first lookup launches. Device
resident memory is the compact byte length up to 4 decimal GB, 528 bytes
of section descriptors, at most 24,903,680 bytes of client-side batch
buffers, and a 64 MiB reserve in the live admission check. The count image
loads only on compute capability 8.9. Dispatch uses Wick's RTX 4090 under
the existing exclusive device lock, refusing other compute clients and
setting `CUDA_DISABLE_PTX_JIT=1`. No model fit or scoring is in this sheet.
