"""One frozen comparison with independent inputs and isolated sequential arms.

One temporary oracle stream is consumed a batch at a time by the adapter arm.
Complete row tensor bytes decide equality; retained evidence contains ordered,
domain-tagged digests, bounded decisions and actual work/resource receipts.
"""

from __future__ import annotations

import argparse
import base64
import collections
import hashlib
import json
import math
import os
import resource
import signal
import struct
import subprocess
import sys
import time
from pathlib import Path

from . import contract
from .adapter import POLICY_SHA256, READER_PREFIX, RESULT_SCHEMA, segment_request

FIELDS = ("inputs", "lengths", "clause_spans", "token_spans",
          "raw_text_sha256", "derived_input_sha256")
SUCCESS_FIELDS = {"schema", "status", "policy_sha256", *FIELDS}
REFUSAL_FIELDS = {"schema", "status", "byte_offset"}
REFUSAL_TAGS = {"UNSUPPORTED_SCHEMA", "INPUT_LIMIT", "INVALID_ENCODING",
                "UNKNOWN_LEXEME", "UNSUPPORTED_BOUNDARY", "UNSUPPORTED_SYNTAX",
                "UNAVAILABLE_ARTIFACT"}
TENSOR_LAYOUT = {
    "inputs": ("int64", (5, 13)), "lengths": ("int64", (5,)),
    "role_attention": ("float32", (5, 3, 13)),
    "role_vectors": ("float32", (5, 3, 64)),
    "binding_attention": ("float32", (5,)), "logits": ("float32", (4096,)),
    "predictions": ("int64", ()), "role_positions": ("int64", (5, 3)),
}
TENSORS = tuple(TENSOR_LAYOUT)
METADATA = ("row_id", "partition", "kind", "group_id", "variant", "form", "profile")
REFUSAL_FAMILIES = (
    "invalid_schema_extra_field", "oversized_buffer", "invalid_utf8", "non_ascii",
    "bare_cr", "unknown_word", "literal_padding", "missing_period",
    "extra_period_empty_clause", "fewer_facts", "extra_fact", "overlong_clause",
    "missing_query_suffix", "appended_answer", "unsupported_mixed_fact_form",
    "unsupported_query_equal_owner_distractor",
)
EXPECTED_COUNTS = {
    "authoring": {"valid": 320, "refusal": 16, "boundary_control": 0, "total": 336},
    "withheld": {"valid": 1280, "refusal": 64, "boundary_control": 16, "total": 1360},
}


class CampaignFailure(Exception):
    def __init__(self, status: str, reason: str, evidence: dict | None = None):
        super().__init__(reason)
        self.status, self.evidence = status, evidence


def read_jsonl(path: Path) -> list[dict]:
    return [json.loads(line) for line in path.read_bytes().splitlines() if line]


def actual_request(row: dict) -> dict:
    request = {"schema": row["request_schema"],
               "text": base64.b64decode(row["text_base64"], validate=True)}
    request.update(row.get("request_extras", {}))
    return request


def transport(row: dict) -> dict:
    return {"schema": row["request_schema"], "text_base64": row["text_base64"],
            "request_extras": row.get("request_extras", {})}


def joint_rows(raw: list[dict], refs: list[dict]) -> list[tuple[dict, dict]]:
    if (len(raw) != len(refs) or len({r["row_id"] for r in raw}) != len(raw)
            or len({r["row_id"] for r in refs}) != len(refs)):
        raise ValueError("duplicate row IDs or incomplete annotations")
    for row, ref in zip(raw, refs, strict=True):
        for key in METADATA:
            if row[key] != ref[key]:
                raise ValueError(f"raw/reference metadata differs: {key}")
        for key in ("refusal_family", "repetition", "source_row_id"):
            if row.get(key) != ref.get(key):
                raise ValueError(f"raw/reference control metadata differs: {key}")
    return list(zip(raw, refs, strict=True))


def valid_reference(ref: dict) -> bool:
    return ref["expected_status"] == "SEGMENTED"


def cell(row: dict) -> str:
    return f"{row['partition']}/form-{row['form']}/profile-{row['profile']}"


def _reference_input_hash(ref: dict) -> str:
    """Independent implementation of the normative ordered tensor framing."""
    digest = hashlib.sha256()
    for value in ("uor-r4.text-to-clauses-input/1", POLICY_SHA256,
                  "blake3:571d5fbc282b17c8726eebd7b23c3ae55212a3de81b35d27722a0fa5979b8c5b",
                  "i64le"):
        encoded = value.encode()
        digest.update(struct.pack("<I", len(encoded)))
        digest.update(encoded)
    digest.update(struct.pack("<5I", 1, 5, 13, 1, 5))
    for clause in ref["inputs"][0]:
        digest.update(struct.pack("<13q", *clause))
    digest.update(struct.pack("<5q", *ref["lengths"][0]))
    return digest.hexdigest()


def _valid_reference_integrity(row: dict, ref: dict) -> None:
    raw = base64.b64decode(row["text_base64"], validate=True)
    if (contract.digest(raw) != ref["raw_text_sha256"]
            or row["request_schema"] != "uor-r4.text-to-clauses/1"
            or row.get("request_extras", {})
            or len(ref["inputs"]) != 1 or len(ref["inputs"][0]) != 5
            or len(ref["lengths"]) != 1 or len(ref["lengths"][0]) != 5
            or len(ref["token_spans"]) != 5 or len(ref["clause_spans"]) != 5
            or len(ref["role_positions"]) != 5):
        raise ValueError("valid reference shape, raw identity or request differs")
    cursor, used_roles = 0, 0
    for index in range(5):
        ids, length = ref["inputs"][0][index], ref["lengths"][0][index]
        spans, roles = ref["token_spans"][index], ref["role_positions"][index]
        if (type(length) is not int or not 1 <= length <= 13 or len(ids) != 13
                or any(type(token) is not int or not 0 <= token < 4096 for token in ids)
                or ids[length:] != [57] * (13-length) or len(spans) != length
                or len(roles) != 3):
            raise ValueError("valid reference length, padding or roles differs")
        if ref["clause_spans"][index] != [spans[0][0], spans[-1][1]]:
            raise ValueError("clause span does not exactly enclose its token spans")
        for token_id, span in zip(ids[:length], spans, strict=True):
            if (len(span) != 2 or any(type(v) is not int for v in span)
                    or not cursor <= span[0] < span[1] <= len(raw)
                    or token_id >= len(READER_PREFIX)
                    or raw[span[0]:span[1]] != READER_PREFIX[token_id].encode()
                    or raw[cursor:span[0]].replace(b"\r\n", b"").strip(b" \t\n")):
                raise ValueError("token annotation does not preserve raw bytes/order")
            cursor = span[1]
        for role, position in enumerate(roles):
            if index == 4 and role == 2:
                if position != -100:
                    raise ValueError("unused query-location diagnostic differs")
            elif type(position) is not int or not 0 <= position < length:
                raise ValueError("diagnostic role position is outside its clause")
            else:
                used_roles += 1
    if (raw[cursor:].replace(b"\r\n", b"").strip(b" \t\n") or used_roles != 14
            or _reference_input_hash(ref) != ref["derived_input_sha256"]
            or type(ref["target_id"]) is not int or not 0 <= ref["target_id"] < 4096
            or type(ref["supported"]) is not bool
            or ref["supported"] != (ref["target_id"] != 11)):
        raise ValueError("reference framing, target or role count differs")


