# #1037 — #973: add learned associative readout campaign and record no-capacity result

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-01T20:11:07Z
- **Updated:** 2026-10-09T14:16:58Z
- **Closed:** 2026-09-01T20:12:31Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1037
- **Merged:** 2026-09-01T20:12:31Z
- **Merge commit:** 3e768fa520bfb2fbc811610cd5b746cf89b41413
- **Base / Head:** main / UOR-Foundation:issue-973-learned-associative-readout
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

- implement the frozen `R4LearnedCandidateLeafAssociativeReadoutV1` and equal-budget pooled control, create-once population/reveal lifecycle, deterministic campaign runner, independent verifier, CLI surface, and focused tests
- execute the one sealed CPU/Apple Accelerate campaign under the predeclared 7,200-second hard wall
- record the verified terminal `LEARNED_ASSOCIATIVE_READOUT_NO_CAPACITY` across the canonical research, roadmap, programme, evaluation, ADR, glossary, and contributor documentation
- bind the next #973 action to a separately frozen retained-value write/binding-law change; do not retry this readout or run generation from it

## Result

- geometric prompt gain: `0.0063767854348491465`, `299/512` wins
- pooled prompt gain: `0.010263234571452827`, `324/512` wins
- frozen V1 prompt gain: `0.006423652456300697`; neither learned arm met the frozen absolute or incremental capacity floor
- geometry attribution: fail versus pooled (`-0.0038864491366036808`, `209/512`) and deranged (`-0.0002888663472835149`, `251/512`)
- pooled fresh-language NLL/top-1: `3.8737562215878296` / `30.042756%`, versus V1 `3.9036360153193317` / `29.628509%`
- pooled state-off cost: `0.3654355381796077` nats and `17,808` correct decisions
- all 10 mechanics gates and independent fresh-process replay passed; post-reveal optimizer steps were zero
- result CID: `blake3:cedba37738ee249457bb589f716ee75afb16a0c4937c2a22ae9f917dd3eb97c1`
- verification CID: `blake3:443d711ce9a228e26e2eb2eebb55c582848424e2677c3473d41deaf8afd69ec7`

## Validation

- focused Python suite: 45/45 passed
- `cargo test --workspace --offline --no-fail-fast`
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`
- `cargo fmt --check`
- graph-format no-std and alloc checks
- Python compile and claim-wording checks
- docs diff check

Generation, reasoning, exact/table lowering, geometry-native lowering, and a transformerless general model remain `NOT_RUN` or `NOT_ESTABLISHED`.

References #973


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

