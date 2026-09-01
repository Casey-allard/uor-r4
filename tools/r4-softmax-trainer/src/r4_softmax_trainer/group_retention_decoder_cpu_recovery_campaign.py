"""Resource-only Apple CPU recovery for the frozen #973 retained decoder."""

from __future__ import annotations

import copy
import os
import platform
import resource
import time
from collections.abc import Callable, Mapping
from pathlib import Path
from typing import Any

import torch

from .group_retention_decoder_campaign import (
    H4_SPECIFIC_MISS,
    H4_SPECIFIC_NOT_EVALUATED,
    H4_SPECIFIC_PASS,
    ISSUE,
    TERMINAL_FAIL,
    TERMINAL_PASS,
    DecoderPreflightConfig,
    DeviceTelemetry,
    PreflightExecutor,
    _contract as _mps_contract,
    _execute_preflight,
    _initialization_identity,
    _load_prepared,
    _with_cid,
    _write_exclusive_json,
    prepare_group_retention_decoder_data,
)
from .provenance import trainer_implementation_contract


POLICY = "R4GroupAddressedRetentionDecoderV1CpuRecovery"
PREPARATION_MANIFEST_NAME = (
    "group-retention-decoder-cpu-recovery-preparation-manifest.json"
)
STARTED_RELATIVE_PATH = "preflight/group-retention-decoder-cpu-recovery-started.json"
RESULT_RELATIVE_PATH = "preflight/group-retention-decoder-cpu-recovery-result.json"
EXACT_FITTED_RELATIVE_PATH = "fitted/cpu-recovery-exact-h4.safetensors"
SCRAMBLED_FITTED_RELATIVE_PATH = "fitted/cpu-recovery-scrambled-h4.safetensors"

PREPARATION_SCHEMA = (
    "uor-r4.group-addressed-retention-decoder-cpu-recovery-preparation/1"
)
STARTED_SCHEMA = "uor-r4.group-addressed-retention-decoder-cpu-recovery-started/1"
RESULT_SCHEMA = "uor-r4.group-addressed-retention-decoder-cpu-recovery-result/1"

TERMINAL_UNAVAILABLE = "UNAVAILABLE_FULLER_DECODER_CPU_RECOVERY"
MPS_TERMINAL_RESULT_CID = (
    "blake3:aef070691138c7a333d84c0b25437abf3e7d8dc87b3244ab7b6acfff89e73a5b"
)
CPU_THREADS = 4
WALL_CEILING_SECONDS = 900.0
THREAD_ENVIRONMENT = {
    "OMP_NUM_THREADS": "4",
    "VECLIB_MAXIMUM_THREADS": "4",
    "OPENBLAS_NUM_THREADS": "4",
}


class _CpuTelemetry(DeviceTelemetry):
    """Process RSS against physical RAM; both values are bytes on Darwin."""

    def synchronize(self) -> None:
        return None

    def empty_cache(self) -> None:
        return None

    def recommended_memory(self) -> int:
        page_size = int(os.sysconf("SC_PAGE_SIZE"))
        physical_pages = int(os.sysconf("SC_PHYS_PAGES"))
        return page_size * physical_pages

    def allocated_memory(self) -> int:
        return int(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss)


CpuDeviceProvider = Callable[
    [str], tuple[torch.device, DeviceTelemetry, Mapping[str, Any]]
]


