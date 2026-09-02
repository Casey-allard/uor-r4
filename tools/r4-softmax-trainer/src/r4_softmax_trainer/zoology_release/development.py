# SPDX-License-Identifier: Apache-2.0
"""CPU reproduction of Zoology's released width-64 attention positive (#1050).

The executable authority is HazyResearch/Zoology commit ``de4e258``.  This
module freezes its Figure-2 T=64 attention configuration, data, optimizer,
cosine schedule, DataLoader shuffling, and strict ``valid/accuracy > 0.99``
early stop.  The two integration exceptions inherited from the credited #1047
port are CPU placement and query-only tied-head projection; C0 checks the
source port before any long arm is admitted.
"""

from __future__ import annotations

import json
import math
import multiprocessing as mp
import os
import platform
import queue as queue_module
import resource
import time
import traceback
from collections.abc import Mapping, Sequence
from dataclasses import asdict
from pathlib import Path
from typing import Any

import torch
import numpy as np
from blake3 import blake3
from safetensors.torch import load as load_safetensors
from safetensors.torch import save as save_safetensors
from torch import Tensor
from torch.nn import functional as F
from torch.utils.data import DataLoader, TensorDataset

from ..provenance import canonical_json_bytes, cid_bytes
from ..zoology_control import data as source_data
from ..zoology_control import development as control_development
from ..zoology_control.model import (
    SOURCE_COMMIT,
    ZoologyFigure2Config,
    ZoologyFigure2Model,
    set_zoology_seed,
)
from ..zoology_control.provenance import zoology_source_attribution


ISSUE = 1050
POLICY = "ZoologyFigure2ReleaseReproductionV1"
SOURCE_MODEL_SEED = 123
VOCAB_SIZE = 8_192
INPUT_SEQ_LEN = 64
NUM_KV_PAIRS = 4
TRAIN_ROWS = 100_000
TEST_ROWS = 3_000
TRAIN_SEED = 0
TEST_SEED = 10
BATCH_SIZE = 512
MAXIMUM_EPOCHS = 64
WEIGHT_DECAY = 0.1
EARLY_STOP_THRESHOLD = 0.99

# Source np.logspace(-4, -2, 4), reordered only as frozen by issue #1050:
# run #1049's already-bound rate first, then the remaining source rates.
SOURCE_LEARNING_RATES = tuple(float(value) for value in np.logspace(-4, -2, 4))
LEARNING_RATE_SOURCE_INDICES = (1, 0, 2, 3)
LEARNING_RATES = tuple(
    SOURCE_LEARNING_RATES[index] for index in LEARNING_RATE_SOURCE_INDICES
)
LEARNING_RATE_KEYS = (
    "4p6415888336127773e-4",
    "1e-4",
    "2p154434690031882e-3",
    "1e-2",
)

ELIGIBLE_THREADS = (1, 4, 8)
TIMED_TRAINING_BATCHES = 16
TIMING_REPEATS = 2
PROJECTION_SAFETY_FACTOR = 1.25
ARM_HARD_WALL_SECONDS = 3_000.0
MEMORY_CEILING_BYTES = 8 * 1024**3
PROBE_TIMEOUT_SECONDS = 300.0

EXPECTED_1049_RESULT_CID = (
    "blake3:9b36540d81d0967a3f7e2ccabed80900d31c904b6c747d9ba0d539b325b13373"
)
EXPECTED_1049_VERDICT = "SCALED_SOURCE_CALIBRATION_MISS"

PREPARATION_RELATIVE_PATH = "zoology-release-preparation.json"
DATA_RELATIVE_PATH = "data/zoology-figure2-t64.safetensors"
PREFLIGHT_STARTED_RELATIVE_PATH = "preflight/zoology-release-started.json"
PREFLIGHT_RELATIVE_PATH = "preflight/zoology-release-preflight.json"
RUN_STARTED_RELATIVE_PATH = "run/zoology-release-started.json"
RESULT_RELATIVE_PATH = "run/zoology-release-result.json"
PREDECESSOR_RESULT_RELATIVE_PATH = "run/zoology-control-result.json"

PREPARATION_SCHEMA = "uor-r4.zoology-release-preparation/1"
PREFLIGHT_SCHEMA = "uor-r4.zoology-release-preflight/1"
STARTED_SCHEMA = "uor-r4.zoology-release-started/1"
ARM_SCHEMA = "uor-r4.zoology-release-arm/1"
RESULT_SCHEMA = "uor-r4.zoology-release-result/1"


def _with_cid(value: Mapping[str, Any], field: str) -> dict[str, Any]:
    result = dict(value)
    if field in result:
        raise ValueError(f"{field} already exists")
    result[field] = cid_bytes(canonical_json_bytes(value))
    return result


def _verify_self_cid(value: Mapping[str, Any], field: str) -> None:
    unsigned = dict(value)
    observed = unsigned.pop(field, None)
    if observed != cid_bytes(canonical_json_bytes(unsigned)):
        raise ValueError(f"{field} does not reproduce")


def _read_json(path: Path, *, cid_field: str) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise ValueError(f"cannot read {path}: {error}") from error
    if not isinstance(value, dict):
        raise ValueError(f"{path.name} is not a JSON object")
    _verify_self_cid(value, cid_field)
    return value


def _write_exclusive(path: Path, payload: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o644)
    try:
        with os.fdopen(descriptor, "wb") as output:
            output.write(payload)
            output.flush()
            os.fsync(output.fileno())
    except BaseException:
        try:
            path.unlink()
        except OSError:
            pass
        raise


def _write_exclusive_json(path: Path, value: Mapping[str, Any]) -> None:
    _write_exclusive(path, canonical_json_bytes(value))


