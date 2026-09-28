"""Write a pure byte-level Hugging Face tokenizer.json.

ids: <|endoftext|>=0, <|im_start|>=1, <|im_end|>=2, then byte b -> id b + 3
(the vocab key is the GPT-2 byte-to-unicode character of b). No merges, so
every byte is one token. The specials appear both in model.vocab (the repo
parser requires a dense 0..N-1 id prefix) and in added_tokens (atomic match).
"""
import json
import sys

SPECIALS = ["<|endoftext|>", "<|im_start|>", "<|im_end|>"]
OFFSET = len(SPECIALS)


def bytes_to_unicode():
    """GPT-2 table: printable latin-1 bytes map to themselves, the other 68
    bytes map to U+0100.. in ascending byte order."""
    bs = list(range(ord("!"), ord("~") + 1)) + list(range(0xA1, 0xAC + 1)) + list(range(0xAE, 0xFF + 1))
    cs = bs[:]
    n = 0
    for b in range(256):
        if b not in bs:
            bs.append(b)
            cs.append(256 + n)
            n += 1
    return {b: chr(c) for b, c in zip(bs, cs)}


def build():
    table = bytes_to_unicode()
    vocab = {s: i for i, s in enumerate(SPECIALS)}
    for b in range(256):
        vocab[table[b]] = b + OFFSET
    assert len(vocab) == 259 and sorted(vocab.values()) == list(range(259))
    added = [
        {"id": i, "content": s, "single_word": False, "lstrip": False, "rstrip": False,
         "normalized": False, "special": True}
        for i, s in enumerate(SPECIALS)
    ]
    return {
        "version": "1.0",
        "truncation": None,
        "padding": None,
        "added_tokens": added,
        "normalizer": None,
        "pre_tokenizer": {"type": "ByteLevel", "add_prefix_space": False, "trim_offsets": True, "use_regex": True},
        "post_processor": None,
        "decoder": {"type": "ByteLevel", "add_prefix_space": True, "trim_offsets": True, "use_regex": True},
        "model": {
            "type": "BPE",
            "dropout": None,
            "unk_token": None,
            "continuing_subword_prefix": None,
            "end_of_word_suffix": None,
            "fuse_unk": False,
            "byte_fallback": False,
            "ignore_merges": False,
            "vocab": vocab,
            "merges": [],
        },
    }


if __name__ == "__main__":
    out = sys.argv[1]
    with open(out, "w", encoding="utf-8") as f:
        json.dump(build(), f, ensure_ascii=False, indent=2)
        f.write("\n")
    print("wrote", out)