def _selection_integrity(selection: dict) -> dict[str, dict]:
    groups = selection["selection"]
    result, all_sources = {}, set()
    for partition, per_family in (("authoring", 2), ("withheld", 8)):
        selected = groups[partition]
        if (len(selected) != per_family * 2
                or collections.Counter(g["pair_type"] for g in selected) != {0: per_family, 1: per_family}):
            raise ValueError("selection group/family counts differ")
        for group in selected:
            identity = group["group_id"]
            if (set(group) != {"source_group_id", "pair_type", "group_id"}
                    or type(group["source_group_id"]) is not int
                    or not isinstance(identity, str) or len(identity) != 64
                    or any(c not in "0123456789abcdef" for c in identity)
                    or identity in result or group["source_group_id"] in all_sources):
                raise ValueError("selection identities are malformed or partitions overlap")
            result[identity] = {**group, "partition": partition}
            all_sources.add(group["source_group_id"])
        for family in (0, 1):
            identities = [g["group_id"] for g in selected if g["pair_type"] == family]
            if identities != sorted(identities):
                raise ValueError("selection family identity order differs")
    for family in (0, 1):
        left = [g["group_id"] for g in groups["authoring"] if g["pair_type"] == family]
        right = [g["group_id"] for g in groups["withheld"] if g["pair_type"] == family]
        if max(left) >= min(right):
            raise ValueError("authoring/withheld do not follow frozen family order")
    return result


def population_integrity(pairs, selection: dict, partition: str) -> dict:
    selected = _selection_integrity(selection)
    rows_by_id = {row["row_id"]: (row, ref) for row, ref in pairs}
    counts, cells, families = collections.Counter(), collections.Counter(), collections.Counter()
    groups, surfaces, boundary_cells = collections.defaultdict(list), collections.defaultdict(set), set()
    for row, ref in pairs:
        if row["partition"] != partition or row["kind"] not in ("valid", "refusal", "boundary_control"):
            raise ValueError("row partition/kind differs")
        group = selected.get(row["group_id"])
        if group is None or group["partition"] != partition or ref["pair_type"] != group["pair_type"]:
            raise ValueError("row group/family differs from independent selection")
        counts[row["kind"]] += 1
        raw = base64.b64decode(row["text_base64"], validate=True)
        if contract.digest(raw) != ref["raw_text_sha256"]:
            raise ValueError("reference raw-text identity differs")
        if row["kind"] == "valid":
            if (not valid_reference(ref) or any(type(row[k]) is not int or row[k] not in range(4)
                                               for k in ("form", "profile"))
                    or type(row["variant"]) is not int or row["variant"] not in range(5)):
                raise ValueError("valid row cell or variant differs")
            _valid_reference_integrity(row, ref)
            cells[(row["form"], row["profile"])] += 1
            groups[(row["group_id"], row["form"], row["profile"])].append(row["variant"])
            surfaces[(row["group_id"], row["variant"], row["form"])].add(ref["derived_input_sha256"])
        else:
            if ref["expected_status"] not in REFUSAL_TAGS:
                raise ValueError("refusal row lacks a fixed refusal tag")
            if "expected_byte_offset" in ref:
                offset = ref["expected_byte_offset"]
                if offset is not None and (type(offset) is not int or not 0 <= offset <= max(4096, len(raw))):
                    raise ValueError("expected refusal offset is invalid")
            if row["kind"] == "refusal":
                families[row["refusal_family"]] += 1
            else:
                source = rows_by_id.get(row["source_row_id"])
                if source is None or source[0]["kind"] != "valid":
                    raise ValueError("boundary control does not bind a valid parent")
                original = base64.b64decode(source[0]["text_base64"], validate=True)
                first = original.find(b".")
                if (first < 0 or raw != original[:first] + original[first+1:]
                        or ref["expected_status"] != "UNSUPPORTED_BOUNDARY"
                        or any(row[k] != source[0][k] for k in ("group_id", "variant", "form", "profile"))
                        or row["variant"] != 0):
                    raise ValueError("boundary control is not first-period removal")
                boundary_cells.add((row["form"], row["profile"]))
    expected = EXPECTED_COUNTS[partition]
    per_cell = 20 if partition == "authoring" else 80
    per_family = 1 if partition == "authoring" else 4
    if (len(pairs) != expected["total"]
            or any(counts[k] != expected[k] for k in ("valid", "refusal", "boundary_control"))
            or cells != {(f, p): per_cell for f in range(4) for p in range(4)}
            or families != {family: per_family for family in REFUSAL_FAMILIES}
            or any(sorted(variants) != [0, 1, 2, 3, 4] for variants in groups.values())
            or any(len(identities) != 1 for identities in surfaces.values())
            or len(groups) != (4 if partition == "authoring" else 16)*16
            or (partition == "withheld" and boundary_cells != {(f, p) for f in range(4) for p in range(4)})):
        raise ValueError("population counts, cells, families or complete group structure differs")
    return {"counts": dict(counts), "groups": len(groups), "cells": 16,
            "refusal_families": dict(families), "independent_selection": True}


def adapter_fidelity(out: dict, ref: dict) -> bool:
    return (set(out) == SUCCESS_FIELDS and out.get("schema") == RESULT_SCHEMA
            and out.get("status") == "SEGMENTED" and out.get("policy_sha256") == POLICY_SHA256
            and all(out.get(key) == ref[key] for key in FIELDS))


def refusal_fidelity(out: dict, ref: dict) -> bool:
    return (set(out) == REFUSAL_FIELDS and out.get("schema") == RESULT_SCHEMA
            and out.get("status") == ref["expected_status"]
            and (out.get("byte_offset") is None or (type(out.get("byte_offset")) is int
                                                     and 0 <= out["byte_offset"] < 2**32))
            and (out.get("status") not in ("UNSUPPORTED_SCHEMA", "UNAVAILABLE_ARTIFACT")
                 or out.get("byte_offset") is None)
            and ("expected_byte_offset" not in ref or out.get("byte_offset") == ref["expected_byte_offset"]))


