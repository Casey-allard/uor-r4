"""Focused contract tests for the #973 Apple CPU recovery."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest import mock

import torch

from r4_softmax_trainer import group_retention_decoder_cpu_recovery_campaign as subject
from r4_softmax_trainer.cli import parser
from r4_softmax_trainer.paths import (
    default_group_retention_decoder_cpu_recovery_root,
)


class _Telemetry:
    def synchronize(self) -> None:
        return None

    def empty_cache(self) -> None:
        return None

    def recommended_memory(self) -> int:
        return 1_000_000

    def allocated_memory(self) -> int:
        return 1


def _cpu_contract() -> dict[str, object]:
    return {
        "platform": "Darwin",
        "blas": "Apple Accelerate",
        "torch_intraop_threads": 4,
        "torch_interop_threads": 4,
        "thread_environment": dict(subject.THREAD_ENVIRONMENT),
        "processes": 1,
        "arm_execution": "SEQUENTIAL",
        "mps_used": False,
        "cuda_used": False,
        "deterministic_algorithms": True,
    }


class CpuBackendContractTests(unittest.TestCase):
    def test_requires_darwin_accelerate_exact_environment_and_four_threads(self) -> None:
        environment = dict(subject.THREAD_ENVIRONMENT)
        with (
            mock.patch.dict(subject.os.environ, environment, clear=True),
            mock.patch.object(subject.platform, "system", return_value="Darwin"),
            mock.patch.object(
                subject.torch.__config__,
                "show",
                return_value="BLAS_INFO=accelerate, LAPACK_INFO=accelerate",
            ),
            mock.patch.object(subject.torch, "use_deterministic_algorithms"),
            mock.patch.object(subject.torch, "manual_seed"),
            mock.patch.object(subject.torch, "set_num_threads") as set_threads,
            mock.patch.object(subject.torch, "get_num_threads", return_value=4),
            mock.patch.object(subject.torch, "get_num_interop_threads", return_value=4),
            mock.patch.object(
                subject.torch, "are_deterministic_algorithms_enabled", return_value=True
            ),
        ):
            device, _, evidence = subject._require_cpu_device("cpu")
        self.assertEqual(device, torch.device("cpu"))
        set_threads.assert_called_once_with(4)
        self.assertEqual(evidence["thread_environment"], environment)
        self.assertEqual(evidence["processes"], 1)
        self.assertEqual(evidence["arm_execution"], "SEQUENTIAL")
        self.assertIs(evidence["mps_used"], False)
        self.assertIs(evidence["cuda_used"], False)

    def test_environment_mismatch_fails_before_torch_configuration(self) -> None:
        with (
            mock.patch.dict(subject.os.environ, {}, clear=True),
            mock.patch.object(subject.platform, "system", return_value="Darwin"),
            mock.patch.object(
                subject.torch.__config__,
                "show",
                return_value="BLAS_INFO=accelerate, LAPACK_INFO=accelerate",
            ),
            mock.patch.object(subject.torch, "set_num_threads") as set_threads,
            self.assertRaisesRegex(RuntimeError, "OMP_NUM_THREADS=4"),
        ):
            subject._require_cpu_device("cpu")
        set_threads.assert_not_called()

    def test_non_darwin_or_non_accelerate_is_rejected(self) -> None:
        with (
            mock.patch.object(subject.platform, "system", return_value="Linux"),
            self.assertRaisesRegex(RuntimeError, "requires Darwin"),
        ):
            subject._require_cpu_device("cpu")
        with (
            mock.patch.dict(
                subject.os.environ, subject.THREAD_ENVIRONMENT, clear=True
            ),
            mock.patch.object(subject.platform, "system", return_value="Darwin"),
            mock.patch.object(subject.torch.__config__, "show", return_value="MKL"),
            self.assertRaisesRegex(RuntimeError, "Apple Accelerate"),
        ):
            subject._require_cpu_device("cpu")


class CpuRecoveryTerminalTests(unittest.TestCase):
    def _run(
        self,
        root: Path,
        execution: dict[str, object],
        capture: dict[str, object] | None = None,
    ) -> dict[str, object]:
        geometry = SimpleNamespace(
            artifact_cid="blake3:" + "1" * 64,
            arms={},
        )
        preparation = {"manifest_cid": "blake3:" + "2" * 64}
        sequences = torch.zeros(32, 129, dtype=torch.long)
        initialization = {
            "seed": 9_737,
            "learned_initialization_cid": "blake3:" + "3" * 64,
            "arm_cids": {},
            "byte_identical": True,
            "ledgers": {},
            "equal_ledgers": True,
        }

        def executor(*args: object, **kwargs: object) -> dict[str, object]:
            if capture is not None:
                capture.update(kwargs)
            return execution

        with (
            mock.patch.object(
                subject,
                "_load_prepared",
                return_value=(preparation, geometry, sequences, sequences.clone()),
            ),
            mock.patch.object(
                subject,
                "_initialization_identity",
                return_value=(initialization, {}),
            ),
            mock.patch.object(
                subject,
                "trainer_implementation_contract",
                return_value={"files": [], "tree_cid": "blake3:" + "4" * 64},
            ),
        ):
            return subject.run_group_retention_decoder_cpu_recovery_preflight(
                root,
                backend="cpu",
                _executor=executor,
                _device_provider=lambda backend: (
                    torch.device("cpu"),
                    _Telemetry(),
                    _cpu_contract(),
                ),
            )

    def test_timing_is_telemetry_only_and_science_is_unchanged(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            capture: dict[str, object] = {}
            result = self._run(
                Path(directory),
                {
                    "available": True,
                    "wall_passed": True,
                    "mechanical": {"passed": True},
                    "retained_decoder_pass": True,
                    "h4_specific_pass": False,
                    "passed": True,
                },
                capture,
            )
        self.assertIs(capture["timing_is_admission_gate"], False)
        self.assertEqual(capture["wall_ceiling_seconds"], 900.0)
        self.assertEqual(
            capture["fitted_relative_paths"],
            (
                subject.EXACT_FITTED_RELATIVE_PATH,
                subject.SCRAMBLED_FITTED_RELATIVE_PATH,
            ),
        )
        contract = result["contract"]
        self.assertEqual(contract["mechanical"]["timing_role"], "TELEMETRY_ONLY_NOT_ADMISSION")
        self.assertEqual(contract["optimizer"]["seed"], 9_737)
        self.assertEqual(contract["optimizer"]["steps_per_trained_arm"], 256)
        self.assertEqual(contract["optimizer"]["total_presentations"], 524_288)
        self.assertEqual(contract["thresholds"]["h4_nll_delta_nats"], 0.02)
        self.assertEqual(result["verdict"], subject.TERMINAL_PASS)

    def test_wall_stop_is_unavailable_not_scientific_failure(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            result = self._run(
                Path(directory),
                {
                    "available": False,
                    "classification": "UNAVAILABLE",
                    "wall_passed": False,
                    "mechanical": {"passed": False},
                    "optimization": {
                        "status": "PARTIAL_WALL_STOP",
                        "completed_steps_per_arm": {
                            "exact_h4": 256,
                            "scrambled_h4": 20,
                        },
                    },
                    "retained_decoder_pass": False,
                    "h4_specific_pass": False,
                    "passed": False,
                },
            )
        self.assertEqual(result["verdict"], subject.TERMINAL_UNAVAILABLE)
        self.assertEqual(
            result["construction_execution"]["optimization"]["status"],
            "PARTIAL_WALL_STOP",
        )

    def test_backend_failure_is_before_create_once_started_marker(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with self.assertRaisesRegex(RuntimeError, "thread contract"):
                subject.run_group_retention_decoder_cpu_recovery_preflight(
                    root,
                    _device_provider=lambda backend: (_ for _ in ()).throw(
                        RuntimeError("thread contract mismatch")
                    ),
                )
            self.assertFalse((root / subject.STARTED_RELATIVE_PATH).exists())
            self.assertFalse((root / subject.RESULT_RELATIVE_PATH).exists())

    def test_cli_and_default_root_are_recovery_specific(self) -> None:
        predecessor = "/tmp/frozen-group-retention"
        prepared = parser().parse_args(
            [
                "prepare-group-retention-decoder-cpu-recovery",
                "--predecessor",
                predecessor,
            ]
        )
        self.assertEqual(
            prepared.command, "prepare-group-retention-decoder-cpu-recovery"
        )
        preflight = parser().parse_args(
            ["preflight-group-retention-decoder-cpu-recovery", "--backend", "cpu"]
        )
        self.assertEqual(
            preflight.command, "preflight-group-retention-decoder-cpu-recovery"
        )
        self.assertTrue(
            str(default_group_retention_decoder_cpu_recovery_root()).endswith(
                "issue-973-group-retention-decoder-v1-cpu-recovery"
            )
        )


if __name__ == "__main__":
    unittest.main()
