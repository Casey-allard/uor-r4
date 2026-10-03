#!/usr/bin/env python3
"""One admitted B=1 Python reference worker for the frozen #1102 comparison.

The external supervisor owns the consumed envelope, durable evidence, combined
RSS and all startup/exit time. This process has an additional 120-second/3-GiB
guard. It receives raw requests only; fixture/reference files are inaccessible.
No command from the historical #1094 worker or its consumed envelope is used.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import resource
import signal
import stat
import sys
import time
import traceback
from pathlib import Path


CONTRACT_SHA256 = "e3565728dfa872d9c260dd3d391ae59e0b0d454ebead20cd15cb075b24796115"
MAX_SECONDS = 120
MAX_RSS_BYTES = 3 * 1024**3
MAX_PACKET_BYTES = 2 * 1024**2
VALID_ROWS = 320
REFUSAL_ROWS = 16
TENSOR_SHAPES = {
    "role_attention": [1, 5, 3, 13],
    "role_vectors": [1, 5, 3, 64],
    "binding_attention": [1, 5],
    "logits": [1, 4096],
}
PARSED_FIELDS = (
    "inputs", "lengths", "token_spans", "clause_spans",
    "raw_text_sha256", "derived_input_sha256",
)


class BudgetExceeded(RuntimeError):
    """A local backstop fired; external supervisor bounds still take priority."""


def _unique_object(pairs: list[tuple[str, object]]) -> dict:
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def _decode(raw: bytes) -> dict:
    value = json.loads(raw, object_pairs_hook=_unique_object)
    if type(value) is not dict:
        raise ValueError("JSON record must be an object")
    return value


def _emit(value: dict) -> None:
    print(json.dumps(value, allow_nan=False, separators=(",", ":")), flush=True)


def _resources(started: float) -> dict:
    peak = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
    return {
        "elapsed_seconds": time.monotonic() - started,
        "peak_rss_bytes": int(peak if sys.platform == "darwin" else peak * 1024),
    }


def _guard(started: float) -> dict:
    observation = _resources(started)
    if observation["elapsed_seconds"] > MAX_SECONDS:
        raise BudgetExceeded("reference worker exceeded its 120-second backstop")
    if observation["peak_rss_bytes"] > MAX_RSS_BYTES:
        raise BudgetExceeded("reference worker exceeded its 3-GiB RSS backstop")
    return observation


def _alarm(_number, _frame) -> None:
    raise BudgetExceeded("reference worker 120-second wall-clock alarm")


def _regular_path(value: object) -> Path:
    if type(value) is not str:
        raise ValueError("bound path must be a string")
    path = Path(value)
    if not path.is_absolute() or any(part == ".." for part in path.parts):
        raise ValueError("bound path must be absolute without parent traversal")
    for part in (path, *path.parents):
        if part.is_symlink():
            raise ValueError(f"bound path has a symlink component: {path.name}")
    if not stat.S_ISREG(path.stat().st_mode):
        raise ValueError(f"bound path is not a regular file: {path.name}")
    return path


def _verify_source(record: dict) -> Path:
    if type(record) is not dict or not {"path", "sha256", "bytes"} <= set(record):
        raise ValueError("source binding requires path, sha256 and bytes")
    path = _regular_path(record["path"])
    if type(record["bytes"]) is not int or record["bytes"] < 0:
        raise ValueError("source byte length must be an unsigned integer")
    payload = path.read_bytes()
    if len(payload) != record["bytes"] or hashlib.sha256(payload).hexdigest() != record["sha256"]:
        raise ValueError(f"source binding differs: {path.name}")
    return path


def _configuration(release_path: Path, expected_sha256: str) -> tuple[dict, dict]:
    if len(expected_sha256) != 64 or any(c not in "0123456789abcdef" for c in expected_sha256):
        raise ValueError("release SHA256 must be 64 lowercase hexadecimal characters")
    path = _regular_path(str(release_path))
    if path.stat().st_size > 4 * 1024**2:
        raise ValueError("release exceeds the worker metadata limit")
    payload = path.read_bytes()
    if hashlib.sha256(payload).hexdigest() != expected_sha256:
        raise ValueError("externally admitted release SHA256 differs")
    release = _decode(payload)
    if (release.get("schema") != "uor-r4.native-bridge-release/1"
            or type(release.get("issue")) is not int or release["issue"] != 1102
            or release.get("contract_sha256") != CONTRACT_SHA256):
        raise ValueError("release does not identify the frozen #1102 contract")
    config = release["reference"]
    if type(config) is not dict or set(config) != {"bindings", "probes", "source_files", "interpreter"}:
        raise ValueError("reference configuration has extra or missing fields")
    interpreter = config["interpreter"]
    if type(interpreter) is not dict or set(interpreter) != {"launcher", "resolved", "torch_file"}:
        raise ValueError("interpreter binding has extra or missing fields")
    if (sys.executable != interpreter["launcher"]
            or str(Path(sys.executable).resolve()) != interpreter["resolved"]
            or not Path(interpreter["torch_file"]).is_absolute()):
        raise ValueError("executing interpreter differs from the release")
    sources = config["source_files"]
    if type(sources) is not list or not sources:
        raise ValueError("reference source closure is absent")
    paths = [_verify_source(record) for record in sources]
    if len(set(paths)) != len(paths) or Path(__file__).resolve() not in paths:
        raise ValueError("source closure is duplicated or omits this worker")
    workers = [path for path in paths if path.parts[-3:] == (
        "r4_softmax_trainer", "text_clause_adapter", "worker.py")]
    if len(workers) != 1:
        raise ValueError("release must bind exactly one accepted worker module")
    package = workers[0].parents[1]
    required = {
        package / "__init__.py",
        package / "text_clause_adapter" / "__init__.py",
        package / "text_clause_adapter" / "adapter.py",
        package / "text_clause_adapter" / "policy.json",
    }
    if not required <= set(paths):
        raise ValueError("source closure omits pre-import package or policy files")
    bindings = config["bindings"]
    if type(bindings) is not dict or not isinstance(bindings.get("source_files"), list):
        raise ValueError("accepted source/asset bindings are absent")
    if not {Path(record["path"]) for record in bindings["source_files"]} <= set(paths):
        raise ValueError("pre-import source closure omits an accepted source file")
    probes = config["probes"]
    if type(probes) is not dict or set(probes) != {"corpus", "reference", "history", "results"}:
        raise ValueError("all four denied-path sentinel probes are required")
    denied = {}
    for name, sentinel in sorted(probes.items()):
        if type(sentinel) is not str or not Path(sentinel).is_absolute():
            raise ValueError("probe must be an absolute harmless sentinel path")
        try:
            with Path(sentinel).open("rb"):
                pass
        except PermissionError:
            denied[name] = True
        except OSError as error:
            raise ValueError(f"{name} probe did not fail with PermissionError") from error
        else:
            raise ValueError(f"OS isolation allowed the {name} sentinel")
    # No project/model imports precede source verification and denied probes.
    sys.path.insert(0, str(package.parent))
    return config, denied


def _merge_audit(total: dict, row: dict) -> None:
    for name, value in row.items():
        if type(value) is int:
            total[name] = total.get(name, 0) + value
        elif type(value) is list:
            total[name] = sorted(set(total.get(name, [])) | set(value))
        else:
            raise ValueError("accepted inference audit has an unexpected value type")


def _result(worker, packet: dict, vocabulary: list[str], adapter_record, started: float) -> dict:
    import torch
    from r4_softmax_trainer.zoology_language_r4.attention import frame_assignment

    receipt = adapter_record(packet)
    _guard(started)
    if receipt["status"] != "SEGMENTED":
        if worker.refusal_rows >= REFUSAL_ROWS:
            raise BudgetExceeded("a seventeenth reference refusal is not admitted")
        worker.refusal_rows += 1
        return {"kind": "result", "result": receipt, "parsed": None,
                "tensors": {}, "diagnostics": None, "logical_forwards": 0}
    if worker.row_forwards >= VALID_ROWS:
        raise BudgetExceeded("a 321st reference forward is not admitted")
    inputs = torch.tensor(receipt["inputs"], dtype=torch.long, device="cpu")
    lengths = torch.tensor(receipt["lengths"], dtype=torch.long, device="cpu")
    if tuple(inputs.shape) != (1, 5, 13) or tuple(lengths.shape) != (1, 5):
        raise ValueError("accepted adapter changed the fixed B=1 input shape")
    worker.inference.reset_audit()
    # Count an attempted forward even if the accepted reference raises.
    worker.row_forwards += 1
    worker.batch_forwards += 1
    with torch.inference_mode():
        output = worker.inference(inputs, lengths, control="none")
        token_frames, clause_frames = frame_assignment(inputs, lengths, worker.inference.frames)
        tensors = {}
        for name, shape in TENSOR_SHAPES.items():
            tensor = output[name]
            if (tensor.device.type != "cpu" or tensor.dtype != torch.float32
                    or list(tensor.shape) != shape or not bool(torch.isfinite(tensor).all())):
                raise ValueError(f"reference {name} is not the required finite CPU f32 tensor")
            tensors[name] = tensor.detach().contiguous().numpy().tobytes(order="C").hex()
        for clause, length in enumerate(receipt["lengths"][0]):
            padding = output["role_attention"][0, clause, :, length:]
            if bool((padding != 0).any()) or bool(torch.signbit(padding).any()):
                raise ValueError("reference padding attention is not positive zero")
        token_id = int(output["logits"].argmax(dim=-1).item())
        roles = output["role_attention"].argmax(dim=-1).flatten().tolist()
    _guard(started)
    bindings = worker.bridge_bindings
    result = {
        "schema": "uor-r4.text-binding-result/1", "status": "MODEL_TOKEN",
        "policy_sha256": bindings["policy_sha256"],
        "raw_text_sha256": receipt["raw_text_sha256"],
        "derived_input_sha256": receipt["derived_input_sha256"],
        "reader_file_cid": bindings["assets"]["reader"]["cid"],
        "core_file_cid": bindings["assets"]["core"]["cid"],
        "frame_tree_cid": bindings["frame_tree_cid"],
        "token_id": token_id, "token": vocabulary[token_id],
    }
    return {
        "kind": "result", "result": result,
        "parsed": {name: receipt[name] for name in PARSED_FIELDS}, "tensors": tensors,
        "diagnostics": {"role_argmax": roles,
                        "token_frame_indices": token_frames.flatten().tolist(),
                        "clause_frame_indices": clause_frames.flatten().tolist()},
        "logical_forwards": 1,
    }


def main() -> int:
    started = time.monotonic()
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--release", type=Path, required=True)
    parser.add_argument("--release-sha256", required=True)
    parser.add_argument("--phase", choices=("execution", "replay"), required=True)
    args = parser.parse_args()
    worker = None
    load_started = False
    total_audit = {}
    signal.signal(signal.SIGALRM, _alarm)
    signal.setitimer(signal.ITIMER_REAL, MAX_SECONDS)
    try:
        config, denied = _configuration(args.release, args.release_sha256)
        _guard(started)
        from r4_softmax_trainer.text_clause_adapter.worker import (
            Worker, _adapter_record, _configure_runtime, _verify_bindings,
        )

        worker = Worker(started)
        worker.runtime = _configure_runtime()
        if str(Path(sys.modules["torch"].__file__).resolve()) != config["interpreter"]["torch_file"]:
            raise ValueError("Torch imported from an unbound runtime path")
        _guard(started)
        payloads, vocabulary = _verify_bindings(config["bindings"])
        worker.bridge_bindings = config["bindings"]
        _guard(started)
        load_started = True
        worker.load(config["bindings"], payloads)
        del payloads
        _guard(started)
        _emit({"kind": "ready", "phase": args.phase, "release_sha256": args.release_sha256,
               "runtime": worker.runtime, "states_before": worker.states_before,
               "model_loads": worker.model_loads, "logical_forwards": 0,
               "denied_probes": denied})
        requests = 0
        while True:
            line = sys.stdin.buffer.readline(MAX_PACKET_BYTES + 1)
            if not line:
                break
            _guard(started)
            if len(line) > MAX_PACKET_BYTES:
                raise ValueError("reference raw-request packet exceeds its transport limit")
            if requests >= VALID_ROWS + REFUSAL_ROWS:
                raise BudgetExceeded("a 337th reference request is not admitted")
            requests += 1
            response = _result(worker, _decode(line), vocabulary, _adapter_record, started)
            if response["logical_forwards"]:
                _merge_audit(total_audit, worker.inference.audit)
            _emit(response)
            _guard(started)
        states_after = worker.states()
        if states_after != worker.states_before:
            raise ValueError("reference inference changed the accepted parameter state")
        if worker.row_forwards != VALID_ROWS or worker.refusal_rows != REFUSAL_ROWS:
            raise ValueError("reference input ended without exactly 320 valid and 16 refusal rows")
        observation = _guard(started)
        _emit({"kind": "done", "phase": args.phase,
               "states_before": worker.states_before, "states_after": states_after,
               "valid_rows": worker.row_forwards, "refusal_rows": worker.refusal_rows,
               "logical_forwards": worker.row_forwards, "model_loads": worker.model_loads,
               "parameter_updates": 0, "audit": total_audit, "resources": observation})
        _guard(started)
        return 0
    except BaseException as error:
        status = ("ABORTED_NATIVE_REFERENCE_BUDGET" if isinstance(error, BudgetExceeded)
                  else "UNAVAILABLE_NATIVE_REFERENCE")
        _emit({"kind": "error", "phase": args.phase, "status": status,
               "error_type": type(error).__name__, "error": str(error),
               "traceback": "".join(traceback.format_exception(error)),
               "states_before": None if worker is None else worker.states_before,
               "model_loads": 0 if worker is None else worker.model_loads,
               "model_load_call_started": load_started,
               "partial_model_state_load_upper_bound": 2 if load_started else 0,
               "logical_forwards": 0 if worker is None else worker.row_forwards,
               "refusal_rows": 0 if worker is None else worker.refusal_rows,
               "parameter_updates": 0, "resources": _resources(started)})
        return 2
    finally:
        signal.setitimer(signal.ITIMER_REAL, 0)


if __name__ == "__main__":
    raise SystemExit(main())
