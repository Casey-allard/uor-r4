# [codex] D37 frozen seven-arm DEV-tune comparison

The first synthetic launch failed at its final batch seam. The host fixture
generated an invented mass of 281,478,734,798,849, above the Q48 unit
mass of 281,474,976,710,656. The CUDA scorer rejected it before returning
the row. The host-only correction brought that mass to 281,470,144,733,185.
The corrected finite run compared 49,158 bucket, mass, and bit words over
8,193 rows per class rule with zero differences. The standalone CUDA
scorer and the four Pick images embedded in the host retained their byte
identities.
The first Vast instance also lost SSH access before D37 data transferred.
It was retired and the execution used a fresh, isolated RTX 4090.

The [finite source diff](evidence-20261121/score-finite-v2.diff) identifies
the only change to the measured Rust scorer. Its SHA-256 is
`291138913ec297b69449a73cb00ac9919444fe369c65ef42d6979d48e6ff0417`.
The V2 Rust binary is
`86638d458b4ea3c5a8700bbbb70d454d36bef1489dabfcb93d605164fdc86757`.
The [execution manifest](evidence-20261121/stage.sha256) is
`c8d01f39b4030a51c0fc3d6e05e0fe47c94469601d7f89e4093c74311518417d`.
The repository's Rust and CUDA sources agree with the Wick build sources.
The previously admitted C2 tape used the V1 host for validation. The
source diff affects only the invented finite fixture.

**Selected A1s, the five-class carrier `UORS5W02`.** A1s minus A1 had
a 95% paired all-target band of **[−50, −31] micro-BPB**, calculated
from exact Q12 endpoints [−4,402,866, −2,812,588]. The upper endpoint
cleared the registered zero cutoff within the preregistered time budget.
The novel band was [−64, −41] micro-BPB, exact Q12
[−4,166,459, −2,691,079].

All seven arms scored 10,807,417 targets in 2,670 original DEV-tune
records. The all-target denominator was 21,928,475 bytes. Novel covered
7,566,174 targets and 15,995,822 bytes, response covered 9,523,958
targets and 19,286,759 bytes, and novel response covered 7,258,229
targets and 15,302,949 bytes. Each arm passed 8,493 native count and
mixture probes. The separate CUDA C2 predecessor tape had 10,807,417
words and passed all 8,493 native predecessor probes.

| Arm | All micro-BPB | Novel | Response | Novel response |
| --- | ---: | ---: | ---: | ---: |
| A0 pure count | 1,681,112 | 1,950,506 | 1,791,340 | 1,953,146 |
| A1 identity, three classes | 1,581,484 | 1,826,884 | 1,684,857 | 1,830,073 |
| A1s identity, five classes | 1,581,444 | 1,826,832 | 1,684,813 | 1,830,021 |
| C1 masked | 1,682,847 | 1,952,051 | 1,793,043 | 1,954,695 |
| C2 latest exact pair | 1,586,259 | 1,833,339 | 1,689,979 | 1,836,436 |
| C3 permuted, three classes | 1,581,612 | 1,827,083 | 1,684,970 | 1,830,239 |
| C3s permuted, five classes | 1,581,601 | 1,827,058 | 1,684,958 | 1,830,217 |

| Paired contrast | All band, micro-BPB | Novel band, micro-BPB |
| --- | ---: | ---: |
| A1 − A0 | [−106,452, −92,992] | [−132,328, −114,970] |
| A1 − C1 | [−108,140, −94,654] | [−133,978, −116,521] |
| A1 − C2 | [−5,232, −4,298] | [−7,089, −5,802] |
| A1 − C3 | [−233, −23] | [−341, −62] |
| A1s − A1 | [−50, −31] | [−64, −41] |
| A1s − C2 | [−5,274, −4,334] | [−7,145, −5,847] |
| A1s − C3s | [−266, −55] | [−374, −89] |
| C1 − A0 | [1,565, 1,916] | [1,371, 1,728] |
| C3s − C3 | [−27, 5] | [−48, −5] |

The scorer used 1,000 complete-record paired resamples with seed 4390.
An independent Rust implementation read the retained per-record sums,
verified the integer totals and reconstructed both endpoints of all
nine contrasts in both cohorts. [Independent check](independent-check/main.rs) · [Reconstruction](evidence/d37-reconstructed.json)

A1s fired at distance zero for 4,111,759 targets with
2,253,627 hits, distance two for 36,362 with 6,160 hits, and distance
three or more for 6,653,956 with 505,294 hits. No distance-one read
occurred. The gate stayed closed for 5,340 targets.

The full score ran in 8:54.06 with peak host RSS 5,448,700 KiB. The
D37 lease was released after verifying the result custody and idle lock.
Estimated D37 GPU rental through release was $0.2805 at the listed
$0.869/h, including 97 seconds for the failed instance and 1,065 seconds
for the replacement. The replacement passed to the state21 session.

The Wick evidence root is
`/mnt/plexpool/uor-r4-corpus/runs/20261120-pruned-r-fit-37/tune/vast-61843938-evidence/revision-v2`.
Its `real-v1/selection.sha256` binds the selection and per-record
arrays, with manifest digest
`ccdca21900b388b24f556408b9bc516905c225cab03c722fd151605059a753b5`.
The `frozen/freeze.sha256` manifest digest is
`4b36ae670b05216d1a83e3e5c256154c84d4a10984db1c10629b42d587a62b4d`.
It binds the selected weight
`b6b3da9e94ac33d0d8dc6c192c3117afc4fdc1f8e9c83dbf051010f5c2f31bb0`,
manifest
`133e691cca11e6663163271c63d1f248aa8a9c575d7adf42ae4131316b74ca97`,
fit seal, and selection receipt. The [selection receipt](evidence/selection.json)
and [per-record sums](evidence/per-record-q12.json) contain the class
counts, Q12 totals, bands and arm identities.
This run scored DEV-tune. DEV-test remained sealed.
