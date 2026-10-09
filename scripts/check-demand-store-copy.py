#!/usr/bin/env python3
"""Independent verification of an emitted UOR-R4 demand store, reading ONLY the bytes on disk.

This is a second implementation of the checks the Rust generator performs, written against the
emitted artifact (tokens.u16 / response_mask.u8 / manifest.json) rather than against the generator's
internal state. It deliberately re-derives everything it can:

  * the UORT header (token count as u64 little-endian at byte offset 8) and both sha256 digests;
  * document boundaries from BOS positions alone;
  * the sixteen clauses of the demand-store generators, including the two this store reads with an
    extra condition (`document_vocabulary` = no acknowledgement is a response, and
    `response_no_interior_eos` = turn 2's answer is not turn 1's and occurs in no user turn);
  * requirement (b): for EVERY document, turn 2's answer does not occur in ANY user turn, and is not
    token-identical to turn 1's answer;
  * requirement (c): no response equals an acknowledgement;
  * the round-trip to the training path's own mask rule (crates/uor-r4-training/src/dialogue_episodes.rs
    `record_response`: the five assistant-marker ids immediately precede each response run with mask
    0, the run's last token is EOS id 1, no EOS inside the run, every other token mask 0).

Usage: python3 check_demand_store_copy.py <store-dir>
Exit status 0 only when every clause count is zero and every requirement above holds.
"""

import hashlib
import json
import struct
import sys
from pathlib import Path

BOS_ID = 0
EOS_ID = 1
UNK_ID = 2
VOCAB_SIZE = 4096
USER_MARKER = [55, 2728, 28]
ASSISTANT_MARKER = [35, 560, 652, 714, 28]
SEPARATOR_ID = 201
COMMA_ID = 14
PERIOD_ID = 16

CLAUSES = [
    "store_mask_length",
    "document_bos_unmasked",
    "document_bos_follows_eos",
    "document_terminal_masked_eos",
    "document_no_interior_eos",
    "document_no_interior_bos",
    "document_vocabulary",
    "document_mask_binary",
    "user_marker_exact",
    "response_single_run",
    "response_nonempty",
    "response_marker_present",
    "response_marker_unmasked",
    "response_marker_exact",
    "response_terminal_eos",
    "response_no_interior_eos",
]

ACKNOWLEDGEMENTS = [
    "Noted.",
    "I see.",
    "Got it.",
    "Understood.",
    "Right.",
    "Sure.",
    "Okay.",
    "OK.",
    "Alright.",
    "Very well.",
    "Of course.",
    "Certainly.",
]


def build_byte_decoder(tokenizer_path):
    """The byte-level BPE decoding table, built from the tokenizer's own vocabulary."""
    vocabulary = json.loads(Path(tokenizer_path).read_text())
    inverse = {identifier: token for token, identifier in vocabulary["model"]["vocab"].items()}
    table = {}
    extra = 0
    for byte in range(256):
        if (0x21 <= byte <= 0x7E) or (0xA1 <= byte <= 0xAC) or (0xAE <= byte <= 0xFF):
            table[chr(byte)] = byte
        else:
            table[chr(256 + extra)] = byte
            extra += 1
    return inverse, table


def decode_ids(ids, inverse, table):
    """Decode token ids to text with the byte-level BPE rule, independently of the Rust tool."""
    raw = b"".join(bytes([table[character]]) for identifier in ids for character in inverse[identifier])
    return raw.decode("utf-8", errors="replace")


class Ledger:
    def __init__(self):
        self.counts = {clause: 0 for clause in CLAUSES}
        self.offenders = []
        self.ack_hits = 0
        self.dependency_hits = 0

    def hit(self, clause, document, detail):
        self.counts[clause] += 1
        if clause == "document_vocabulary":
            self.ack_hits += 1
        if clause == "response_no_interior_eos":
            self.dependency_hits += 1
        if len(self.offenders) < 20:
            self.offenders.append((document, clause, detail))

    def total(self):
        return sum(self.counts.values())


def mask_runs(mask, start=0, end=None):
    """Maximal masked runs of a document, computed from the byte mask alone."""
    end = len(mask) if end is None else end
    runs = []
    offset = start
    while offset < end:
        if mask[offset] == 1:
            run_start = offset
            while offset < end and mask[offset] == 1:
                offset += 1
            runs.append((run_start, offset))
        else:
            offset += 1
    return runs


