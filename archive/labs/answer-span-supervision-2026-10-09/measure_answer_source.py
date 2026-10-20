#!/usr/bin/env python3
"""Measure what the response loss is actually supervised on, per source.

Why this exists: the answer-span round asked whether supervising the value's own
span instead of the whole response run could remove the template prefix the
failure carries. It cannot if the supervised answers ARE the value, or if the
prefix is never supervised at all, and this script measures exactly that on a
prepared chat store (`uor-r4-chat-corpus/v1`: `tokens.u16`, `response_mask.u8`,
`tokenizer.json`) with no model and no GPU.

    python3 measure_answer_source.py TOKENS.u16 RESPONSE_MASK.u8 TOKENIZER.json
    python3 measure_answer_source.py DCOPY/tokens.u16 DCOPY/response_mask.u8 tok.json \
        CHAT/tokens.u16 CHAT/response_mask.u8

Reports, per store: the payload/header split, documents (BOS), supervised
response runs, content tokens per run, single-word answers, answers carrying
English sentence scaffolding (`is/are/was/were/my/your/the/that/did/does`), and
the occurrences of the phrases a "My name is X." reply would need. All counts
are of the bytes on disk; nothing is read from a manifest, because the `dcopy`
manifest on the pod does not describe its own files (its `tokens` and
`tokens_sha256` disagree with the file, while `mask_sha256` agrees).

With a second store it also reports the response-INITIAL token distribution,
which is how the ` My| name| is` response prior was counted in the record: 13 of
129,465 supervised chat responses, against 0 occurrences of `my name` anywhere
in the copy source.
"""

import argparse
import collections
import json
import re
import struct
import sys


def byte_level_decoder():
    """GPT-2 byte-level BPE: the character -> the byte it stands for."""
    printable = (
        list(range(ord("!"), ord("~") + 1))
        + list(range(ord("\u00a1"), ord("\u00ac") + 1))
        + list(range(ord("\u00ae"), ord("\u00ff") + 1))
    )
    base = printable[:]
    chars = printable[:]
    extra = 0
    for byte in range(256):
        if byte not in base:
            base.append(byte)
            chars.append(256 + extra)
            extra += 1
    return {chr(c): bytes([b]) for b, c in zip(base, chars)}


class Decoder:
    def __init__(self, tokenizer_path):
        spec = json.load(open(tokenizer_path))
        self.inverse = {v: k for k, v in spec["model"]["vocab"].items()}
        self.added = {a["id"]: a["content"] for a in spec.get("added_tokens", [])}
        self.byte = byte_level_decoder()

    def piece(self, token):
        if token in self.added:
            return self.added[token]
        surface = self.inverse.get(token)
        if surface is None:
            return "<unk>"
        return b"".join(self.byte.get(c, c.encode()) for c in surface).decode(
            "utf-8", "replace"
        )

    def text(self, tokens):
        return "".join(self.piece(token) for token in tokens)

    def ids_of(self, surface):
        """The id sequence a piece-wise search for `surface` can use."""
        out = []
        for want in surface:
            for token, piece in self.inverse.items():
                if self.piece(token) == want:
                    out.append(token)
                    break
            else:
                return None
        return out


def payload_and_mask(tokens_path, mask_path):
    raw = open(tokens_path, "rb").read()
    tokens = list(struct.unpack("<%dH" % (len(raw) // 2), raw))
    mask = open(mask_path, "rb").read()
    if len(tokens) < len(mask):
        raise SystemExit("%s holds fewer ids than %s mask bytes" % (tokens_path, mask_path))
    payload = tokens[len(tokens) - len(mask):]
    return payload, mask, len(tokens) - len(mask)


def runs_of(mask):
    runs, start = [], None
    for index, value in enumerate(mask):
        if value == 1 and start is None:
            start = index
        elif value == 0 and start is not None:
            runs.append((start, index))
            start = None
    if start is not None:
        runs.append((start, len(mask)))
    return runs


SCAFFOLD = re.compile(r"\b(is|are|was|were|my|your|the|that|did|does)\b", re.I)
PHRASES = ["My name is", "my name", "name is", "is called", "Your ", "What is my"]


def report(name, tokens_path, mask_path, decoder):
    payload, mask, header = payload_and_mask(tokens_path, mask_path)
    runs = runs_of(mask)
    bodies = [decoder.text(payload[s:e]).replace("<|eos|>", "") for s, e in runs]
    text = decoder.text(payload)
    content = [e - s - 1 for s, e in runs]
    scaffold = [body for body in bodies if SCAFFOLD.search(body)]
    single = [body for body in bodies if len(body.split()) == 1]
    print("=== %s" % name)
    print("header u16 %d, payload %d, mask bytes %d, mask ones %d"
          % (header, len(payload), len(mask), sum(mask)))
    print("documents (BOS) %d, supervised response runs %d"
          % (sum(1 for token in payload if token == 0), len(runs)))
    if content:
        print("content tokens per run: min %d mean %.2f max %d"
              % (min(content), sum(content) / len(content), max(content)))
    print("single-word answers %d of %d (%.4f)"
          % (len(single), len(bodies), len(single) / max(1, len(bodies))))
    print("answers with sentence scaffolding %d of %d (%.4f)"
          % (len(scaffold), len(bodies), len(scaffold) / max(1, len(bodies))))
    print("occurrences in the whole source: "
          + ", ".join("%r %d" % (phrase, text.count(phrase)) for phrase in PHRASES))
    return payload, mask


def report_response_initials(name, payload, mask, decoder):
    starts = [index for index in range(1, len(mask)) if mask[index] == 1 and mask[index - 1] == 0]
    first = collections.Counter(payload[index] for index in starts)
    print("=== %s response-initial tokens" % name)
    print("supervised response runs %d" % len(starts))
    print("top response-initial ids %s" % (first.most_common(6),))
    for surface in (" My", " My name is"):
        ids = decoder.ids_of(surface)
        if ids is None:
            print("%r: no piece-wise id sequence" % surface)
            continue
        count = sum(
            1 for index in starts if payload[index:index + len(ids)] == ids
        )
        print("runs starting with %r: %d (ids %s)" % (surface, count, ids))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("tokens")
    parser.add_argument("mask")
    parser.add_argument("tokenizer")
    parser.add_argument("second_tokens", nargs="?")
    parser.add_argument("second_mask", nargs="?")
    args = parser.parse_args()
    decoder = Decoder(args.tokenizer)
    report("source", args.tokens, args.mask, decoder)
    if args.second_tokens and args.second_mask:
        payload, mask = report("second source", args.second_tokens, args.second_mask, decoder)
        report_response_initials("second source", payload, mask, decoder)
    return 0


if __name__ == "__main__":
    sys.exit(main())
