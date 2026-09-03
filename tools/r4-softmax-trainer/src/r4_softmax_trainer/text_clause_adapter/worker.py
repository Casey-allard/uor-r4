"""Isolated, read-only JSON-lines worker for the frozen #1094 comparison.

The driver supplies only raw requests to the adapter arm and only clause tensors
to the oracle arm. This module never reads a population, annotation, preparation
report or fit report. All source/asset bindings and the OS isolation probe must
pass before model construction, and every result binds the configuration bytes.
"""

from __future__ import annotations

import argparse
import base64
import binascii
import hashlib
import json
import os
import platform
import resource
import signal
import sys
import time
from pathlib import Path

from .adapter import (
    POLICY_SHA256,
    READER_PREFIX,
    VOCABULARY_FILE_CID,
    derived_input_sha256,
    segment_request,
    unavailable_artifact,
)


READER_FILE_CID = "blake3:c11d21817bff818fa242f653279e9e0c12d21641ff63df3a5f7a6680bcc732a7"
CORE_FILE_CID = "blake3:9c055cc6ea09548bf960e37288276535b30515b94a50a96aa929b5e55afea3c4"
READER_STATE_CID = "blake3:7c659422df2e65a0ce24c08738dc9f08dca99775de1702251097a0fc6483404e"
CORE_STATE_CID = "blake3:abbdbcaafc2d9eb36543ce75fbb0101b6788119d80a6ed9c017bb9d06fbeac59"
FRAME_TREE_CID = "blake3:94762441a43b03f596a66131ec34af15bba3afbc2bbc5d28ab7dfdabd9b6d68c"
MAX_RSS_BYTES = 3 * 1024**3
MAX_SECONDS = 120
MAX_PACKET_BYTES = 2 * 1024**2
MAX_LOGICAL_ROWS = 1600
_ASSET_NAMES = {"reader", "core", "vocabulary", "h4_frames", "token_frames"}
_EXPECTED_RUNTIME = {
    "python": "3.12.14",
    "torch": "2.7.1",
    "device": "cpu",
    "threads": 4,
    "interop_threads": 1,
    "workers": 1,
    "blas": "accelerate",
}


class WorkerFailure(Exception):
    def __init__(self, status: str, reason: str) -> None:
        super().__init__(reason)
        self.status = status


def _unique_object(pairs: list[tuple[str, object]]) -> dict:
    value = {}
    for key, item in pairs:
        if key in value:
            raise ValueError(f"duplicate JSON key: {key}")
        value[key] = item
    return value


def _json(raw: bytes) -> dict:
    value = json.loads(raw, object_pairs_hook=_unique_object)
    if type(value) is not dict:
        raise ValueError("record must be a JSON object")
    return value


def _emit(value: dict) -> None:
    print(
        json.dumps(value, ensure_ascii=False, allow_nan=False, separators=(",", ":")),
        flush=True,
    )


def _peak_rss_bytes() -> int:
    peak = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
    return int(peak if sys.platform == "darwin" else peak * 1024)


def _cid(payload: bytes) -> str:
    from blake3 import blake3

    return f"blake3:{blake3(payload).hexdigest()}"


def _canonical(value: object) -> bytes:
    return (
        json.dumps(
            value, ensure_ascii=False, allow_nan=False, sort_keys=True,
            separators=(",", ":"),
        )
        + "\n"
    ).encode("utf-8")


def _bound_file(record: dict, *, require_bytes: bool = True) -> bytes:
    path = Path(record["path"])
    if not path.is_absolute() or path.is_symlink() or not path.is_file():
        raise ValueError("bound files must be absolute regular non-symlink paths")
    payload = path.read_bytes()
    if hashlib.sha256(payload).hexdigest() != record["sha256"]:
        raise ValueError(f"bound SHA256 differs: {path.name}")
    if require_bytes or "bytes" in record:
        if type(record["bytes"]) is not int or len(payload) != record["bytes"]:
            raise ValueError(f"bound byte length differs: {path.name}")
    if record.get("cid") is not None and _cid(payload) != record["cid"]:
        raise ValueError(f"bound BLAKE3 file CID differs: {path.name}")
    return payload


def _isolation_probe() -> bool:
    probe = os.environ.get("UOR_ISOLATION_PROBE")
    if not probe:
        raise ValueError("required denied-reference isolation probe is absent")
    try:
        with Path(probe).open("rb"):
            pass
    except PermissionError:
        return True
    except OSError as error:
        raise ValueError("isolation probe failed without permission denial") from error
    raise ValueError("OS sandbox allowed the denied-reference isolation probe")