class Budget:
    def __init__(self, args, phase: str, carried: float = 0.0, carried_forwards: int = 0):
        self.started, self.phase = time.monotonic(), phase
        self.output, self.corpus, self.carried = args.output, args.corpus, carried
        self.carried_forwards, self.row_forwards = carried_forwards, 0
        self.worker_peak = 0
        self.progress = []
        args._active_budget = self

    @property
    def elapsed(self) -> float:
        return time.monotonic() - self.started

    def snapshot(self) -> dict:
        coordinator = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
        if sys.platform != "darwin":
            coordinator *= 1024
        output_bytes = sum(p.stat().st_size for p in self.output.rglob("*") if p.is_file())
        population = self.corpus / "population.json"
        # The frozen manifest accounts for sealed bytes without opening or
        # traversing withheld files during authoring preparation.
        corpus_bytes = json.loads(population.read_bytes())["total_bytes"] + population.stat().st_size
        probe = self.corpus / "isolation-probe.txt"
        if probe.exists():
            corpus_bytes += probe.stat().st_size
        return {"phase_elapsed_seconds": self.elapsed,
                "cumulative_elapsed_seconds": self.carried + self.elapsed,
                "coordinator_peak_rss_bytes": int(coordinator),
                "worker_peak_rss_bytes": self.worker_peak,
                "combined_peak_rss_bound_bytes": int(coordinator) + self.worker_peak,
                "new_bytes": output_bytes + corpus_bytes,
                "logical_row_forwards": self.row_forwards,
                "cumulative_logical_row_forwards": self.carried_forwards + self.row_forwards}

    def check(self) -> None:
        observed = self.snapshot()
        if (observed["phase_elapsed_seconds"] > contract.LIMITS["phase_seconds"]
                or observed["cumulative_elapsed_seconds"] > contract.LIMITS["cumulative_seconds"]
                or observed["combined_peak_rss_bound_bytes"] > contract.LIMITS["peak_rss_bytes"]
                or observed["new_bytes"] > contract.LIMITS["new_bytes"]
                or observed["cumulative_logical_row_forwards"] > contract.LIMITS["logical_row_forwards"]):
            raise CampaignFailure("INCOMPLETE_RESOURCE", "frozen resource cap exceeded", observed)

    def record_event(self, arm: str, event: dict) -> None:
        self.worker_peak = max(self.worker_peak, event.get("peak_rss_bytes", 0))
        retained = {k: v for k, v in event.items() if k not in ("tensors", "receipts", "model_tokens")}
        retained.update(arm=arm, phase=self.phase)
        self.progress.append(retained)
        with (self.output / f"{self.phase}-progress.jsonl").open("ab") as stream:
            stream.write(contract.canonical(retained))


def alarm(_number, _frame):
    raise CampaignFailure("INCOMPLETE_RESOURCE", "frozen phase deadline expired")


def _worker_identity(event: dict, bindings: dict, binding_sha: str, *, ready: bool, readiness: bool) -> None:
    expected_states = {"reader": bindings["reader_state_cid"], "core": bindings["core_state_cid"]}
    if (event.get("bindings_sha256") != binding_sha or event.get("runtime") != contract.RUNTIME
            or event.get("deterministic_algorithms") is not True
            or event.get("model_loads") != (0 if readiness else 2)):
        raise CampaignFailure("UNAVAILABLE_REFERENCE_REPLAY", "worker identity/runtime/load receipt differs", event)
    if ready:
        if (event.get("event") != "ready" or event.get("isolation_denied") is not True
                or event.get("states") != (None if readiness else expected_states)
                or event.get("row_forwards") != 0 or event.get("batch_forwards") != 0):
            raise CampaignFailure("UNAVAILABLE_REFERENCE_REPLAY", "worker startup was not isolated and fixed", event)
    elif (event.get("event") != "done"
          or event.get("states_before") != (None if readiness else expected_states)
          or event.get("states_after") != (None if readiness else expected_states)
          or event.get("audit", {}).get("isolation_denied") is not True
          or event.get("audit", {}).get("optimizer_updates") != 0):
        raise CampaignFailure("UNAVAILABLE_REFERENCE_REPLAY", "worker final state/audit differs", event)


def arm_process(args, arm: str, packets, budget: Budget, callback=None, *, readiness=False):
    """Consume one packet/receipt at a time; never retain full-arm tensors."""
    bindings = json.loads((args.output / "bindings.json").read_bytes())
    binding_sha = contract.record(args.output / "bindings.json")["sha256"]
    env = dict(os.environ)
    env.update(PYTHONPATH=str(args.repo / "tools/r4-softmax-trainer/src"),
               PYTHONDONTWRITEBYTECODE="1", PYTHONUNBUFFERED="1",
               OMP_NUM_THREADS="4", VECLIB_MAXIMUM_THREADS="4",
               UOR_ISOLATION_PROBE=str(args.corpus / "isolation-probe.txt"))
    command = ["/usr/bin/sandbox-exec", "-f", str(args.output / "worker.sb"),
               str(args.python), "-m", "r4_softmax_trainer.text_clause_adapter.worker",
               "--bindings", str(args.output / "bindings.json"), "--arm", arm]
    if readiness:
        command.append("--readiness-only")
    budget.record_event(arm, {"event": "worker-started", "bindings_sha256": binding_sha,
                              "readiness_only": readiness, "model_forwards": 0})
    process = subprocess.Popen(command, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                               stderr=subprocess.PIPE, env=env, cwd="/", text=True,
                               bufsize=1, start_new_session=True)
    processed_rows, processed_batches = 0, 0
    try:
        def receive():
            budget.check()
            line = process.stdout.readline()
            if not line:
                error = process.stderr.read(8192)
                raise CampaignFailure("UNAVAILABLE_REFERENCE_REPLAY", f"worker ended without receipt: {error}")
            event = json.loads(line)
            budget.record_event(arm, event)
            if event.get("event") == "error":
                additional = max(0, event.get("row_forwards", 0) - processed_rows)
                budget.row_forwards += additional
                raise CampaignFailure(event.get("status", "UNAVAILABLE_REFERENCE_REPLAY"),
                                      event.get("reason", "worker failed"), event)
            return event
        initial = receive()
        _worker_identity(initial, bindings, binding_sha, ready=True, readiness=readiness)
        for batch_no, packet in enumerate(packets):
            if readiness:
                raise ValueError("readiness cannot receive model packets")
            process.stdin.write(json.dumps(packet, separators=(",", ":")) + "\n")
            process.stdin.flush()
            event = receive()
            indices = event.get("valid_indices")
            if (event.get("event") != "batch" or event.get("bindings_sha256") != binding_sha
                    or type(indices) is not list or any(type(i) is not int for i in indices)
                    or indices != sorted(set(indices))
                    or any(not 0 <= i < len(packet["records"]) for i in indices)
                    or len(event.get("receipts", [])) != len(packet["records"])
                    or event.get("row_forwards") != len(indices)
                    or event.get("batch_forwards") != int(bool(indices))):
                raise CampaignFailure("UNAVAILABLE_REFERENCE_REPLAY", "worker packet accounting differs", event)
            processed_rows += len(indices)
            processed_batches += int(bool(indices))
            budget.row_forwards += len(indices)
            if (event.get("cumulative_row_forwards") != processed_rows
                    or event.get("cumulative_batch_forwards") != processed_batches):
                raise CampaignFailure("UNAVAILABLE_REFERENCE_REPLAY", "worker cumulative counts differ")
            if callback is not None:
                callback(batch_no, packet, event)
            budget.check()
        process.stdin.close()
        final = receive()
        _worker_identity(final, bindings, binding_sha, ready=False, readiness=readiness)
        remaining = min(contract.LIMITS["phase_seconds"]-budget.elapsed,
                        contract.LIMITS["cumulative_seconds"]-budget.carried-budget.elapsed)
        if process.wait(timeout=max(0.01, remaining)) != 0:
            raise CampaignFailure("UNAVAILABLE_REFERENCE_REPLAY", "worker exited nonzero after terminal")
        if (final.get("row_forwards") != processed_rows or final.get("batch_forwards") != processed_batches
                or (not readiness and final["audit"].get("rows") != processed_rows)):
            raise CampaignFailure("UNAVAILABLE_REFERENCE_REPLAY", "worker final work accounting differs", final)
        final["initial"] = initial
        budget.check()
        return final
    finally:
        if process.poll() is None:
            os.killpg(process.pid, signal.SIGKILL)
            process.wait()
        for stream in (process.stdin, process.stdout, process.stderr):
            if stream and not stream.closed:
                stream.close()


