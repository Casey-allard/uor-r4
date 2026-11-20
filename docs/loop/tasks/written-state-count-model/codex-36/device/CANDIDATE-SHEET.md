# D36 device readback at the C1 and full-C candidate doses

Request independent FIT and a candidate-specific GO after the candidate's
whole-context retention audit. Run the 18 retained and dropped original-ID
probes from that audit. Each probe reports both target counts, both context
totals and the Q48 mass at each causal suffix order. No fit, language score,
R input or held input is part of this readback.

Begin with C1 at the registered 268,435,456-byte compact cap, threshold 5,
and 242,774,057 occupied compact bytes. The candidate SHA-256 is
`1ba0ae5473ad8154e2144891f999c049b17eb335e0f6f106754d1cf7dbdeab81`.
The C1 audit supplies eleven retained and seven dropped probes.
Prepare full C only after it has a sealed compact artifact at or below
the deployment-derived compact allowance and an independently pinned
18-probe whole-context audit. Issue a separate FIT and GO for that artifact.

The device uses the same AOT sm_89 binary as the successful C128 check
(`9c576eba1e78807cc5fea60f55b15c5c4d1ab5c828ba882ed488b76e32b70dc3`).
It uploads actual compact bytes once per fresh process with an 8 MiB host
buffer. Device state is bounded by occupied artifact bytes plus the small
packet, output and status buffers. The launcher requires 64 MiB additional
free VRAM. For 18 probes those buffers occupy less than 8 KiB beyond the
artifact. The Rust witness maps an immutable, digest-pinned artifact
read-only. Allow up to 4 GB of resident file pages and measure actual
RSS. Only the packet and exact requested context keys have separate
host allocations. No expanded database or precomputed target answers
are passed to the device. The native comparator runs after device release.

Before dispatch, the revised witness must reproduce the legacy witness's
C128 packets and receipts from the retained hardware tapes byte for byte.
For each candidate, both fresh device processes must agree on all words and
match the native comparator at every causal order, including all Q48 words
after a dropped context. The pinned independent audit also binds each final
mass and raw trace. Both implementations must emit zeros beyond the causal
history. A mismatch leaves device qualification open and blocks that
candidate's progression to the R-only fit. Report artifact identity,
device output and receipt hashes, GPU device, runtime and peak host RSS.
