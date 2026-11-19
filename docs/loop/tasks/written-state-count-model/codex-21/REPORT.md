# [codex] Clause-to-entity scoring and the earlier connector veto

## What did not work

Transferring the frozen role evidence from each query clause to its entity selected the **excluded person on every supported prompt**. The D19 reference selected the second query name on all 96. Independently varying clause and name order made that shortcut right on exactly 48. The clause rule recovered zero spans and lost all 48. The failure occurred at the comparison of the two eligible people, before any value selection or writing.

The supplied-span value improvement was confined to **“could still find small glass jars.”** On the new-object transfer block both value policies had zero exact replies in 32 attempts. That block included 16 prompts with a genuinely early gold value and 16 with a late one. The new veto fired on none of them. No held release tested a truly early out-of-radius gold under the triggering signature. A planted pair with the same geometric packet and different correct onsets demonstrates the resulting risk. [Veto definition and finite counterexample](analysis/earlier-connector.md)

## What changed

Both comparisons used the same 96 supported prompts and six unsupported controls on each frozen artifact. Each block independently crossed affirmative/exclusion clause order, which name appeared first in the query, fact order, object assignment and connector assignment. There were two wording templates across three lexical blocks. Both independent readers agreed on all 102 prompts. On every supported prompt both names matched actual assertion heads. The treatment pooled signed native role-cell evidence within each punctuation-bounded clause and transferred it to the one matching person. The value diagnostic supplied only the correct person's query endpoints to the original assertion binder. Both value policies then saw the same original candidate packet and bank. The source-address copy, span cursor, token emission and EOS operator stayed fixed. No training update was made. [Registered test](SHEET.md) · [Native role operator](operator/README.md) · [Panel construction](panel/README.md)

## Measurements

Counts below are per artifact. The denominator for exactness is the same 96 supported prompts. First-error offsets are zero-based. The right source column requires the original gold value-start address.

| Arm | Correct query span | Right source | Wrong source | No source | Exact through EOS | First error at 0 |
|---|---:|---:|---:|---:|---:|---:|
| D19 ordinary | 48 | 8 | 32 | 56 | 8 | 88 |
| Clause-to-entity ordinary | 0 | 0 | 40 | 56 | 0 | 96 |
| Correct query supplied, ordered value | 96 supplied | 16 | 24 | 56 | 16 | 80 |
| Correct query supplied, earlier connector veto | 96 supplied | 24 | 24 | 48 | 24 | 72 |
| Opposing query supplied | 0 | 0 | 40 | 56 | 0 | 96 |
| Read disabled | 0 | 0 | 0 | 96 | 0 | 96 |
| Gold value onset forced | 96 supplied | 96 forced | 0 | 0 | 96 | 0 |

### Where the span failed

| Clause order | Supported prompts | Correct D19 spans | Correct clause spans |
|---|---:|---:|---:|
| Request first, exclusion last | 48 | 0 | 0 |
| Exclusion first, request last | 48 | 48 | 0 |

The requested and excluded clauses respectively earned **41 and 50** with “copy the exact item wording for” / “leave out the entry for,” and **28 and 84** with “quote the words naming the object for” / “disregard the description for.” The same contrasts held with the clauses reversed and with either person requested. Both mentions were eligible. The larger contrast selected the excluded person in either position.

For example, in “Leave out the entry for Rosalind; copy the exact item wording for Benedict,” the D19 selector returned Benedict at [47,50). The new rule returned Rosalind at [31,35). In the fact order with Benedict's brushes first, the reference already failed at output 0, with 1155 (“ G”) in place of 564 (“ br”). The treatment copied Rosalind's jars and changed the first wrong ID to 563 (“ small”), still at offset 0. Across the panel, eight previously exact ordinary replies gained a first error at 0. Another 24 changed the wrong token at 0. The other 64 kept the same first mismatch. No ordinary reply improved.

### Where the veto helped

All eight recoveries crossed both names (Rosalind and Benedict), both query-clause positions and both fact orders. On the first such prompt the supplied Rosalind span was [36,40). The earlier connector onset at source 9 had positive distance 52, negative distance 44 and radius 48. Source 10 had signed margin −4/3 and source 11, the start of “small,” had +16/4. The ordered reader withheld the source and emitted 1155 instead of 563 at output 0. The new rule selected 11. The original writer emitted [563, 2537, 1731, 85, 1], **“small glass jars” through EOS**. On all eight recovered prompts the first error disappeared. Every remaining failure still began at output 0.

| Gold object and connector, correct person supplied | Ordered exact | Veto exact |
|---|---:|---:|
| small glass jars / could still find | 0/8 | 8/8 |
| small glass jars / took | 8/8 | 8/8 |
| green glass beads / kept | 8/8 | 8/8 |
| Other nine object–connector combinations | 0/72 | 0/72 |