def document_bounds(tokens):
    """(start, end) of every document, from BOS positions alone, plus the end of the stream."""
    starts = [i for i, token in enumerate(tokens) if token == BOS_ID]
    starts.append(len(tokens))
    return starts


def user_content_spans(tokens):
    """Content span of every user turn: after a User: marker up to the separator closing the turn."""
    spans = []
    for marker in range(len(tokens) - len(USER_MARKER) + 1):
        if tokens[marker:marker + len(USER_MARKER)] != USER_MARKER:
            continue
        end = marker + len(USER_MARKER)
        while end < len(tokens):
            if tokens[end] == SEPARATOR_ID:
                break
            if tokens[end:end + len(ASSISTANT_MARKER)] == ASSISTANT_MARKER:
                break
            end += 1
        spans.append((marker + len(USER_MARKER), end))
    return spans


def contains_span(haystack, needle):
    if not needle or len(needle) > len(haystack):
        return False
    return any(haystack[i:i + len(needle)] == needle for i in range(len(haystack) - len(needle) + 1))


def answer_core(answer):
    return answer.strip().rstrip(".").strip().lower()


def contains_answer_word(text, core):
    """Does `text` contain `core` as a whole word (space before, non-alphanumeric after)?"""
    if not core:
        return False
    haystack = text.strip().lower()
    from_index = 0
    while True:
        found = haystack.find(core, from_index)
        if found < 0:
            return False
        start, end = found, found + len(core)
        before_ok = start > 0 and haystack[start - 1] in " \n\t"
        after_ok = end == len(haystack) or not haystack[end].isalnum()
        if before_ok and after_ok:
            return True
        from_index = start + 1


def is_acknowledgement(text):
    return text.strip() in [ack.strip() for ack in ACKNOWLEDGEMENTS]


