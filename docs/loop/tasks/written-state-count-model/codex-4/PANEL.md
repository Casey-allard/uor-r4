# [codex] Content selector panel preregistration

The test asks whether the learned selector can bind a named query occurrence to the correct written fact under independent changes in question position, fact order, exclusion placement and trailing text. Each seed receives identical token IDs. We fixed the data and fingerprints before model fitting or scoring. This designed panel measures transfer over its declared grammar and vocabulary.

## Population

Each split has 288 rows, comprising 144 paired role reversals. We cross two name/value families, two requested-name positions within the question, two fact orders, three exclusion conditions, three trailing-text lengths, two requested entities and two assignments of values to entities. For a paired question we keep the written sentences, fact order, value assignment, form of the question, exclusion placement and wording, and trailer fixed. The queried and excluded entities exchange roles. When exclusion is absent only the queried name changes. Where exclusion is present both complete prefixes in a pair have the same multiset of token IDs.

| Factor | TRAIN | Held-out |
|---|---|---|
| Names and target families | Tom/Sam with black/white. Amy/Max with gold coin/gold key. | Kim/Mia with green/brown. Ada/Noah with silver bell/silver leaf. |
| Written facts | Each entity “picked” its assigned value. | Each entity “took” its assigned value. |
| Query templates | “For X, what was picked?” or “Which item was picked by X?” | “About X, which item was taken?” or “What was the item that X took?” |
| Exclusion | Absent, or before/after the question with “Ignore Y.” or “Leave Y out.” | Absent, or before/after the question with “Set Y aside.” or “Skip Y.” |
| Text after the final clause | Empty, “ Answer.” or “ Give only the requested item in your answer.” | Empty, “ Respond.” or “ Write just the name of the item here.” |

No held-out entity name or answer phrase appears as a TRAIN name or answer. The splits share the dialogue framing, punctuation, sentence layout and the local two-token offset from the last fact-name token to the first value token. This limits conclusions about arbitrary fact syntax. Template variation within the question moves the queried entity independently of both fact order and trailing filler. For an exclusion present before or after the question, wording index is lexical-family index XOR assignment XOR order. Each position has 48 examples of each wording. Absent exclusion has 96 examples and no mention of the other entity after its fact. Wording is balanced across placement, but is derived from other factors.

The separate sixteen original prompt rows give eight regression pairs. They do not enter fitting or held-out denominators. Their original bytes are bound in the manifest. We have specified no random subsampling or performance-based exclusions.

## Measured token positions before fitting

Indices are zero-based. Within-question indices include the leading-space BPE on its first word. Inverse offset zero denotes the last prefix token.

| Quantity | TRAIN | Held-out |
|---|---|---|
| Distinct complete prefixes | 288 | 288 |
| Length in IDs | 26–49 | 27–59 |
| Inverse offset of final requested-name ID | 14 distinct values from 7 to 31 | 19 distinct values from 8 to 33 |
| Inverse offset of first source-value ID | 15–42 | 16–50 |
| Added tail IDs | 0, 5, 14 | 0, 5, 12 |
| Number of original-panel rows fitted | 0 | 0 |

For held-out Kim/Mia the anchor is at index 3 or 6 within the question. For Ada/Noah it is at 5 or 8 because both names occupy three IDs. We label the last ID in the entire name. The held-out terminal IDs for Kim, Mia, Ada and Noah are 4053, 655, 67 and 74, respectively. The last IDs are identical across each name's own query and fact occurrences. These terminal IDs can appear elsewhere as subword fragments. The blind input contains the full prefix and no entity-span feature.

The target sequences include the native EOS token 1. TRAIN has black [1588,1], white [1605,1], gold coin [2551,2126,1] and gold key [2551,1569,1]. Held-out has green [1074,1], brown [1730,1], silver bell [3585,322,2046,1] and silver leaf [3585,322,1667,1]. The silver pair shares its first two tokens.

## Framing, labels and blinding

The Rust generator uses the project's ByteBpeTokenizer and DialogueProtocol::literal_roles_v2. The prompt includes BOS 0, User:, space-bearing content, newline, and Assistant:. Independently encoding the completed dialogue must reproduce the complete prefix followed by exactly the source value tokens and EOS. Both public seed tokenizers reproduced all six files byte-for-byte.

- train.json rows contain only prefix, query, query_other, fact, other_fact, source and other_source. The two query fields and two fact fields name absolute indices of terminal entity IDs. The two source fields are absolute indices of the first payload IDs. query_other is null for absent exclusion.
- held-blind.json and original-blind.json each contain only objects with the single field prefix. No label, grouping, row ID or expected length is an input to inference. Row order binds the output to the independent grading files.
- held-grade.json and train-grade.json give case id, pair, role, prefix, asked_start, asked_end, other_start, other_end, expected, factors, token_positions, prompt, answer_text and other_text. All source spans are unique and half-open. original-grade.json preserves the frozen original core fields and adds a split marker. Keep the grading files out of fitting and inference.
- Each generated prefix and both expected source spans are validated by encoding the corresponding surfaces. The generator checks complete-dialogue target agreement, matching terminal IDs and occurrence counts, distinct full prefixes and maximum prefix length 192.

## Reproduction

From the repository root, run the tokenizer-only program separately for each seed. OUT is a new directory. The original input is docs/loop/tasks/written-state-count-model/codex-1/panel.json. The measured files on Wick are in /home/callard/uor-codex/20261118-content-selector/panels/ and in the main checkout’s local/codex/content-selector-panel/panels/ on the workstation. These are generated diagnostic data.

```sh
~/.cargo/bin/cargo run --locked -p uor-r4-tokenizer --example content-selector-panel -- "$TOKENIZER" docs/loop/tasks/written-state-count-model/codex-1/panel.json "$OUT"
```

The file PANEL-SHA256SUMS records SHA-256 of the exact generator, original fixture, both tokenizer files and every generated array. Verification on Wick reproduced the local checksums. The wider role-copy and milestone panels retain their own denominators.

## Independent finite checks before the run

The Studio ByteLevel BPE encoder independently reproduced all 576 Rust prefixes and 576 expected replies from decoded prompt and answer text with seed 1. Seed 2 reproduced all six Rust output files byte-for-byte. In each split, all 288 prefixes are unique, each exclusion level occurs 96 times, and each pair has identical fact sentences and exchanges the two value spans. The 96 present-clause role pairs per split also have identical full-prefix token multisets. The 96 rows per split without an exclusion contain the distractor entity only in its fact, and have null query_other. Both blind files contain only prefix in every row in the exact grade-row order. Each training target has length two or three IDs including EOS (144 each). Each held-out target has length two or four (144 each).

In-memory planted changes in both splits each triggered the expected independent check. Inserting the distractor fact position as a false absent-clause label or clearing a true before-clause label failed label presence. Relabeling an after-clause mention with the fact position failed occurrence order. Removing EOS failed the full completion comparison. Repeating a prompt reduced the unique-prefix count from 288 to 287. No changed input was written.

The pre-fit generator SHA-256 was 04b0bfa20e19825be752287de91dc443028e6ca1d76b9a0b738675ad97d875f0 (commit 70c96f313). Review subsequently strengthened its rejection of malformed original spans. The revised generator SHA-256 is 2678ff4e3e7f7260c5a95cd2165cee02e310bd029c3b50310345af820527ffa4 and is listed in PANEL-SHA256SUMS. It reproduced all six frozen arrays byte for byte. Both the out-of-range empty target and the in-range empty value with EOS-only target were rejected.
