#!/usr/bin/env python3
"""The one externally supervised export/gate/comparison/replay for #1102.

Only this coordinator reads the frozen authoring annotations. Isolated workers
receive the original raw bytes and original request fields, one row at a time.
All outputs are exclusive or append-only under the admitted run directory.
"""
from __future__ import annotations

import argparse
import base64
import copy
import hashlib
import json
import math
import os
from pathlib import Path
import struct
import subprocess
import sys
import traceback

import r4_native_bridge_1102_supervisor as supervisor
from r4_native_bridge_1102_export import export_artifact
from r4_native_bridge_1102_mutations import build_mutations

SHAPES = {"role_attention": [1, 5, 3, 13], "role_vectors": [1, 5, 3, 64],
          "binding_attention": [1, 5], "logits": [1, 4096]}
PARSED = ("inputs", "lengths", "token_spans", "clause_spans",
          "raw_text_sha256", "derived_input_sha256")


def canonical(value):
    return (json.dumps(value, sort_keys=True, separators=(",", ":"),
                       ensure_ascii=False, allow_nan=False) + "\n").encode()


def digest(data):
    return hashlib.sha256(data).hexdigest()


def write(path, data):
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("xb") as f:
        f.write(data)
        f.flush()
        os.fsync(f.fileno())
    return {"path": str(path), "bytes": len(data), "sha256": digest(data)}


def bound(record):
    path = Path(record["path"])
    if not path.is_absolute() or path.resolve() != path or not path.is_file():
        raise ValueError("bound file is not an absolute regular original")
    data = path.read_bytes()
    if len(data) != record["bytes"] or digest(data) != record["sha256"]:
        raise ValueError(f"bound identity changed: {path.name}")
    return data


class WorkerError(Exception):
    def __init__(self, event):
        self.event = event
        super().__init__(str(event))


def receive(process, last=None):
    line = process.stdout.readline(2 * 1024**2 + 1)
    if last is not None:
        last[0] = line
    if not line or len(line) > 2 * 1024**2 or not line.endswith(b"\n"):
        raise WorkerError({"kind": "error", "message": "incomplete or excessive worker event"})
    event = json.loads(line)
    if event.get("kind") == "error":
        raise WorkerError(event)
    return event


def launch(release, arm, phase, release_path, release_sha):
    paths = release["paths"]
    if arm == "native":
        command = [release["native"]["binary"], "run", str(release_path), release_sha,
                   str(Path(paths["run_root"]) / "exports/a.r4lr"),
                   str(Path(paths["run_root"]) / "exports/expected.json")]
    else:
        command = [release["reference"]["interpreter"]["launcher"],
                   str(Path(release["export"]["repo_root"]) / "scripts/r4_native_bridge_1102_reference.py"),
                   "--release", str(release_path), "--release-sha256", release_sha, "--phase", phase]
    profile = paths[f"{arm}_profile"]
    return ["/usr/bin/sandbox-exec", "-f", profile, *command]


