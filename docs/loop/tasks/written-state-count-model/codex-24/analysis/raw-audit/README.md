# Independent CUDA regrade

The Rust verifier reconstructs all three rational L1 distances independently with signed 128-bit cross-products. It reads the original GPU bank and scored coordinates without fitting a prototype. It checks the GPU coordinate tape against the sealed Rust projector for every row, compares the bank, coordinates and predictions with each corresponding cold run, and verifies 33,792 GPU distance and decision words over both artifacts and both arms. The artifact paths yield identical role evidence within each arm. Held role grades exclude the 32 unresolved prompts.

| Measure | Identity | Positional |
|---|---:|---:|
| Exact requested set, 1,024 prompts | 320 | 192 |
| Exact requested set, 640 gold-positive prompts | 96 | 96 |
| False positive requests, 384 gold-empty prompts | 160 | 288 |
| All mention classes correct, 1,024 prompts | 256 | 96 |
| Exact sets with changed cues, 512 prompts | 96 | 96 |
| Exact sets with shared cues, 512 prompts | 224 | 96 |
| Ties, 2,112 raw scored mentions including unresolved | 196 | 400 |
| Distinct vectors, 2,048 unambiguous mentions | 11 | 12 |
| Mentions sharing a vector with a different class, in unspecified / requested / excluded order | 0 / 192 / 128 | 0 / 0 / 0 |
| Maximum correct classes from any fixed vector-only rule, 2,048 mentions | 1,920 | 2,048 |
| Maximum exact sets from any fixed vector-only request rule, 1,024 prompts | 896 | 1,024 |

The class ceiling sums the largest gold class count for each distinct vector. The exact-set ceiling enumerates every possible binary assignment to the 11 or 12 distinct vectors and counts the greatest number of simultaneously correct prompt sets. The verifier publishes no assignment and trains nothing. These are held-panel bounds and establish neither transfer nor the capabilities of a fitted decoder.

On the 64 matched same-person polarity pairs in the root-order family, all 64 shared the same local root multiset. Identity gave an identical vector on all 64 pairs. Positional gave different vectors on all 64 pairs. Both fitted arms got both decisions right in zero pairs. For the match using “choose to keep instead of drop Emily” and “choose to drop instead of keep Emily,” the intended roles were requested and excluded. Identity returned requested twice. Positional returned a tie and then excluded.

On the changed-cue panel, Identity's 96 exact sets all occur in the near formulation. The Positional arm's 96 all occur in the right-context formulation. On the shared-cue panel, Identity scored 96/256 on the far formulation and 128/256 on root order. Positional scored 0/256 and 96/256 respectively. These are descriptive counts on four fixed wording realizations.

From this directory, run the verifier against the restored D24 evidence directory containing `panel/`, `panel-seed2/` and `result/`.

```sh
cargo run --locked --quiet -- /absolute/path/to/clause-role-24
```

The JSON output includes confusion matrices, grouped grades, raw-input hashes and one paired trace. Its SHA-256 for the reviewed evidence is `220d03695c8e1a470065ff42c1cd8a33e64ba7e8c4dfa832587613213e4e1c9e`.
