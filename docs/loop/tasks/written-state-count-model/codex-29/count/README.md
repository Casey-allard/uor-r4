# [codex] Training-only C/R preparation and complete count lookup replay

## Verified preparation on Wick

The exporter read the verified C439 sources inside an isolated CPU-only container using the local image `sha256:154bd2b08b4ccf152598c633958f4da7426d27a160d877fd8ccecb266285e35a`. The historical workspace volume was mounted read-only with networking disabled. Only the count lab's new output mount was writable. The Wick artifact directory is `/home/callard/uor-codex/20261119-rival-admission-29/count/cr-v1`. After readback its directory mode is 555 and all six files are mode 444. No dataset bytes left Wick.

| Output | Bytes | Tokens | Records |
|---|---:|---:|---:|
| 1% C UORT | 41,508,666 | 20,754,301 | 5,402 |
| Reserved R UORT | 31,903,352 | 15,951,644 | 4,060 |
| R classes | 15,951,644 | one byte per R token | |
| R segments | 15,951,644 | one byte per R token | |
| Original ID-byte census | 4,096 | one byte per original ID | |
| **Data total** | **105,319,402** | | |

The source prefix hashes were independently pinned before export. The C cut is the last BOS at or before the 20,756,083-token mark. The exporter reads the bounded C remainder through that mark to confirm the omitted BOS and absence of a later one. Every output record begins with BOS and ends with EOS. The R query, classes and segments are read only through the end of segment zero. The exporter checks every R class against the original BOS marker and otherwise requires class zero. It never reads DEV-tune or later query payloads, writes scores or performs training. In each UORT header, bytes 8 through 15 alone carry the new token count.

The independent second read used GNU `cmp` directly against every original token payload and immutable header byte. It also compared the R class/segment prefixes and original ID-byte table bit for bit and rehashed every sealed output independently. The manifest SHA-256 is `84a340eed728583a6ea978c5e78dcd4f7e5b8bf38b68e7c407e06d0b6bcffc31`. The Wick exporter executable SHA-256 is `b28123331c0fec845f351c69e70eed1978c5efec85a791b43072549dec911ccc`. [Source and output identities](manifest.json) · [Independent readback](readback.txt) · [Original split custody](../../codex-28/count/INPUTS.md)

To reproduce from the isolated container, mount the verified workspace at `/source` read-only and a fresh lab-owned output root at `/work`. Compile the locked exporter on the Linux host and expose its release binary at `/work/d29-prep-cr`. The program requires a new output directory, writes its manifest last and refuses the wrong source prefix or a mixed-segment R.

```sh
/work/d29-prep-cr /source/out/c439/split-v1 /source/out/c420/run1 /work/cr-fresh
```

## Query-independent lookup slice and size limit

`UORKNC01` stores the original raw and modified KN count records in canonical order and binds the serialized lookup to a supplied training identity. It carries all orders, the original 22-byte triple words and 40-byte context words, and all four count-of-counts words per order. The loader pins the artifact hash, checks lengths and complete context census, and reuses `RawTables` and `KnTables` for one source-token target lookup. Its integer raw count, total and KN Q48 words have the same boundary as the D28 fitted-mixture scorer.

A finite two-table fixture used a query-independent training tape, a distinct probe and different raw versus continuation rows. The reloaded **3,516-byte** artifact matched all **11** KN Q48 target masses and all **44** raw count words and **44** total words from the direct original tables. The probe includes record restart, a repeated context, changed left context and an unseen token. Negative fixtures reject a query-filtered row set, inconsistent counts, a stale training binding, truncated bytes and a BOS target. The full `kn_ceiling` suite passed **23 tests**. No corpus count tables were constructed or inspected beyond the published census.

The 1%-C historical global census contains 200,733,726 triples and 169,847,686 contexts across the raw and adjusted tables. Applying this codec's exact row widths and table headers gives a projected **11,210,050,512 bytes** before the trunk, weights and any alternate indexes. That projection charges the order-ten table twice under this straightforward layout. The older query-conditioned retained rows cannot represent arbitrary contexts. Serving a cap-feasible complete or deliberately bounded count model remains open. A bounded table changes the experts available to fitting and needs its own registered train/serve parity and P1 comparisons. [Historical C1 census](../../claude-439/run2/pick-p10.json) · [D28 fitted-weight replay](../../codex-28/count/README.md)
