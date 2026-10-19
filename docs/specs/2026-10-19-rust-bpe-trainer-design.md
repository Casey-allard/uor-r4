# Deterministic Rust BPE trainer

Authority: owner directs parallel useful work; PM6260043935 specifies merge BPE, descending weighted count then Unicode code-point pair order, BOS/EOS/UNK ids0/1/2, shared parser/byte encoder, target16,384/u16. Existing tokenizer format and runtime remain compatible. This is host data preparation only; no corpus campaign/model execution or vocabulary migration is released.

Use all256 byte symbols after specials, weighted unique pre-tokens, a pair-to-word index and a lazy max heap. Update only words containing the selected pair. Ties compare left then right token strings by Unicode scalar order (UTF8 lexical order agrees); apply non-overlapping occurrences left-to-right. Reuse an existing concatenated symbol when needed; retain its new merge rank. Stop at target vocabulary or minimum pair frequency, reporting the actual attained size. Never pad with fabricated merges.

Alternatives: repeated full-corpus recount is simple but unsuitable at16k; external tokenizer dependencies duplicate the shared policy. Incremental indexing keeps deterministic semantics while avoiding whole-corpus rescans per merge.

Training shares the exact segment/pretoken iteration used by encoding, including Digits, prefix spaces and atomic specials. Output HF-shaped tokenizer.json, rank-ordered merges, dense model vocabulary, atomic BOS/EOS/UNK. Full byte alphabet preserves unseen UTF8. Input files are separate documents; no pair crosses files, pre-token boundaries or specials. Configurable byte/unique-pretoken limits, u16 size ceiling, checked counters; fail before writing if input/limits invalid. Output create-new prevents accidental overwrite.

Validation: deterministic ties/order/repeated runs, overlap, duplicate symbol reuse, weighted counts, Unicode/unseen-byte roundtrip, Digits/prefix/special boundaries, early exhaustion, invalid/oversized inputs, CLI roundtrip and refusal to overwrite. Compare incremental merge sequence to a small independent exhaustive trainer in tests. Source review before delivery; no speedup claim without measurements.
