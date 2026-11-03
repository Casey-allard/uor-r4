# REPORT: matched full-path record controls, deepseek-92

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Order: [ORDER.md](ORDER.md). Base:
`1fa3d1724f7eeb130351f2c1e4b49e168cea24f4`.

## What did not work

- **This is not a capability comparison.** Every arm starts from the same
  synthetic-initial full-path artifact and is run on two authored sequences.
  It tests mechanism isolation and serving parity, not language quality.
- **Hamming-off does not isolate the point decision.** On the point sequence,
  both the full and Hamming-off arms choose token 40. Hamming still changes the
  query from row 2, and the no-memory Hamming-on versus stripped pair isolates
  that contribution, but removal of Hamming alone does not remove the point
  behavior.
- **Memory-off and point-off agree on the point sequence.** Both choose token
  1295 because a point key cannot be served without the turn-memory container.
  The point effect is isolated by `point_off`, which retains memory and changes
  the decision from 1295 to 40.
- **The comparison has two sequences and eleven decisions.** It is a control
  harness, not statistical evidence.
- **There is still no public or corpus-trained full-path model.** The source
  trunk and examples are finite development fixtures.

## What changed

The full-path construction from deepseek-91 now emits five deterministic arms
from the same trained writer and records:

- `full`: Hamming, opening/reply memory and point key;
- `memory_off`: Hamming only;
- `point_off`: Hamming and memory, no point key;
- `hamming_off`: memory and point key, no Hamming;
- `stripped`: none of the three records.

The test parses the mechanism records to ensure each control carries the
mechanisms its name claims. Every arm is run through the complete served-forward
audit on the reply sequence and the point-fallback sequence.

## Measurements

| Arm | SHA-256 | Hamming | Memory | Point |
|---|---|---|---|---|
| full | `bc83c038…5195b35` | yes | yes | yes |
| memory-off | `9b82d4d8…987f7b` | yes | no | no |
| point-off | `d85d4a27…b66098` | yes | yes | no |
| Hamming-off | `ab8ad83d…fba4cb` | no | yes | yes |
| stripped | `7c4c1400…885b5` | no | no | no |

Every arm passed both audits:

| Reading | Per arm |
|---|---:|
| Sequences | 2 |
| Q12 query coordinates | 704 |
| Query-coordinate mismatches | 0 |
| Token decisions | 11 |
| Token mismatches | 0 |
| Kept cells | 88 |
| Kept-cell mismatches | 0 |
| Instrument fault | false |

Point-sequence decisions:

| Arm | Token |
|---|---:|
| full | 40 |
| memory-off | 1,295 |
| point-off | 1,295 |
| Hamming-off | 40 |
| stripped | 3,936 |

The first query divergence at the reply sequence was row 2 for memory-off and
stripped, and row 3 for Hamming-off. On the point sequence Hamming-off first
diverged at row 2. The memory-off arm changed the opening query and both reply
queries; point-off retained those changes but replaced the target opening 40
with 1,295.

The compact evidence record is
[full-path-controls.json](evidence/full-path-controls.json).

## Reading

This supplies matched record ablations on one trained full-path artifact and
shows that:

- memory removal changes every memory lookup row;
- point removal changes the opening decision while preserving memory;
- Hamming removal changes the query path;
- stripping all records changes both query and decision paths;
- all five artifacts retain exact CPU/serving parity.

It does not rank the arms on language quality and does not constitute the next
corpus-scale training comparison. The next comparison must use held-out targets
and a registered outcome rather than this finite mechanism-isolation panel.

## Checks run at the head

- Full release crate suite: 138 library tests passed with four release
  benchmarks ignored; the CPU corpus, counterfactual-corpus and
  discrete-corpus binaries passed 6, 1 and 5 tests respectively.
- Strict package Clippy with `--all-targets --no-deps -- -D warnings`,
  package formatting, `git diff --check` and evidence JSON validation passed.
- Five-arm deterministic reconstruction: passed.
- Parsed mechanism presence/absence for every control: passed.
- Full served audit for every arm and both sequences: exact, with no instrument
  fault.
- Report, roadmap and local PR checks pass.

## Skills used

No available Codex skill applied. The trained full-path fixture, FBS artifact
formats and served-forward audit were inspected directly.

## Still open

- A materially new held-out CPU training comparison.
- A public or corpus-trained full-path artifact.
- Complete-reply and fresh-phrasing qualification.

## Plain summary

Five matched artifacts now separate the trained full path from memory-off,
point-off, Hamming-off and fully stripped controls. Each mechanism changes the
query or decision where expected, each control carries exactly its named
records, and all five pass the complete CPU/serving audit exactly. This is
mechanism evidence, not a model-quality result.