def validate_documents(tokens, mask, documents, texts, id_texts=None):
    """The sixteen clauses over the flat stream, in the multi-turn reading (plus the two extras)."""
    ledger = Ledger()
    if len(tokens) != len(mask):
        ledger.hit("store_mask_length", 0, f"{len(tokens)} tokens against {len(mask)} mask bytes")
        return ledger
    if not tokens:
        ledger.hit("document_bos_unmasked", 0, "empty stream")
        return ledger

    starts = document_bounds(tokens)
    if tokens[0] != BOS_ID:
        ledger.hit("document_bos_unmasked", 0, f"first token is {tokens[0]} not BOS")
    for position, token in enumerate(tokens):
        if token == BOS_ID and position != 0 and tokens[position - 1] != EOS_ID:
            ledger.hit(
                "document_bos_follows_eos",
                0,
                f"BOS at {position} follows token {tokens[position - 1]}",
            )

    for document in range(len(starts) - 1):
        start, end = starts[document], starts[document + 1]
        doc_tokens = tokens[start:end]
        doc_mask = mask[start:end]
        last = len(doc_tokens) - 1
        premise, demand, answer1, answer2 = texts[document]
        if id_texts is not None:
            answer1_ids, answer2_ids = id_texts[document][2], id_texts[document][3]
            if answer1_ids == answer2_ids:
                ledger.hit(
                    "response_no_interior_eos",
                    document,
                    "turn 2's answer is token-identical to turn 1's answer",
                )

        if doc_tokens[0] != BOS_ID or doc_mask[0] != 0:
            ledger.hit(
                "document_bos_unmasked",
                document,
                f"document start {start} is {doc_tokens[0]} with mask {doc_mask[0]}",
            )
        if doc_tokens[last] != EOS_ID or doc_mask[last] != 1:
            ledger.hit(
                "document_terminal_masked_eos",
                document,
                f"final token {doc_tokens[last]} carries mask {doc_mask[last]}",
            )
        if any(
            token == EOS_ID and doc_mask[offset] == 0
            for offset, token in enumerate(doc_tokens[:last])
        ):
            ledger.hit(
                "document_no_interior_eos",
                document,
                "an unmasked EOS appears before the document end",
            )
        if any(token == BOS_ID for token in doc_tokens[1:]):
            ledger.hit("document_no_interior_bos", document, "a BOS appears inside the document")
        for offset, token in enumerate(doc_tokens):
            if token >= VOCAB_SIZE or token == UNK_ID:
                ledger.hit("document_vocabulary", document, f"token {token} at offset {offset}")
        # This store's extra condition on the clause: no response content is an acknowledgement.
        for text in (answer1, answer2):
            if is_acknowledgement(text):
                ledger.hit("document_vocabulary", document, f"response {text!r} is an acknowledgement")
        for offset, value in enumerate(doc_mask):
            if value > 1:
                ledger.hit("document_mask_binary", document, f"mask byte {value} at offset {offset}")

        # Every User: marker must be the exact literal with mask 0.
        if (
            len(doc_tokens) < 4
            or doc_tokens[1:4] != USER_MARKER
            or any(value != 0 for value in doc_mask[1:4])
        ):
            ledger.hit("user_marker_exact", document, "the opening User: marker is not exact")
        for cursor in range(len(doc_tokens) - len(USER_MARKER) + 1):
            if doc_tokens[cursor:cursor + len(USER_MARKER)] == USER_MARKER and any(
                value != 0 for value in doc_mask[cursor:cursor + len(USER_MARKER)]
            ):
                ledger.hit(
                    "user_marker_exact",
                    document,
                    f"a User: marker at offset {cursor} carries a masked token",
                )
                break

        # Every assistant marker opens exactly one contiguous masked run closing at its own EOS,
        # followed by an unmasked token or the document end.
        for marker in range(len(doc_tokens) - len(ASSISTANT_MARKER) + 1):
            if doc_tokens[marker:marker + len(ASSISTANT_MARKER)] != ASSISTANT_MARKER:
                continue
            run_start = marker + len(ASSISTANT_MARKER)
            if run_start >= len(doc_tokens) or doc_mask[run_start] != 1:
                ledger.hit(
                    "response_single_run",
                    document,
                    f"assistant marker at offset {marker} opens no masked response",
                )
                continue
            run_end = run_start
            while run_end < len(doc_mask) and doc_mask[run_end] == 1:
                run_end += 1
            eos = next(
                (p for p in range(run_start, run_end) if doc_tokens[p] == EOS_ID),
                None,
            )
            if eos is None or eos + 1 != run_end or (
                eos + 1 != len(doc_tokens) and doc_mask[eos + 1] != 0
            ):
                ledger.hit(
                    "response_single_run",
                    document,
                    f"the masked stretch after the marker at offset {marker} does not close at its "
                    "own EOS followed by an unmasked token",
                )

        # Per-run clauses.
        for run_start, run_end in mask_runs(doc_mask):
            if run_start >= run_end:
                ledger.hit("response_nonempty", document, "empty response run")
                continue
            if run_start <= len(ASSISTANT_MARKER):
                ledger.hit(
                    "response_marker_present",
                    document,
                    f"response run starts at document offset {run_start}",
                )
                continue
            marker_start = run_start - len(ASSISTANT_MARKER)
            if any(value != 0 for value in doc_mask[marker_start:run_start]):
                ledger.hit(
                    "response_marker_unmasked",
                    document,
                    f"mask {doc_mask[marker_start:run_start]} over the marker",
                )
            if doc_tokens[marker_start:run_start] != ASSISTANT_MARKER:
                ledger.hit(
                    "response_marker_exact",
                    document,
                    f"tokens {doc_tokens[marker_start:run_start]} before the response run",
                )
            if doc_tokens[run_end - 1] != EOS_ID or doc_mask[run_end - 1] != 1:
                ledger.hit(
                    "response_terminal_eos",
                    document,
                    f"last response token is {doc_tokens[run_end - 1]} with mask "
                    f"{doc_mask[run_end - 1]}",
                )
            if any(token == EOS_ID for token in doc_tokens[run_start:run_end - 1]):
                ledger.hit(
                    "response_no_interior_eos",
                    document,
                    "an EOS appears inside the response run",
                )

        # This store's extra condition on the dependency clause.
        if answer1 == answer2:
            ledger.hit(
                "response_no_interior_eos",
                document,
                f"turn 2's answer {answer2!r} is identical to turn 1's answer",
            )
        core = answer_core(answer2)
        if contains_answer_word(premise, core) or contains_answer_word(demand, core):
            ledger.hit(
                "response_no_interior_eos",
                document,
                f"turn 2's answer core {core!r} occurs as a word in a user turn "
                f"(premise {premise!r}, demand {demand!r})",
            )
    return ledger


