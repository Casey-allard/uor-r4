# D32 role comparison on the exposed D30 prompts

No arm correctly assigned both designated roles on any of the 16 crossed
prompts. Opening the requested region raised correct requests from 4/16
to 12/16 and exact output from 1/16 to 2/16. It also released two
unsupported copies. Repeating exact factual heads changed no decision
word or emitted ID within either fence condition.

| Arm | Both present | Requested admitted / correct | Excluded admitted / correct | Joint correct | Any source | Correct source / exact through EOS | Unsupported copy |
|---|---:|---:|---:|---:|---:|---:|---:|
| Historical, inherited | 8/16 | 4/16 / 4/16 | 0/16 / 0/16 | 0/16 | 15/16 | 1/16 / 1/16 | 0/4 |
| Historical, repeated | 8/16 | 4/16 / 4/16 | 0/16 / 0/16 | 0/16 | 15/16 | 1/16 / 1/16 | 0/4 |
| Full, inherited | 8/16 | 12/16 / 12/16 | 0/16 / 0/16 | 0/16 | 6/16 | 2/16 / 2/16 | 2/4 |
| Full, repeated | 8/16 | 12/16 / 12/16 | 0/16 / 0/16 | 0/16 | 6/16 | 2/16 / 2/16 | 2/4 |

Each designated endpoint was present on 12/16 crossed prompts. Each
direct arm returned both correct requested spans and copied one exact
reply through EOS. That reply also had a spurious excluded span.
Neither unspecified control released a copy. Both incidental-only
controls in the full arms copied “a separate line” from original source
15, through EOS. The selected query spans were [50,51) and [46,47).
The single new exact crossed reply was 30116, which copied “glass
beads” at original source 24. Its first emitted ID changed from 261
to 2537. All fourteen remaining crossed errors in the full arms began
at output offset zero.

The full arms never bound the expected assertion on rows 30104–30107.
On the ten other crossed failures, the expected assertion was bound
and the original source was eligible but lost at ranking. For example,
row 30100 regained Natalia [60,64) and bound [20,24), but source 25
had margin zero and the source reader withheld it. Quentin remained
absent in the same row even with both interventions. Quentin's
factual and query IDs matched exactly, but its first ID 223 had role
and assertion class 0 in the frozen D26 packet. Neither iterator
could seed the complete span.

Each of the 88 ordinary records was byte-identical on a second call.
Historical role words, source, token IDs, cursor steps, stop and text
reproduced all 22 D30 ordinary captures. The two CUDA launches matched
all 704 ordinary decision words per launch, including endpoints and
both score pairs. The join first verified each packet's original IDs,
framed user bounds and prefix SHA against the independent ordinary
capture.

The Wick root `/home/callard/uor-codex/20261120-d32-role-grade` retains
the primary and cold ordinary captures, grades and join. Its
`joined-evidence.sha256` binds those outputs to the device outputs and
packet identity. The join SHA-256 is
`306a13d98105a96575617519de59b57818d085bdcaa788586285292c1c9b9e68`.
The ordinary grades SHA-256 is
`2c01bf0c761d53229a15ba53b21df4fed3b11f82fde11494c043c6e9913db515`.
These measurements concern the sealed D26 reader on known D30 prompts.
