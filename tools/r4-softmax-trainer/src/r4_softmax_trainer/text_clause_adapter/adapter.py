"""Deterministic raw-byte entry to the frozen five-clause reader (#1094).

Only lexical IDs, lengths and raw-byte provenance are produced. The local
grammar recognizer returns booleans; it does not reconstruct a semantic world,
select a role, compute an answer or call the model. Artifact verification and
model execution belong to the wrapper, which must verify startup before
calling ``segment_request``.
"""

from __future__ import annotations

import hashlib
import json
import struct
from collections.abc import Sequence
from pathlib import Path
from types import MappingProxyType
from typing import NamedTuple


POLICY_BYTES = Path(__file__).with_name("policy.json").read_bytes()
POLICY_SHA256 = hashlib.sha256(POLICY_BYTES).hexdigest()
_POLICY = json.loads(POLICY_BYTES)

REQUEST_SCHEMA = _POLICY["request"]["schema"]
RESULT_SCHEMA = _POLICY["result_schema"]
VOCABULARY_FILE_CID = _POLICY["lexical_artifact"]["cid"]
MAX_BYTES = _POLICY["limits"]["max_bytes"]
MAX_TOKENS = _POLICY["limits"]["tokens_per_clause"]
PAD_ID = _POLICY["limits"]["padding_id"]
READER_PREFIX = tuple(_POLICY["lexical_artifact"]["reader_prefix_by_id"])
_REJECTED_IDS = frozenset(_POLICY["lexical_artifact"]["rejected_input_ids"])
TOKEN_IDS = MappingProxyType(
    {
        word: index
        for index, word in enumerate(READER_PREFIX)
        if index not in _REJECTED_IDS
    }
)
OWNERS = tuple(_POLICY["syntax"]["owners"])
OBJECTS = tuple(_POLICY["syntax"]["objects"])
LOCATIONS = tuple(_POLICY["syntax"]["locations"])
_CATEGORY_MEMBERS = {
    "O": frozenset(OWNERS),
    "D": frozenset(OWNERS),
    "X": frozenset(OBJECTS),
    "L": frozenset(LOCATIONS),
}
_FACT_FORMS = tuple(
    tuple(form.split()) for form in _POLICY["syntax"]["fact_forms"]
)
_QUERY_FORM = tuple(_POLICY["syntax"]["query_form"].split())
_PUNCTUATION = frozenset(b".,?:")
_QUERY_SUFFIX = ("?", "answer", ":")


class _Token(NamedTuple):
    word: str
    token_id: int
    start: int
    end: int


def _refuse(status: str, byte_offset: int | None) -> dict:
    return {
        "schema": RESULT_SCHEMA,
        "status": status,
        "byte_offset": byte_offset,
    }


def unavailable_artifact() -> dict:
    """Return the startup refusal without reading or parsing any request."""

    return _refuse("UNAVAILABLE_ARTIFACT", None)


def _lex(raw: bytes) -> list[_Token] | dict:
    """Scan accepted ASCII directly; every accepted byte is strict UTF-8.

    Valid non-ASCII UTF-8 and invalid UTF-8 have the same required refusal tag.
    Rejecting their first non-ASCII byte also keeps encoding/lexical failures
    in byte order rather than letting a later error replace an earlier one.
    """

    tokens: list[_Token] = []
    cursor = 0
    while cursor < len(raw):
        byte = raw[cursor]
        if byte in (32, 9, 10):
            cursor += 1
            continue
        if byte == 13:
            if cursor + 1 == len(raw) or raw[cursor + 1] != 10:
                return _refuse("INVALID_ENCODING", cursor)
            cursor += 2
            continue
        start = cursor
        if 97 <= byte <= 122:
            cursor += 1
            while cursor < len(raw) and 97 <= raw[cursor] <= 122:
                cursor += 1
            word = raw[start:cursor].decode("ascii")
            token_id = TOKEN_IDS.get(word)
            if token_id is None:
                return _refuse("UNKNOWN_LEXEME", start)
        elif byte in _PUNCTUATION:
            cursor += 1
            word = chr(byte)
            token_id = TOKEN_IDS[word]
        else:
            return _refuse("INVALID_ENCODING", cursor)
        tokens.append(_Token(word, token_id, start, cursor))
    return tokens


def _boundaries(tokens: list[_Token], end: int) -> list[list[_Token]] | dict:
    """Recover literal delimiter spans before examining admitted grammar."""

    clauses: list[list[_Token]] = []
    start = 0
    for index, token in enumerate(tokens):
        if token.word != ".":
            continue
        if index == start:
            return _refuse("UNSUPPORTED_BOUNDARY", token.start)
        if len(clauses) == 4:
            return _refuse("UNSUPPORTED_BOUNDARY", token.start)
        clauses.append(tokens[start : index + 1])
        start = index + 1
    if len(clauses) != 4 or start == len(tokens):
        return _refuse("UNSUPPORTED_BOUNDARY", end)

    query = tokens[start:]
    question_mark = next(
        (index for index, token in enumerate(query) if token.word == "?"), None
    )
    if question_mark is None:
        return _refuse("UNSUPPORTED_BOUNDARY", end)
    for offset, expected in enumerate(_QUERY_SUFFIX):
        index = question_mark + offset
        if index == len(query):
            return _refuse("UNSUPPORTED_BOUNDARY", end)
        if query[index].word != expected:
            return _refuse("UNSUPPORTED_BOUNDARY", query[index].start)
    following = question_mark + len(_QUERY_SUFFIX)
    if following != len(query):
        return _refuse("UNSUPPORTED_BOUNDARY", query[following].start)
    clauses.append(query)
    return clauses


