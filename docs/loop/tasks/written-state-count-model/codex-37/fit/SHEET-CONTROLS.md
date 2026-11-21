# [codex] D37 reserved-R control fits

Use the same pinned R, full-C physical counts, count tape, D11, KN start,
96 optimizer updates and exact state/probe criteria as the identity fit.
The four controls are A0 count-only, C1 masked Pick expert with A1's
observed classes, C2 exact pair, and C3/C3s the registered permuted
code map. C3 uses splitmix64 seed 439 and preserves original BOS and EOS
code rows. Use the same original scored ordinal order in every run.

A0 uses only the first 176 buckets of the three-class shape, exactly the
count-only state embedded in the wider carrier. C1 derives its three-class
buckets and all count expert words directly from the sealed A1 optimizer
rows and masks the copy lane. It must preserve all class counts and have
zero weight in the copy lane of every bucket at all 97 captures.
C2 chooses the most recent exact pair within the original prefix. Its
copied successor may contribute only when that successor equals the held
target. C3 and C3s have one shared causal CUDA read, with the permuted native
code rule as the probe oracle, and differ only by class partition.
Run the ordinary optimizer for A0, C2, C3 and C3s. Run the separately
compiled masked optimizer for C1.

The fixed C1 finite fixture must match all 97 Rust Q12 sums, 717,024 Q32
weights and the exact final Q48 probe at bucket 527 under the oracle mask.
The real control dispatch requires the primary finite result, this masked
finite check, admitted primary identity receipt, independent FIT, explicit
GO and the same Wick device lock. Neither this producer nor its optimizer
reads DEV observations. A mismatched class, probe, count value or image
invalidates the affected arm. Complete controls permit the distinct paired
DEV-tune comparison under its admission.

C2 real reads use the standalone AOT sm_89 selector over original UORT
prefixes. Its device input consists of the original integer token words
and per-target half-open prefix bounds. The selector reads only the
bounded prefix and returns the latest matching predecessor as a relative
u32 index or an absent sentinel. The fixed 234-target device fixture
must match the shared native exact-pair rule at every ordinal. It covers
an empty framed record, latest tied match, window eviction and BOS reset.
Real preparation checks that tape's SHA-256 and compares independent
native decisions at the fixed replay ordinals. Gold IDs enter only the
later hit check.
