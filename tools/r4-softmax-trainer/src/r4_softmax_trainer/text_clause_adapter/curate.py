"""Independent, model-free input curation for the frozen #1085/#1094 contract.

This module has no production adapter, model, Torch, parser, renderer or decoder
imports.  It reads only pinned historical construction tensors and the lexical
artifact.  ``freeze`` records selection and byte rules before ``generate`` can
write text.  Withheld payloads are sealed on creation; the curator releases them
only after the implementation and independent source review are frozen.
"""

from __future__ import annotations

import argparse
import base64
import hashlib
import json
import math
import os
from pathlib import Path
import resource
import struct
import time

import blake3


SOURCE_CID = "blake3:d767fafdf544f01db99d9acb317c76df55e9f9d28f99785d2a6ae62b663731a2"
CLAUSE_SOURCE_CID = "blake3:ae68b7dd3e6b88304634647c24066f284a98877b7d554360076fae22c4712c1f"
VOCABULARY_CID = "blake3:571d5fbc282b17c8726eebd7b23c3ae55212a3de81b35d27722a0fa5979b8c5b"
POLICY_SHA256 = "91cce30a0b78c48130595369d3ea2a47c4de89cab5db1d4219d1874198cf52d0"
SPECIFICATION_SHA256 = "85f928fec94fa0f6793cff4c35e1fc8c9cba691739d34db272465766c7c9dab1"
SPECIFICATION_COMMIT = "3e894820c520f3b7803a48c6a2eeeb5b7d7021c5"
SOURCE_RELATIVE = ".uor-models/research/issue-1073-zoology-compound-binding/data/construction.safetensors"
CLAUSE_SOURCE_RELATIVE = ".uor-models/research/issue-1077-language-interface/data/construction.safetensors"
VOCABULARY_RELATIVE = ".uor-models/research/issue-1077-language-interface/data/vocabulary.json"
POLICY_RELATIVE = "tools/r4-softmax-trainer/src/r4_softmax_trainer/text_clause_adapter/policy.json"
REQUEST_SCHEMA = "uor-r4.text-to-clauses/1"
FAMILY_NAMES = ("same_owner", "same_object")
VARIANTS = ("base_q0", "base_q1", "swapped_q0", "swapped_q1", "absent_q0")
FACT_WORDS = (
    ("O", ",", "not", "D", ",", "put", "the", "X", "in", "the", "L", "."),
    ("in", "the", "L", ",", "not", "D", "but", "O", "put", "the", "X", "."),
    ("not", "D", "but", "O", "put", "the", "X", "in", "the", "L", "."),
    ("in", "the", "L", ",", "O", ",", "not", "D", ",", "put", "the", "X", "."),
)
FACT_ROLES = ((0, 7, 10), (7, 10, 2), (3, 6, 9), (4, 11, 2))
QUERY_WORDS = ("where", "is", "the", "X", "owned", "by", "O", ",", "not", "D", "?", "answer", ":")
REFUSAL_FAMILIES = (
    "invalid_schema_extra_field", "oversized_buffer", "invalid_utf8", "non_ascii",
    "bare_cr", "unknown_word", "literal_padding", "missing_period",
    "extra_period_empty_clause", "fewer_facts", "extra_fact", "overlong_clause",
    "missing_query_suffix", "appended_answer", "unsupported_mixed_fact_form",
    "unsupported_query_equal_owner_distractor",
)
REFUSAL_TAGS = (
    "UNSUPPORTED_SCHEMA", "INPUT_LIMIT", "INVALID_ENCODING", "INVALID_ENCODING",
    "INVALID_ENCODING", "UNKNOWN_LEXEME", "INVALID_ENCODING", "UNSUPPORTED_BOUNDARY",
    "UNSUPPORTED_BOUNDARY", "UNSUPPORTED_BOUNDARY", "UNSUPPORTED_BOUNDARY", "INPUT_LIMIT",
    "UNSUPPORTED_BOUNDARY", "UNSUPPORTED_BOUNDARY", "UNSUPPORTED_SYNTAX", "UNSUPPORTED_SYNTAX",
)
DESCRIPTION_SERIALIZATION = {
    "encoding": "UTF-8; json.dumps(value,sort_keys=True,separators=(',',':'),ensure_ascii=True) plus one LF",
    "identity": "SHA256 of the exact serialized group description",
    "group_fields": ["family", "rows"],
    "row_order": "five original rows in variant order 0,1,2,3,4; no ordering change",
    "row_fields": ["variant", "input_ids_1073", "target_id", "fact_distractor_ids", "query_distractor_id"],
    "source_group_index": "excluded from description; retained separately as provenance",
    "source_inputs": "all original 41 integer IDs, including BOS and unchanged query, in original order",
    "distractors": "read only from pinned #1077 construction view0 fact slot3 and query slot9; independently checked against the frozen distractor policy",
    "family_selection": "sort group SHA256 hex strings separately for same_owner and same_object; first2 authoring, next8 withheld",
}
REFUSAL_RULES = {
    "base_selection": "family f, repetition r: partition group index (4*f+r) modulo partition group count; variant f modulo5; form f modulo4; profile floor(f/4) modulo4",
    "partition_group_order": "family0 selected hashes ascending, then family1 selected hashes ascending",
    "authoring_repetitions": 1,
    "withheld_repetitions": 4,
    "mutations_by_family": [
        "authoring: extra lengths=[]; withheld repetition0 wrong schema suffix,1 extra clauses=[],2 extra roles=[],3 extra prior_state=null",
        "append ASCII spaces to exactly4097 bytes",
        "append one0xff byte",
        "append ASCII space then UTF-8 e-acute",
        "replace first ASCII space with a bare CR",
        "prefix quux then space",
        "prefix literal <pad> then space",
        "remove first literal period byte",
        "insert a second period immediately after the first period",
        "remove first fact span and its clause join",
        "prefix an exact copy of the first fact and its clause join",
        "insert the known word the immediately before the query question mark, making query length14",
        "remove final colon byte",
        "append ASCII space then drawer after the colon",
        "replace only fact0 with next fact form modulo4, retaining semantic IDs and spacing profile",
        "authoring and withheld0 set query O equal D; withheld1 set fact0 O equal D; withheld2 replace query where with in; withheld3 replace query owned with put",
    ],
    "offset_assertions": "only unambiguous schema,raw-limit,encoding,unknown,padding,missing-period,missing-colon,and appended-answer offsets; exact tags required for every row",
}