def _atomic_json(path: Path, value: Mapping[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_suffix(path.suffix + ".tmp")
    payload = canonical_json_bytes(value)
    with temporary.open("wb") as output:
        output.write(payload)
        output.flush()
        os.fsync(output.fileno())
    os.replace(temporary, path)


def _file_record(path: Path, *, relative_path: str) -> dict[str, Any]:
    payload = path.read_bytes()
    return {
        "path": relative_path,
        "source_path": str(path),
        "bytes": len(payload),
        "file_cid": cid_bytes(payload),
    }


def _tensor_mapping_cid(tensors: Mapping[str, Tensor]) -> str:
    digest = blake3()
    for name in sorted(tensors):
        tensor = tensors[name].detach().cpu().contiguous()
        digest.update(
            canonical_json_bytes(
                {"name": name, "shape": list(tensor.shape), "dtype": str(tensor.dtype)}
            )
        )
        digest.update(tensor.numpy().tobytes(order="C"))
    return f"blake3:{digest.hexdigest()}"


def _canonical_safetensors(tensors: Mapping[str, Tensor]) -> bytes:
    """Emit deterministic tensor bytes; provenance lives in the JSON envelope."""

    ordered = {
        name: tensors[name].detach().cpu().contiguous() for name in sorted(tensors)
    }
    return save_safetensors(ordered)


def _implementation_contract() -> dict[str, Any]:
    trainer_root = Path(__file__).resolve().parents[3]
    package_root = Path(__file__).resolve().parent
    paths = sorted(package_root.glob("*.py"))
    paths.extend(
        trainer_root / relative
        for relative in (
            "src/r4_softmax_trainer/zoology_control/data.py",
            "src/r4_softmax_trainer/zoology_control/model.py",
            "src/r4_softmax_trainer/zoology_control/provenance.py",
            "src/r4_softmax_trainer/zoology_control/NOTICE.md",
            "src/r4_softmax_trainer/zoology_control/LICENSE-APACHE-2.0.md",
        )
    )
    paths.extend(sorted((trainer_root / "tests").glob("test_zoology_release*.py")))
    records: list[dict[str, Any]] = []
    digest = blake3()
    for path in sorted(set(paths)):
        payload = path.read_bytes()
        relative = str(path.relative_to(trainer_root))
        record = {"path": relative, "bytes": len(payload), "cid": cid_bytes(payload)}
        records.append(record)
        digest.update(canonical_json_bytes(record))
    body = {
        "schema": "uor-r4/zoology-release-implementation/v1",
        "issue": ISSUE,
        "policy": POLICY,
        "source_commit": SOURCE_COMMIT,
        "files": records,
        "tree_cid": f"blake3:{digest.hexdigest()}",
    }
    return _with_cid(body, "implementation_cid")


def _learning_rate_contract() -> dict[str, Any]:
    expected = (
        0.0001,
        0.00046415888336127773,
        0.002154434690031882,
        0.01,
    )
    if SOURCE_LEARNING_RATES != expected:
        raise ValueError("locked NumPy no longer reproduces the source learning rates")
    source = [
        {
            "source_index": index,
            "decimal": repr(value),
            "float_hex": value.hex(),
        }
        for index, value in enumerate(SOURCE_LEARNING_RATES)
    ]
    return {
        "source_expression": "numpy.logspace(-4,-2,4)",
        "source_order": source,
        "execution_source_indices": list(LEARNING_RATE_SOURCE_INDICES),
        "execution_order": [source[index] for index in LEARNING_RATE_SOURCE_INDICES],
    }


def _source_pass(rate: float) -> bool:
    """Apply the executable source's strict early-stop comparison."""

    return rate > EARLY_STOP_THRESHOLD


def _bind_predecessor(predecessor_root: Path) -> dict[str, Any]:
    path = predecessor_root / PREDECESSOR_RESULT_RELATIVE_PATH
    result = _read_json(path, cid_field="result_cid")
    decision = result.get("decision")
    if (
        result.get("result_cid") != EXPECTED_1049_RESULT_CID
        or not isinstance(decision, Mapping)
        or decision.get("verdict") != EXPECTED_1049_VERDICT
    ):
        raise ValueError("#1049 predecessor differs from the frozen result")
    return {
        **_file_record(path, relative_path=PREDECESSOR_RESULT_RELATIVE_PATH),
        "result_cid": result["result_cid"],
        "verdict": decision["verdict"],
        "fitted_artifact_access": "FORBIDDEN_NOT_READ",
    }


def _source_attribution_record() -> dict[str, Any]:
    """Wrap #1047's credited source inventory without inheriting its run policy."""

    credited = zoology_source_attribution()
    return {
        "credited_inventory": credited,
        "credited_inventory_cid": credited["attribution_cid"],
        "reuse_scope": "UPSTREAM_FILE_CIDS_AND_LICENSE_ONLY",
        "current_execution": "EXACT_FIGURE2_T64_CONFIGURATION_AND_DATALOADER_SEMANTICS",
        "current_integration_exceptions": [
            "CPU placement instead of the source's CUDA-only fit line",
            "query-only tied-head projection equivalent under ignore_index",
        ],
        "not_inherited_from_1047": [
            "8,192/1,024 reduced population",
            "batch 64",
            "BLAKE3-seeded independent epoch shuffle",
            "two-consecutive train/development qualification",
            "C2 and binding-permuted controls",
        ],
    }


def _selected(labels: Tensor) -> tuple[Tensor, Tensor]:
    if labels.ndim != 2:
        raise ValueError("released labels must be [rows,time]")
    active = labels.ne(-100)
    if not bool(torch.all(active.sum(dim=1) == NUM_KV_PAIRS)):
        raise ValueError("released labels do not contain four queries per row")
    positions = active.nonzero(as_tuple=False)[:, 1].reshape(
        labels.shape[0], NUM_KV_PAIRS
    )
    targets = torch.gather(labels, 1, positions)
    return positions.contiguous(), targets.contiguous()


def _build_dataset_payload() -> tuple[bytes, dict[str, Any]]:
    train_inputs, train_labels = source_data._released_mqar(
        vocab_size=VOCAB_SIZE,
        num_examples=TRAIN_ROWS,
        input_seq_len=INPUT_SEQ_LEN,
        seed=TRAIN_SEED,
        num_kv_pairs=NUM_KV_PAIRS,
    )
    test_inputs, test_labels = source_data._released_mqar(
        vocab_size=VOCAB_SIZE,
        num_examples=TEST_ROWS,
        input_seq_len=INPUT_SEQ_LEN,
        seed=TEST_SEED,
        num_kv_pairs=NUM_KV_PAIRS,
    )
    train_positions, train_targets = _selected(train_labels)
    test_positions, test_targets = _selected(test_labels)
    tensors = {
        "train_inputs": train_inputs.contiguous(),
        "train_positions": train_positions,
        "train_targets": train_targets,
        "test_inputs": test_inputs.contiguous(),
        "test_positions": test_positions,
        "test_targets": test_targets,
    }
    tensor_cid = _tensor_mapping_cid(tensors)
    metadata = {
        "schema": "uor-r4.zoology-figure2-data/1",
        "source_commit": SOURCE_COMMIT,
        "tensor_cid": tensor_cid,
        "config": canonical_json_bytes(
            {
                "vocab_size": VOCAB_SIZE,
                "input_seq_len": INPUT_SEQ_LEN,
                "num_kv_pairs": NUM_KV_PAIRS,
                "train_rows": TRAIN_ROWS,
                "test_rows": TEST_ROWS,
                "train_seed": TRAIN_SEED,
                "test_seed": TEST_SEED,
                "power_a": source_data.SOURCE_NATIVE_POWER_A,
                "random_non_queries": False,
            }
        ).decode("utf-8"),
    }
    payload = _canonical_safetensors(tensors)
    return payload, {
        **metadata,
        "tensor_shapes": {k: list(v.shape) for k, v in tensors.items()},
    }


def _load_dataset(path: Path) -> dict[str, Tensor]:
    tensors = {
        name: tensor.contiguous()
        for name, tensor in load_safetensors(path.read_bytes()).items()
    }
    expected_shapes = {
        "train_inputs": [TRAIN_ROWS, INPUT_SEQ_LEN],
        "train_positions": [TRAIN_ROWS, NUM_KV_PAIRS],
        "train_targets": [TRAIN_ROWS, NUM_KV_PAIRS],
        "test_inputs": [TEST_ROWS, INPUT_SEQ_LEN],
        "test_positions": [TEST_ROWS, NUM_KV_PAIRS],
        "test_targets": [TEST_ROWS, NUM_KV_PAIRS],
    }
    if set(tensors) != set(expected_shapes) or any(
        list(tensors[name].shape) != shape or tensors[name].dtype != torch.long
        for name, shape in expected_shapes.items()
    ):
        raise ValueError("released dataset tensor contract differs")
    return tensors


def _dataset_record(root: Path, preparation: Mapping[str, Any]) -> dict[str, Any]:
    path = root / DATA_RELATIVE_PATH
    record = _file_record(path, relative_path=DATA_RELATIVE_PATH)
    expected = preparation.get("dataset")
    if not isinstance(expected, Mapping) or any(
        record.get(field) != expected.get(field)
        for field in ("path", "bytes", "file_cid")
    ):
        raise ValueError("released dataset file changed")
    tensors = _load_dataset(path)
    if _tensor_mapping_cid(tensors) != expected.get("tensor_cid"):
        raise ValueError("released dataset tensors changed")
    return dict(expected)


def prepare_release_reproduction(
    root: Path,
    *,
    predecessor_root: Path,
) -> dict[str, Any]:
    """Create and bind the exact 100k/3k released population once."""

    root = root.resolve()
    predecessor_root = predecessor_root.resolve()
    if not predecessor_root.is_dir():
        raise FileNotFoundError("#1049 predecessor root is absent")
    if root == predecessor_root:
        raise ValueError("run and predecessor roots must differ")
    path = root / PREPARATION_RELATIVE_PATH
    implementation = _implementation_contract()
    predecessor = _bind_predecessor(predecessor_root)
    attribution = _source_attribution_record()
    if path.exists():
        preparation = _read_json(path, cid_field="preparation_cid")
        if (
            preparation.get("implementation") != implementation
            or preparation.get("predecessor_root") != str(predecessor_root)
            or preparation.get("predecessor") != predecessor
            or preparation.get("source_attribution") != attribution
        ):
            raise ValueError("cached #1050 preparation no longer reproduces")
        _dataset_record(root, preparation)
        return preparation
    if root.exists() and any(root.iterdir()):
        raise FileExistsError("#1050 preparation requires an empty run root")

    payload, dataset_metadata = _build_dataset_payload()
    dataset_path = root / DATA_RELATIVE_PATH
    _write_exclusive(dataset_path, payload)
    dataset = {
        "path": DATA_RELATIVE_PATH,
        "bytes": len(payload),
        "file_cid": cid_bytes(payload),
        **dataset_metadata,
    }
    body = {
        "schema": PREPARATION_SCHEMA,
        "issue": ISSUE,
        "policy": POLICY,
        "predecessor_root": str(predecessor_root),
        "predecessor": predecessor,
        "implementation": implementation,
        "source_attribution": attribution,
        "source_contract": {
            "authority": f"HazyResearch/Zoology@{SOURCE_COMMIT}",
            "vocab_size": VOCAB_SIZE,
            "input_seq_len": INPUT_SEQ_LEN,
            "num_kv_pairs": NUM_KV_PAIRS,
            "train_rows": TRAIN_ROWS,
            "test_rows": TEST_ROWS,
            "train_seed": TRAIN_SEED,
            "test_seed": TEST_SEED,
            "batch_size": BATCH_SIZE,
            "d_model": 64,
            "n_layers": 2,
            "num_heads": 1,
            "state_mixer": "Identity",
            "model_seed": SOURCE_MODEL_SEED,
            "weight_decay": WEIGHT_DECAY,
            "schedule": "CosineAnnealingLR(epoch,to_zero)",
            "maximum_epochs": MAXIMUM_EPOCHS,
            "early_stop": "test_top1_strictly_greater_than_0.99",
            "learning_rates": _learning_rate_contract(),
        },
        "integration_exceptions": {
            "device": "CPU_INSTEAD_OF_SOURCE_CUDA",
            "vocabulary_projection": "QUERY_ONLY_MATHEMATICALLY_EQUIVALENT_TO_IGNORE_INDEX",
        },
        "dataset": dataset,
        "forbidden_inputs": {
            "uor_bytes": "NOT_READ",
            "r4_geometry": "NOT_READ",
            "roles": "NOT_READ",
            "teacher_provider_ollama_gemma": "NOT_CALLED",
            "cuda": "FORBIDDEN",
            "mps": "FORBIDDEN",
        },
        "mod256_boundary": {
            "later_discrete_lowering": "IN_SCOPE_AFTER_POSITIVE",
            "softmax_probability_normalization": "REAL_VALUED_NOT_MOD256",
        },
    }
    preparation = _with_cid(body, "preparation_cid")
    _write_exclusive_json(path, preparation)
    return preparation


def _configure_cpu(threads: int) -> torch.device:
    if threads not in ELIGIBLE_THREADS:
        raise ValueError("CPU threads are outside the frozen plans")
    os.environ["CUDA_VISIBLE_DEVICES"] = ""
    os.environ["PYTORCH_ENABLE_MPS_FALLBACK"] = "0"
    os.environ["OMP_NUM_THREADS"] = str(threads)
    os.environ["VECLIB_MAXIMUM_THREADS"] = str(threads)
    torch.set_num_threads(threads)
    if torch.get_num_interop_threads() != 1:
        try:
            torch.set_num_interop_threads(1)
        except RuntimeError as error:
            if torch.get_num_interop_threads() != 1:
                raise RuntimeError(
                    "CPU inter-op thread plan was initialized early"
                ) from error
    return torch.device("cpu")


def _new_model() -> ZoologyFigure2Model:
    return ZoologyFigure2Model(
        ZoologyFigure2Config(
            vocab_size=VOCAB_SIZE,
            d_model=64,
            n_layers=2,
            num_heads=1,
            max_position_embeddings=INPUT_SEQ_LEN,
            attention_dropout=0.1,
            embed_dropout=0.1,
            resid_dropout=0.0,
        )
    )


def _loaders(tensors: Mapping[str, Tensor]) -> tuple[DataLoader[Any], DataLoader[Any]]:
    train = DataLoader(
        TensorDataset(
            tensors["train_inputs"],
            tensors["train_positions"],
            tensors["train_targets"],
        ),
        batch_size=BATCH_SIZE,
        num_workers=0,
        shuffle=True,
    )
    test = DataLoader(
        TensorDataset(
            tensors["test_inputs"],
            tensors["test_positions"],
            tensors["test_targets"],
        ),
        batch_size=BATCH_SIZE,
        num_workers=0,
        shuffle=True,
    )
    return train, test


def _train_batch(
    model: ZoologyFigure2Model,
    optimizer: torch.optim.Optimizer,
    batch: Sequence[Tensor],
) -> float:
    inputs, positions, targets = batch
    optimizer.zero_grad()
    output = model.forward_selected(inputs, positions, targets)
    if output.loss is None:
        raise RuntimeError("released training batch lacks query loss")
    output.loss.backward()
    optimizer.step()
    return float(output.loss.detach())


def _score_loader(
    model: ZoologyFigure2Model, loader: DataLoader[Any]
) -> dict[str, Any]:
    model.eval()
    correct = 0
    decisions = 0
    loss_sum = 0.0
    digest = blake3()
    with torch.inference_mode():
        for inputs, positions, targets in loader:
            output = model.forward_selected(inputs, positions, targets)
            flat_logits = output.logits.float().reshape(-1, VOCAB_SIZE)
            flat_targets = targets.reshape(-1)
            predictions = flat_logits.argmax(dim=-1)
            decisions += int(flat_targets.numel())
            correct += int(torch.count_nonzero(predictions == flat_targets))
            loss_sum += float(
                F.cross_entropy(flat_logits, flat_targets, reduction="sum")
            )
            cpu_logits = output.logits.detach().cpu().contiguous()
            digest.update(cpu_logits.numpy().tobytes(order="C"))
    if decisions != TEST_ROWS * NUM_KV_PAIRS:
        raise RuntimeError("released test decision count differs")
    return {
        "decisions": decisions,
        "top1_correct": correct,
        "top1_rate": correct / decisions,
        "nll_nats": loss_sum / decisions,
        "selected_logits_cid": f"blake3:{digest.hexdigest()}",
    }


def _peak_rss_bytes() -> int:
    peak = int(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss)
    return peak if platform.system() == "Darwin" else peak * 1024


def _probe_once(threads: int, data_path: Path) -> dict[str, Any]:
    _configure_cpu(threads)
    tensors = _load_dataset(data_path)
    repeat_seconds: list[float] = []
    losses: list[list[float]] = []
    evaluation_seconds: list[float] = []
    for _repeat in range(TIMING_REPEATS):
        set_zoology_seed(SOURCE_MODEL_SEED)
        model = _new_model()
        optimizer = torch.optim.AdamW(
            model.parameters(),
            lr=LEARNING_RATES[0],
            weight_decay=WEIGHT_DECAY,
        )
        train_loader, test_loader = _loaders(tensors)
        iterator = iter(train_loader)
        _train_batch(model, optimizer, next(iterator))
        model.train()
        began = time.monotonic()
        observed_losses: list[float] = []
        for _ in range(TIMED_TRAINING_BATCHES):
            observed_losses.append(_train_batch(model, optimizer, next(iterator)))
        repeat_seconds.append(time.monotonic() - began)
        losses.append(observed_losses)
        evaluation_began = time.monotonic()
        _score_loader(model, test_loader)
        evaluation_seconds.append(time.monotonic() - evaluation_began)

    seconds_per_training_batch = max(repeat_seconds) / TIMED_TRAINING_BATCHES
    seconds_per_test = max(evaluation_seconds)
    train_batches = math.ceil(TRAIN_ROWS / BATCH_SIZE)
    projected_before_safety = MAXIMUM_EPOCHS * (
        train_batches * seconds_per_training_batch + seconds_per_test
    )
    projected = projected_before_safety * PROJECTION_SAFETY_FACTOR
    stability_ratio = max(repeat_seconds) / min(repeat_seconds)
    return {
        "plan": {
            "name": f"cpu-{threads}t-b{BATCH_SIZE}",
            "device": "cpu",
            "threads": threads,
            "workers": 1,
            "batch_size": BATCH_SIZE,
            "cuda": "FORBIDDEN",
            "mps": "FORBIDDEN",
        },
        "timed_training_batches_per_repeat": TIMED_TRAINING_BATCHES,
        "timing_repeats": TIMING_REPEATS,
        "training_seconds": repeat_seconds,
        "training_losses": losses,
        "full_test_evaluation": True,
        "test_evaluation_seconds": evaluation_seconds,
        "stability_ratio": stability_ratio,
        "stable": stability_ratio <= 1.35,
        "projected_arm_seconds_before_safety": projected_before_safety,
        "projected_arm_seconds": projected,
        "projection_safety_factor": PROJECTION_SAFETY_FACTOR,
        "peak_rss_bytes": _peak_rss_bytes(),
    }


def _probe_worker(result_queue: Any, threads: int, data_path: str) -> None:
    try:
        result_queue.put({"ok": True, "record": _probe_once(threads, Path(data_path))})
    except BaseException as error:
        result_queue.put(
            {
                "ok": False,
                "error": {
                    "type": type(error).__name__,
                    "reason": str(error),
                    "traceback": traceback.format_exc(),
                },
            }
        )


def _spawn_probe(threads: int, data_path: Path) -> dict[str, Any]:
    context = mp.get_context("spawn")
    result_queue = context.Queue()
    process = context.Process(
        target=_probe_worker,
        args=(result_queue, threads, str(data_path)),
    )
    process.start()
    process.join(PROBE_TIMEOUT_SECONDS)
    if process.is_alive():
        process.terminate()
        process.join(timeout=10.0)
        return {
            "ok": False,
            "error": {"type": "TimeoutError", "reason": "probe timed out"},
        }
    try:
        return dict(result_queue.get(timeout=2.0))
    except queue_module.Empty:
        return {
            "ok": False,
            "error": {
                "type": "RuntimeError",
                "reason": "probe exited without evidence",
                "exitcode": process.exitcode,
            },
        }
    finally:
        result_queue.close()
        result_queue.join_thread()


def _started_record(
    *, phase: str, preparation: Mapping[str, Any], implementation: Mapping[str, Any]
) -> dict[str, Any]:
    return _with_cid(
        {
            "schema": STARTED_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            "phase": phase,
            "preparation_cid": preparation["preparation_cid"],
            "implementation_cid": implementation["implementation_cid"],
            "implementation_tree_cid": implementation["tree_cid"],
        },
        "started_cid",
    )


def _select_plan(records: Sequence[Mapping[str, Any]]) -> dict[str, Any]:
    if {int(record["plan"]["threads"]) for record in records} != set(ELIGIBLE_THREADS):
        raise ValueError("1/4/8-thread timing matrix is incomplete")
    eligible = [
        dict(record)
        for record in records
        if record.get("stable") is True
        and float(record.get("projected_arm_seconds", math.inf))
        <= ARM_HARD_WALL_SECONDS
        and int(record.get("peak_rss_bytes", MEMORY_CEILING_BYTES + 1))
        <= MEMORY_CEILING_BYTES
    ]
    selected = min(
        eligible,
        key=lambda record: (
            float(record["projected_arm_seconds"]),
            int(record["plan"]["threads"]),
        ),
        default=None,
    )
    return {
        "available": selected is not None,
        "plans": [dict(record) for record in records],
        "selected_plan": None if selected is None else selected["plan"],
        "selected_projection_seconds": (
            None if selected is None else selected["projected_arm_seconds"]
        ),
        "arm_hard_wall_seconds": ARM_HARD_WALL_SECONDS,
        "memory_ceiling_bytes": MEMORY_CEILING_BYTES,
    }


def preflight_release_reproduction(root: Path) -> dict[str, Any]:
    """Repeat source C0, then select the fastest stable batch-512 CPU plan."""

    root = root.resolve()
    path = root / PREFLIGHT_RELATIVE_PATH
    started_path = root / PREFLIGHT_STARTED_RELATIVE_PATH
    preparation = _read_json(
        root / PREPARATION_RELATIVE_PATH, cid_field="preparation_cid"
    )
    implementation = _implementation_contract()
    if preparation.get("implementation") != implementation:
        raise ValueError("#1050 implementation changed after preparation")
    _dataset_record(root, preparation)
    if path.exists():
        preflight = _read_json(path, cid_field="preflight_cid")
        if preflight.get("implementation") != implementation:
            raise ValueError("cached #1050 preflight no longer reproduces")
        return preflight
    if started_path.exists():
        raise FileExistsError("#1050 preflight already started")
    _write_exclusive_json(
        started_path,
        _started_record(
            phase="preflight", preparation=preparation, implementation=implementation
        ),
    )

    began = time.monotonic()
    _configure_cpu(4)
    c0 = control_development._run_c0(
        source_data.build_source_calibration(),
        device=torch.device("cpu"),
    )
    records: list[dict[str, Any]] = []
    if c0.get("passed") is True:
        for threads in ELIGIBLE_THREADS:
            envelope = _spawn_probe(threads, root / DATA_RELATIVE_PATH)
            record = envelope.get("record")
            if envelope.get("ok") is not True or not isinstance(record, Mapping):
                raise RuntimeError(
                    f"{threads}-thread probe failed: {envelope.get('error')}"
                )
            records.append(dict(record))
    selection = (
        _select_plan(records)
        if records
        else {
            "available": False,
            "plans": [],
            "selected_plan": None,
            "selected_projection_seconds": None,
            "arm_hard_wall_seconds": ARM_HARD_WALL_SECONDS,
            "memory_ceiling_bytes": MEMORY_CEILING_BYTES,
        }
    )
    body = {
        "schema": PREFLIGHT_SCHEMA,
        "issue": ISSUE,
        "policy": POLICY,
        "preparation_cid": preparation["preparation_cid"],
        "implementation": implementation,
        "dataset_cid": preparation["dataset"]["tensor_cid"],
        "c0": c0,
        "selection": selection,
        "elapsed_seconds": time.monotonic() - began,
        "passed": bool(c0.get("passed") is True and selection["available"]),
        "read_ledger": {
            "uor_byte_reads": 0,
            "r4_geometry_reads": 0,
            "role_reads": 0,
            "teacher_calls": 0,
            "provider_calls": 0,
            "future_value_reads": 0,
        },
        "cuda": "FORBIDDEN",
        "mps": "FORBIDDEN",
    }
    if implementation != _implementation_contract():
        raise ValueError("#1050 implementation changed during preflight")
    preflight = _with_cid(body, "preflight_cid")
    _write_exclusive_json(path, preflight)
    return preflight


def _artifact_payload(model: ZoologyFigure2Model, *, learning_rate: float) -> bytes:
    tensors = {
        name: tensor.detach().cpu().contiguous()
        for name, tensor in sorted(model.state_dict().items())
        if name != "lm_head.weight"
    }
    return _canonical_safetensors(tensors)


def _save_checkpoint(
    path: Path,
    *,
    model: ZoologyFigure2Model,
    optimizer: torch.optim.Optimizer,
    scheduler: torch.optim.lr_scheduler.CosineAnnealingLR,
    completed_epochs: int,
    history: Sequence[Mapping[str, Any]],
    elapsed_seconds: float,
) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_suffix(".tmp")
    torch.save(
        {
            "model": model.state_dict(),
            "optimizer": optimizer.state_dict(),
            "scheduler": scheduler.state_dict(),
            "torch_rng_state": torch.get_rng_state(),
            "completed_epochs": completed_epochs,
            "history": [dict(record) for record in history],
            "elapsed_seconds": elapsed_seconds,
        },
        temporary,
    )
    os.replace(temporary, path)


def _score_test(
    model: ZoologyFigure2Model, test_loader: DataLoader[Any]
) -> dict[str, Any]:
    return _score_loader(model, test_loader)


def _run_arm(
    root: Path,
    tensors: Mapping[str, Tensor],
    *,
    arm_index: int,
    learning_rate: float,
    threads: int,
) -> dict[str, Any]:
    key = LEARNING_RATE_KEYS[arm_index]
    arm_root = root / "arms" / f"{arm_index:02d}-{key}"
    result_path = arm_root / "result.json"
    progress_path = arm_root / "progress.json"
    checkpoint_path = arm_root / "checkpoint.pt"
    artifact_path = arm_root / "model.safetensors"
    if result_path.exists():
        return _read_json(result_path, cid_field="arm_cid")

    _configure_cpu(threads)
    set_zoology_seed(SOURCE_MODEL_SEED)
    model = _new_model()
    optimizer = torch.optim.AdamW(
        model.parameters(),
        lr=learning_rate,
        weight_decay=WEIGHT_DECAY,
    )
    scheduler = torch.optim.lr_scheduler.CosineAnnealingLR(
        optimizer,
        T_max=MAXIMUM_EPOCHS,
        eta_min=0.0,
    )
    train_loader, test_loader = _loaders(tensors)
    history: list[dict[str, Any]] = []
    completed_epochs = 0
    carried_elapsed = 0.0
    if checkpoint_path.exists():
        checkpoint = torch.load(checkpoint_path, map_location="cpu", weights_only=False)
        model.load_state_dict(checkpoint["model"])
        optimizer.load_state_dict(checkpoint["optimizer"])
        scheduler.load_state_dict(checkpoint["scheduler"])
        completed_epochs = int(checkpoint["completed_epochs"])
        history = [dict(record) for record in checkpoint["history"]]
        carried_elapsed = float(checkpoint["elapsed_seconds"])
        torch.set_rng_state(checkpoint["torch_rng_state"])

    began = time.monotonic()
    status = "COMPLETE_MISS"
    for epoch in range(completed_epochs, MAXIMUM_EPOCHS):
        if carried_elapsed + time.monotonic() - began >= ARM_HARD_WALL_SECONDS:
            status = "INCOMPLETE_HARD_WALL"
            break
        model.train()
        decisions = 0
        correct = 0
        loss_sum = 0.0
        epoch_began = time.monotonic()
        learning_rate_used = float(optimizer.param_groups[0]["lr"])
        for inputs, positions, targets in train_loader:
            optimizer.zero_grad()
            output = model.forward_selected(inputs, positions, targets)
            if output.loss is None:
                raise RuntimeError("released arm lacks query loss")
            output.loss.backward()
            optimizer.step()
            batch_decisions = int(targets.numel())
            decisions += batch_decisions
            loss_sum += float(output.loss.detach()) * batch_decisions
            correct += int(
                torch.count_nonzero(output.logits.detach().argmax(dim=-1) == targets)
            )
        test = _score_test(model, test_loader)
        passed = _source_pass(float(test["top1_rate"]))
        if not passed:
            scheduler.step()
        elapsed = carried_elapsed + time.monotonic() - began
        epoch_record = {
            "epoch": epoch + 1,
            "learning_rate": learning_rate_used,
            "train": {
                "decisions": decisions,
                "online_top1_correct": correct,
                "online_top1_rate": correct / decisions,
                "online_nll_nats": loss_sum / decisions,
            },
            "test": test,
            "strict_source_pass": passed,
            "epoch_seconds": time.monotonic() - epoch_began,
            "elapsed_seconds": elapsed,
        }
        history.append(epoch_record)
        _atomic_json(
            progress_path,
            {
                "schema": "uor-r4.zoology-release-progress/1",
                "issue": ISSUE,
                "arm_index": arm_index,
                "learning_rate": learning_rate,
                "latest": epoch_record,
                "completed_epochs": epoch + 1,
            },
        )
        _save_checkpoint(
            checkpoint_path,
            model=model,
            optimizer=optimizer,
            scheduler=scheduler,
            completed_epochs=epoch + 1,
            history=history,
            elapsed_seconds=elapsed,
        )
        print(
            f"#1050 arm={arm_index + 1}/4 lr={learning_rate:.17g} "
            f"epoch={epoch + 1}/64 test={test['top1_rate']:.6%} "
            f"nll={test['nll_nats']:.6f} wall={elapsed:.1f}s",
            flush=True,
        )
        if passed:
            status = "SOURCE_REPRODUCTION_POSITIVE"
            break

    elapsed = carried_elapsed + time.monotonic() - began
    artifact_payload = _artifact_payload(model, learning_rate=learning_rate)
    _write_exclusive(artifact_path, artifact_payload)
    final_test = history[-1]["test"] if history else None
    body = {
        "schema": ARM_SCHEMA,
        "issue": ISSUE,
        "policy": POLICY,
        "arm_index": arm_index,
        "learning_rate": learning_rate,
        "status": status,
        "passed": status == "SOURCE_REPRODUCTION_POSITIVE",
        "epochs": len(history),
        "history": history,
        "final_test": final_test,
        "best_test": (
            None
            if not history
            else max(
                (record["test"] for record in history),
                key=lambda score: float(score["top1_rate"]),
            )
        ),
        "elapsed_seconds": elapsed,
        "optimizer": {
            "name": "AdamW",
            "learning_rate": learning_rate,
            "weight_decay": WEIGHT_DECAY,
            "betas": [0.9, 0.999],
            "epsilon": 1e-8,
            "schedule": "CosineAnnealingLR_after_failed_test_only",
            "maximum_epochs": MAXIMUM_EPOCHS,
        },
        "artifact": {
            "schema": "uor-r4.zoology-release-model/1",
            "path": str(artifact_path.relative_to(root)),
            "bytes": len(artifact_payload),
            "cid": cid_bytes(artifact_payload),
            "source_commit": SOURCE_COMMIT,
            "learning_rate": learning_rate,
            "config": asdict(model.config),
            "tied_omission": "lm_head.weight",
            "state_cid": control_development._tensor_mapping_cid(
                {
                    name: tensor
                    for name, tensor in model.state_dict().items()
                    if name != "lm_head.weight"
                }
            ),
        },
        "work": {
            "train_query_presentations": sum(
                int(record["train"]["decisions"]) for record in history
            ),
            "test_query_presentations": sum(
                int(record["test"]["decisions"]) for record in history
            ),
            "future_value_reads": 0,
            "role_reads": 0,
            "r4_geometry_reads": 0,
            "uor_byte_reads": 0,
            "provider_calls": 0,
            "teacher_calls": 0,
        },
    }
    result = _with_cid(body, "arm_cid")
    _write_exclusive_json(result_path, result)
    return result


def _finish_result(
    root: Path,
    *,
    preparation: Mapping[str, Any],
    preflight: Mapping[str, Any],
    plan: Mapping[str, Any] | None,
    arms: Sequence[Mapping[str, Any]],
    verdict: str,
    action: str,
    elapsed_seconds: float,
) -> dict[str, Any]:
    body = {
        "schema": RESULT_SCHEMA,
        "issue": ISSUE,
        "policy": POLICY,
        "preparation_cid": preparation["preparation_cid"],
        "preflight_cid": preflight["preflight_cid"],
        "implementation": preflight["implementation"],
        "dataset_cid": preparation["dataset"]["tensor_cid"],
        "plan": None if plan is None else dict(plan),
        "arms": [dict(arm) for arm in arms],
        "decision": {
            "verdict": verdict,
            "passed": verdict == "SOURCE_REPRODUCTION_POSITIVE",
            "strict_threshold": EARLY_STOP_THRESHOLD,
            "comparison": ">",
            "action": action,
        },
        "elapsed_seconds": elapsed_seconds,
        "read_ledger": {
            "uor_byte_reads": 0,
            "r4_geometry_reads": 0,
            "role_reads": 0,
            "teacher_calls": 0,
            "provider_calls": 0,
            "future_value_reads": 0,
        },
        "nonclaims": [
            "R4 or geometric attention",
            "English generation",
            "reasoning",
            "modulo-256 softmax",
            "exact runtime lowering",
            "product or release readiness",
        ],
    }
    result = _with_cid(body, "result_cid")
    _write_exclusive_json(root / RESULT_RELATIVE_PATH, result)
    return result


def run_release_reproduction(root: Path) -> dict[str, Any]:
    """Run the frozen rates in order and stop on the first strict source pass."""

    root = root.resolve()
    result_path = root / RESULT_RELATIVE_PATH
    if result_path.exists():
        return verify_release_reproduction(root)
    preparation = _read_json(
        root / PREPARATION_RELATIVE_PATH, cid_field="preparation_cid"
    )
    preflight = _read_json(root / PREFLIGHT_RELATIVE_PATH, cid_field="preflight_cid")
    implementation = _implementation_contract()
    if (
        preparation.get("implementation") != implementation
        or preflight.get("implementation") != implementation
        or preflight.get("preparation_cid") != preparation.get("preparation_cid")
    ):
        raise ValueError("#1050 execution binding differs")
    _dataset_record(root, preparation)

    started_path = root / RUN_STARTED_RELATIVE_PATH
    if not started_path.exists():
        _write_exclusive_json(
            started_path,
            _started_record(
                phase="run", preparation=preparation, implementation=implementation
            ),
        )
    selection = preflight.get("selection")
    plan = selection.get("selected_plan") if isinstance(selection, Mapping) else None
    if preflight.get("passed") is not True or not isinstance(plan, Mapping):
        return _finish_result(
            root,
            preparation=preparation,
            preflight=preflight,
            plan=None,
            arms=(),
            verdict="NOT_RUN_PREFLIGHT",
            action="repair only C0/resource admission; make no attention inference",
            elapsed_seconds=0.0,
        )

    tensors = _load_dataset(root / DATA_RELATIVE_PATH)
    began = time.monotonic()
    arms: list[dict[str, Any]] = []
    verdict = "SOURCE_REPRODUCTION_MISS"
    action = "audit source-port parity and stop without changing R4"
    for arm_index, learning_rate in enumerate(LEARNING_RATES):
        arm = _run_arm(
            root,
            tensors,
            arm_index=arm_index,
            learning_rate=learning_rate,
            threads=int(plan["threads"]),
        )
        arms.append(arm)
        if arm.get("status") == "SOURCE_REPRODUCTION_POSITIVE":
            verdict = "SOURCE_REPRODUCTION_POSITIVE"
            action = "open a fresh exact-#1045-byte C2 transfer issue"
            break
        if arm.get("status") == "INCOMPLETE_HARD_WALL":
            verdict = "INCOMPLETE_HARD_WALL"
            action = (
                "stop and inspect the measured CPU wall; do not infer a mechanism miss"
            )
            break
    return _finish_result(
        root,
        preparation=preparation,
        preflight=preflight,
        plan=plan,
        arms=arms,
        verdict=verdict,
        action=action,
        elapsed_seconds=time.monotonic() - began,
    )


def verify_release_reproduction(root: Path) -> dict[str, Any]:
    """Verify lifecycle bindings, arm CIDs, artifacts, and final decision."""

    root = root.resolve()
    preparation = _read_json(
        root / PREPARATION_RELATIVE_PATH, cid_field="preparation_cid"
    )
    preflight = _read_json(root / PREFLIGHT_RELATIVE_PATH, cid_field="preflight_cid")
    result = _read_json(root / RESULT_RELATIVE_PATH, cid_field="result_cid")
    preflight_started = _read_json(
        root / PREFLIGHT_STARTED_RELATIVE_PATH, cid_field="started_cid"
    )
    run_started = _read_json(root / RUN_STARTED_RELATIVE_PATH, cid_field="started_cid")
    implementation = _implementation_contract()
    _dataset_record(root, preparation)
    predecessor_root = Path(str(preparation.get("predecessor_root")))
    if preparation.get("predecessor") != _bind_predecessor(predecessor_root):
        raise ValueError("#1049 predecessor changed after #1050 execution")
    selection = preflight.get("selection")
    if (
        not isinstance(selection, Mapping)
        or _select_plan(selection.get("plans", [])) != selection
    ):
        raise ValueError("#1050 CPU plan selection does not reproduce")
    if (
        preparation.get("schema") != PREPARATION_SCHEMA
        or preflight.get("schema") != PREFLIGHT_SCHEMA
        or result.get("schema") != RESULT_SCHEMA
        or preparation.get("issue") != ISSUE
        or preflight.get("issue") != ISSUE
        or result.get("issue") != ISSUE
        or preparation.get("implementation") != implementation
        or preflight.get("implementation") != implementation
        or result.get("implementation") != implementation
        or result.get("preparation_cid") != preparation.get("preparation_cid")
        or result.get("preflight_cid") != preflight.get("preflight_cid")
        or preflight_started
        != _started_record(
            phase="preflight",
            preparation=preparation,
            implementation=implementation,
        )
        or run_started
        != _started_record(
            phase="run",
            preparation=preparation,
            implementation=implementation,
        )
    ):
        raise ValueError("#1050 lifecycle envelope differs")
    arms = result.get("arms")
    if not isinstance(arms, list) or len(arms) > len(LEARNING_RATES):
        raise ValueError("#1050 arm list is malformed")
    passed_indices: list[int] = []
    for index, arm in enumerate(arms):
        if not isinstance(arm, Mapping):
            raise ValueError("#1050 arm record is malformed")
        _verify_self_cid(arm, "arm_cid")
        if (
            arm.get("arm_index") != index
            or arm.get("learning_rate") != LEARNING_RATES[index]
        ):
            raise ValueError("#1050 arm order differs")
        artifact = arm.get("artifact")
        if not isinstance(artifact, Mapping):
            raise ValueError("#1050 arm artifact is missing")
        payload = (root / str(artifact["path"])).read_bytes()
        if len(payload) != artifact.get("bytes") or cid_bytes(payload) != artifact.get(
            "cid"
        ):
            raise ValueError("#1050 arm artifact changed")
        if arm.get("passed") is True:
            passed_indices.append(index)
        key = LEARNING_RATE_KEYS[index]
        arm_file = _read_json(
            root / "arms" / f"{index:02d}-{key}" / "result.json",
            cid_field="arm_cid",
        )
        if arm_file != dict(arm):
            raise ValueError("#1050 per-arm result differs from the final envelope")
    decision = result.get("decision")
    if not isinstance(decision, Mapping):
        raise ValueError("#1050 decision is malformed")
    verdict = decision.get("verdict")
    if verdict == "SOURCE_REPRODUCTION_POSITIVE":
        if len(passed_indices) != 1 or passed_indices[0] != len(arms) - 1:
            raise ValueError("#1050 positive early stop differs")
    elif verdict == "SOURCE_REPRODUCTION_MISS":
        if len(arms) != len(LEARNING_RATES) or passed_indices:
            raise ValueError("#1050 all-rate falsifier differs")
    elif verdict == "INCOMPLETE_HARD_WALL":
        if not arms or arms[-1].get("status") != "INCOMPLETE_HARD_WALL":
            raise ValueError("#1050 hard-wall stop differs")
    elif verdict == "NOT_RUN_PREFLIGHT":
        if arms or preflight.get("passed") is True:
            raise ValueError("#1050 preflight stop differs")
    else:
        raise ValueError("#1050 invented a verdict")
    ledger = result.get("read_ledger")
    if not isinstance(ledger, Mapping) or any(
        int(value) != 0 for value in ledger.values()
    ):
        raise ValueError("#1050 reports a forbidden read")
    return result


def execute_release_reproduction(
    root: Path,
    *,
    predecessor_root: Path,
) -> dict[str, Any]:
    """Prepare, preflight, run, and verify the exact-source reproduction."""

    prepare_release_reproduction(root, predecessor_root=predecessor_root)
    preflight_release_reproduction(root)
    run_release_reproduction(root)
    return verify_release_reproduction(root)


__all__ = [
    "LEARNING_RATES",
    "execute_release_reproduction",
    "prepare_release_reproduction",
    "preflight_release_reproduction",
    "run_release_reproduction",
    "verify_release_reproduction",
]