def _matches_clause(words: tuple[str, ...], form: tuple[str, ...]) -> bool:
    """Boolean-only recognition: no captured roles or semantic assignments."""

    if len(words) != len(form):
        return False
    for word, predicate in zip(words, form, strict=True):
        members = _CATEGORY_MEMBERS.get(predicate)
        if members is None:
            if word != predicate:
                return False
        elif word not in members:
            return False
    return words[form.index("O")] != words[form.index("D")]


def _syntax_refusal(clauses: list[list[_Token]]) -> dict | None:
    words = [tuple(token.word for token in clause) for clause in clauses]
    admitted_form = next(
        (form for form in _FACT_FORMS if _matches_clause(words[0], form)), None
    )
    if admitted_form is None:
        return _refuse("UNSUPPORTED_SYNTAX", clauses[0][0].start)
    for index in range(1, 4):
        if not _matches_clause(words[index], admitted_form):
            return _refuse("UNSUPPORTED_SYNTAX", clauses[index][0].start)
    if not _matches_clause(words[4], _QUERY_FORM):
        return _refuse("UNSUPPORTED_SYNTAX", clauses[4][0].start)
    return None


def derived_input_sha256(
    inputs: Sequence[Sequence[Sequence[int]]], lengths: Sequence[Sequence[int]]
) -> str:
    """Hash the normative ordered i64 little-endian tensor framing.

    These are the transport-independent Python integer arrays of the success
    record. The caller converts them to int64 tensors only at the model seam.
    """

    if (
        len(inputs) != 1
        or len(inputs[0]) != 5
        or any(len(clause) != MAX_TOKENS for clause in inputs[0])
        or len(lengths) != 1
        or len(lengths[0]) != 5
        or any(
            type(token_id) is not int or not 0 <= token_id < 4096
            for clause in inputs[0]
            for token_id in clause
        )
        or any(
            type(length) is not int or not 1 <= length <= MAX_TOKENS
            for length in lengths[0]
        )
    ):
        raise ValueError("derived input requires i64 [1,5,13] and lengths [1,5]")
    digest = hashlib.sha256()
    for value in (
        "uor-r4.text-to-clauses-input/1",
        POLICY_SHA256,
        VOCABULARY_FILE_CID,
        "i64le",
    ):
        encoded = value.encode("utf-8")
        digest.update(struct.pack("<I", len(encoded)))
        digest.update(encoded)
    digest.update(struct.pack("<5I", 1, 5, 13, 1, 5))
    for clause in inputs[0]:
        digest.update(struct.pack("<13q", *clause))
    digest.update(struct.pack("<5q", *lengths[0]))
    return digest.hexdigest()


def segment_request(request: object) -> dict:
    """Segment one exact-schema raw request after wrapper startup verification.

    A successful result contains no role, target, semantic-world, answer or
    view fields. Refusals expose only their status and optional byte offset.
    This function has no model or artifact handles and performs zero forwards.
    """

    if (
        type(request) is not dict
        or set(request) != {"schema", "text"}
        or type(request["schema"]) is not str
        or request["schema"] != REQUEST_SCHEMA
        or type(request["text"]) is not bytes
    ):
        return _refuse("UNSUPPORTED_SCHEMA", None)
    raw = request["text"]
    if len(raw) > MAX_BYTES:
        return _refuse("INPUT_LIMIT", MAX_BYTES)

    tokens = _lex(raw)
    if isinstance(tokens, dict):
        return tokens
    clauses = _boundaries(tokens, len(raw))
    if isinstance(clauses, dict):
        return clauses
    for clause in clauses:
        if len(clause) > MAX_TOKENS:
            return _refuse("INPUT_LIMIT", clause[MAX_TOKENS].start)
    refusal = _syntax_refusal(clauses)
    if refusal is not None:
        return refusal

    inputs = [
        [
            [token.token_id for token in clause]
            + [PAD_ID] * (MAX_TOKENS - len(clause))
            for clause in clauses
        ]
    ]
    lengths = [[len(clause) for clause in clauses]]
    return {
        "schema": RESULT_SCHEMA,
        "status": "SEGMENTED",
        "policy_sha256": POLICY_SHA256,
        "raw_text_sha256": hashlib.sha256(raw).hexdigest(),
        "derived_input_sha256": derived_input_sha256(inputs, lengths),
        "clause_spans": [[clause[0].start, clause[-1].end] for clause in clauses],
        "token_spans": [
            [[token.start, token.end] for token in clause] for clause in clauses
        ],
        "inputs": inputs,
        "lengths": lengths,
    }
