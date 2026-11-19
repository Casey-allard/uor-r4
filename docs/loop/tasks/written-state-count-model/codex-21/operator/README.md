# Clause-to-entity requested role

D21 adds the opt-in D11 schema `uor-r4.lut-discrete-requested-occurrence/3`.
The `clause_to_entity` policy stores protocol-2 framing and the comma and
semicolon token IDs obtained from the frozen tokenizer in that order. The
envelope binds the original paired parent, span state and ordered value
bank. They remain byte-identical across policies. The old schemas keep
their original role policies and serialized bytes.

The original assertion-agreement rule locates the final query sentence and
finds its exact assertion-supported mentions on the original tape. The new
policy partitions that sentence at the packed clause separators. A clause
transfers evidence only if it contains exactly one assertion-supported
mention. Its evidence uses every token root in the clause outside the
mention's half-open span, skipping the original sentence delimiters.
Each remaining root addresses the six frozen role-offset cells. If a cell
has total count `n` and class count `c`, its evidence for that class is
`(16*c - 3*n)/(n + 2)`, using signed integer division toward zero.
Each offset retains its original weight `[1, 2, 4, 4, 2, 1]`. Missing
cells contribute zero. The operator sums the two class scores over the
clause and passes their requested-minus-excluded contrast to the supported
mention. The existing D19 rule chooses a unique highest contrast and allows
negative winners.

Permuting the non-mention token roots within a clause preserves its pooled
evidence. Moving that clause to the other clause position also preserves
its evidence. Entity IDs determine assertion agreement and the endpoints
of the transfer. The token tape supplies punctuation boundaries. A
clause's position, the entity identity and answer or source labels supply
no sign or rank weight. The original full-prefix excluded rank still runs
independently. The ordinary ordered value reader, source mask, original
span cursor and EOS use the same code and frozen state. There is no
optimizer step.

```bash
pack-all-starts "$paired" "$span" "$bank" "$profile" "$out" \
  --requested-policy clause_to_entity "$tokenizer"
operator=docs/loop/tasks/written-state-count-model/codex-19/operator
cargo build --release --locked --manifest-path "$operator/Cargo.toml" --target-dir "$build"
nvcc -O3 -std=c++17 -gencode arch=compute_89,code=sm_89 "$operator/device.cu" -o "$device"
"$build/release/requested-role-device" --finite-clause-to-entity "$device" "$finite_receipt"
"$build/release/requested-role-device" "$out" "$blind" "$device" "$role_receipt"
```

Build the ordinary packer and client with `--features serving-wasm`.
The Rust driver marks site flags for sentence and clause fences in the
original four-word site record. Packet mode 3 enables clause fences. The
device checks all eight output words against the ordinary Rust decision.
The host census records supporting heads, local six-cell evidence, and the
clause bounds, pooled class scores and multiple-mention status used to
re-rank the requested role. The planted fixture covers both identities in
both clause positions with the signed evidence outside the original local
window. It also checks positive ties, negative-only evidence, a shared
clause and a positive clause without an eligible entity. In the latter
case the sole supported mention retains its negative contrast and wins
under the inherited rank rule. The four
independent excluded-decision words must agree with D19 on each fixture.