def main():
    if len(sys.argv) != 2:
        print("usage: check_demand_store_copy.py <store-dir>")
        return 2
    directory = Path(sys.argv[1])
    tokens_bytes = (directory / "tokens.u16").read_bytes()
    mask_bytes = (directory / "response_mask.u8").read_bytes()
    manifest = json.loads((directory / "manifest.json").read_text())

    failures = []

    # --- header and digests -------------------------------------------------
    if len(tokens_bytes) < 64:
        print("FAIL: tokens.u16 is shorter than the UORT header")
        return 1
    declared = struct.unpack_from("<Q", tokens_bytes, 8)[0]
    payload = tokens_bytes[64:]
    tokens = list(struct.unpack(f"<{len(payload) // 2}H", payload))
    mask = list(mask_bytes)
    tokens_sha = hashlib.sha256(tokens_bytes).hexdigest()
    mask_sha = hashlib.sha256(mask_bytes).hexdigest()
    print(f"file tokens.u16: {len(tokens_bytes)} bytes, header token count {declared}, "
          f"payload {len(tokens)} u16 tokens")
    print(f"file response_mask.u8: {len(mask)} bytes")
    print(f"sha256 tokens.u16      = {tokens_sha}")
    print(f"sha256 response_mask.u8= {mask_sha}")
    print(f"sha256 manifest.json   = {hashlib.sha256((directory / 'manifest.json').read_bytes()).hexdigest()}")
    if declared != len(tokens):
        failures.append(f"header token count {declared} != payload tokens {len(tokens)}")
    if len(tokens) != len(mask):
        failures.append(f"{len(tokens)} tokens against {len(mask)} mask bytes")
    if manifest["tokens_sha256"] != tokens_sha:
        failures.append("manifest tokens_sha256 does not match the file")
    if manifest["mask_sha256"] != mask_sha:
        failures.append("manifest mask_sha256 does not match the file")
    if manifest["tokens"] != len(tokens) or manifest["tokens_bytes"] != len(tokens_bytes):
        failures.append("manifest token counts do not match the file")
    if manifest["mask_bytes"] != len(mask):
        failures.append("manifest mask_bytes does not match the file")
    if manifest["drops"]["special_token_occurrences"] != 0:
        failures.append("manifest drops.special_token_occurrences is not 0")
    if manifest["response_tokens"] != sum(mask):
        failures.append("manifest response_tokens does not match the mask")

    # --- document count and the two-turn text of every document --------------
    starts = document_bounds(tokens)
    documents = len(starts) - 1
    print(f"documents (BOS-delimited): {documents}")

    # The independent reader recovers each document's premise, demand and both answers from the
    # byte stream: user turns are the unmasked stretches after a User: marker, response runs are the
    # masked stretches. The comparison texts the containment check needs are the decoded ids only by
    # way of the manifest's own copy rule, so the check is re-run from ids where it can be: the
    # token-identical reading, and the token span of turn 2's answer inside a user turn.
    texts = []
    for document in range(documents):
        start, end = starts[document], starts[document + 1]
        doc_tokens = tokens[start:end]
        doc_mask = mask[start:end]
        spans = user_content_spans(doc_tokens)
        runs = mask_runs(doc_mask)
        runs = [run for run in runs if run[0] > len(ASSISTANT_MARKER)]
        if len(spans) != 2 or len(runs) != 2:
            texts.append((None, None, None, None))
            continue
        premise = doc_tokens[spans[0][0]:spans[0][1]]
        demand = doc_tokens[spans[1][0]:spans[1][1]]
        answer1 = doc_tokens[runs[0][0]:runs[0][1] - 1]
        answer2 = doc_tokens[runs[1][0]:runs[1][1] - 1]
        texts.append((premise, demand, answer1, answer2))

    # --- requirement (b) and (c) from the ids alone --------------------------
    equal_count = 0
    token_span_in_user = 0
    answer2_in_user_tokens = 0
    for document, entry in enumerate(texts):
        if entry[0] is None:
            failures.append(f"document {document} does not hold exactly two user turns and two runs")
            continue
        premise, demand, answer1, answer2 = entry
        if answer1 == answer2:
            equal_count += 1
        if contains_span(premise, answer2) or contains_span(demand, answer2):
            token_span_in_user += 1
        if contains_span(premise, answer2):
            answer2_in_user_tokens += 1
    print(f"(b) id-level: turn-2 answer token-identical to turn-1 answer: {equal_count}")
    print(f"(b) id-level: turn-2 answer token span inside a user turn: {token_span_in_user}")
    print(f"(b) id-level: turn-2 answer token span inside the PREMISE: {answer2_in_user_tokens}")

    # --- requirement (b)/(c) at the phrase level, from the manifest's own texts
    copy_rule = manifest.get("copy_rule", {})
    word_in_user = copy_rule.get("turn2_answer_in_a_user_turn")
    acks = copy_rule.get("acknowledgements_present")
    acks_loose = copy_rule.get("acknowledgements_present_case_insensitive")
    print(f"(c) manifest acknowledgements_present = {acks} (case-insensitive {acks_loose})")
    print(f"(b) manifest turn2_answer_in_a_user_turn = {word_in_user}")
    print(f"(b) manifest turn2_answer_equals_turn1_answer = "
          f"{copy_rule.get('turn2_answer_equals_turn1_answer')}")
    print(f"(b) manifest turn1_answer_not_a_premise_span = "
          f"{copy_rule.get('turn1_answer_not_a_premise_span')}")
    if acks != 0 or acks_loose != 0:
        failures.append("responses are acknowledgements")
    if word_in_user != 0:
        failures.append("turn 2's answer occurs in a user turn")
    if equal_count != 0:
        failures.append("turn 2's answer is token-identical to turn 1's answer")
    if copy_rule.get("turn2_answer_equals_turn1_answer") != 0:
        failures.append("manifest reports answer2 == answer1")

    # --- the sixteen clauses -------------------------------------------------
    # The clause pass needs each document's TEXT for the two augmented conditions (no acknowledgement
    # is a response; turn 2's answer is in no user turn). The text is decoded here from the emitted
    # ids with this file's own byte-level BPE decoder, so nothing is taken from the generator.
    inverse, table = build_byte_decoder(manifest["tokenizer"]["path"])
    decoded = []
    for entry in texts:
        if entry[0] is None:
            decoded.append(("", "", "", ""))
        else:
            premise = decode_ids(entry[0], inverse, table)
            demand = decode_ids(entry[1], inverse, table)
            answer1 = decode_ids(entry[2], inverse, table)
            answer2 = decode_ids(entry[3], inverse, table)
            decoded.append((premise.strip(), demand.strip(), answer1.strip(), answer2.strip()))
    if decoded and decoded[0][0]:
        print(f"decoded document 0: premise {decoded[0][0]!r} -> turn-1 {decoded[0][2]!r} -> "
              f"demand {decoded[0][1]!r} -> turn-2 {decoded[0][3]!r}")
    word_in_user_independent = 0
    for premise, demand, _answer1, answer2 in decoded:
        core = answer_core(answer2)
        if contains_answer_word(premise, core) or contains_answer_word(demand, core):
            word_in_user_independent += 1
    print(f"(b) independent word-level: turn-2 answer core in a user turn: "
          f"{word_in_user_independent} of {len(decoded)}")
    if word_in_user_independent != 0:
        failures.append("independent word-level containment found turn 2's answer in a user turn")

    id_texts = []
    for entry in texts:
        if entry[0] is None:
            id_texts.append(("", "", [], []))
        else:
            id_texts.append(("", "", entry[2], entry[3]))
    ledger = validate_documents(tokens, mask, documents, decoded, id_texts)
    print(f"validation (independent Python reader, 16 clauses): {ledger.total()} offenders "
          f"(acknowledgement hits {ledger.ack_hits}, dependency hits {ledger.dependency_hits})")
    for clause in CLAUSES:
        print(f"  clause {clause}: {ledger.counts[clause]}")
    if ledger.total() != 0:
        for document, clause, detail in ledger.offenders[:8]:
            print(f"  offending document {document} clause {clause}: {detail}")
        failures.append("clause offenders are non-zero")

    # --- counts the brief asks to see ---------------------------------------
    lengths1 = [len(entry[2]) for entry in texts if entry[0] is not None]
    lengths2 = [len(entry[3]) for entry in texts if entry[0] is not None]
    response_tokens = sum(mask)
    print(f"response tokens (mask sum): {response_tokens}")
    print(f"turn-1 answer tokens: total {sum(lengths1)}, mean {sum(lengths1) / len(lengths1):.4f}, "
          f"max {max(lengths1)}")
    print(f"turn-2 answer tokens: total {sum(lengths2)}, mean {sum(lengths2) / len(lengths2):.4f}, "
          f"max {max(lengths2)}")
    print(f"manifest diversity: {json.dumps(manifest.get('diversity', {}).get('distinct_value_template_combinations'))} "
          f"(value, template) combinations over "
          f"{manifest.get('diversity', {}).get('distinct_values')} values and "
          f"{manifest.get('diversity', {}).get('distinct_templates')} templates")
    print(f"manifest reader: {json.dumps(manifest.get('reader'))}")

    if failures:
        print("\nFAILURES:")
        for failure in failures:
            print(f"  - {failure}")
        return 1
    print("\nOK: every independent check passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