def canonical_bytes(value: object) -> bytes:
    return (json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True) + "\n").encode("utf-8")


def sha256(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def read_verified(path: Path, cid: str) -> bytes:
    payload = path.read_bytes()
    actual = "blake3:" + blake3.blake3(payload).hexdigest()
    if actual != cid:
        raise ValueError(f"historical construction artifact identity differs: {path}")
    return payload


class IntegerTensors:
    """Small independent reader of the source's uncompressed I64 wire schema."""

    def __init__(self, payload: bytes) -> None:
        header_size = struct.unpack("<Q", payload[:8])[0]
        self.header = json.loads(payload[8:8 + header_size])
        self.payload = payload
        self.start = 8 + header_size
        for name, entry in self.header.items():
            if name == "__metadata__":
                continue
            first, last = entry["data_offsets"]
            if entry["dtype"] != "I64" or last - first != 8 * math.prod(entry["shape"]):
                raise ValueError("unexpected source tensor representation")
            if first < 0 or self.start + last > len(payload):
                raise ValueError("source tensor bounds differ")

    def row(self, name: str, index: int) -> list[int]:
        entry = self.header[name]
        if not 0 <= index < entry["shape"][0]:
            raise ValueError("source row index outside tensor")
        width = math.prod(entry["shape"][1:])
        offset = self.start + entry["data_offsets"][0] + index * width * 8
        return list(struct.unpack_from("<" + str(width) + "q", self.payload, offset))


def historical_inputs(asset_root: Path) -> tuple[list[dict], list[str], list[dict]]:
    source_bytes = read_verified(asset_root / SOURCE_RELATIVE, SOURCE_CID)
    clause_bytes = read_verified(asset_root / CLAUSE_SOURCE_RELATIVE, CLAUSE_SOURCE_CID)
    vocabulary_bytes = read_verified(asset_root / VOCABULARY_RELATIVE, VOCABULARY_CID)
    source, clause = IntegerTensors(source_bytes), IntegerTensors(clause_bytes)
    vocabulary = json.loads(vocabulary_bytes)["vocabulary"]
    if len(vocabulary) != 4096 or vocabulary[52:58] != ["not", "but", ",", "owned", "by", "<pad>"]:
        raise ValueError("reader codec differs")
    if source.header["inputs"]["shape"] != [10240, 41] or clause.header["inputs"]["shape"] != [20480, 5, 13]:
        raise ValueError("construction population shape differs")
    groups = []
    for group_index in range(2048):
        rows = []
        pair_type = group_index % 2
        for variant in range(5):
            index = 5 * group_index + variant
            if source.row("group_ids", index) != [group_index] or source.row("variant_ids", index) != [variant] or source.row("pair_types", index) != [pair_type]:
                raise ValueError("original five-row grouping differs")
            ids = source.row("inputs", index)
            if ids[0] != 0 or [ids[33], ids[34], ids[36], *ids[38:]] != [5, 6, 7, 8, 9, 10]:
                raise ValueError("original query layout differs")
            historical = clause.row("inputs", index)
            facts, distractors = [], []
            for fact in range(4):
                old = ids[1 + fact * 8:9 + fact * 8]
                if [old[p] for p in (1, 2, 4, 5, 7)] != [1, 2, 3, 2, 4]:
                    raise ValueError("original fact layout differs")
                owner, obj, location = old[0], old[3], old[6]
                if not (12 <= owner < 28 and 28 <= obj < 44 and 44 <= location < 52):
                    raise ValueError("original semantic ID outside known lexica")
                negative = historical[13 * fact + 3]
                if negative != 12 + (owner - 12 + 4) % 16:
                    raise ValueError("inherited fact distractor differs")
                expected = [owner, 54, 52, negative, 54, 1, 2, obj, 3, 2, location, 4, 57]
                if historical[13 * fact:13 * (fact + 1)] != expected:
                    raise ValueError("historical view0 does not preserve source fact")
                facts.append([owner, obj, location])
                distractors.append(negative)
            query_owner, query_object, query_negative = ids[35], ids[37], historical[61]
            if historical[52:] != [5, 6, 2, query_object, 55, 56, query_owner, 54, 52, query_negative, 8, 9, 10]:
                raise ValueError("historical query does not preserve source query")
            if clause.row("lengths", index) != [12, 12, 12, 12, 13]:
                raise ValueError("historical clause lengths differ")
            target = source.row("targets", index)[0]
            matches = [loc for owner, obj, loc in facts if (owner, obj) == (query_owner, query_object)]
            if len(matches) > 1 or target != (matches[0] if matches else 11):
                raise ValueError("original source semantic target differs")
            rows.append({"variant": variant, "input_ids_1073": ids, "target_id": target,
                         "fact_distractor_ids": distractors, "query_distractor_id": query_negative})
        q0, q1 = rows[0]["input_ids_1073"], rows[1]["input_ids_1073"]
        if pair_type == 1:
            expected_negatives = [q1[35], q0[35], q1[35], q0[35], q1[35]]
        else:
            owners = {q0[1 + 8 * fact] for fact in range(4)}
            options = sorted(owner for owner in owners if owner != q0[35] and all(
                ((owner - 12 + obj - 28) % 4 == 0) == ((q0[35] - 12 + obj - 28) % 4 == 0)
                for obj in (q0[37], q1[37])))
            negative = options[0] if options else 12 + (q0[35] - 12 + 4) % 16
            expected_negatives = [negative] * 5
        if [row["query_distractor_id"] for row in rows] != expected_negatives:
            raise ValueError("inherited query distractors differ")
        description = {"family": FAMILY_NAMES[pair_type], "rows": rows}
        groups.append({"source_group_id": group_index, "pair_type": pair_type,
                       "group_id": sha256(canonical_bytes(description)), "description": description})
    sources = [
        {"path": SOURCE_RELATIVE, "cid": SOURCE_CID, "sha256": sha256(source_bytes), "bytes": len(source_bytes)},
        {"path": CLAUSE_SOURCE_RELATIVE, "cid": CLAUSE_SOURCE_CID, "sha256": sha256(clause_bytes), "bytes": len(clause_bytes)},
        {"path": VOCABULARY_RELATIVE, "cid": VOCABULARY_CID, "sha256": sha256(vocabulary_bytes), "bytes": len(vocabulary_bytes)},
    ]
    return groups, vocabulary, sources


def selected_groups(groups: list[dict]) -> dict[str, list[dict]]:
    selected = {"authoring": [], "withheld": []}
    for family in range(2):
        candidates = sorted((g for g in groups if g["pair_type"] == family), key=lambda g: g["group_id"])
        if len(candidates) != 1024 or len({g["group_id"] for g in candidates}) != 1024:
            raise ValueError("canonical group identities are not unique per family")
        selected["authoring"].extend(candidates[:2])
        selected["withheld"].extend(candidates[2:10])
    return selected


def immutable_write(path: Path, payload: bytes) -> dict:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("xb") as handle:
        handle.write(payload)
        handle.flush()
        os.fsync(handle.fileno())
    path.chmod(0o444)
    return {"path": str(path), "sha256": sha256(payload), "bytes": len(payload)}


def freeze(asset_root: Path, checkout: Path, output: Path) -> dict:
    started = time.monotonic()
    policy_bytes = (checkout / POLICY_RELATIVE).read_bytes()
    if sha256(policy_bytes) != POLICY_SHA256:
        raise ValueError("production policy is not the agreed pre-generation bytes")
    policy = json.loads(policy_bytes)
    if sha256((checkout / "docs/integration/clause-segmentation-1085.md").read_bytes()) != SPECIFICATION_SHA256:
        raise ValueError("specification bytes differ")
    groups, _, sources = historical_inputs(asset_root)
    selected = selected_groups(groups)
    receipt = {
        "schema": "uor-r4.clause-curation-selection/1", "issue": 1094,
        "status": "FROZEN_BEFORE_TEXT_GENERATION", "model_access": False,
        "model_forwards": 0, "new_fitted_parameters": 0,
        "specification": {"commit": SPECIFICATION_COMMIT, "sha256": SPECIFICATION_SHA256},
        "policy_sha256": POLICY_SHA256, "sources": sources,
        "curator_source_sha256": sha256(Path(__file__).read_bytes()),
        "description_serialization": DESCRIPTION_SERIALIZATION,
        "surface_profiles": policy["surface_profiles"], "generated_surfaces": policy["generated_surfaces"],
        "selection": {partition: [{k: group[k] for k in ("source_group_id", "pair_type", "group_id")}
                                  for group in values] for partition, values in selected.items()},
        "candidate_counts": {"same_owner": 1024, "same_object": 1024},
        "candidate_identity_sha256": sha256(canonical_bytes(sorted(g["group_id"] for g in groups))),
        "valid_row_order": "partition authoring then withheld; selected group order, then variant0..4, form0..3, profile0..3",
        "valid_counts": {"authoring": 320, "withheld": 1280},
        "valid_cell_counts": {"authoring": 20, "withheld": 80},
        "refusal_families": list(REFUSAL_FAMILIES), "refusal_tags": list(REFUSAL_TAGS),
        "refusal_rules": REFUSAL_RULES, "refusal_counts": {"authoring": 16, "withheld": 64},
        "boundary_control": {"count": 16, "source": "first withheld selected group, variant0, all16 form/profile cells",
                             "mutation": "remove the first period byte", "expected_status": "UNSUPPORTED_BOUNDARY"},
        "annotation": "independent token construction with spans accumulated from emitted bytes; independent fixed role indices; only source targets and distractors reused",
        "role_positions": {"fact_forms": [list(x) for x in FACT_ROLES], "query": [6, 3, -100]},
        "raw_row_schema": "row_id,partition,kind,group_id,variant,form,profile,request_schema,text_base64,request_extras; refusal rows add refusal_family/repetition; boundary rows add source_row_id",
        "reference_row_schema": "row_id,partition,kind,group_id,variant,form,profile,pair_type,expected_status; valid rows add raw_text_sha256,derived_input_sha256,clause_spans,token_spans,inputs[1,5,13],lengths[1,5],role_positions[5,3],target_id,supported; selected refusal rows add expected_byte_offset",
        "independence_scope": "text preparation and annotation only; original construction worlds were already observed during fitting",
    }
    output.mkdir(parents=True, exist_ok=True, mode=0o700)
    result = immutable_write(output / "selection.json", canonical_bytes(receipt))
    immutable_write(output / "policy.json", policy_bytes)
    result.update({"selection_counts": {k: len(v) for k, v in selected.items()}, "elapsed_seconds": time.monotonic() - started,
                   "model_forwards": 0, "text_generation": "NOT_RUN"})
    return result


def make_token_clauses(row: dict, form: int, vocabulary: list[str]) -> tuple[list[list[str]], list[list[int]]]:
    ids = row["input_ids_1073"]
    clauses = []
    for index in range(4):
        fact = ids[1 + 8 * index:9 + 8 * index]
        assignment = {"O": vocabulary[fact[0]], "X": vocabulary[fact[3]], "L": vocabulary[fact[6]],
                      "D": vocabulary[row["fact_distractor_ids"][index]]}
        clauses.append([assignment.get(item, item) for item in FACT_WORDS[form]])
    assignment = {"O": vocabulary[ids[35]], "X": vocabulary[ids[37]], "D": vocabulary[row["query_distractor_id"]]}
    clauses.append([assignment.get(item, item) for item in QUERY_WORDS])
    return clauses, [list(FACT_ROLES[form]) for _ in range(4)] + [[6, 3, -100]]


def emit_bytes(clauses: list[list[str]], profile: int) -> tuple[bytes, list, list]:
    result = bytearray()
    clause_spans, token_spans = [], []
    clause_join = (b" ", b"\n", b"\r\n", b" ")[profile]
    for clause_index, words in enumerate(clauses):
        if clause_index:
            result.extend(clause_join)
        clause_start, spans = len(result), []
        for index, word in enumerate(words):
            if index:
                if profile < 2:
                    gap = b"" if word in (".", ",", "?", ":") else b" "
                elif profile == 2:
                    gap = b" "
                else:
                    gap = (b" ", b"\t", b"\n")[(index - 1) % 3]
                result.extend(gap)
            first = len(result)
            result.extend(word.encode("ascii"))
            spans.append([first, len(result)])
        clause_spans.append([clause_start, len(result)])
        token_spans.append(spans)
    return bytes(result), clause_spans, token_spans


def input_identity(inputs: list[list[int]], lengths: list[int]) -> str:
    chunks = []
    for value in ("uor-r4.text-to-clauses-input/1", POLICY_SHA256, VOCABULARY_CID, "i64le"):
        encoded = value.encode("utf-8")
        chunks.extend([struct.pack("<I", len(encoded)), encoded])
    chunks.append(struct.pack("<5I", 1, 5, 13, 1, 5))
    flattened = [value for clause in inputs for value in clause] + lengths
    chunks.append(struct.pack("<70q", *flattened))
    return sha256(b"".join(chunks))


def row_identity(metadata: dict) -> str:
    return sha256(b"uor-r4.clause-comparison-row/1\x00" + canonical_bytes(metadata))


def valid_row(partition: str, group: dict, variant: int, form: int, profile: int,
              vocabulary: list[str], word_ids: dict[str, int]) -> tuple[dict, dict, list[list[str]]]:
    row = group["description"]["rows"][variant]
    words, roles = make_token_clauses(row, form, vocabulary)
    payload, clause_spans, token_spans = emit_bytes(words, profile)
    lengths = [len(clause) for clause in words]
    inputs = [[word_ids[word] for word in clause] + [57] * (13 - len(clause)) for clause in words]
    metadata = {"partition": partition, "kind": "valid", "group_id": group["group_id"], "variant": variant,
                "form": form, "profile": profile}
    identifier = row_identity(metadata)
    raw = {"row_id": identifier, **metadata, "request_schema": REQUEST_SCHEMA,
           "text_base64": base64.b64encode(payload).decode("ascii"), "request_extras": {}}
    reference = {"row_id": identifier, **metadata, "pair_type": group["pair_type"], "expected_status": "SEGMENTED",
                 "raw_text_sha256": sha256(payload), "derived_input_sha256": input_identity(inputs, lengths),
                 "clause_spans": clause_spans, "token_spans": token_spans, "inputs": [inputs], "lengths": [lengths],
                 "role_positions": roles, "target_id": row["target_id"], "supported": row["target_id"] != 11}
    return raw, reference, words


def refusal_row(partition: str, groups: list[dict], family: int, repetition: int,
                vocabulary: list[str], word_ids: dict[str, int]) -> tuple[dict, dict]:
    group = groups[(4 * family + repetition) % len(groups)]
    variant, form, profile = family % 5, family % 4, (family // 4) % 4
    raw, reference, words = valid_row(partition, group, variant, form, profile, vocabulary, word_ids)
    payload = base64.b64decode(raw["text_base64"])
    spans = reference["token_spans"]
    offset = None
    assert_offset = False
    if family == 0:
        if partition == "authoring":
            raw["request_extras"] = {"lengths": []}
        elif repetition == 0:
            raw["request_schema"] += "/unsupported"
        else:
            key, value = (("clauses", []), ("roles", []), ("prior_state", None))[repetition - 1]
            raw["request_extras"] = {key: value}
        assert_offset = True
    elif family == 1:
        payload += b" " * (4097 - len(payload))
        offset, assert_offset = 4096, True
    elif family == 2:
        offset, assert_offset = len(payload), True
        payload += b"\xff"
    elif family == 3:
        offset, assert_offset = len(payload) + 1, True
        payload += b" \xc3\xa9"
    elif family == 4:
        offset, assert_offset = payload.index(b" "), True
        payload = payload[:offset] + b"\r" + payload[offset + 1:]
    elif family == 5:
        payload = b"quux " + payload
        offset, assert_offset = 0, True
    elif family == 6:
        payload = b"<pad> " + payload
        offset, assert_offset = 0, True
    elif family == 7:
        period = payload.index(b".")
        payload = payload[:period] + payload[period + 1:]
        offset, assert_offset = len(payload), True
    elif family == 8:
        period = payload.index(b".")
        payload = payload[:period + 1] + b"." + payload[period + 1:]
    elif family == 9:
        payload = payload[reference["clause_spans"][1][0]:]
    elif family == 10:
        payload = payload[:reference["clause_spans"][1][0]] + payload
    elif family == 11:
        question = spans[4][10][0]
        payload = payload[:question] + b" the " + payload[question:]
    elif family == 12:
        payload = payload[:-1]
        offset, assert_offset = len(payload), True
    elif family == 13:
        offset, assert_offset = len(payload) + 1, True
        payload += b" drawer"
    elif family == 14:
        alternative, _ = make_token_clauses(group["description"]["rows"][variant], (form + 1) % 4, vocabulary)
        words[0] = alternative[0]
        payload, _, _ = emit_bytes(words, profile)
    elif family == 15:
        if partition == "authoring" or repetition == 0:
            words[4][6] = words[4][9]
        elif repetition == 1:
            owner_slot = FACT_ROLES[form][0]
            distractor_slot = (3, 5, 1, 7)[form]
            words[0][owner_slot] = words[0][distractor_slot]
        elif repetition == 2:
            words[4][0] = "in"
        else:
            words[4][4] = "put"
        payload, _, _ = emit_bytes(words, profile)
    metadata = {key: raw[key] for key in ("partition", "group_id", "variant", "form", "profile")}
    metadata.update({"kind": "refusal", "refusal_family": REFUSAL_FAMILIES[family], "repetition": repetition})
    identifier = row_identity(metadata)
    raw.update(metadata)
    raw.update({"row_id": identifier, "text_base64": base64.b64encode(payload).decode("ascii")})
    reference = {"row_id": identifier, **metadata, "pair_type": group["pair_type"],
                 "expected_status": REFUSAL_TAGS[family], "raw_text_sha256": sha256(payload)}
    if assert_offset:
        reference["expected_byte_offset"] = offset
    return raw, reference


def generate(asset_root: Path, output: Path, selection_sha256: str) -> dict:
    started = time.monotonic()
    selection_bytes = (output / "selection.json").read_bytes()
    selection = json.loads(selection_bytes)
    if sha256(selection_bytes) != selection_sha256 or selection["curator_source_sha256"] != sha256(Path(__file__).read_bytes()):
        raise ValueError("curator source or frozen selection identity differs")
    if sha256((output / "policy.json").read_bytes()) != POLICY_SHA256:
        raise ValueError("frozen policy bytes differ")
    groups, vocabulary, sources = historical_inputs(asset_root)
    if sources != selection["sources"]:
        raise ValueError("source identities differ from selection freeze")
    selected = selected_groups(groups)
    selection_projection = {partition: [{key: group[key] for key in ("source_group_id", "pair_type", "group_id")}
                                       for group in values] for partition, values in selected.items()}
    if selection_projection != selection["selection"]:
        raise ValueError("selected original groups differ")
    word_ids = {word: index for index, word in enumerate(vocabulary)}
    records, row_counts = [], {}
    all_row_ids = set()
    for partition in ("authoring", "withheld"):
        raw_rows, references = [], []
        for group in selected[partition]:
            for variant in range(5):
                for form in range(4):
                    for profile in range(4):
                        raw, reference, _ = valid_row(partition, group, variant, form, profile, vocabulary, word_ids)
                        raw_rows.append(raw)
                        references.append(reference)
        valid_count = len(raw_rows)
        for family in range(16):
            for repetition in range(1 if partition == "authoring" else 4):
                raw, reference = refusal_row(partition, selected[partition], family, repetition, vocabulary, word_ids)
                raw_rows.append(raw)
                references.append(reference)
        if partition == "withheld":
            group = selected[partition][0]
            for form in range(4):
                for profile in range(4):
                    raw, reference, _ = valid_row(partition, group, 0, form, profile, vocabulary, word_ids)
                    payload = base64.b64decode(raw["text_base64"])
                    first = payload.index(b".")
                    payload = payload[:first] + payload[first + 1:]
                    metadata = {key: raw[key] for key in ("partition", "group_id", "variant", "form", "profile")}
                    metadata.update({"kind": "boundary_control", "source_row_id": raw["row_id"]})
                    identifier = row_identity(metadata)
                    raw = {"row_id": identifier, **metadata, "request_schema": REQUEST_SCHEMA,
                           "text_base64": base64.b64encode(payload).decode("ascii"), "request_extras": {}}
                    reference = {"row_id": identifier, **metadata, "pair_type": group["pair_type"],
                                 "raw_text_sha256": sha256(payload), "expected_status": "UNSUPPORTED_BOUNDARY",
                                 "expected_byte_offset": len(payload)}
                    raw_rows.append(raw)
                    references.append(reference)
        if valid_count != selection["valid_counts"][partition] or len(raw_rows) != (336 if partition == "authoring" else 1360):
            raise ValueError("population count differs")
        for raw, reference in zip(raw_rows, references, strict=True):
            if raw["row_id"] != reference["row_id"] or raw["row_id"] in all_row_ids:
                raise ValueError("row identity overlap or annotation misalignment")
            all_row_ids.add(raw["row_id"])
            if reference["expected_status"] == "SEGMENTED":
                payload = base64.b64decode(raw["text_base64"])
                for index, spans in enumerate(reference["token_spans"]):
                    for token, (first, last) in zip(reference["inputs"][0][index], spans, strict=False):
                        if payload[first:last] != vocabulary[token].encode("ascii"):
                            raise ValueError("curator byte-span annotation differs from emitted bytes")
        for name, values in (("raw.jsonl", raw_rows), ("reference.jsonl", references)):
            destination = output / partition / name
            record = immutable_write(destination, b"".join(canonical_bytes(value) for value in values))
            record["path"] = str(destination.relative_to(output))
            records.append(record)
        row_counts[partition] = {"valid": valid_count, "refusal": 16 if partition == "authoring" else 64,
                                 "boundary_control": 0 if partition == "authoring" else 16, "total": len(raw_rows)}
    (output / "withheld").chmod(0o000)
    total_bytes = sum(record["bytes"] for record in records) + len(selection_bytes) + (output / "policy.json").stat().st_size
    if total_bytes >= 128 * 1024 * 1024:
        raise ValueError("curator population exceeds frozen total corpus/result cap")
    receipt = {"schema": "uor-r4.clause-curation-population/1", "issue": 1094,
               "status": "INDEPENDENT_INPUTS_FROZEN_WITHHELD_SEALED", "selection_sha256": selection_sha256,
               "policy_sha256": POLICY_SHA256, "curator_source_sha256": sha256(Path(__file__).read_bytes()),
               "files": records, "counts": row_counts, "total_bytes": total_bytes,
               "model_access": False, "model_forwards": 0, "optimizer_updates": 0,
               "adapter_source_inspected_or_imported": False,
               "historical_render_parse_decode_helpers_reused": False,
               "withheld_access": "directory mode000 after writes; no root or model access before source/review freeze",
               "authoring_preparation_elapsed_seconds": time.monotonic() - started,
               "authoring_preparation_peak_rss_native_units": resource.getrusage(resource.RUSAGE_SELF).ru_maxrss,
               "resource_scope": "independent input authoring; separate from the timed120s preparation integrity/model comparison stages"}
    result = immutable_write(output / "population.json", canonical_bytes(receipt))
    result.update({"counts": row_counts, "total_bytes": total_bytes, "withheld_status": "SEALED", "model_forwards": 0})
    return result


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("freeze", "generate"))
    parser.add_argument("--asset-root", type=Path, required=True)
    parser.add_argument("--checkout", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--selection-sha256")
    args = parser.parse_args()
    if args.action == "freeze":
        if args.checkout is None:
            parser.error("freeze requires --checkout")
        result = freeze(args.asset_root, args.checkout, args.output)
    else:
        if args.selection_sha256 is None:
            parser.error("generate requires --selection-sha256")
        result = generate(args.asset_root, args.output, args.selection_sha256)
    print(json.dumps(result, sort_keys=True))


if __name__ == "__main__":
    main()