All 24 previously wrong source copies survived. Sixteen began in a connector and eight began inside the value. The rule introduced zero new wrong source copies on supported prompts. On one unsupported conflict prompt it released one of two assertions. The second assertion still abstained, so the final binder withheld the source. All six unsupported prompts retained zero final factual writes. The registered value criterion passes on this panel. The ordinary role criterion fails on span recovery and exact replies.

## Checks run at the head

The integer-library suite with the serving feature passed 711 tests with one ignored. The crate-wide format check reported existing differences in 13 files, all byte-identical to the base revision. The D19 baseline rebuilt byte-for-byte. The new artifact preserves its paired parent, original span state and ordered value bank. Planted role tests verified all eight GPU decision words against the Rust operator and exercised ties, missing support and a surviving lone negative winner. The independent blind grades, finite GPU checks and source identities were sealed before held scoring. Forced gold value onsets wrote 96/96 exact on each supported panel. Forced opposing values wrote 0/96, and independently regrading the correct tapes against the wrong keys accepted 0/96. [Independent FIT](FIT.md) · [Pre-run review](PRE-RUN.md) · [GO](authorization.json)

A fresh-process repeat of each seed reproduced all 426 deterministic files and the entire summary. All 851 raw file checks passed on each of four runs. An independent verifier compared source selections, token IDs and 10,960 cursor transitions across 1,224 ordinary client/GPU rows. Both ordinary policies terminated at EOS on every prompt. In each run one read-disabled control hit the 16-token limit on both the client and the GPU. Across 104 assertion records per run the veto changed nine, including the conflict whose final source stayed empty. Both supplied arms used identical inputs and kept all 259 candidate geometries. [Verifier source](analysis/writer-check/src/main.rs) · [Writer verification](evidence/writer-parity.json) · [Full check output](evidence/writer-check.txt) · [Cold replay](evidence/cold-replay.txt)

Wick’s RTX 4090 was occupied. The four held runs took 11 minutes 44.63 seconds on one rented RTX 4090. The estimated rental cost including setup and archival was $0.26, and the instance was terminated. The workstation restored 4,277 regular files and six symlinks. Wick verified its copy of the archive and all 14 model pins. A standalone restore preserves the text of a build-stage symlink whose target is the tracked [kernel header](../../../../../docs/labs/m1-d11-artifact-scorer/instrument/src/kernel_core.h). Rebuilding that stage requires the source checkout. All held traces replay from the archive. The private Hugging Face store rejected the full held and finite upload at its quota. Two smaller deposits succeeded. Revision-pinned downloads verified the paired measurements and 30 original primary observation files, including the native writer and source traces. The workstation and Wick retain the complete evidence. [Compute ledger](evidence/compute.json) · [Archive and private-store revisions](evidence/storage.json)

## Skills used

The native-space cycle supplied the shared operator, artifact identity and original-token replay checks.

## Still open

Keep the ordinary reference. The original bank learned six offsets relative to each candidate, with counts for requested, excluded and unspecified starts. D21 looked up each non-name clause root at all six offsets. The result identifies the first wrong decision and motivates a test of the feature coordinates used at fit time. Preserve each offset’s original side and distance, clipping at the candidate’s own clause fence with the existing missing-root value. Compare that projection to both frozen arms on untouched wording, checking that a clause-order flip preserves the named person and that a polarity flip switches people. If that fails, fit the clause projection itself on a disjoint training set and retest on new affirmative, exclusion and unsupported wordings. A sign reversal chosen after seeing these held answers would require a separate test. [Original role training](../codex-12/ROLE-SHEET.md) · [Candidate-relative CUDA count](../codex-12/role/device.cu) For values, test a genuinely early gold under the exact release signature alongside a later gold under the same signature. An identical packet producing different labels requires additional observable state, such as evidence of the connector endpoint, before a semantic distinction is possible. Retain the 24 wrong-source and 48 no-source rows as distinct outcomes. Training scale, equal-byte efficiency, unrestricted prose and useful chat remain outside this test.

## Plain summary

Clause scoring tracked the wording across positions, but ranked exclusion above the request. Supplying the right person exposed a narrower success. Eight formerly generic replies copied the complete correct phrase. The fixed writer copied every forced correct start. Both measured bottlenecks are upstream of the writer. [Paired cases](evidence/paired-findings.md) · [First artifact, all rows](evidence/readout-seed1.md) · [Second artifact, all rows](evidence/readout-seed2.md) · [Matched ordinals and counts](evidence/paired-summary.json)
