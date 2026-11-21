# [codex] Non-author FIT for the DEV-tune count capture

TEST FITNESS: FIT

I reviewed the exact [sheet](SHEET.md) with SHA-256
`dc3dc114ecb22206c968a2a1b2d24681a3289f231bee97f685ea2abfe608fd4a`
and [dispatcher](dispatch.sh) with SHA-256
`3c4b03fb412c28698ee8f5f5ac9dfafcdfa595e1eb8ee47e7449c0d2ff26a1de`.
A separate top-session GO must bind these files, this FIT and a fresh
output. I ran host reads and shell syntax checking only.

I rehashed the exact Wick tune UORT, C420 census and both executable
files. Their hashes matched the dispatcher. The C420 census reports
`dev-tune`, 2,670 records, 10,810,087 token IDs, 10,807,417 targets and
21,928,475 target bytes. Independently decoding the 21,620,238-byte UORT
payload found 2,670 BOS and 2,670 EOS words, 10,807,417 non-BOS targets,
valid original IDs, zero malformed transitions and terminal EOS. Counting
the union of first targets, EOS, paired 65,536 seams and ordinals congruent
to 197 modulo 4,093 gave 8,308 native probes. The checked native replay
separately validates all row framing and feature ranges and compares
23 words for each selected ordinal. The retained changed-mass negative
failed at ordinal zero.

Both widths require 165 launches. Each final tape occupies
1,513,038,492 bytes. The D36 driver uses 24 bytes of probe, 352 bytes of
result and four bytes of status per row. Its eleven descriptors use
528 bytes. Charging the 3,912,334,481-byte compact payload and the
64 MiB device reserve yields 4,004,347,553 bytes at width 65,536 and
4,004,341,853 bytes at width 65,521. The dispatcher checks these exact
admission values from the process receipts and enforces the separate
pre-context 5,120 MiB GPU check, 8 GiB host memory and 8 GiB disk. Wick
readers share the exclusive GPU lock, verify the specified RTX 4090 and
abort on active compute clients. After the cold byte comparison and the
native parity check, the new output directory receives the receipt hash
manifest. A failed invocation leaves its evidence in that fresh directory.

The tested dependencies are the deployed compact table, original tune
IDs, the C420 census, the same admitted D36 AOT resident reader and the
D37 native checker. The scripts never supply fitter state, observed arm
scores or DEV-test IDs. This admission captures reusable tune count
features. Paired class selection and sealed DEV-test each require their
own admission.
