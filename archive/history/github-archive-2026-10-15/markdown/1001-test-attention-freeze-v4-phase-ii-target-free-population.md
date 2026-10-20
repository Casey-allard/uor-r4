# #1001 — test(attention): freeze V4 Phase II target-free population

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-29T14:53:31Z
- **Updated:** 2026-10-09T14:16:56Z
- **Closed:** 2026-08-29T15:13:16Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1001
- **Merged:** 2026-08-29T15:13:16Z
- **Merge commit:** a567edd43ec4840c0bce495339c6416777c4c883
- **Base / Head:** main / UOR-Foundation:issue-973-v4-target-free-freeze
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Outcome

Freeze the independent target-free Phase-II population for `ConnectionGaugeCovarianceV4` before any V4 validation prediction or scoring-label join.

The first public commit contains only the executable generator checkpoint. Independent code-path and byte-level review accepted its exact NUL domains, LE encodings, all-65,536 counter order, tie rules, first-12 scan, dynamic antichain, involution, and exact 16/8/12 legacy fixtures. It cannot reach attention compilation, prediction, labels, or scoring and publishes no selected population value.

The second append-only commit binds the 12 matched pairs / 24 target-free rows, raw case IDs, forbidden/prefix/input roots, exact-24-row salted commitment schema, sealed commitment, prelabel freeze, comparator bindings, and synchronized living documentation. The generator checkpoint remains byte-identical. No Phase-I mechanism, configuration, frame, control, threshold, or artifact changed.

## Frozen identities

- Phase-I protected merge: `b054197acb92e3dd23d88d81bd859379ea8fac67`
- generator checkpoint: `c9c3b13ca7c1d346b5c4b3a5d624907b7f765461`
- target-free freeze commit: `476146711cac308a67c4093596b9307d0bfa64c4`
- generator policy: `blake3:73b4233b0b91ba85ffb6cd8c3d86132a954e4fbda5c7ec57510cc30bd9fb5dca`
- forbidden root: `blake3:877707eff60857b9c790cfb0e8a2a5a12bbcadb51d3448c9bd7119d5b86b6c42`
- validation-prefix root: `blake3:a3321b13d808d553d7588997f8fb7951be33e254724d45a1223460dd775a3ad8`
- validation-input CID: `blake3:5a17c5526d866f2862b042750cb70f5183f6a8fc09ab53a067d79d28d1c989d1`
- salted label commitment: `blake3:9773355914ed171f0d14950a4db554f5f543252804c703e8e0bbbbf17fe7b602`
- prelabel freeze: `blake3:170419cfcf80b2b0e48cc74faff13c9791dd9106045a1ff59a82efe4f6b205aa`

Counts: `12 pairs / 24 cases / 12:12 aggregate balance / 0 validation predictions / 0 scoring-label joins`.

Labels and nonce remain sealed; predictions, scores, and verdict remain `NOT_RUN`. The commitment steward accessed the structurally inferable preimage only to calculate its integrity commitment. Zero-call counters are declarations corroborated by the restricted imports and absence of model/prediction/scoring paths.

## Focused evidence

```text
cargo test -p uor-r4-core --test connection_gauge_covariance_v4_generator_973 --offline -- --nocapture
cargo test -p uor-r4-core --test connection_gauge_covariance_v4_target_free_973 --offline -- --nocapture
cargo fmt --check
python3 scripts/check_claim_wording.py
```

Both focused tests: `1 passed; 0 failed`. Formatting, claim wording, diff checks, exact generator replay, raw32/schema audit, and independent scope review passed. Focused strict clippy reaches the unrelated pre-existing `uor-r4-model-source/src/geometric_decoder.rs:438` `needless_range_loop`; this PR does not touch that file.

After protected merge, Phase III may reproduce the freezes, create two byte-identical label-free prediction streams, reveal the committed nonce/preimage once, and score once. Any mechanism/input/configuration/control/threshold repair is V5 with a new population.

References #973



## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

