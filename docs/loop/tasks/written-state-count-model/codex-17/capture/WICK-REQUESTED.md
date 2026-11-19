# [codex] Sealed requested-span capture for #4106

The requested lane is a diagnostic intervention on the existing reader. It replaces only words 0–1 of the original public role tuple. Words 2–7, including the excluded span, both scores and both runners-up, remain bit-identical. The input and selection region stay the full original protocol-2 context. The original selector runs through `writer-c select public`. The Rust adapter calls the existing `paired_copy_witness` with those original role words and the independently supplied requested span. The downstream chain still uses the original assertion enumeration, CUDA fence, and unchanged writer.

## Input custody

The raw input is a strict JSON array of `{"index":0,"text":"..."}` rows. The unmarked text and endpoint file are separately SHA-256 sealed before staging. The endpoint file accepts exactly this shape:

```json
{
  "schema": "uor-r4.sealed-requested-names/1",
  "raw_sha256": "<sealed raw JSON SHA-256>",
  "rows": [
    {
      "index": 0,
      "text_sha256": "<SHA-256 of the row text UTF-8 bytes>",
      "requested": {"bytes": [0, 4], "name": "Ames"}
    }
  ]
}
```

Offsets are half-open UTF-8 byte positions in the raw text. Use `"requested":null` for an independently absent requested endpoint. The fixture tests also distinguish a present requested word from absent factual support. The preparer verifies exact decoded protocol-2 bytes and name boundaries, resolves offsets using the selected seed tokenizer, and emits only absolute spans and provenance to `resolved-requested.json`. Nested and top-level extra fields reject. The command accepts no grade, label, family, item, source, answer, fixture bounds, or outcome path. A contradictory spelling and byte range reject. If the independently read semantic request itself names the wrong valid occurrence and seals its exact bytes, this tool cannot infer intent. Reader independence and adjudication belong to the custody receipt.

## Build and stage two seeds

Use only the reviewed source tree and SHA. These commands write new directories.

```bash
reviewed_repo=/absolute/path/to/reviewed/source
reviewed_head=REVIEWED_GIT_SHA
capture_root=/absolute/path/to/fresh/run/adapter
./build-for-wick.sh "$reviewed_repo" "$reviewed_head" "$capture_root"
cap="$capture_root/target/release/uor-value-rank-capture"
```

For each seed, run the data-only preparation with separately pinned raw and endpoint files. Example commands below use externally recorded SHA values for the protected inputs.

```bash
"$cap" prepare "$tokenizer1" "$tokenizer1_sha" "$sealed_raw" "$raw_sha" \
  "$sealed_names" "$names_sha" "$stage/prepared-seed1"
"$cap" prepare "$tokenizer2" "$tokenizer2_sha" "$sealed_raw" "$raw_sha" \
  "$sealed_names" "$names_sha" "$stage/prepared-seed2"
```

For each seed the trusted launcher pins a stage-spec JSON file with `schema` equal to `uor-r4.requested-capture-stage/1`, a numeric `seed`, absolute `wick_root`, and `inputs`. The exact `inputs` keys are `base`, `blind`, `capture`, `paired`, `resolved`, `span` and `writer`. Each takes `{"path":"/absolute/known/file","sha256":"<pinned SHA-256>"}`. `blind` and `resolved` point into that seed's new prepared directory. The original historical chain must also have its required D11, exact/span banks and AOT images under the normal Wick deployed paths. Bind those source assets in the reviewed stage receipt.

The following is the deciding capture command sequence for a unit with an accepted FIT. The older chain obtains its normal device lock internally. The script takes the same sealed prepared files in cold runs. It never opens sibling files by discovery.

```bash
./run-wick.sh "$spec1" "$spec1_sha" "$stage/primary-seed1"
./run-wick.sh "$spec2" "$spec2_sha" "$stage/primary-seed2"
./run-wick.sh "$spec1" "$spec1_sha" "$stage/repeat-seed1"
./run-wick.sh "$spec2" "$spec2_sha" "$stage/repeat-seed2"
for seed in 1 2; do
  cmp "$stage/primary-seed$seed/cold.sha256" "$stage/repeat-seed$seed/cold.sha256"
done
```

The common launcher runs both original public and disabled commands with literal `- -` arguments. It derives requested diagnostics through the endpoint-only adapter and calls the same downstream chain for the three selections. The capture verifies the original public selection again before exporting. Use `requested-diagnostic/candidates.json` for the native packet bridge. Preserve `archive-audit.json` separately. This script stages capture, no fit, scoring or post-seal grade join.

## Released author fixture

The local sample is an explicitly authored check over already-released seed-1 text, not an independently produced non-author seal for the planned suite. Its three supplied names are Sabine, Vale and Nina. Raw text re-encodes to the exact previously archived blind bytes. On the Sabine sample the public role abstained. The independently specified span restores source 19 through the unchanged witness. Vale retains known source 7. Nina retains a requested span with no matching fact or source. All 32 rows preserve the six non-request role words and full selector region. A wrong name at the sealed bytes and a forged `gold_start` field both fail. Changing a neighboring synthetic `grades.json` does not change the diagnostic output.

For a full parser/receipt check on old released data, `released-replay.sh` explicitly constructs an **offline replay index** from immutable archived writer mappings whose row tokens, assertion and chosen source agree. The test points the new requested capture mixer at those old GPU rows. It does not create a new GPU receipt or claim that the future Wick stage has run. The three-lane Wick commands are prepared and unexecuted. No #4106 protected models, fit texts, held texts or grades were opened. [Released test hashes](released-stage-fixture/SHA256SUMS)