def _require_cpu_device(
    backend: str,
) -> tuple[torch.device, DeviceTelemetry, Mapping[str, Any]]:
    """Enforce the public CPU/Accelerate/thread freeze before consuming the run."""
    if backend != "cpu":
        raise ValueError("#973 CPU recovery permits only backend='cpu'")
    if platform.system() != "Darwin":
        raise RuntimeError("#973 CPU recovery requires Darwin")
    build = torch.__config__.show()
    lowered = build.lower()
    if "blas_info=accelerate" not in lowered or "lapack_info=accelerate" not in lowered:
        raise RuntimeError("#973 CPU recovery requires Apple Accelerate BLAS/LAPACK")
    observed_environment = {name: os.environ.get(name) for name in THREAD_ENVIRONMENT}
    if observed_environment != THREAD_ENVIRONMENT:
        raise RuntimeError(
            "#973 CPU recovery requires OMP_NUM_THREADS=4, "
            "VECLIB_MAXIMUM_THREADS=4, and OPENBLAS_NUM_THREADS=4"
        )

    torch.use_deterministic_algorithms(True)
    torch.manual_seed(9_737)
    torch.set_num_threads(CPU_THREADS)
    if torch.get_num_interop_threads() != CPU_THREADS:
        try:
            torch.set_num_interop_threads(CPU_THREADS)
        except RuntimeError as error:
            raise RuntimeError(
                "#973 CPU recovery could not establish four interop threads"
            ) from error
    if (
        torch.get_num_threads() != CPU_THREADS
        or torch.get_num_interop_threads() != CPU_THREADS
        or not torch.are_deterministic_algorithms_enabled()
    ):
        raise RuntimeError(
            "#973 CPU recovery thread or deterministic contract differs from the freeze"
        )

    return (
        torch.device("cpu"),
        _CpuTelemetry(),
        {
            "platform": platform.system(),
            "blas": "Apple Accelerate",
            "torch_intraop_threads": torch.get_num_threads(),
            "torch_interop_threads": torch.get_num_interop_threads(),
            "thread_environment": observed_environment,
            "processes": 1,
            "arm_execution": "SEQUENTIAL",
            "mps_used": False,
            "cuda_used": False,
            "deterministic_algorithms": torch.are_deterministic_algorithms_enabled(),
        },
    )


def prepare_group_retention_decoder_cpu_recovery_data(
    root: Path, *, predecessor: Path
) -> dict[str, Any]:
    """Create the independent recovery root without touching terminal MPS files."""
    return prepare_group_retention_decoder_data(
        root,
        predecessor=predecessor,
        _manifest_name=PREPARATION_MANIFEST_NAME,
        _schema=PREPARATION_SCHEMA,
        _policy=POLICY,
    )


def _contract(
    config: DecoderPreflightConfig, cpu_contract: Mapping[str, Any]
) -> dict[str, Any]:
    contract = copy.deepcopy(_mps_contract(config))
    contract["backend"] = "cpu"
    contract["execution"] = {
        **dict(cpu_contract),
        "required_thread_environment": dict(THREAD_ENVIRONMENT),
        "whole_process_wall_ceiling_seconds": WALL_CEILING_SECONDS,
        "mps": "FORBIDDEN",
        "cuda": "FORBIDDEN",
    }
    contract["mechanical"].update(
        {
            "timing_role": "TELEMETRY_ONLY_NOT_ADMISSION",
            "eta_ceiling_seconds": WALL_CEILING_SECONDS,
        }
    )
    contract["optimizer"].update(
        {
            "cpu": True,
            "cpu_fallback": False,
            "mps": False,
            "cuda": False,
            "processes": 1,
            "arm_execution": "SEQUENTIAL",
            "torch_intraop_threads": CPU_THREADS,
            "torch_interop_threads": CPU_THREADS,
            "wall_ceiling_seconds": WALL_CEILING_SECONDS,
        }
    )
    contract["recovery_basis"] = {
        "terminal_policy": "R4GroupAddressedRetentionDecoderV1",
        "terminal_result_cid": MPS_TERMINAL_RESULT_CID,
        "terminal_verdict": "UNAVAILABLE_FULLER_DECODER_CONSTRUCTION",
        "optimization": "NOT_RUN",
        "changed_variable": "EXECUTION_PLAN_ONLY",
    }
    return contract