def _verify_bindings(bindings: dict) -> tuple[dict[str, bytes], list[str]]:
    if (
        bindings["policy_sha256"] != POLICY_SHA256
        or bindings["reader_state_cid"] != READER_STATE_CID
        or bindings["core_state_cid"] != CORE_STATE_CID
        or bindings["frame_tree_cid"] != FRAME_TREE_CID
        or bindings["runtime"] != _EXPECTED_RUNTIME
        or set(bindings["assets"]) != _ASSET_NAMES
    ):
        raise ValueError("bindings differ from fixed policy, states, frames or runtime")
    source_files = bindings["source_files"]
    if type(source_files) is not list or not source_files:
        raise ValueError("source closure is absent")
    source_paths = [str(Path(record["path"]).resolve()) for record in source_files]
    if len(set(source_paths)) != len(source_paths):
        raise ValueError("duplicate file in source closure")
    package = Path(__file__).resolve().parent
    if not {
        str(package / "worker.py"),
        str(package / "adapter.py"),
        str(package / "__init__.py"),
    }.issubset(source_paths):
        raise ValueError("source closure omits the executing adapter or worker")
    for record in source_files:
        _bound_file(record)
    payloads = {
        name: _bound_file(bindings["assets"][name]) for name in sorted(_ASSET_NAMES)
    }
    for name, expected in (
        ("reader", READER_FILE_CID),
        ("core", CORE_FILE_CID),
        ("vocabulary", VOCABULARY_FILE_CID),
    ):
        if _cid(payloads[name]) != expected:
            raise ValueError(f"{name} is not the accepted file")

    frame_paths = {
        name: Path(bindings["assets"][name]["path"])
        for name in ("h4_frames", "token_frames")
    }
    if (
        frame_paths["h4_frames"].name != "h4-frames.json"
        or frame_paths["token_frames"].name != "token-frames.json"
        or frame_paths["h4_frames"].parent != frame_paths["token_frames"].parent
    ):
        raise ValueError("frame files must be the two fixed siblings")
    frame_records = sorted(
        [
            {
                "path": frame_paths[name].name,
                "bytes": len(payloads[name]),
                "cid": _cid(payloads[name]),
            }
            for name in ("h4_frames", "token_frames")
        ],
        key=lambda record: record["path"],
    )
    if _cid(_canonical(frame_records)) != FRAME_TREE_CID:
        raise ValueError("frame tree identity differs")

    vocabulary = _json(payloads["vocabulary"])
    core_vocabulary = vocabulary["core_vocabulary"]
    expected_core = list(READER_PREFIX[:52]) + [
        f"<unused-{index:04d}>" for index in range(52, 4096)
    ]
    expected_reader = list(READER_PREFIX) + expected_core[58:]
    if (
        core_vocabulary != expected_core
        or vocabulary["vocabulary"] != expected_reader
        or vocabulary["padding_id"] != 57
    ):
        raise ValueError("core vocabulary or reader lexical aliases differ")

    source = bindings["core_model"]["source"]
    core_record = source["model"]
    core_path = (Path(source["root"]) / core_record["path"]).resolve()
    if (
        core_path != Path(bindings["assets"]["core"]["path"]).resolve()
        or core_record["bytes"] != len(payloads["core"])
        or core_record["cid"] != CORE_FILE_CID
        or core_record["state_cid"] != CORE_STATE_CID
    ):
        raise ValueError("compact core model metadata identifies another asset")
    return payloads, core_vocabulary


def _configure_runtime() -> dict:
    os.environ["CUDA_VISIBLE_DEVICES"] = ""
    os.environ["PYTORCH_ENABLE_MPS_FALLBACK"] = "0"
    os.environ["OMP_NUM_THREADS"] = "4"
    os.environ["VECLIB_MAXIMUM_THREADS"] = "4"
    import torch

    torch.set_num_threads(4)
    if torch.get_num_interop_threads() != 1:
        torch.set_num_interop_threads(1)
    torch.use_deterministic_algorithms(True)
    runtime = {
        "python": platform.python_version(),
        "torch": str(torch.__version__),
        "device": "cpu",
        "threads": torch.get_num_threads(),
        "interop_threads": torch.get_num_interop_threads(),
        "workers": 1,
        "blas": "accelerate"
        if "BLAS_INFO=accelerate" in torch.__config__.show()
        else "other",
    }
    if (
        runtime != _EXPECTED_RUNTIME
        or sys.byteorder != "little"
        or not torch.are_deterministic_algorithms_enabled()
    ):
        raise ValueError("runtime does not reproduce the fixed deterministic CPU plan")
    return runtime


