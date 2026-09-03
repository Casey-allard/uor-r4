# Original-source audit for #1085

**2026-09-03 — `SOURCE_INSPECTED`.** The independent source auditor queried public
`uor_knowledge` for clause segmentation, NEMESIS/W33 carrying, and UOR canonical
identity/ordered operands, fetched the relevant records, and followed the
original source pins. Small fresh raw downloads matched the prior local source
snapshots byte for byte. No source was vendored, no model or upstream witness
was executed, and no repository-wide reading or formal verification is claimed.

The live issue refresh established #1085 open with its sole blocker #1082 closed,
parent #973 open and consumer #954 blocked. The indexed #1085 snapshot was
`kb:e7cc9f7af577cf8b27950871b0dc2ba0aa34bce71ddb6a9b5bd0fe126604b8cd`, revision
`2026-09-03T04:25:49Z`, body SHA256
`85225824b064acf7065277dec79be252ff05835e5bfd697ecbceec87bcde8644`.
The fetched [#1082 independent review](../r4_token_exposure_1082_review.md)
was checked against current `origin/main`; its retrieval origin was commit
`c1c45fa098d24f4ceae3c8fc207f0ecd07d62e20`, SHA256
`85334c1f01fd2675a2ed844e3012a2e32b285067eaf75d983fb817dc0340b7f1`.
Snapshots are discovery aids; current native state governs eligibility.

## Original external sources

| Original source | Revision; verified raw SHA256 | Inspected support and limit |
|---|---|---|
| [NEMESIS, Technical Report: Integration of Hypercomplex Geometries as UOR Structure Carrying Substrates](https://github.com/markrnd87-cmd/NEMESIS-Theory/blob/0d106967843c2c96477cf3e57aeff213e7db1c97/Technical%20Report_%20Integration%20of%20Hypercomplex%20Geometries%20as%20UOR%20Structure%20Carrying%20Substrates.pdf), pp. 1–3; source attribution Mark / NEMESIS 3D Studio | `0d106967843c2c96477cf3e57aeff213e7db1c97`; `697d48b70a1499a1fd70d8f1a4c285606a198a3831250425ae11439f37b395cc`; 99,374 bytes | Separates state mapping, transition preservation and primitive interpretation. These are useful interface questions. Later complexity/energy assertions do not establish parsing or model capability. No license was found in the pinned intake; link and attribute, no vendoring. |
| [W33 runtime](https://github.com/wilcompute/W33-Theory/blob/5674aa2e74c7de81864556b1bbc90aa9a1b1bd3d/analysis/w33_fractal_microvm_runtime.py#L38), especially lines 38–58 and 314–329 | `5674aa2e74c7de81864556b1bbc90aa9a1b1bd3d`; `875b53408cc5312b60b5a6254dbac80a9a1324c89cdf24936488d7a4744e90ca`; 58,882 bytes | Explicit admission, canonical payload serialization and immutable content-addressed records inform receipts. VM/routing geometry supplies no clause-boundary rule or parsing evidence. MIT at this pin. |
| [UOR-ADDR JSON carrier](https://github.com/UOR-Foundation/uor-addr/blob/165b51e3e2113ee5d032730cde709335d4fe9b60/crates/uor-addr/src/json/value.rs#L21) | `165b51e3e2113ee5d032730cde709335d4fe9b60`; `8cfcc7ccd1684013dc7f583bfd284631627f671c200ec052400b926fc7cf3db0`; 36,378 bytes | NFC normalization and object-member sorting mean canonical structural identity and original text bytes/spans need different contracts. |
| [UOR composition canonicalization](https://github.com/UOR-Foundation/uor-addr/blob/165b51e3e2113ee5d032730cde709335d4fe9b60/crates/uor-addr/src/composition/canonicalize.rs#L68), lines 68–118 | Same UOR-ADDR pin; `d9032fc9bc95a4f86ddbb8c0db3753865ee6de36a8f915fc446597243b8a6d89`; 15,372 bytes | `check_axis` preserves declared digest-axis compatibility. G2 canonicalization is deliberately commutative and cannot identify ordered clauses or role positions by itself. |

These sources motivate retaining raw-byte identity, ordered token/clause spans,
typed admission and separate schema/codec/artifact identities. Whitespace
normalization is many-to-one; a reversible text claim needs original bytes or
reconstruction data. No general UOR arithmetic, CRT, H4 or W33 mapping is added
under #1085. Any mathematical bridge remains separately owned by #1091 and any
typed UOR adapter by #1083.

## Original learned-reference source audit

Read at `a7f62b025c707640058e48721ef4971f8be789c5`:

- [data.py](../../tools/r4-softmax-trainer/src/r4_softmax_trainer/zoology_language_interface/data.py), lines 19–36 and 198–220: exact four fact templates, fixed query, width 13 and padding 57. `_clause` uses `template.split()`; no existing raw-text tokenizer is qualified. The question includes `? answer :` before the target.
- [core lexical data](../../tools/r4-softmax-trainer/src/r4_softmax_trainer/zoology_english_binding/data.py), lines 21–85: fixed owner/object/location lists and ID ordering. Reader aliases and core output spellings differ at IDs 52–57. Output must decode with the core vocabulary, with `unknown` at ID 11.
- [model.py](../../tools/r4-softmax-trainer/src/r4_softmax_trainer/zoology_language_interface/model.py), lines 66–85, 108–123 and 145–192: only IDs/lengths enter the model; padding alone is masked; all soft mixtures remain; the full output vocabulary is used.
- [data.py](../../tools/r4-softmax-trainer/src/r4_softmax_trainer/zoology_language_interface/data.py), lines 129–159: `_parse_clause` returns assignments, gold role positions and view. It is an evaluator audit helper, unsuitable as the inference adapter. A boolean recognizer may validate declared grammar without forwarding its captures or re-rendering input.
- [R4 attention.py](../../tools/r4-softmax-trainer/src/r4_softmax_trainer/zoology_language_r4/attention.py), lines 66–88, 177–214 and 273–277: frames fold continuously through valid punctuation/words, use clause-end positions, exclude padding and require frozen CPU/eval state. The R4 wrapper's underlying model control must remain `none`.

Fact lengths are 12, 12, 11 and 13 for views 0–3; the query has 13 tokens.
In view 3 all five clauses saturate width 13, so a nonzero boundary shift with
fixed total tokens cannot remain valid. The chosen comparison therefore uses
malformed-input refusal as its boundary negative control; it does not invent
an across-view answer sensitivity requirement or widen the reader.

The [specification](clause-segmentation-1085.md) is an interface definition plus
future empirical criterion. Source inspection is not execution, a proof of
parser correctness, or a model result. Prior #1079 weak-control and #1082
descriptive evidence remain unchanged.
