# User-text-only requested occurrence

The opt-in D11 schema `uor-r4.lut-discrete-requested-occurrence/2` carries
`requested_policy: {"user_text_only": <protocol-2 framing>}`. The D19 baseline
uses `uor-r4.lut-discrete-requested-occurrence/1` and its existing
`assertion_agreement` policy. The packer derives the same single-turn framing
from the frozen carrier's tokenizer for both modes. The onset bank, span state
and paired carrier are embedded byte for byte and verified at load. Either
requested policy requires the ordered value profile. The loader checks the
schema against the policy tag. A pre-D20 reader rejects the new schema.

The treatment reads the same six role feature offsets at a candidate's
absolute positions `start - 3`, `start - 2`, `start - 1`, `end`, `end + 1`
and `end + 2`. When an offset falls outside the half-open protocol-2 user
content, its root becomes 65,536, the existing missing-context sentinel. The
same frozen sparse cells and signed integer scoring apply to those keys. The
requested contrast uses both scoped role scores. The independent excluded-role
decision uses the original full-prefix scores, including its runner-up and
evidence floor. Candidate enumeration, exact assertion-head agreement and the
requested winner and tie rules run on the original tape. The ordered value
reader, literal source mask, copy cursor and EOS retain the original path.
There are no learned updates.

The ordinary protocol-2 client selects the mode from the artifact. Build the
packer and client with `--features serving-wasm`. Given the original frozen
paired artifact, span state, ordered bank, profile and bound tokenizer:

```bash
pack-all-starts "$paired" "$span" "$bank" "$profile" "$out" \
  --requested-policy user_text_only "$tokenizer"
```

The D19 Rust driver and CUDA device executable use their original packet ABI.
Mode 0 is the legacy ranker, mode 1 is D19 assertion agreement on full-prefix
keys, and mode 2 is D20 assertion agreement on user-text keys. Older device
executables reject mode 2. The driver chooses the mode from the D11 artifact
and accepts a JSON list of blind records with a `prefix` field containing
original token IDs. The receipt records the selected policy and compares all
eight device role words to Rust. The host census separately reranks the
candidates and checks the same eight words. It records each absolute candidate
span, supporting head, eligibility, effective six feature keys and the two
scores used by requested contrast. Where those keys differ from the original,
`excluded_decision_scores` retains both full-prefix scores used for the
excluded decision.

```bash
operator=docs/loop/tasks/written-state-count-model/codex-19/operator
cargo build --release --locked --manifest-path "$operator/Cargo.toml" --target-dir "$build"
nvcc -O3 -std=c++17 -gencode arch=compute_89,code=sm_89 "$operator/device.cu" -o "$device"
"$build/release/requested-role-device" "$artifact" "$blind" "$device" "$role_receipt"
"$build/release/requested-role-device" --finite "$device" "$d19_finite_receipt"
"$build/release/requested-role-device" --finite-user-text-only "$device" "$d20_finite_receipt"
```

The new planted pair holds user content fixed at `[2,16)` and changes a
single root at absolute offset 16. D19 selects `[14,16)` with contrast 56
over 40, then `[11,13)` with contrast 40 over −56. D20 selects `[11,13)`
with contrast 40 over −12 in both rows. The −12 comes from a deliberately
nonzero missing-root cell. Both modes must agree on the four excluded-decision
words within each prompt. The CUDA finite command compares all 32 raw words
across the two modes. The D19 finite command retains its six original fixtures.