class Worker:
    def __init__(self, started: float) -> None:
        self.started = started
        self.bindings_sha256 = None
        self.runtime = None
        self.states_before = None
        self.model_loads = 0
        self.row_forwards = 0
        self.batch_forwards = 0
        self.refusal_rows = 0
        self.isolation_denied = False
        self.inference = None

    def resources(self) -> dict:
        return {
            "elapsed_seconds": time.monotonic() - self.started,
            "peak_rss_bytes": _peak_rss_bytes(),
        }

    def check_resources(self) -> None:
        observation = self.resources()
        if (
            observation["elapsed_seconds"] > MAX_SECONDS
            or observation["peak_rss_bytes"] > MAX_RSS_BYTES
        ):
            raise WorkerFailure("INCOMPLETE_RESOURCE", "worker time or RSS cap exceeded")

    def counts(self) -> dict:
        return {
            "model_loads": self.model_loads,
            "row_forwards": self.row_forwards,
            "batch_forwards": self.batch_forwards,
        }

    def states(self) -> dict:
        from ..zoology_release.development import _tensor_mapping_cid

        return {
            "reader": _tensor_mapping_cid(self.inference.reader.state_dict()),
            "core": _tensor_mapping_cid(
                {
                    name: value
                    for name, value in self.inference.core.state_dict().items()
                    if name != "lm_head.weight"
                }
            ),
        }

    def load(self, bindings: dict, payloads: dict[str, bytes]) -> None:
        from safetensors.torch import load as load_safetensors
        from ..zoology_compound_binding.model import (
            MODEL_CONFIG,
            MODEL_POLICY,
            CompoundBindingConfig,
            CompoundBindingModel,
        )
        from ..zoology_language_interface.model import (
            LanguageInterfaceModel,
            LearnedRoleReader,
        )
        from ..zoology_language_r4.attention import R4LanguageInterfaceInference
        from ..zoology_r4_inference.frames import load_frames
        from ..zoology_release.development import _tensor_mapping_cid

        core_record = bindings["core_model"]["source"]["model"]
        if core_record["config"] != MODEL_CONFIG or core_record["model_policy"] != MODEL_POLICY:
            raise ValueError("fixed core configuration or execution policy differs")
        core_state = load_safetensors(payloads["core"])
        reader_state = load_safetensors(payloads["reader"])
        if (
            _tensor_mapping_cid(core_state) != CORE_STATE_CID
            or _tensor_mapping_cid(reader_state) != READER_STATE_CID
        ):
            raise ValueError("serialized reader/core tensor identity differs")
        core = CompoundBindingModel(CompoundBindingConfig(**core_record["config"]))
        missing, unexpected = core.load_state_dict(core_state, strict=False)
        if missing != ["lm_head.weight"] or unexpected:
            raise ValueError("core must omit exactly the tied lm_head.weight")
        self.model_loads += 1
        reader = LearnedRoleReader()
        reader.load_state_dict(reader_state, strict=True)
        self.model_loads += 1
        model = LanguageInterfaceModel(core, reader)
        model.eval().requires_grad_(False)
        if (
            core.parameter_count() != 286976
            or reader.parameter_count() != 141571
            or core.lm_head.weight is not core.embedding.weight
        ):
            raise ValueError("fixed parameter counts or tied core head differ")
        frames = load_frames(Path(bindings["assets"]["h4_frames"]["path"]).parent)
        if (
            frames.frame_file_cid != _cid(payloads["h4_frames"])
            or frames.file_cid != _cid(payloads["token_frames"])
        ):
            raise ValueError("loaded frame bytes differ from verified assets")
        self.inference = R4LanguageInterfaceInference(model, frames, execution="r4")
        self.states_before = self.states()
        if self.states_before != {"reader": READER_STATE_CID, "core": CORE_STATE_CID}:
            raise ValueError("loaded model state differs from accepted reader/core")

    def ready(self, *, readiness_only: bool) -> None:
        _emit(
            {
                "event": "ready",
                "status": "ARTIFACTS_READY" if readiness_only else "MODEL_READY",
                "bindings_sha256": self.bindings_sha256,
                "runtime": self.runtime,
                "deterministic_algorithms": True,
                "isolation_denied": self.isolation_denied,
                "states": self.states_before,
                **self.counts(),
                **self.resources(),
            }
        )

    def batch(self, packet: dict, arm: str, core_vocabulary: list[str]) -> None:
        import torch

        if set(packet) != {"records"} or type(packet["records"]) is not list:
            raise WorkerFailure("UNAVAILABLE_COMPARISON_INPUT", "invalid packet fields")
        records = packet["records"]
        if not 1 <= len(records) <= 128:
            raise WorkerFailure("UNAVAILABLE_COMPARISON_INPUT", "packet must contain 1..128 rows")
        receipts, valid_indices, inputs, lengths = [], [], [], []
        for index, record in enumerate(records):
            if arm == "adapter":
                receipt = _adapter_record(record)
            else:
                if type(record) is not dict or set(record) != {"inputs", "lengths"}:
                    raise WorkerFailure("UNAVAILABLE_COMPARISON_INPUT", "oracle has extra or missing fields")
                try:
                    derived_input_sha256(record["inputs"], record["lengths"])
                except (ValueError, TypeError, IndexError) as error:
                    raise WorkerFailure("UNAVAILABLE_COMPARISON_INPUT", "invalid oracle tensor shape or values") from error
                receipt = {"status": "ORACLE"}
            receipts.append(receipt)
            if receipt["status"] in ("SEGMENTED", "ORACLE"):
                valid_indices.append(index)
                value = receipt if arm == "adapter" else record
                inputs.append(value["inputs"][0])
                lengths.append(value["lengths"][0])
            else:
                self.refusal_rows += 1
        self.check_resources()
        tensors, model_tokens = {}, []
        if valid_indices:
            if self.row_forwards + len(valid_indices) > MAX_LOGICAL_ROWS:
                raise WorkerFailure("INCOMPLETE_RESOURCE", "fixed per-arm logical-row cap would be exceeded")
            input_tensor = torch.tensor(inputs, dtype=torch.long, device="cpu")
            length_tensor = torch.tensor(lengths, dtype=torch.long, device="cpu")
            # Counters record attempted model work, including a forward that fails.
            self.row_forwards += len(valid_indices)
            self.batch_forwards += 1
            with torch.inference_mode():
                output = self.inference(input_tensor, length_tensor)
                predictions = output["logits"].argmax(dim=-1)
                role_positions = output["role_attention"].argmax(dim=-1)
            self.check_resources()
            tensors = {
                name: _tensor_record(tensor)
                for name, tensor in {
                    "inputs": input_tensor,
                    "lengths": length_tensor,
                    "role_attention": output["role_attention"],
                    "role_vectors": output["role_vectors"],
                    "binding_attention": output["binding_attention"],
                    "logits": output["logits"],
                    "predictions": predictions,
                    "role_positions": role_positions,
                }.items()
            }
            for valid_row, token_id in enumerate(predictions.tolist()):
                receipt = receipts[valid_indices[valid_row]]
                model_tokens.append(
                    {
                        "schema": "uor-r4.text-binding-result/1" if arm == "adapter"
                        else "uor-r4.oracle-binding-diagnostic/1",
                        "status": "MODEL_TOKEN" if arm == "adapter" else "ORACLE_TOKEN",
                        "policy_sha256": POLICY_SHA256,
                        "raw_text_sha256": receipt.get("raw_text_sha256"),
                        "derived_input_sha256": receipt.get("derived_input_sha256")
                        if arm == "adapter"
                        else derived_input_sha256([inputs[valid_row]], [lengths[valid_row]]),
                        "reader_file_cid": READER_FILE_CID,
                        "core_file_cid": CORE_FILE_CID,
                        "frame_tree_cid": FRAME_TREE_CID,
                        "token_id": token_id,
                        "token": core_vocabulary[token_id],
                    }
                )
        self.check_resources()
        _emit(
            {
                "event": "batch",
                "bindings_sha256": self.bindings_sha256,
                "receipts": receipts,
                "valid_indices": valid_indices,
                "tensors": tensors,
                "model_tokens": model_tokens,
                "row_forwards": len(valid_indices),
                "batch_forwards": int(bool(valid_indices)),
                "cumulative_row_forwards": self.row_forwards,
                "cumulative_batch_forwards": self.batch_forwards,
            }
        )

    def done(self) -> None:
        states_after = self.states() if self.inference is not None else None
        if states_after != self.states_before:
            raise WorkerFailure("UNAVAILABLE_REFERENCE_REPLAY", "inference mutated model state")
        self.check_resources()
        audit = dict(self.inference.audit) if self.inference is not None else {}
        audit.update(
            {
                "refusal_rows": self.refusal_rows,
                "refusal_row_forwards": 0,
                "optimizer_updates": 0,
                "oracle_or_label_file_reads": 0,
                "isolation_denied": self.isolation_denied,
            }
        )
        _emit(
            {
                "event": "done",
                "bindings_sha256": self.bindings_sha256,
                "runtime": self.runtime,
                "deterministic_algorithms": True,
                "states_before": self.states_before,
                "states_after": states_after,
                **self.counts(),
                **self.resources(),
                "audit": audit,
            }
        )


