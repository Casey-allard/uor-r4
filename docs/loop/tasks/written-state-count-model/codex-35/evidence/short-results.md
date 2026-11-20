# D35 matched short-object stress result

The two forward ranks lost ordinary exact replies and copied an unsupported
value on more prompts than the D26 baseline. Reversing either bank gave more
exact replies than its forward bank in both modes. The supplied fact-prefix
forward arm gained six exact replies over baseline with 46 recoveries and 40
losses. It also made 22 more wrong source selections overall.

| Mode | Result | D26 | Fact forward | Fact reversed | Candidate forward | Candidate reversed |
|---|---|---:|---:|---:|---:|---:|
| Ordinary | Exact through EOS, 216 supported | 32 | 24 | 40 | 20 | 33 |
| Ordinary | Wrong supported source, 216 supported | 84 | 120 | 92 | 120 | 109 |
| Ordinary | Unsupported copy, 18 controls | 7 | 9 | 7 | 8 | 7 |
| Supplied | Exact through EOS, 216 supported | 42 | 48 | 56 | 42 | 48 |
| Supplied | Wrong supported source, 216 supported | 74 | 96 | 76 | 98 | 94 |

The supplied diagnostic admitted all 216 supported gold onsets and complete
source spans. Both forward banks still missed at the first output token on all
their nonexact supported replies. Ordinary requested-role accuracy was
144/216 supported and 8/18 unsupported, or 152/234 overall. The correct assertion
and both gold eligibility measures reached 144/216 supported. All arms shared
these role and eligibility counts. Each ordinary arm assigned zero of 225
explicit exclusions correctly. The 18 unsupported controls have no unique
supplied assertion and were inapplicable to that mode.

| Mode | Forward arm vs D26 | Exact gains | Exact losses | New wrong sources | New unsupported copies |
|---|---|---:|---:|---:|---:|
| Ordinary | Fact prefix | 22 | 30 | 76 | 3 |
| Ordinary | Candidate prefix | 18 | 30 | 73 | 3 |
| Supplied | Fact prefix | 46 | 40 | 68 | N/A |
| Supplied | Candidate prefix | 40 | 40 | 68 | N/A |

Both cold captures reproduced 2,250 transcripts byte for byte. All eight
geometry × mode × pass checks matched GPU winner and candidate words for both
orientations. Per geometry and pass, the ordinary check covered 2,716 ordered
pairs across 242 pools, and the supplied check covered 2,480 pairs across 216
pools. The same frozen banks and writer served every arm. No GPU fit ran for
the supplement. The test supports a negative for these two trained directions
on matched writer-admissible objects. The supplement reuses primary names,
templates and scenario families.

- [Original expected and emitted IDs, source starts and first differences](short-rows.tsv)
- [Paired transitions and exact reply deltas by family](short-transitions.json)
- [First-error stages and offsets](short-first-error-counts.json)
- [Candidate envelope by mode, family and modifier](short-envelope.tsv)
- [Input and device custody](short-input-device.sha256)
- [Native and grade custody](short-native-grade.sha256)
- [Full release file hashes](short-release.sha256)
