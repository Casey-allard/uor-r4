# ORDER: one-model-end-to-end, deepseek-264 (real artifact audit)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `402b4091e`.

## Goal

**Audit the real claude-339 carried artifact and exact `−C` control end to end, and leave
the same audit ready for every final 16,384 artifact.**

Claude-339 has published real seed-1 L-aC artifacts at steps 8,192 and 12,288, with
ask-preserving controls, panel reports, the pinned tokenizer, and the sealed m2-fresh256
inputs. These are trend points, not the registered final read. The support unit must:

- verify the published hashes;
- reproduce the exact `−C` bytes;
- compare CPU and integer panel reports bit-for-bit after timing and engine identity;
- expose first-target and paired-first rank changes under the carry cut;
- produce one deterministic receipt that can be run unchanged on step 16,384.

This is an instrument and diagnostic unit. It trains nothing, changes no serving arithmetic,
moves no bar, and makes no capability claim from the early checkpoints.

## Cancel probe

1. PR #3894 contains real stored artifacts for L-aC seed 1 at 8,192 and 12,288.
2. The artifacts and m2-fresh256 inputs have published SHA-256 values.
3. The CPU port and exact `−C` tool are merged on `main`.
4. No open PR owns this branch or the new audit binary.

**Result at claim:** pass.

## Design

Add `real-artifact-audit` to the one-model instrument. It accepts:

- the carried artifact and its published `−C`;
- carried and `−C` CPU panel reports;
- carried and `−C` integer panel reports;
- optional CPU rank reports from `--teacher-forced 1 --paired-first 1`;
- the expected panel row count;
- an output path.

It refuses unless:

- both artifacts parse, have the same ask inventory, and differ in the expected carry;
- optional `--carried-sha256` and `--minus-c-sha256` equal the artifacts' published
  `SHA256SUMS` values;
- `drop_hamming_carry_uorlut01(carried)` equals the supplied `−C` byte-for-byte;
- every report is for the artifact it scores, the CPU reports are not integer reports,
  and all six reports hold exactly the expected row count with the same settings,
  rendering and non-artifact inputs;
- CPU and integer reports equal after only timing fields and `settings.engine`;
- ranked reports have the same cases as the CPU generation reports and compare by case id,
  not row position. `changed_by_cut` is true only for a target/EOS or paired-rank change;
  raw-nearest and diagnostic-field changes are counted separately.

It writes a receipt with artifact identities and sizes, ask depths, carry rows, report
equality, first `reply_ids` divergence, and carried-versus-`−C` teacher/paired rank deltas.

## Files in scope

- `docs/labs/one-model-end-to-end/instrument/Cargo.toml`
- `docs/labs/one-model-end-to-end/instrument/src/bin/real-artifact-audit.rs`
- `docs/loop/tasks/one-model-end-to-end/deepseek-264/**`
- `docs/research/notebook/float-training.md`
- `docs/harness/claims.json`

## Done when

1. The tool and negative tests pass.
2. It passes both real early pairs: 8,192 and 12,288.
3. Its run records whether the carry changes first-target and paired-first ranks and where
   the first generation divergence occurs.
4. The command is ready for the final 16,384 artifacts without code changes.
5. Focused tests, full gates, required checks, fresh exact-head review and protected merge pass.

## Stop and ask if

Nothing waits on the owner. Do not alter Claude's artifacts, reports, pods, branch or the
registered outcome map. A CPU/integer disagreement or failed exact `−C` reproduction is an
instrument finding; report the first differing row and preserve the files.