def run_group_retention_decoder_cpu_recovery_preflight(
    root: Path,
    *,
    backend: str = "cpu",
    _executor: PreflightExecutor | None = None,
    _device_provider: CpuDeviceProvider | None = None,
) -> dict[str, Any]:
    """Run the one create-once CPU recovery; no reveal or main path exists."""
    process_started = time.monotonic()
    root = root.resolve()
    config = DecoderPreflightConfig.production()
    if backend != "cpu":
        raise ValueError("#973 CPU recovery permits only backend='cpu'")
    if any(
        (root / relative).exists() or (root / relative).is_symlink()
        for relative in (STARTED_RELATIVE_PATH, RESULT_RELATIVE_PATH)
    ):
        raise FileExistsError("the sole #973 CPU recovery is already terminal")

    # Backend, BLAS, environment, and actual torch thread counts are checked
    # before the create-once started marker is written.
    device_provider = _require_cpu_device if _device_provider is None else _device_provider
    device, telemetry, cpu_contract = device_provider(backend)
    if device.type != "cpu":
        raise RuntimeError("#973 CPU recovery provider did not return a CPU device")

    preparation, geometry, train_sequences, validation_sequences = _load_prepared(
        root,
        manifest_name=PREPARATION_MANIFEST_NAME,
        schema=PREPARATION_SCHEMA,
        policy=POLICY,
    )
    initialization, initial_exports = _initialization_identity(geometry.arms, config.model)
    contract = _contract(config, cpu_contract)
    started = _with_cid(
        {
            "schema": STARTED_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            "preparation_manifest_cid": preparation["manifest_cid"],
            "geometry_artifact_cid": geometry.artifact_cid,
            "implementation": trainer_implementation_contract(),
            "initialization": initialization,
            "contract": contract,
            "model_heldout": {"status": "NOT_RUN", "reads": 0},
        },
        "started_cid",
    )
    _write_exclusive_json(root / STARTED_RELATIVE_PATH, started)

    executor = _execute_preflight if _executor is None else _executor
    execution: Mapping[str, Any] | None = None
    failure: dict[str, str] | None = None
    try:
        execution = executor(
            root,
            train_sequences,
            validation_sequences,
            geometry.arms,
            device=device,
            telemetry=telemetry,
            config=config,
            initial_exports=initial_exports,
            wall_ceiling_seconds=WALL_CEILING_SECONDS,
            timing_is_admission_gate=False,
            started_monotonic=process_started,
            fitted_relative_paths=(
                EXACT_FITTED_RELATIVE_PATH,
                SCRAMBLED_FITTED_RELATIVE_PATH,
            ),
        )
        if not isinstance(execution, Mapping):
            raise RuntimeError("CPU recovery executor returned no evidence mapping")
    except Exception as error:
        failure = {"type": type(error).__name__, "reason": str(error)}

    mechanical_pass = bool(
        failure is None
        and execution is not None
        and isinstance(execution.get("mechanical"), Mapping)
        and execution["mechanical"].get("passed") is True
    )
    scientific_available = bool(
        mechanical_pass
        and execution is not None
        and execution.get("available") is True
        and execution.get("wall_passed") is True
    )
    retained_pass = bool(
        scientific_available
        and execution is not None
        and execution.get("retained_decoder_pass") is True
        and execution.get("passed") is True
    )
    h4_pass = bool(retained_pass and execution.get("h4_specific_pass") is True)
    if not scientific_available:
        verdict = TERMINAL_UNAVAILABLE
    elif retained_pass:
        verdict = TERMINAL_PASS
    else:
        verdict = TERMINAL_FAIL
    if not retained_pass:
        h4_verdict = H4_SPECIFIC_NOT_EVALUATED
    elif h4_pass:
        h4_verdict = H4_SPECIFIC_PASS
    else:
        h4_verdict = H4_SPECIFIC_MISS
    result = _with_cid(
        {
            "schema": RESULT_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            "started_cid": started["started_cid"],
            "preparation_manifest_cid": preparation["manifest_cid"],
            "geometry_artifact_cid": geometry.artifact_cid,
            "initialization": initialization,
            "contract": contract,
            "construction_execution": dict(execution) if execution is not None else None,
            "verdict": verdict,
            "h4_specific_verdict": h4_verdict,
            "failure": failure,
            "model_heldout": {"status": "NOT_RUN", "reads": 0},
            "promotion": "NOT_AUTHORIZED_BY_CONSTRUCTION",
            "main_command": "ABSENT",
            "reveal_command": "ABSENT",
        },
        "result_cid",
    )
    _write_exclusive_json(root / RESULT_RELATIVE_PATH, result)
    return result
