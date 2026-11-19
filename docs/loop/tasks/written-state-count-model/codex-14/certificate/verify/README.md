# Independent P1(c) certificate verifier

Run after the independent fit verifier has validated the sealed frame and fit with its query and fit checks. Use the same frame manifest, fitter, fit directory and verifier output passed to the certificate orchestrator.

```sh
CARGO_TARGET_DIR="$scratch/certificate-verify" cargo build --release --locked \
  --manifest-path "$verify_dir/Cargo.toml"

"$scratch/certificate-verify/release/kn-ceiling-certificate-verify" \
  "$frame_dir" "$verified_fit_json" "$fit_dir" "$fitter" \
  "$certificate_dir/device-certificate.json" "$cuda_worker" "$certificate_host" \
  --reverify "$primary_verifier" "$collector" "$query_inputs"
```

The device receipt and its two executable paths are optional as a group. Omitting them reports the feasible upper witness and a missing lower certificate. If supplied, a malformed certificate exits nonzero. A verified interval that crosses a cutoff reports `loose`. Only an interval wholly in one registered regime of the full frozen DEV-tune run reports a proved branch. Synthetic and smaller cuts report their intervals within the same cut. The `--reverify` arguments are required for production and may be omitted for synthetic fixtures.

For production, the verifier reruns the primary independent verifier and checks for the identical verification output and unchanged fit result and executable digests. It validates saved feasible weight vectors and their matching mode receipts, then recomputes the tangent bound from all ordered bucket and expert sums with checked integer arithmetic. The upper witness is the best exact score among pure KN and the saved endpoints of both U_count fits and the near-KN restricted control. An independently rescored pass-zero anchor can also supply the upper witness. The receipt records an interior best as an observation and marks the corresponding tangent anchor `weights_unavailable`. The reviewed CUDA worker supplies the corpus directional sums and row-maximum scores.

The verifier independently reopens, hashes and rescans synthetic tapes of at most 65,536 rows. On those tapes it checks score, directional sums and row-maximum score word for word against the device receipt. Production verification uses receipt arithmetic without host corpus scoring.

```sh
CARGO_TARGET_DIR="$scratch/certificate-verify" cargo test --locked \
  --manifest-path "$verify_dir/Cargo.toml"
```

The finite tests independently enumerate every feasible vector on an eight-step toy simplex for each fixture bucket and check enclosure. Controls cover the floor-to-one case, a live zero-mass lane, an absent lane, a live floor-weight lane at the anchor whose vertex is optimal, mutually conflicting row optima, ties, wide integer numerators, exact cutoffs, reversed rounding, missing buckets, permuted sums and a different mass tape.
