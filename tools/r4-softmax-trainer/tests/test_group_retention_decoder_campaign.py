"""Focused preparation, terminal, and CLI checks for the fuller decoder."""

from __future__ import annotations

import copy
import json
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest import mock

import torch

from r4_softmax_trainer import group_retention_decoder_campaign as subject
from r4_softmax_trainer.cli import parser
from r4_softmax_trainer.group_retention_decoder_data import (
    ConstructionData,
    ConstructionPartition,
)
from r4_softmax_trainer.provenance import canonical_json_bytes


class _Telemetry:
    def synchronize(self) -> None:
        return None

    def empty_cache(self) -> None:
        return None

    def recommended_memory(self) -> int:
        return 1_000_000

    def allocated_memory(self) -> int:
        return 1


def _partition(name: str, ordinals: tuple[int, ...], fill: int) -> ConstructionPartition:
    story_cids = tuple(f"blake3:{ordinal + 1:064x}" for ordinal in ordinals)
    span_cids = tuple(f"blake3:{ordinal + 10_000:064x}" for ordinal in ordinals)
    tokens = bytes([fill, 0]) * (len(ordinals) * 129)
    index = b"".join(
        canonical_json_bytes(
            {
                "construction_ordinal": construction_ordinal,
                "construction_partition": name,
                "copied_token_count": 129,
                "copied_token_offset": construction_ordinal * 129,
                "scored_next_tokens": 128,
                "selected_span_cid": span_cids[construction_ordinal],
                "source_fit_ordinal": source_ordinal,
                "source_full_span_cid": f"blake3:{source_ordinal + 20_000:064x}",
                "story_cid": story_cids[construction_ordinal],
            }
        )
        for construction_ordinal, source_ordinal in enumerate(ordinals)
    )
    return ConstructionPartition(
        name=name,
        ordinals=ordinals,
        tokens=tokens,
        index=index,
        story_cids=story_cids,
        span_cids=span_cids,
    )


def _expected_predecessor() -> dict[str, str]:
    return {
        "training_view_manifest_cid": subject.EXPECTED_PREDECESSOR_TRAINING_VIEW_CID,
        "population_manifest_cid": subject.EXPECTED_PREDECESSOR_POPULATION_CID,
        "fit_store_cid": subject.EXPECTED_PREDECESSOR_FIT_STORE_CID,
        "fit_index_cid": subject.EXPECTED_PREDECESSOR_FIT_INDEX_CID,
        "tokenizer_cid": subject.EXPECTED_TOKENIZER_CID,
    }


def _rebind_preparation(root: Path, manifest: dict[str, object]) -> dict[str, object]:
    payload = {
        key: value
        for key, value in manifest.items()
        if key not in {"artifacts", "tree_cid", "manifest_cid"}
    }
    return subject.write_bound_manifest(
        root / subject.PREPARATION_MANIFEST_NAME,
        payload,
        artifact_root=root,
        relative_paths=(
            subject.TRAIN_TOKENS_RELATIVE_PATH,
            subject.TRAIN_INDEX_RELATIVE_PATH,
            subject.VALIDATION_TOKENS_RELATIVE_PATH,
            subject.VALIDATION_INDEX_RELATIVE_PATH,
            subject.GEOMETRY_RELATIVE_PATH,
        ),
    )


def _prepare_loadable_root(
    root: Path,
) -> tuple[dict[str, object], SimpleNamespace, dict[str, object]]:
    predecessor = root.parent / "predecessor"
    geometry_path = predecessor / subject.GEOMETRY_RELATIVE_PATH
    geometry_path.parent.mkdir(parents=True)
    geometry_path.write_bytes(b"geometry")
    data = ConstructionData(
        predecessor=_expected_predecessor(),
        train=_partition("train", tuple(range(8, 40)), 1),
        validation=_partition("validation", tuple(range(40, 72)), 2),
    )
    geometry = SimpleNamespace(
        artifact_cid=subject.EXPECTED_GEOMETRY_ARTIFACT_CID,
        geometry_file_cid="blake3:" + "6" * 64,
        h4_generated_count=120,
        c120_generated_count=120,
        scrambled_generated_count=120,
    )
    implementation: dict[str, object] = {
        "files": [],
        "tree_cid": "blake3:" + "7" * 64,
    }
    with (
        mock.patch.object(subject, "build_decoder_construction_data", return_value=data),
        mock.patch.object(subject, "load_group_geometry_artifacts", return_value=geometry),
        mock.patch.object(
            subject, "trainer_implementation_contract", return_value=implementation
        ),
    ):
        manifest = subject.prepare_group_retention_decoder_data(
            root, predecessor=predecessor
        )
    return manifest, geometry, implementation


