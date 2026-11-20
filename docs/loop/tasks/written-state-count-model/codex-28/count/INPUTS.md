# [codex] Recover the S5 C/R inputs from Wick

Wick's existing `uor-r4-claude` container exposes the historical files under `/workspace`, backed by the named volume `uor-r4-claude-workspace`. On 20 November UTC, all four files under `/workspace/out/c439/split-v1` passed the published cumulative `/workspace/out/c439/run2/split.sha256` (manifest digest `5c51ad29fbd027731df25e9f89f69d4e6fd1e7e207c4c9de03472f9dc5a9f74a`). The split manifest also matches the committed copy, SHA-256 `9e267941f440c892479184faac02477f6816520f4abaf81b01f49cc428032779`. The four streams plus the manifest occupy 4,472,305,441 bytes. [Published split manifest](../../claude-439/run2/split-manifest.json) · [Published hashes](../../claude-439/run2/split.sha256)

| Live C420 input under `/workspace/out/c420/run1` | Verified SHA-256 |
|---|---|
| `train.tokens.u16` | `c6cba29ead4c745fb47b3f87007d982e72defc9f04102a268709ba0d34d51460` |
| `dev-tune.tokens.u16` | `75c6d1ac479bf120bac3aa60ac160536e8146f8052aa49c002209500799b7d0a` |
| `dev-tune.novel.u8` | `7df42437260bb767e2b1bdfd0cc55a242bd6e0b632a5bcd754852a7ab8aca6af` |
| `id-bytes.u8` | `befb22f6908751ae522a80336afa584eb2c24e36a10b90ad3105e39271a48ac9` |

The private `caseyallard/uor-r4-store` reported revision `535d12e6ca8d6ffa738f48ad73301cee84b277d9`. Its file-list metadata requests returned HTTP 429, including nonrecursive fallback requests. The historical split's Hub availability is unverified. Wick is a positively verified source and requires no download. During inspection the workspace filesystem had **296 GiB** available, the separate Plexpool mount had **40 TiB** reported available, and Wick reported **25 GiB** available memory. These are point-in-time observations. No volume, job, allocation or file content was changed. No sealed DEV-test asset was opened.

## Exact small preparation boundary

The original recipe reserves every TRAIN record with index divisible by 128 into R before selecting any C prefix. The historical 1% rule then ends C at the last BOS at or before the 10-per-mille token mark. A read-only scan of the real C prefix found the cut at **20,754,301** tokens and **5,402** records, with EOS immediately before the cut. The prefix excludes the BOS at the cut. The historical run's target count independently confirms 20,748,899 non-BOS targets. A separate read-only scan confirmed that the first **15,951,644** tokens of the query carry complete BOS/EOS-framed R with **4,060** records and 15,947,584 non-BOS targets. These scans inspect only C and the R prefix. [Historical C1 log](../../claude-439/run2/pick-p10.log) · [Historical C1 receipt](../../claude-439/run2/pick-p10.json)

| Proposed prep output | Exact bytes |
|---|---:|
| 1% C, including its 64-byte UORT header | 41,508,666 |
| Full R, including its 64-byte UORT header | 31,903,352 |
| **Token streams** | **73,412,018** |
| Optional R class and segment files, one byte each per R token | 31,903,288 |
| **Streams plus both optional files** | **105,315,306** |

These are split artifacts only. They exclude count indexes, KN tables, lookup tapes, fitting working memory, and deployment bytes. A bounded exporter should preserve token order and token payload bytes, rewrite each UORT count word, and pin each result against the source identity and split boundary. The existing `pick-split` CLI can reproduce the original full bundle, but always includes DEV-tune and the plus controls. It has no R-only output mode. Likewise, `--per-mille 10 --pick-arms` slices C and still processes the whole original query, so it is a historical replay rather than a C/R-only fit.

## Next commands

Read-only verification can run immediately without building anything:

```sh
ssh -o BatchMode=yes wick 'docker exec uor-r4-claude sha256sum -c /workspace/out/c439/run2/split.sha256'
```

If the original bundle ever needs repair, build the unchanged `pick-split` CLI from the historical source and run the following with `pick_split` set to that binary and `fresh` set to an unused lab-owned output path in the same container:

```sh
base=/workspace/out/c420/run1
"$pick_split" "$base/train.tokens.u16" "$base/dev-tune.tokens.u16" \
  "$base/dev-tune.novel.u8" "$fresh"
(cd "$fresh" && awk '{name=$2; sub(/^.*\//,"",name); print $1"  "name}' \
  /workspace/out/c439/run2/split.sha256 | sha256sum -c -)
```

For the bounded new path, add a training-only Rust exporter that writes precisely the C and R cuts above from the verified split, then demonstrate exact raw and adjusted lookup words on R before seeking FIT for a new 1%-C mixture. Keep the historical original replay and any new R-only comparison separately labeled.