def arm_run(release, arm, phase, release_path, release_sha, raw_rows):
    root = Path(release["paths"]["run_root"])
    prefix = root / "results" / f"{phase}-{arm}"
    prefix.parent.mkdir(exist_ok=True)
    results = []
    ready = None
    last = [b""]
    env = dict(os.environ, PYTHONDONTWRITEBYTECODE="1", PYTHONNOUSERSITE="1")
    # Worker processes receive no supervisor control descriptors.
    env.pop("R4_NATIVE_BRIDGE_EVENTS_FD", None)
    env.pop("R4_NATIVE_BRIDGE_ACK_FD", None)
    with Path(str(prefix) + ".stderr").open("xb") as err, \
            Path(str(prefix) + ".f32").open("xb") as tensors, \
            Path(str(prefix) + ".jsonl").open("xb") as records:
        process = subprocess.Popen(launch(release, arm, phase, release_path, release_sha),
                                   stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=err, env=env)
        try:
            candidate_ready = receive(process, last)
            if candidate_ready["kind"] != "ready" or candidate_ready["model_loads"] != 2 or candidate_ready["logical_forwards"] != 0:
                raise ValueError("worker did not establish one reader/core engine")
            ready = candidate_ready
            write(Path(str(prefix) + ".ready.json"), canonical(ready))
            for index, raw in enumerate(raw_rows):
                text = base64.b64decode(raw["text_base64"], validate=True)
                packet = {"schema": raw["request_schema"], "request_extras": raw["request_extras"]}
                packet["text_hex" if arm == "native" else "text_base64"] = (
                    text.hex() if arm == "native" else base64.b64encode(text).decode("ascii"))
                process.stdin.write(canonical(packet))
                process.stdin.flush()
                event = receive(process, last)
                if event["kind"] != "result":
                    raise ValueError("worker result event missing")
                persisted = copy.deepcopy(event)
                if event["logical_forwards"]:
                    if set(event["tensors"]) != set(SHAPES):
                        raise ValueError("complete tensor set missing")
                    for name, shape in SHAPES.items():
                        data = bytes.fromhex(event["tensors"][name])
                        if len(data) != 4 * math.prod(shape):
                            raise ValueError("tensor byte count differs")
                        values = struct.unpack("<" + "f" * (len(data) // 4), data)
                        if not all(math.isfinite(x) for x in values):
                            raise ValueError("nonfinite worker output")
                        persisted["tensors"][name] = {"offset": tensors.tell(), "bytes": len(data),
                                                      "shape": shape, "sha256": digest(data)}
                        tensors.write(data)
                        event["tensors"][name] = data
                elif event["tensors"] or event["parsed"] is not None or event["diagnostics"] is not None:
                    raise ValueError("refusal contains model diagnostics")
                persisted["coordinator_row_index"] = index
                persisted["coordinator_row_id"] = raw["row_id"]
                records.write(canonical(persisted))
                records.flush()
                tensors.flush()
                results.append(event)
            process.stdin.close()
            done = receive(process, last)
            if (done["kind"] != "done" or done["logical_forwards"] != 320
                    or done["model_loads"] != 2 or done["parameter_updates"] != 0
                    or done["valid_rows"] != 320 or done["refusal_rows"] != 16):
                raise ValueError("worker completion counts differ")
            if process.stdout.read() or process.wait() != 0:
                raise ValueError("worker exit/trailing bytes differ")
            write(Path(str(prefix) + ".done.json"), canonical(done))
            return {"rows": results, "done": done, "error": None, "ready":ready}
        except BaseException as error:
            if last[0]:
                write(Path(str(prefix) + ".failure-event.bin"), last[0])
            event = error.event if isinstance(error, WorkerError) else {
                "kind": "error", "error": str(error), "traceback": traceback.format_exc()}
            write(Path(str(prefix) + ".error.json"), canonical(event))
            return {"rows": results, "done": None, "error": event, "ready":ready}
        finally:
            if process.poll() is None:
                process.kill()
            process.wait()
            tensors.flush()
            records.flush()
            os.fsync(tensors.fileno())
            os.fsync(records.fileno())


def attempted_work(arm):
    completed = sum(row["logical_forwards"] for row in arm["rows"])
    done = arm.get("done") or {}
    error = arm.get("error") or {}
    exact = done.get("logical_forwards", error.get("work", {}).get("logical_forwards", error.get("logical_forwards")))
    return {"exact":exact, "known_lower_bound":max(completed, exact or 0),
            "admitted_upper_bound":320, "unavailable_counter":exact is None}


def score(arm, refs):
    errors, answers, roles, refusals = [], 0, 0, 0
    if arm["error"] is not None or len(arm["rows"]) != len(refs):
        errors.append({"error": "incomplete worker", "detail": arm["error"]})
    for i, (row, ref) in enumerate(zip(arm["rows"], refs)):
        result = row["result"]
        if ref["kind"] == "valid":
            if result.get("status") != "MODEL_TOKEN" or row["logical_forwards"] != 1:
                errors.append({"row": i, "error": "valid row did not forward"})
                continue
            for name in PARSED:
                if row["parsed"][name] != ref[name]:
                    errors.append({"row": i, "error": f"annotation mismatch: {name}"})
            answers += result["token_id"] == ref["target_id"]
            if result["token_id"] != ref["target_id"]:
                errors.append({"row":i,"error":"answer","expected":ref["target_id"],"actual":result["token_id"]})
            expected_roles = [v for clause in ref["role_positions"] for v in clause][:14]
            actual_roles = row["diagnostics"]["role_argmax"][:14]
            roles += sum(a == b for a, b in zip(actual_roles, expected_roles))
            for role, (actual, expected) in enumerate(zip(actual_roles, expected_roles)):
                if actual != expected:
                    errors.append({"row":i,"error":"consumed_role","role":role,"expected":expected,"actual":actual})
            attention = row["tensors"]["role_attention"]
            for clause, length in enumerate(row["parsed"]["lengths"][0]):
                for role in range(3):
                    for token in range(length, 13):
                        offset = ((clause * 3 + role) * 13 + token) * 4
                        if attention[offset:offset + 4] != b"\0" * 4:
                            errors.append({"row": i, "error": "nonzero padding probability"})
        else:
            good = (result.get("status") == ref["expected_status"] and row["logical_forwards"] == 0
                    and set(result) == {"schema", "status", "byte_offset"})
            if "expected_byte_offset" in ref:
                good &= result.get("byte_offset") == ref["expected_byte_offset"]
            refusals += bool(good)
            if not good:
                errors.append({"row": i, "error": "refusal mismatch", "actual": result})
    if (answers, roles, refusals) != (320, 4480, 16):
        errors.append({"error": "required floors", "answers": answers, "roles": roles, "refusals": refusals})
    return {"answers": answers, "roles": roles, "refusals": refusals, "errors": errors}


def compare(reference, native):
    errors, rows = [], []
    for index, (a, b) in enumerate(zip(reference["rows"], native["rows"])):
        for field in ("result", "parsed", "logical_forwards"):
            if a[field] != b[field]:
                errors.append({"row": index, "field": field})
        if a["diagnostics"] is not None and b["diagnostics"] is not None:
            for field in ("token_frame_indices", "clause_frame_indices"):
                if a["diagnostics"][field] != b["diagnostics"][field]:
                    errors.append({"row": index, "field": field})
            if a["diagnostics"]["role_argmax"][:14] != b["diagnostics"]["role_argmax"][:14]:
                errors.append({"row": index, "field": "consumed_role_argmax"})
        maxima = {}
        for name in SHAPES:
            if name not in a["tensors"] or name not in b["tensors"]:
                continue
            av = struct.unpack("<" + "f" * (len(a["tensors"][name]) // 4), a["tensors"][name])
            bv = struct.unpack("<" + "f" * (len(b["tensors"][name]) // 4), b["tensors"][name])
            location, difference = max(enumerate(abs(x - y) for x, y in zip(av, bv)), key=lambda x: x[1])
            maxima[name] = {"max_abs": difference, "flat_index": location}
            if difference > 1e-5:
                errors.append({"row": index, "tensor": name, **maxima[name]})
        rows.append({"row": index, "maxima": maxima})
    if len(reference["rows"]) != len(native["rows"]):
        errors.append({"error": "different completed row counts"})
    return {"errors": errors, "rows": rows}


def replay_equal(a, b):
    def normalized_done(v):
        return {k: x for k, x in (v or {}).items() if k not in ("phase", "resources")}
    return (a["error"] is None and b["error"] is None and a["rows"] == b["rows"]
            and normalized_done(a["done"]) == normalized_done(b["done"]))


def perform(release_path, release_sha, review_path, review_sha):
    release_bytes = release_path.read_bytes()
    if digest(release_bytes) != release_sha:
        raise ValueError("release digest mismatch")
    release = json.loads(release_bytes)
    if canonical(release) != release_bytes:
        raise ValueError("release is not canonical")
    review_bytes = review_path.read_bytes()
    review = json.loads(review_bytes)
    if digest(review_bytes) != review_sha or review.get("release_sha256") != release_sha \
            or review.get("status") != "ACCEPTED_FOR_ONE_NATIVE_BRIDGE_ATTEMPT":
        raise ValueError("independent release acceptance missing")
    root = Path(release["paths"]["run_root"])
    # Reconcile all executable/config bytes before asset access.
    for record in release["launch_files"]:
        bound(record)
    raw_data = bound(release["fixtures"]["raw"])
    reference_data = bound(release["fixtures"]["reference"])
    raw_rows = [json.loads(line) for line in raw_data.splitlines()]
    refs = [json.loads(line) for line in reference_data.splitlines()]
    if len(raw_rows) != 336 or len(refs) != 336:
        raise ValueError("original authoring population count differs")
    for i, (raw, ref) in enumerate(zip(raw_rows, refs)):
        if raw["row_id"] != ref["row_id"] or raw["kind"] != ref["kind"] \
                or raw["kind"] != ("valid" if i < 320 else "refusal") \
                or raw["partition"] != "authoring" or ref["partition"] != "authoring":
            raise ValueError("original population order/identity differs")
    artifacts = []
    for suffix in ("a", "b"):
        artifact, expected = export_artifact(release["asset_paths"], release, release_sha)
        artifacts.append((artifact, expected))
        write(root / f"exports/{suffix}.r4lr", artifact)
    if artifacts[0] != artifacts[1]:
        return "NATIVE_REFERENCE_MISMATCH", {"failure": "duplicate export mismatch"}
    artifact, expected = artifacts[0]
    write(root / "exports/expected.json", canonical(expected))
    fixtures = build_mutations(artifact, expected)
    if len(fixtures) != 11:
        raise ValueError("mutation count differs")
    manifest = []
    for i, fixture in enumerate(fixtures):
        path = root / f"mutations/{i:02d}.r4lr"
        ep = root / f"mutations/{i:02d}.expected.json"
        manifest.append({"name": fixture["name"], "expected_error": fixture["expected_error"],
                         "artifact": write(path, fixture["artifact"]),
                         "expected_binding": write(ep, canonical(fixture["expected_binding"]))})
    # Every copy, digest and expected error is durable before the first call.
    write(root / "mutations/frozen.json", canonical(manifest))
    gate = []
    for i, item in enumerate(manifest + [{"artifact": {"path": str(root / "exports/a.r4lr")},
                                        "expected_binding": {"path": str(root / "exports/expected.json")}}]):
        command = ["/usr/bin/sandbox-exec", "-f", release["paths"]["native_profile"],
                   release["native"]["binary"], "gate", str(release_path), release_sha,
                   item["artifact"]["path"], item["expected_binding"]["path"]]
        with (root / f"mutations/gate-{i:02d}.stderr").open("xb") as err:
            process = subprocess.run(command, stdout=subprocess.PIPE, stderr=err, check=False)
        write(root / f"mutations/gate-{i:02d}.json", process.stdout)
        event = json.loads(process.stdout)
        gate.append(event)
        if process.returncode != 0 or event.get("kind") != "gate":
            return "NATIVE_REFERENCE_MISMATCH", {"gate": gate, "failure": "native loader worker failed", "numerical_verdict": "NOT_RUN"}
        if i < 11 and (event["error"] != item["expected_error"] or event.get("logical_forwards") != 0 or event.get("model_loads") != 0):
            return "NATIVE_REFERENCE_MISMATCH", {"gate": gate, "failure": "frozen loader error mismatch", "numerical_verdict": "NOT_RUN"}
        if i == 11 and (event["error"] is not None or event["missing_qualification"]["tag"] != "UNAVAILABLE_NATIVE_QUALIFICATION"
                        or event["capability"]["native_behavior"] != "NOT_RUN" or event["logical_forwards"] != 0):
            return "NATIVE_REFERENCE_MISMATCH", {"gate": gate, "failure": "valid loader/qualification gate", "numerical_verdict": "NOT_RUN"}
    write(root / "results-gate.json", canonical(gate))
    supervisor.phase("execution")
    initial_ref = arm_run(release, "reference", "execution", release_path, release_sha, raw_rows)
    ref_score = score(initial_ref, refs)
    if ref_score["errors"]:
        status = (initial_ref["error"] or {}).get("status", "UNAVAILABLE_NATIVE_REFERENCE")
        return status, {"reference": ref_score, "failure": "initial reference invalid"}
    initial_native = arm_run(release, "native", "execution", release_path, release_sha, raw_rows)
    if (initial_native["error"] or {}).get("status") == "ABORTED_NATIVE_REFERENCE_BUDGET":
        return "ABORTED_NATIVE_REFERENCE_BUDGET", {"native_error":initial_native["error"]}
    initial_comparison = compare(initial_ref, initial_native)
    write(root / "comparison-initial.json", canonical(initial_comparison))
    supervisor.phase("replay")
    replay_ref = arm_run(release, "reference", "replay", release_path, release_sha, raw_rows)
    replay_ref_score = score(replay_ref, refs)
    ref_replay = replay_equal(initial_ref, replay_ref)
    if replay_ref_score["errors"] or not ref_replay:
        status = (replay_ref["error"] or {}).get("status", "UNAVAILABLE_NATIVE_REFERENCE")
        return status, {"reference": ref_score, "replay_reference": replay_ref_score,
                        "reference_exact_replay": ref_replay, "failure": "reference replay invalid"}
    replay_native = arm_run(release, "native", "replay", release_path, release_sha, raw_rows)
    if (replay_native["error"] or {}).get("status") == "ABORTED_NATIVE_REFERENCE_BUDGET":
        return "ABORTED_NATIVE_REFERENCE_BUDGET", {"native_error":replay_native["error"]}
    replay_comparison = compare(replay_ref, replay_native)
    write(root / "comparison-replay.json", canonical(replay_comparison))
    native_score = score(initial_native, refs)
    native_replay_score = score(replay_native, refs)
    native_replay = replay_equal(initial_native, replay_native)
    ok = not (native_score["errors"] or native_replay_score["errors"] or initial_comparison["errors"]
              or replay_comparison["errors"]) and native_replay
    return ("NATIVE_REFERENCE_PRESERVED" if ok else "NATIVE_REFERENCE_MISMATCH"), {
        "reference": ref_score, "replay_reference": replay_ref_score, "native": native_score,
        "replay_native": native_replay_score, "reference_exact_replay": ref_replay,
        "native_exact_replay": native_replay, "initial_comparison_errors": initial_comparison["errors"],
        "replay_comparison_errors": replay_comparison["errors"],
        "artifact_sha256": expected["artifact_sha256"], "native_state_sha256": gate[-1]["native_state_sha256"],
        "completed_logical_forwards": sum(x["logical_forwards"] for arm in [initial_ref, initial_native, replay_ref, replay_native] for x in arm["rows"]),
        "successful_engine_loads": 1 + sum(x["ready"] is not None for x in [initial_ref,initial_native,replay_ref,replay_native]),
        "completed_model_state_loads": 2 + sum((x["ready"] or {}).get("model_loads",0) for x in [initial_ref,initial_native,replay_ref,replay_native]), "loader_gate_calls": len(gate),
        "loader_rejected_partial_model_states": sum(len(x["validation_audit"]["partial_model_states"]) for x in gate[:-1]),
        "worker_failures": [x["error"] for x in [initial_ref,initial_native,replay_ref,replay_native] if x["error"]],
        "attempted_forward_counts": {name:attempted_work(arm) for name,arm in zip(
            ["initial_reference","initial_native","replay_reference","replay_native"],
            [initial_ref,initial_native,replay_ref,replay_native])},
        "parameter_updates": 0, "withheld_reads": 0,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for field in ("release", "release-sha256", "review", "review-sha256"):
        parser.add_argument("--" + field, required=True)
    args = parser.parse_args()
    try:
        terminal, details = perform(Path(args.release), args.release_sha256, Path(args.review), args.review_sha256)
    except BaseException as error:
        terminal, details = "UNAVAILABLE_NATIVE_REFERENCE", {"error": str(error), "traceback": traceback.format_exc()}
    root = Path(json.loads(Path(args.release).read_bytes())["paths"]["run_root"])
    result_record = write(root / "coordinator-result.json", canonical({"terminal": terminal, "details": details}))
    supervisor.complete(terminal, details={"result":result_record})
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