class DecoderPreparationTests(unittest.TestCase):
    def test_prepare_binds_fit_only_slices_geometry_and_implementation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            predecessor = base / "predecessor"
            root = base / "successor"
            geometry_path = predecessor / subject.GEOMETRY_RELATIVE_PATH
            geometry_path.parent.mkdir(parents=True)
            geometry_path.write_bytes(b"geometry")
            data = ConstructionData(
                predecessor={
                    "training_view_manifest_cid": "blake3:" + "1" * 64,
                    "population_manifest_cid": "blake3:" + "2" * 64,
                    "fit_store_cid": "blake3:" + "3" * 64,
                    "fit_index_cid": "blake3:" + "4" * 64,
                    "tokenizer_cid": "blake3:" + "5" * 64,
                },
                train=_partition("train", tuple(range(8, 40)), 1),
                validation=_partition("validation", tuple(range(40, 72)), 2),
            )
            geometry = SimpleNamespace(
                artifact_cid=subject.EXPECTED_GEOMETRY_ARTIFACT_CID,
                geometry_file_cid="blake3:" + "6" * 64,
                h4_generated_count=120,
                c120_generated_count=120,
                scrambled_generated_count=120,
            )
            with (
                mock.patch.object(
                    subject, "build_decoder_construction_data", return_value=data
                ),
                mock.patch.object(
                    subject, "load_group_geometry_artifacts", return_value=geometry
                ),
                mock.patch.object(
                    subject,
                    "trainer_implementation_contract",
                    return_value={"files": [], "tree_cid": "blake3:" + "7" * 64},
                ),
            ):
                manifest = subject.prepare_group_retention_decoder_data(
                    root, predecessor=predecessor
                )
                with self.assertRaisesRegex(FileExistsError, "create-once"):
                    subject.prepare_group_retention_decoder_data(
                        root, predecessor=predecessor
                    )
            self.assertEqual(manifest["selection"]["train"]["ordinals"], list(range(8, 40)))
            self.assertEqual(
                manifest["selection"]["validation"]["ordinals"], list(range(40, 72))
            )
            self.assertEqual(manifest["model_heldout"], {"status": "NOT_RUN", "reads": 0})
            self.assertEqual((root / subject.TRAIN_TOKENS_RELATIVE_PATH).stat().st_size, 8_256)
            self.assertEqual(
                (root / subject.VALIDATION_TOKENS_RELATIVE_PATH).stat().st_size,
                8_256,
            )

    def test_load_prepared_rejects_rebound_predecessor_identity_tampering(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "successor"
            manifest, geometry, implementation = _prepare_loadable_root(root)
            tampered = copy.deepcopy(manifest)
            tampered["predecessor"]["fit_store_cid"] = "blake3:" + "f" * 64
            _rebind_preparation(root, tampered)

            with (
                mock.patch.object(
                    subject, "load_group_geometry_artifacts", return_value=geometry
                ),
                mock.patch.object(
                    subject,
                    "trainer_implementation_contract",
                    return_value=implementation,
                ),
                self.assertRaisesRegex(ValueError, "frozen contract"),
            ):
                subject._load_prepared(root)

    def test_load_prepared_rejects_rebound_selection_identity_tampering(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "successor"
            manifest, geometry, implementation = _prepare_loadable_root(root)
            tampered = copy.deepcopy(manifest)
            selection = tampered["selection"]
            selection["train"]["ordinals"][0] = 7
            tampered["selection_cid"] = subject.cid_bytes(
                canonical_json_bytes(selection)
            )
            _rebind_preparation(root, tampered)

            with (
                mock.patch.object(
                    subject, "load_group_geometry_artifacts", return_value=geometry
                ),
                mock.patch.object(
                    subject,
                    "trainer_implementation_contract",
                    return_value=implementation,
                ),
                self.assertRaisesRegex(ValueError, "frozen contract"),
            ):
                subject._load_prepared(root)

    def test_load_prepared_rejects_rebound_index_identity_tampering(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "successor"
            manifest, geometry, implementation = _prepare_loadable_root(root)
            index_path = root / subject.TRAIN_INDEX_RELATIVE_PATH
            lines = index_path.read_bytes().splitlines(keepends=True)
            first = json.loads(lines[0])
            first["source_fit_ordinal"] = 7
            lines[0] = canonical_json_bytes(first)
            index_path.write_bytes(b"".join(lines))
            _rebind_preparation(root, manifest)

            with (
                mock.patch.object(
                    subject, "load_group_geometry_artifacts", return_value=geometry
                ),
                mock.patch.object(
                    subject,
                    "trainer_implementation_contract",
                    return_value=implementation,
                ),
                self.assertRaisesRegex(ValueError, "differs from selection"),
            ):
                subject._load_prepared(root)


class DecoderPreflightTests(unittest.TestCase):
    def _run(
        self, root: Path, execution: dict[str, object] | Exception
    ) -> dict[str, object]:
        geometry = SimpleNamespace(
            artifact_cid=subject.EXPECTED_GEOMETRY_ARTIFACT_CID,
            arms={},
        )
        preparation = {"manifest_cid": "blake3:" + "8" * 64}
        sequences = torch.zeros(32, 129, dtype=torch.long)
        initialization = {
            "seed": 9_737,
            "learned_initialization_cid": "blake3:" + "9" * 64,
            "arm_cids": {},
            "byte_identical": True,
            "ledgers": {},
            "equal_ledgers": True,
        }

        def executor(*args: object, **kwargs: object) -> dict[str, object]:
            if isinstance(execution, Exception):
                raise execution
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
                return_value={"files": [], "tree_cid": "blake3:" + "a" * 64},
            ),
        ):
            return subject.run_group_retention_decoder_preflight(
                root,
                backend="mps",
                _executor=executor,
                _device_provider=lambda backend: (torch.device("cpu"), _Telemetry()),
            )

    def test_pass_is_create_once_and_never_authorizes_reveal_or_main(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            execution = {
                "available": True,
                "wall_passed": True,
                "mechanical": {"passed": True},
                "retained_decoder_pass": True,
                "h4_specific_pass": False,
                "passed": True,
            }
            result = self._run(root, execution)
            self.assertEqual(result["verdict"], subject.TERMINAL_PASS)
            self.assertEqual(result["h4_specific_verdict"], subject.H4_SPECIFIC_MISS)
            self.assertEqual(result["model_heldout"], {"status": "NOT_RUN", "reads": 0})
            self.assertEqual(result["main_command"], "ABSENT")
            self.assertEqual(result["reveal_command"], "ABSENT")
            self.assertFalse((root / "reveal").exists())
            with self.assertRaisesRegex(FileExistsError, "already terminal"):
                self._run(root, execution)

    def test_mechanical_miss_is_unavailable_not_a_model_negative(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            result = self._run(
                Path(directory),
                {
                    "available": False,
                    "wall_passed": False,
                    "mechanical": {"passed": False},
                    "retained_decoder_pass": False,
                    "h4_specific_pass": False,
                    "passed": False,
                },
            )
            self.assertEqual(result["verdict"], subject.TERMINAL_UNAVAILABLE)
            self.assertEqual(
                result["h4_specific_verdict"], subject.H4_SPECIFIC_NOT_EVALUATED
            )

    def test_admitted_nonfinite_is_a_scientific_fail_with_partial_status(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            execution = {
                "available": True,
                "classification": "SCIENTIFIC_FAIL",
                "wall_passed": True,
                "mechanical": {"passed": True},
                "optimization": {
                    "status": "PARTIAL_MODEL_FAILURE",
                    "completed_steps_per_arm": {
                        "exact_h4": 17,
                        "scrambled_h4": 0,
                    },
                },
                "retained_decoder_pass": False,
                "h4_specific_pass": False,
                "passed": False,
            }
            result = self._run(Path(directory), execution)
            self.assertEqual(result["verdict"], subject.TERMINAL_FAIL)
            self.assertEqual(
                result["h4_specific_verdict"], subject.H4_SPECIFIC_NOT_EVALUATED
            )
            self.assertEqual(
                result["construction_execution"]["optimization"]
                ["completed_steps_per_arm"]["exact_h4"],
                17,
            )

    def test_hard_wall_is_unavailable_and_preserves_completed_steps(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            execution = {
                "available": False,
                "classification": "UNAVAILABLE",
                "wall_passed": False,
                "mechanical": {"passed": False},
                "optimization": {
                    "status": "PARTIAL_WALL_STOP",
                    "completed_steps_per_arm": {
                        "exact_h4": 256,
                        "scrambled_h4": 41,
                    },
                },
                "retained_decoder_pass": False,
                "h4_specific_pass": False,
                "passed": False,
            }
            result = self._run(Path(directory), execution)
            self.assertEqual(result["verdict"], subject.TERMINAL_UNAVAILABLE)
            self.assertEqual(
                result["construction_execution"]["optimization"]
                ["completed_steps_per_arm"]["scrambled_h4"],
                41,
            )

    def test_unexpected_executor_error_is_unavailable(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            result = self._run(Path(directory), RuntimeError("MPS device lost"))
            self.assertEqual(result["verdict"], subject.TERMINAL_UNAVAILABLE)
            self.assertEqual(result["failure"]["type"], "RuntimeError")

    def test_emitted_exact_bytes_are_reloaded_into_a_fresh_model(self) -> None:
        artifact = b"fitted-safetensors"
        fresh = mock.Mock()
        fresh.to.return_value = fresh
        replay = {"ce_nats": 1.0, "logits_cid": "blake3:" + "1" * 64}
        state_off = {"ce_nats": 1.2, "logits_cid": "blake3:" + "2" * 64}
        with (
            mock.patch.object(
                subject, "R4GroupAddressedRetentionDecoderV1", return_value=fresh
            ) as constructor,
            mock.patch.object(
                subject, "_evaluate", side_effect=(replay, state_off)
            ) as evaluate,
        ):
            observed = subject._evaluate_emitted_exact(
                artifact,
                SimpleNamespace(),
                subject.DecoderConfig.production(),
                torch.zeros(32, 129, dtype=torch.long),
                device=torch.device("cpu"),
                batch_size=8,
            )
        constructor.assert_called_once()
        fresh.load_learned_artifact.assert_called_once_with(artifact)
        self.assertEqual(observed, (replay, state_off))
        self.assertNotIn("state_off", evaluate.call_args_list[0].kwargs)
        self.assertIs(evaluate.call_args_list[1].kwargs["state_off"], True)

    def test_causality_instrument_compares_the_full_shared_prefix(self) -> None:
        class FakeDecoder:
            def __init__(self, config: object, geometry: object) -> None:
                self.weight = torch.nn.Parameter(torch.tensor(0.0))

            def to(self, device: torch.device) -> FakeDecoder:
                return self

            def load_learned_artifact(self, payload: bytes) -> None:
                return None

            def named_parameters(self):
                return (("weight", self.weight),)

            def __call__(
                self,
                token_ids: torch.Tensor,
                targets: torch.Tensor | None = None,
                **kwargs: object,
            ) -> SimpleNamespace:
                logits = torch.zeros(
                    token_ids.shape[0], token_ids.shape[1], 2, dtype=torch.float32
                )
                if targets is None and bool((token_ids[:, -1] != 0).any()):
                    # Leak the changed final input into the immediately preceding
                    # output.  Comparing ``[:-2]`` misses this; ``[:-1]`` catches it.
                    logits[:, -2, 0] = 1.0
                return SimpleNamespace(
                    logits=logits,
                    loss=self.weight * 0.0,
                    final_state=torch.zeros(1),
                    audit=SimpleNamespace(work_signature=lambda: (1,)),
                )

        sequences = torch.zeros(32, 129, dtype=torch.long)
        arms = {arm: object() for arm in subject.MECHANICAL_ARMS}
        initial_exports = {arm: b"initial" for arm in subject.MECHANICAL_ARMS}
        with tempfile.TemporaryDirectory() as directory:
            with (
                mock.patch.object(
                    subject, "R4GroupAddressedRetentionDecoderV1", FakeDecoder
                ),
                mock.patch.object(subject, "_optimizer", return_value=object()),
                mock.patch.object(
                    subject,
                    "_training_step",
                    return_value=(0.0, SimpleNamespace(work_signature=lambda: (1,))),
                ),
                mock.patch.object(
                    subject, "_gradient_census", return_value={"passed": True}
                ),
                mock.patch.object(subject, "_release", return_value=None),
            ):
                execution = subject._execute_preflight(
                    Path(directory),
                    sequences,
                    sequences.clone(),
                    arms,
                    device=torch.device("cpu"),
                    telemetry=_Telemetry(),
                    config=subject.DecoderPreflightConfig.production(),
                    initial_exports=initial_exports,
                )

        causality = execution["mechanical"]["strict_prefix_causality"]
        self.assertEqual(causality["maximum_shared_prefix_logit_delta"], 1.0)
        self.assertIs(causality["passed"], False)
        self.assertEqual(execution["classification"], "UNAVAILABLE")

    def test_cli_exposes_only_prepare_and_construction_preflight(self) -> None:
        predecessor = "/tmp/frozen-group-retention"
        prepared = parser().parse_args(
            ["prepare-group-retention-decoder", "--predecessor", predecessor]
        )
        self.assertEqual(prepared.command, "prepare-group-retention-decoder")
        self.assertEqual(str(prepared.predecessor), str(Path(predecessor).resolve()))
        preflight = parser().parse_args(
            ["preflight-group-retention-decoder", "--backend", "mps"]
        )
        self.assertEqual(preflight.command, "preflight-group-retention-decoder")
        self.assertFalse(hasattr(preflight, "reveal"))


if __name__ == "__main__":
    unittest.main()
