# REPORT: data-derived full-path training, deepseek-91

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Order: [ORDER.md](ORDER.md). Base:
`d1d465eaeea0063cf83c3bd3260659a6ebf3f976`.

## What did not work

- **The initial trunk is synthetic.** This trains its writer and record
  parameters on finite examples, but it is not a newly trained public
  language model.
- **Only one of eight Hamming head margins changed.** Every read/head was
  searched against the exact score, but this constructed sample selected a
  change at one head. That is a trained-record control, not broad Hamming
  learning evidence.
- **Memory movement weights remain fixed at one.** The opening and reply keys,
  values and sorting are derived from examples; their width-wide movement
  vectors are initialization, not learned parameters.
- **The point key is dataset-exact, not a learned classifier.** It stores the
  exact Q12 opening query and its supervised target, as the serving point-key
  mechanism defines it.
- **There is no language-quality claim.** The examples are authored finite
  sequences. Complete replies, fresh phrasing, corpus scale and GPU
  qualification remain unmeasured.
- **The first two test runs hit the repository's mixed-feature Cargo target
  collision.** Fresh targets passed; this was an instrument fault before the
  test body ran or after only a source compile.

## What changed

Added a finite full-path training control in
`crates/uor-r4-cpu-train/src/train.rs`.

The control:

1. starts from a two-layer width-64 FBS trunk with two four-head Hamming reads;
2. trains the writer table through the exact rank-margin trainer;
3. writes the trained table back to the FBS snapshot and proves the export
   matches it;
4. searches every Hamming read/head margin over
   `{0.5, 1, 2, 4, 8, 16, 32}` against target admission, target rank, margin
   loss and target distance at the exact CPU boundary; one margin improved and
   the resulting Hamming record changed the final query;
5. derives opening and reply memory tables from two prompt/reply examples,
   groups duplicate keys by target count and breaks ties to the smaller token
   id;
6. derives a point-key row from the exact final opening query of a prompt whose
   memory key is absent; and
7. exports the complete artifact, writes and reloads it, then audits opening
   memory, two reply-memory rows and the point fallback through the independent
   CPU and integer-serving paths.

The test also constructs the complete artifact twice and requires byte
identity.

## Measurements

| Stage | SHA-256 |
|---|---|
| Initial Hamming trunk | `1da4e4fc4116c7f250c2ad5ba2fa0f03d4af4c9180e6b888db1c485daa62c769` |
| After 8 exact writer updates | `1dc319ce5c14183ae3c29afc3d0ff67309e2d2aef49aa33c02bb3dea986f18b9` |
| After Hamming-margin training | `9b82d4d8dd492a52473ef7a15d12a3cbd9832bb769c581b9e240e64b7c987f7b` |
| Final Hamming + memory + point artifact | `bc83c038e56c36627672bac90c0a8572fe6ee9f2733913c2c507dccfd5195b35` |

The final artifact is 536,516 bytes. The local artifact is
`local/deepseek/full-path-records/trained-full-path.uorlut`; it is reproducible
with:

```text
UOR_FULL_PATH_ARTIFACT=local/deepseek/full-path-records/trained-full-path.uorlut \
  cargo test --release --locked -p uor-r4-cpu-train \
  data_derived_records_train_and_audit_the_full_path -- --nocapture
```

| Reading | Result |
|---|---:|
| Writer updates accepted | 8 / 8 |
| Hamming head margins considered | 8 |
| Hamming margins changed | 1 |
| Opening memory rows | 1 |
| Reply memory rows | 2 |
| Point-key rows | 1 |
| Served audit sequences | 2 |
| Q12 query coordinates | 704 |
| Query-coordinate mismatches | 0 |
| Token decisions | 11 |
| Token mismatches | 0 |
| Kept cells | 88 |
| Kept-cell mismatches | 0 |

The opening memory changed the query at its lookup row; the two reply-memory
rows each changed their query. Without the point table, the final opening was
token 1,295; with the data-derived point row it was its target, token 40.

The compact evidence record is
[full-path-records.json](evidence/full-path-records.json).

## Reading

The full-path artifact now has evidence that all three previously synthetic
record families were populated from training data or exact-score selection and
then operated in one exported artifact. The writer, Hamming margin, memory keys
and values, and point query/target all reach the composed served path, and the
CPU and integer session agree exactly.

This closes the finite train-to-full-path-record gap. It does not provide a
public trained full-path checkpoint or demonstrate useful learning. The
remaining trained-record question is whether corpus-scale joint training can
improve a held-out ranking or reply result.

## Checks run at the head

- Focused release test in a fresh target: 1 passed; the other 140 tests were
  filtered.
- Deterministic construction: two builds produced the same four stage hashes.
- Export read-back: the final artifact was written and read back byte-identical.
- Full served audit: both sequences had zero query, token and kept-cell
  mismatches and no instrument fault.
- Full release crate suite: 137 library tests passed with four release
  benchmarks ignored; the CPU corpus, counterfactual-corpus and
  discrete-corpus binaries passed 6, 1 and 5 tests respectively.
- Strict package Clippy with `--all-targets --no-deps -- -D warnings`,
  package formatting, `git diff --check` and evidence JSON validation passed.
- `loop report --check` and `loop gates --full`: passed.

## Skills used

No available Codex skill applied. The existing CPU trainer, FBS exporter,
Hamming and memory formats, and served-forward audit were read directly.

## Still open

- A public or corpus-trained full-path model rather than this synthetic initial
  trunk.
- Learned width-wide Hamming and memory weights, rather than a calibrated
  Hamming margin and data-derived tables.
- A held-out ranking or complete-reply bar on the full path.

## Plain summary

The CPU can now train the shared writer, calibrate a Hamming head against the
exact served score, derive opening/reply memory and a point-key fallback from
training examples, export all of them into one artifact, and verify that
Hamming, both memory stages and the point fallback affect execution with exact
CPU/serving parity. This closes the finite trained-record coverage gap while
leaving public-model quality and GPU qualification open.
