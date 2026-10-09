#!/usr/bin/env python3
"""Build a per-token-id byte-length table (lens.u16) from a HF tokenizer.json.

Mirrors uor-r4-tokenizer::Tokenizer::token_byte_lengths exactly:
  - 4096 vocab slots; added tokens decode as their literal content bytes;
  - every other token decodes through the GPT-2 byte-decoder char map, and a
    character outside the map contributes its own UTF-8 encoding;
  - unassigned ids have length 0.

Also reproduces `token-stream-bytes <stream.u16> <tokenizer.json> --skip-header 64`
so the byte basis can be checked against the recorded 2.837427 bytes/token.
"""
import json
import struct
import sys


def bytes_to_unicode():
    assigned = {}
    for byte in list(range(ord("!"), ord("~") + 1)) + list(range(0xA1, 0xAC + 1)) + list(
        range(0xAE, 0xFF + 1)
    ):
        assigned[byte] = chr(byte)
    extra = 0
    for byte in range(256):
        if byte not in assigned:
            assigned[byte] = chr(256 + extra)
            extra += 1
    return assigned


def main():
    tokenizer_path, out_lens = sys.argv[1], sys.argv[2]
    raw = open(tokenizer_path, "rb").read()
    doc = json.loads(raw)
    encoder = bytes_to_unicode()
    decoder = {ch: b for b, ch in encoder.items()}

    vocab = doc["model"]["vocab"]
    added = {t["id"]: t["content"] for t in doc.get("added_tokens", [])}
    size = max(max(vocab.values()), max(added)) + 1

    lens = [0] * size
    for token, tid in vocab.items():
        if tid in added:
            lens[tid] = len(added[tid].encode("utf-8"))
            continue
        n = 0
        for ch in token:
            if ch in decoder:
                n += 1
            else:
                n += len(ch.encode("utf-8"))
        lens[tid] = n

    with open(out_lens, "wb") as fh:
        for value in lens:
            fh.write(struct.pack("<H", value))

    args = sys.argv[3:]
    if args:
        stream = args[0]
        skip = int(args[1]) if len(args) > 1 else 64
        data = open(stream, "rb").read()[skip:]
        tokens = len(data) // 2
        total = 0
        oov = 0
        max_id = 0
        for (tid,) in struct.iter_unpack("<H", data[: tokens * 2]):
            max_id = max(max_id, tid)
            if tid < size and lens[tid]:
                total += lens[tid]
            else:
                oov += 1
        print(
            json.dumps(
                {
                    "path": stream,
                    "tokens": tokens,
                    "bytes": total,
                    "bytes_per_token": round(total / max(tokens, 1), 6),
                    "out_of_vocab": oov,
                    "max_id": max_id,
                    "vocab_slots": size,
                }
            )
        )
    print(f"wrote {out_lens}: {size} u16 entries", file=sys.stderr)


if __name__ == "__main__":
    main()