def _adapter_record(record: object) -> dict:
    if (
        type(record) is not dict
        or set(record) != {"schema", "text_base64", "request_extras"}
        or type(record["text_base64"]) is not str
        or type(record["request_extras"]) is not dict
    ):
        raise WorkerFailure("UNAVAILABLE_COMPARISON_INPUT", "invalid raw request transport")
    try:
        raw = base64.b64decode(record["text_base64"], validate=True)
    except (ValueError, binascii.Error) as error:
        raise WorkerFailure("UNAVAILABLE_COMPARISON_INPUT", "invalid raw request base64") from error
    request = {"schema": record["schema"], "text": raw}
    request.update(record["request_extras"])
    return segment_request(request)


def _tensor_record(tensor) -> dict:
    import torch

    if tensor.device.type != "cpu" or tensor.dtype not in (torch.float32, torch.int64):
        raise WorkerFailure("UNAVAILABLE_REFERENCE_REPLAY", "unexpected output tensor device or dtype")
    contiguous = tensor.detach().contiguous()
    return {
        "dtype": "float32" if tensor.dtype == torch.float32 else "int64",
        "shape": list(contiguous.shape),
        "data_base64": base64.b64encode(contiguous.numpy().tobytes(order="C")).decode("ascii"),
    }


def _alarm(_signal_number, _frame) -> None:
    raise WorkerFailure("INCOMPLETE_RESOURCE", "worker wall-clock cap exceeded")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bindings", required=True, type=Path)
    parser.add_argument("--arm", choices=("adapter", "oracle"), required=True)
    parser.add_argument("--readiness-only", action="store_true")
    args = parser.parse_args()
    worker = Worker(time.monotonic())
    signal.signal(signal.SIGALRM, _alarm)
    signal.setitimer(signal.ITIMER_REAL, MAX_SECONDS)
    startup = True
    try:
        raw_bindings = args.bindings.read_bytes()
        worker.bindings_sha256 = hashlib.sha256(raw_bindings).hexdigest()
        bindings = _json(raw_bindings)
        payloads, vocabulary = _verify_bindings(bindings)
        worker.isolation_denied = _isolation_probe()
        worker.runtime = _configure_runtime()
        worker.check_resources()
        if not args.readiness_only:
            worker.load(bindings, payloads)
        worker.check_resources()
        worker.ready(readiness_only=args.readiness_only)
        startup = False
        if not args.readiness_only:
            while raw_packet := sys.stdin.buffer.readline(MAX_PACKET_BYTES + 1):
                worker.check_resources()
                if len(raw_packet) > MAX_PACKET_BYTES:
                    raise WorkerFailure("UNAVAILABLE_COMPARISON_INPUT", "packet byte limit exceeded")
                worker.batch(_json(raw_packet), args.arm, vocabulary)
        worker.done()
        return 0
    except Exception as error:
        status = (
            error.status
            if isinstance(error, WorkerFailure)
            else "UNAVAILABLE_ARTIFACT" if startup else "UNAVAILABLE_REFERENCE_REPLAY"
        )
        _emit(
            {
                "event": "error",
                "status": status,
                "reason": str(error),
                "bindings_sha256": worker.bindings_sha256,
                "request_refusal": unavailable_artifact() if startup else None,
                "runtime": worker.runtime,
                "states_before": worker.states_before,
                "rows_done": worker.row_forwards,
                **worker.counts(),
                **worker.resources(),
            }
        )
        return 1
    finally:
        signal.setitimer(signal.ITIMER_REAL, 0)


if __name__ == "__main__":
    raise SystemExit(main())