def prepare(args) -> dict:
    """Named authoring/schema/hash/isolation preflight, with zero forwards."""
    args.output.mkdir(parents=True, exist_ok=True)
    budget = Budget(args, "preparation")
    contract.exclusive(args.output / "preparation-started.json", {
        "schema": "uor-r4.text-clause-preparation-started/1", "issue": 1094,
        "limits": contract.LIMITS, "policy_sha256": POLICY_SHA256,
        "selection": contract.record(args.corpus / "selection.json"),
        "corpus_manifest": contract.record(args.corpus / "population.json"),
        "model_forwards": 0, "optimizer_updates": 0,
    })
    manifest = json.loads((args.corpus / "population.json").read_bytes())
    selection = json.loads((args.corpus / "selection.json").read_bytes())
    if (manifest["policy_sha256"] != POLICY_SHA256
            or manifest["selection_sha256"] != contract.record(args.corpus / "selection.json")["sha256"]
            or manifest["curator_source_sha256"] != contract.record(Path(__file__).with_name("curate.py"))["sha256"]
            or manifest["model_forwards"] != 0 or manifest["optimizer_updates"] != 0
            or manifest["adapter_source_inspected_or_imported"]
            or manifest["historical_render_parse_decode_helpers_reused"]
            or manifest["counts"] != EXPECTED_COUNTS):
        raise ValueError("independent curator/policy/source/count provenance differs")
    expected_paths = {f"{part}/{kind}.jsonl" for part in ("authoring", "withheld") for kind in ("raw", "reference")}
    if {item["path"] for item in manifest["files"]} != expected_paths or len(manifest["files"]) != 4:
        raise ValueError("population file inventory differs")
    for item in manifest["files"]:
        if item["path"].startswith("authoring/"):
            contract.verify_record({**item, "path": str(args.corpus / item["path"])})
    bindings = contract.make_bindings(args.repo)
    contract.exclusive(args.output / "bindings.json", bindings)
    profile = contract.sandbox_profile(args.repo, args.python,
                                       args.output / "bindings.json", bindings["assets"])
    with (args.output / "worker.sb").open("x") as stream:
        stream.write(profile)
    probe = args.corpus / "isolation-probe.txt"
    if not probe.exists():
        with probe.open("x") as stream:
            stream.write("isolation control; contains no corpus or target\n")
    pairs = joint_rows(read_jsonl(args.corpus / "authoring/raw.jsonl"),
                       read_jsonl(args.corpus / "authoring/reference.jsonl"))
    integrity = population_integrity(pairs, selection, "authoring")
    checks, failures = collections.Counter(), []
    for row, ref in pairs:
        out = segment_request(actual_request(row))
        ok = adapter_fidelity(out, ref) if valid_reference(ref) else refusal_fidelity(out, ref)
        key = cell(row) if valid_reference(ref) else row["refusal_family"]
        checks[key + "/rows"] += 1
        checks[key + "/exact"] += ok
        if not ok:
            failures.append({"row_id": row["row_id"], "expected_status": ref["expected_status"],
                             "observed": out if not valid_reference(ref) else {k: out.get(k) for k in ("schema", "status", "policy_sha256")},
                             "field_exact": {k: out.get(k) == ref.get(k) for k in FIELDS}})
        budget.check()
    # Declared model-free schema probes exercise the actual transport-independent
    # API with one already-authoring request; no additional language row is made.
    sample = actual_request(next(row for row, ref in pairs if valid_reference(ref)))
    probes = {
        "text_must_be_bytes": segment_request({**sample, "text": sample["text"].decode("ascii")}),
        "external_roles_refused": segment_request({**sample, "roles": []}),
    }
    for name, out in probes.items():
        if out != {"schema": RESULT_SCHEMA, "status": "UNSUPPORTED_SCHEMA", "byte_offset": None}:
            failures.append({"probe": name, "observed": out})
    # Preserve an authoring miss before a separate startup/runtime failure can
    # stop preparation; neither outcome supersedes the other.
    input_preflight = contract.exclusive(args.output / "authoring-input-preflight.json", {
        "schema": "uor-r4.text-clause-authoring-input-preflight/1",
        "status": "AUTHORING_INPUT_EXACT" if not failures else "CLAUSE_ADAPTER_PREFLIGHT_MISS",
        "reference_integrity": integrity, "schema_probes": probes,
        "authoring_counts": dict(checks), "failure_count": len(failures),
        "failures": failures[:32], "model_loads": 0, "model_forwards": 0,
        "withheld_access": "NOT_RUN",
    })
    budget.record_event("coordinator", {"event": "authoring-input-preflight",
        "record": input_preflight, "failure_count": len(failures), "model_forwards": 0})
    readiness = arm_process(args, "adapter", [], budget, readiness=True)
    report = {
        "schema": "uor-r4.text-clause-preparation/1", "issue": 1094,
        "status": "COMPARISON_PREPARED" if not failures else "CLAUSE_ADAPTER_PREFLIGHT_MISS",
        "bindings": contract.record(args.output / "bindings.json"),
        "sandbox": contract.record(args.output / "worker.sb"),
        "selection": contract.record(args.corpus / "selection.json"),
        "corpus_manifest": contract.record(args.corpus / "population.json"),
        "corpus_commitments": manifest, "readiness": readiness,
        "reference_integrity": integrity, "schema_probes": probes,
        "authoring_counts": dict(checks), "failures": failures[:32],
        "failure_count": len(failures), "elapsed_seconds": budget.elapsed,
        "model_loads": 0, "model_forwards": 0, "optimizer_updates": 0,
        "withheld_access": "NOT_RUN", "withheld_evaluation": "NOT_RUN",
    }
    budget.check()
    contract.exclusive(args.output / "preparation.json", report)
    contract.exclusive(args.output / "preparation-closed.json", budget.snapshot())
    budget.check()
    return report


def batch_packets(rows: list[dict]):
    for start in range(0, len(rows), contract.LIMITS["batch_size"]):
        yield {"records": rows[start:start+contract.LIMITS["batch_size"]]}


def _decode_tensors(event: dict, rows: int) -> tuple[dict, list[str]]:
    values, errors = {}, []
    tensors = event.get("tensors")
    if type(tensors) is not dict or set(tensors) != (set(TENSORS) if rows else set()):
        errors.append("tensor inventory")
        tensors = tensors if type(tensors) is dict else {}
    for name, (dtype, tail) in TENSOR_LAYOUT.items():
        if not rows:
            continue
        item = tensors.get(name)
        if (type(item) is not dict or set(item) != {"dtype", "shape", "data_base64"}
                or item.get("dtype") != dtype or item.get("shape") != [rows, *tail]):
            errors.append(name + " shape/dtype")
            continue
        try:
            raw = base64.b64decode(item["data_base64"], validate=True)
        except (ValueError, TypeError):
            errors.append(name + " encoding")
            continue
        stride = math.prod(tail) * (8 if dtype == "int64" else 4)
        if len(raw) != rows * stride:
            errors.append(name + " byte length")
            continue
        values[name] = (raw, stride)
    return values, errors


def _row_bytes(tensors: dict, name: str, row: int | None) -> bytes | None:
    if row is None or name not in tensors:
        return None
    raw, stride = tensors[name]
    return raw[row*stride:(row+1)*stride]


def _row_integers(tensors: dict, name: str, row: int | None) -> list[int]:
    raw = _row_bytes(tensors, name, row)
    return list(struct.unpack("<" + "q" * (len(raw)//8), raw)) if raw is not None else []


def _model_token_exact(token: object, arm: str, ref: dict, prediction: int | None, bindings: dict) -> bool:
    if (type(prediction) is not int or not 0 <= prediction < 4096
            or type(token) is not dict or type(token.get("token_id")) is not int
            or type(token.get("token")) is not str):
        return False
    spelling = READER_PREFIX[prediction] if prediction < 52 else f"<unused-{prediction:04d}>"
    expected = {
        "schema": "uor-r4.text-binding-result/1" if arm == "adapter" else "uor-r4.oracle-binding-diagnostic/1",
        "status": "MODEL_TOKEN" if arm == "adapter" else "ORACLE_TOKEN",
        "policy_sha256": POLICY_SHA256,
        "raw_text_sha256": ref["raw_text_sha256"] if arm == "adapter" else None,
        "derived_input_sha256": ref["derived_input_sha256"],
        "reader_file_cid": bindings["assets"]["reader"]["cid"],
        "core_file_cid": bindings["assets"]["core"]["cid"],
        "frame_tree_cid": bindings["frame_tree_cid"],
        "token_id": prediction, "token": spelling,
    }
    return token == expected


class Comparison:
    def __init__(self, valid, invalid, bindings, budget: Budget):
        self.valid, self.invalid, self.bindings, self.budget = valid, invalid, bindings, budget
        self.checks, self.metrics = collections.Counter(), {}
        self.decisions, self.tensor_receipts = [], []
        self.complete_groups = collections.defaultdict(list)
        self.failures, self.failure_count = [], 0
        self.valid_seen, self.invalid_seen = 0, 0
        self.oracle_batches = math.ceil(len(valid)/contract.LIMITS["batch_size"])

    def fail(self, detail: dict) -> None:
        self.failure_count += 1
        if len(self.failures) < 64:
            self.failures.append(detail)

    def record_tensors(self, arm: str, batch_no: int, event: dict, tensors: dict) -> None:
        for name, (raw, _stride) in tensors.items():
            item = event["tensors"][name]
            identity = {"domain": "uor-r4.text-clause-tensor-receipt/1", "arm": arm,
                        "batch": batch_no, "name": name, "dtype": item["dtype"],
                        "shape": item["shape"], "valid_indices": event["valid_indices"]}
            self.tensor_receipts.append({**identity,
                "sha256": contract.digest(contract.canonical(identity) + raw), "bytes": len(raw)})

    def oracle(self, batch_no: int, packet: dict, event: dict) -> None:
        n = len(packet["records"])
        tensors, errors = _decode_tensors(event, n)
        if (errors or event["valid_indices"] != list(range(n))
                or event["receipts"] != [{"status": "ORACLE"} for _ in range(n)]
                or len(event.get("model_tokens", [])) != n):
            raise CampaignFailure("UNAVAILABLE_REFERENCE_REPLAY", "oracle output schema is incomplete", {"errors": errors})
        offset = batch_no * contract.LIMITS["batch_size"]
        for local in range(n):
            ref = self.valid[offset+local][1]
            prediction = _row_integers(tensors, "predictions", local)[0]
            if (not _model_token_exact(event["model_tokens"][local], "oracle", ref, prediction, self.bindings)
                    or _row_integers(tensors, "inputs", local) != [v for clause in ref["inputs"][0] for v in clause]
                    or _row_integers(tensors, "lengths", local) != ref["lengths"][0]):
                raise CampaignFailure("UNAVAILABLE_REFERENCE_REPLAY", "oracle token or supplied inputs differ")
        self.record_tensors("oracle", batch_no, event, tensors)

    def _metrics(self, arm: str, row: dict, ref: dict, prediction: int | None, positions: list[int]) -> None:
        expected_roles = [p for clause in ref["role_positions"] for p in clause]
        correct_roles = sum(i < len(positions) and positions[i] == target
                            for i, target in enumerate(expected_roles) if target >= 0)
        correct = prediction == ref["target_id"]
        key = cell(row)
        for stratum in ("all", "supported" if ref["supported"] else "unknown"):
            metric = self.metrics.setdefault(f"{arm}/{key}/{stratum}",
                {"rows": 0, "answer_correct": 0, "role_decisions": 0, "role_correct": 0})
            metric["rows"] += 1
            metric["answer_correct"] += correct
            # Missing output still contributes every required diagnostic slot.
            metric["role_decisions"] += 14
            metric["role_correct"] += correct_roles
        self.complete_groups[f"{arm}/{key}/{row['group_id']}"].append(
            {"variant": row["variant"], "correct": correct, "pair_type": ref["pair_type"]})

    def adapter(self, batch_no: int, packet: dict, event: dict, oracle: dict | None) -> None:
        indices = event["valid_indices"]
        mapping = {original: compressed for compressed, original in enumerate(indices)}
        tensors, errors = _decode_tensors(event, len(indices))
        if len(event.get("model_tokens", [])) != len(indices):
            errors.append("model-token count")
        if any((receipt.get("status") == "SEGMENTED") != (i in mapping)
               for i, receipt in enumerate(event["receipts"])):
            errors.append("acceptance/index mapping")
        if errors:
            self.fail({"batch": batch_no, "reason": "adapter output schema", "errors": errors})
        self.record_tensors("adapter", batch_no, event, tensors)
        if oracle is None:
            if indices or event["row_forwards"] or event["batch_forwards"] or tensors or event.get("model_tokens"):
                self.fail({"batch": batch_no, "reason": "refusal caused model work",
                           "actual_row_forwards": event["row_forwards"]})
            for local, out in enumerate(event["receipts"]):
                row, ref = self.invalid[self.invalid_seen]
                good = refusal_fidelity(out, ref) and local not in mapping
                family = row.get("refusal_family", row["kind"])
                key = f"{row['partition']}/{family}"
                self.checks[key + "/refusal"] += 1
                self.checks[key + "/refusal_exact"] += good
                if not good:
                    self.fail({"row_id": row["row_id"], "reason": "refusal", "observed": out,
                               "expected_status": ref["expected_status"],
                               "expected_byte_offset": ref.get("expected_byte_offset")})
                self.decisions.append({"row_id": row["row_id"], "refusal": out,
                                       "model_forwards": int(local in mapping)})
                self.invalid_seen += 1
            return
        n = len(packet["records"])
        oracle_tensors, oracle_errors = _decode_tensors(oracle, n)
        if oracle_errors:
            raise CampaignFailure("UNAVAILABLE_REFERENCE_REPLAY", "oracle spool tensor corruption")
        exact_batch = True
        for local in range(n):
            row, ref = self.valid[self.valid_seen]
            compressed = mapping.get(local)
            out = event["receipts"][local]
            fidelity = adapter_fidelity(out, ref)
            equal = {name: _row_bytes(oracle_tensors, name, local) == _row_bytes(tensors, name, compressed)
                     and _row_bytes(tensors, name, compressed) is not None for name in TENSORS}
            exact = not errors and all(equal.values())
            exact_batch = exact_batch and exact
            key = cell(row)
            self.checks[key + "/valid"] += 1
            self.checks[key + "/input_exact"] += fidelity
            self.checks[key + "/soft_exact"] += exact
            op = _row_integers(oracle_tensors, "predictions", local)[0]
            avalues = _row_integers(tensors, "predictions", compressed)
            ap = avalues[0] if avalues else None
            oroles = _row_integers(oracle_tensors, "role_positions", local)
            aroles = _row_integers(tensors, "role_positions", compressed)
            expected_roles = [p for clause in ref["role_positions"] for p in clause]
            role_exact = all(i < len(aroles) and oroles[i] == aroles[i]
                             for i, target in enumerate(expected_roles) if target >= 0)
            self.checks["role_rows_exact"] += role_exact
            self.checks["answer_rows_exact"] += ap is not None and op == ap
            for arm, prediction, positions in (("oracle", op, oroles), ("adapter", ap, aroles)):
                self._metrics(arm, row, ref, prediction, positions)
            model_token = (event["model_tokens"][compressed]
                           if compressed is not None and compressed < len(event.get("model_tokens", [])) else None)
            token_exact = _model_token_exact(model_token, "adapter", ref, ap, self.bindings)
            if not fidelity or not exact or not role_exact or not token_exact:
                self.fail({"row_id": row["row_id"], "reason": "adapter row fidelity",
                           "status": out.get("status"), "input_exact": fidelity,
                           "tensor_exact": equal, "role_exact": role_exact, "token_exact": token_exact})
            self.decisions.append({"row_id": row["row_id"], "cell": key,
                "oracle_token_id": op, "adapter_token_id": ap, "target_id": ref["target_id"],
                "input_exact": fidelity, "tensor_exact": equal, "model_token": model_token})
            self.valid_seen += 1
        self.checks["tensor_batches"] += 1
        self.checks["tensor_batches_exact"] += exact_batch

    def result(self, oracle_final: dict, adapter_final: dict) -> dict:
        if (self.valid_seen, self.invalid_seen) != (1600, 96) or oracle_final["row_forwards"] != 1600:
            raise CampaignFailure("UNAVAILABLE_REFERENCE_REPLAY", "complete row accounting differs")
        groups = {}
        for key, rows in self.complete_groups.items():
            if sorted(r["variant"] for r in rows) != [0, 1, 2, 3, 4]:
                raise ValueError("five-row group became incomplete")
            groups[key] = {"complete_five": all(r["correct"] for r in rows),
                          "complete_supported_four": all(r["correct"] for r in rows if r["variant"] < 4),
                          "pair_type": rows[0]["pair_type"]}
        if any(self.checks[f"{partition}/form-{f}/profile-{p}/valid"] != count
               for partition, count in (("authoring", 20), ("withheld", 80)) for f in range(4) for p in range(4)):
            raise ValueError("comparison lost valid cell denominators")
        return {"checks": dict(self.checks), "metrics": self.metrics, "groups": groups,
                "decisions": self.decisions, "tensors": self.tensor_receipts,
                "audits": {"oracle": oracle_final["audit"], "adapter": adapter_final["audit"]},
                "states": {"oracle": oracle_final["states_after"], "adapter": adapter_final["states_after"]},
                "work": {"oracle": oracle_final["row_forwards"], "adapter": adapter_final["row_forwards"]},
                "failures": self.failures, "failure_count": self.failure_count}


def _oracle_identity(evidence: dict) -> dict:
    return {"tensors": [r for r in evidence["tensors"] if r["arm"] == "oracle"],
            "decisions": [{"row_id": r["row_id"], "token_id": r["oracle_token_id"]}
                          for r in evidence["decisions"] if "oracle_token_id" in r],
            "metrics": {k: v for k, v in evidence["metrics"].items() if k.startswith("oracle/")},
            "groups": {k: v for k, v in evidence["groups"].items() if k.startswith("oracle/")},
            "audit": evidence["audits"]["oracle"], "states": evidence["states"]["oracle"],
            "work": evidence["work"]["oracle"]}


def run_phase(args, pairs, preparation, phase: str, budget: Budget):
    valid = [(row, ref) for row, ref in pairs if valid_reference(ref)]
    invalid = [(row, ref) for row, ref in pairs if not valid_reference(ref)]
    bindings = json.loads((args.output / "bindings.json").read_bytes())
    comparison = Comparison(valid, invalid, bindings, budget)
    temporary = args.output / f"{phase}-oracle-tensors.tmp.jsonl"
    spool_digest, spool_bytes = hashlib.sha256(), 0
    completed = False
    try:
        with temporary.open("xb") as spool:
            def oracle_callback(batch_no, packet, event):
                nonlocal spool_bytes
                comparison.oracle(batch_no, packet, event)
                payload = contract.canonical(event)
                spool.write(payload)
                spool.flush()
                spool_digest.update(payload)
                spool_bytes += len(payload)
                budget.check()
            oracle_final = arm_process(args, "oracle", batch_packets(
                [{key: ref[key] for key in ("inputs", "lengths")} for _, ref in valid]),
                budget, oracle_callback)
        with temporary.open("rb") as spool:
            consumed_digest = hashlib.sha256()
            def adapter_callback(batch_no, packet, event):
                oracle = None
                if batch_no < comparison.oracle_batches:
                    payload = spool.readline()
                    if not payload:
                        raise CampaignFailure("UNAVAILABLE_REFERENCE_REPLAY", "oracle spool ended early")
                    consumed_digest.update(payload)
                    oracle = json.loads(payload)
                comparison.adapter(batch_no, packet, event, oracle)
                budget.check()
            def packets():
                yield from batch_packets([transport(row) for row, _ in valid])
                yield from batch_packets([transport(row) for row, _ in invalid])
            adapter_final = arm_process(args, "adapter", packets(), budget, adapter_callback)
            if spool.read(1) or consumed_digest.digest() != spool_digest.digest():
                raise CampaignFailure("UNAVAILABLE_REFERENCE_REPLAY", "oracle spool identity/consumption differs")
        evidence = comparison.result(oracle_final, adapter_final)
        budget.check()
        completed = True
        return {"schema": "uor-r4.text-clause-comparison-phase/1", "phase": phase,
                "preparation_sha256": contract.record(args.output / "preparation.json")["sha256"],
                "deterministic": evidence, "deterministic_sha256": contract.digest(contract.canonical(evidence)),
                "oracle_replay_sha256": contract.digest(contract.canonical(_oracle_identity(evidence))),
                "elapsed_seconds": budget.elapsed,
                "worker_receipts": {"oracle": oracle_final, "adapter": adapter_final},
                "logical_row_forwards": oracle_final["row_forwards"] + adapter_final["row_forwards"],
                "temporary_oracle_stream": {"path": str(temporary), "bytes": spool_bytes,
                    "sha256": spool_digest.hexdigest(), "policy": "one sequential-arm spool; remove only after complete deterministic evidence is persisted; retain on exception"},
                "optimizer_updates": 0}
    finally:
        if temporary.exists():
            budget.record_event("coordinator", {"event": "oracle-spool-retained",
                "path": str(temporary), "bytes": spool_bytes,
                "sha256": spool_digest.hexdigest(), "comparison_completed": completed,
                "reason": "awaiting persisted comparison evidence" if completed else "partial comparison evidence"})


def _close_oracle_spool(report: dict, evidence_path: Path, budget: Budget) -> None:
    temporary = Path(report["temporary_oracle_stream"]["path"])
    if temporary != budget.output / f"{budget.phase}-oracle-tensors.tmp.jsonl":
        raise ValueError("unexpected disposable oracle spool path")
    # A complete phase record with the deterministic tensor receipts now exists.
    persisted = contract.record(evidence_path)
    if json.loads(evidence_path.read_bytes())["deterministic_sha256"] != report["deterministic_sha256"]:
        raise ValueError("persisted deterministic comparison identity differs")
    temporary.unlink()
    budget.record_event("coordinator", {"event": "oracle-spool-removed-after-evidence",
        "path": str(temporary), "retained_evidence": persisted,
        "spool_sha256": report["temporary_oracle_stream"]["sha256"]})


def _execution_identity(args, preparation: dict) -> dict:
    for name in ("bindings", "sandbox", "selection", "corpus_manifest"):
        contract.verify_record(preparation[name])
    bindings = json.loads((args.output / "bindings.json").read_bytes())
    for item in bindings["source_files"]:
        contract.verify_record(item)
    if bindings["hardware"] != contract.hardware_identity():
        raise CampaignFailure("UNAVAILABLE_REFERENCE_REPLAY", "physical hardware binding drifted")
    return bindings


def run(args) -> dict:
    budget = Budget(args, "execution")
    prep_path = args.output / "preparation.json"
    prep = json.loads(prep_path.read_bytes())
    prep_closed = json.loads((args.output / "preparation-closed.json").read_bytes())
    budget.carried = prep_closed["phase_elapsed_seconds"]
    if prep["status"] != "COMPARISON_PREPARED":
        raise ValueError("authoring preflight does not admit withheld access")
    bindings = _execution_identity(args, prep)
    review = json.loads(args.review.read_bytes())
    if (review.get("status") != "ACCEPTED_FOR_FROZEN_COMPARISON"
            or review.get("bindings_sha256") != prep["bindings"]["sha256"]
            or review.get("preparation_sha256") != contract.record(prep_path)["sha256"]
            or review.get("corpus_manifest_sha256") != prep["corpus_manifest"]["sha256"]
            or review.get("selection_sha256") != prep["selection"]["sha256"]):
        raise ValueError("independent review does not bind actual preparation and sealed commitments")
    budget.check()
    # This durable receipt precedes the first withheld file read/hash, including
    # integrity verification. The execution clock already includes review/source
    # readiness, and withheld release never renews the original cumulative cap.
    contract.exclusive(args.output / "execution-started.json", {
        "schema": "uor-r4.text-clause-execution-started/1", "issue": 1094,
        "preparation": contract.record(prep_path), "review": contract.record(args.review),
        "bindings_sha256": prep["bindings"]["sha256"],
        "carried_seconds": budget.carried, "hardware": bindings["hardware"],
        "optimizer_updates": 0, "withheld_payload_reads_before_receipt": 0,
    })
    for item in prep["corpus_commitments"]["files"]:
        if not isinstance(item, dict) or "sha256" not in item:
            raise ValueError("missing independent corpus commitment")
        contract.verify_record({**item, "path": str(args.corpus / item["path"])})
        budget.check()
    selection = json.loads((args.corpus / "selection.json").read_bytes())
    pairs, integrity = [], {}
    for partition in ("authoring", "withheld"):
        part = joint_rows(read_jsonl(args.corpus / partition / "raw.jsonl"),
                          read_jsonl(args.corpus / partition / "reference.jsonl"))
        integrity[partition] = population_integrity(part, selection, partition)
        pairs.extend(part)
        budget.check()
    if len({row["row_id"] for row, _ in pairs}) != len(pairs):
        raise ValueError("row identities overlap between partitions")
    phase = run_phase(args, pairs, prep, "execution", budget)
    phase.update(reference_integrity=integrity, hardware=bindings["hardware"])
    contract.exclusive(args.output / "execution.json", phase)
    _close_oracle_spool(phase, args.output / "execution.json", budget)
    contract.exclusive(args.output / "execution-closed.json", budget.snapshot())
    budget.check()
    carried = budget.carried + budget.elapsed
    replay_budget = Budget(args, "replay", carried, phase["logical_row_forwards"])
    signal.setitimer(signal.ITIMER_REAL, min(120, max(0.001, 360-carried)))
    bindings = _execution_identity(args, prep)
    contract.exclusive(args.output / "replay-started.json", {
        "schema": "uor-r4.text-clause-replay-started/1", "issue": 1094,
        "execution": contract.record(args.output / "execution.json"),
        "bindings_sha256": prep["bindings"]["sha256"], "carried_seconds": carried,
        "carried_logical_row_forwards": phase["logical_row_forwards"],
        "hardware": bindings["hardware"], "optimizer_updates": 0,
    })
    replay = run_phase(args, pairs, prep, "replay", replay_budget)
    replay.update(reference_integrity=integrity, hardware=bindings["hardware"])
    contract.exclusive(args.output / "replay.json", replay)
    _close_oracle_spool(replay, args.output / "replay.json", replay_budget)
    contract.exclusive(args.output / "replay-closed.json", replay_budget.snapshot())
    replay_budget.check()
    exact = phase["deterministic"] == replay["deterministic"]
    oracle_exact = phase["oracle_replay_sha256"] == replay["oracle_replay_sha256"]
    if not oracle_exact:
        status = "UNAVAILABLE_REFERENCE_REPLAY"
    elif (not exact or phase["deterministic"]["failure_count"]
          or replay["deterministic"]["failure_count"]):
        status = "CLAUSE_ADAPTER_MISS"
    else:
        status = "CLAUSE_ADAPTER_PRESERVED"
    report = {
        "schema": "uor-r4.text-clause-comparison-result/1", "issue": 1094,
        "status": status, "exact_fresh_process_replay": exact,
        "oracle_exact_fresh_process_replay": oracle_exact,
        "preparation": contract.record(prep_path),
        "execution": contract.record(args.output / "execution.json"),
        "replay": contract.record(args.output / "replay.json"),
        "elapsed_seconds": carried + replay_budget.elapsed,
        "logical_row_forwards": phase["logical_row_forwards"] + replay["logical_row_forwards"],
        "final_resource_path": str(args.output / "final-resources.json"),
        "terminal_authority": "requires completion.json bound to this result and no run-stopped.json; any stopped receipt takes precedence",
        "completion_path": str(args.output / "completion.json"),
        "optimizer_updates": 0, "semantic_world_novelty": False,
        "new_mathematical_proofs": 0, "generation": "NOT_RUN",
        "parent_973": "OPEN", "consumer_954": "BLOCKED",
    }
    contract.exclusive(args.output / "result.json", report)
    # This final snapshot is taken after deterministic evidence and result writes;
    # the subsequent check includes the resource receipt's own bytes and time.
    contract.exclusive(args.output / "final-resources.json", replay_budget.snapshot())
    replay_budget.check()
    contract.exclusive(args.output / "completion.json", {
        "schema": "uor-r4.text-clause-completion/1", "status": status,
        "result": contract.record(args.output / "result.json"),
        "final_resources": contract.record(args.output / "final-resources.json"),
        "authority": "valid only in the absence of run-stopped.json; a stopped receipt takes precedence",
    })
    replay_budget.check()
    return report


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("phase", choices=("prepare", "run"))
    parser.add_argument("--repo", type=Path, required=True)
    parser.add_argument("--corpus", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--python", type=Path, required=True)
    parser.add_argument("--review", type=Path)
    args = parser.parse_args()
    for key in ("repo", "corpus", "output", "python"):
        setattr(args, key, getattr(args, key).absolute())
    signal.signal(signal.SIGALRM, alarm)
    signal.setitimer(signal.ITIMER_REAL, 120)
    exit_code = 0
    try:
        result = prepare(args) if args.phase == "prepare" else run(args)
    except Exception as error:
        exit_code = 1
        status = (error.status if isinstance(error, CampaignFailure)
                  else "INCOMPLETE_RESOURCE" if isinstance(error, (TimeoutError, subprocess.TimeoutExpired))
                  else "UNAVAILABLE_COMPARISON_INPUT")
        budget = getattr(args, "_active_budget", None)
        try:
            resources = budget.snapshot() if budget is not None else None
        except (OSError, ValueError, KeyError):
            resources = None
        result = {"schema": "uor-r4.text-clause-stopped/1", "phase": args.phase,
                  "active_phase": budget.phase if budget is not None else None,
                  "status": status, "reason": str(error), "optimizer_updates": 0,
                  "cause": error.evidence if isinstance(error, CampaignFailure) else None,
                  "resources": resources,
                  "partial_progress": budget.progress if budget is not None else []}
        args.output.mkdir(parents=True, exist_ok=True)
        contract.exclusive(args.output / f"{args.phase}-stopped.json", result)
        # A resource-overrun stop is already terminal; do not replace its typed
        # cause with a second exception while preserving its final footprint.
        if budget is not None:
            try:
                contract.exclusive(args.output / f"{args.phase}-stopped-resources.json", budget.snapshot())
            except (OSError, ValueError, KeyError):
                pass
    finally:
        signal.setitimer(signal.ITIMER_REAL, 0)
    print(json.dumps(result, sort_keys=True))
    return exit_code


if __name__ == "__main__":
    raise SystemExit(main())
