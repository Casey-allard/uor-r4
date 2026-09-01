"""Focused geometry, structural-census, and preflight-orchestration tests."""

from __future__ import annotations

import copy
import itertools
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import torch

from r4_softmax_trainer import cli as cli_subject
from r4_softmax_trainer import group_retention_campaign as subject
from r4_softmax_trainer.group_retention import (
    GroupAddressArtifact,
    GroupRetentionConfig,
)


def _cyclic_table(order: int, identity: int) -> tuple[int, ...]:
    return tuple(
        (identity + ((left - identity) % order) + ((right - identity) % order)) % order
        for left in range(order)
        for right in range(order)
    )


def _inverse_table(table: tuple[int, ...], order: int, identity: int) -> list[int]:
    return [
        next(
            candidate
            for candidate in range(order)
            if table[element * order + candidate] == identity
            and table[candidate * order + element] == identity
        )
        for element in range(order)
    ]


def _rotated_order_classes(
    element_orders: tuple[int, ...], identity: int
) -> tuple[int, ...]:
    permutation = list(range(len(element_orders)))
    for element_order in sorted(set(element_orders)):
        members = [
            element
            for element, observed in enumerate(element_orders)
            if observed == element_order and element != identity
        ]
        if len(members) > 1:
            for offset, element in enumerate(members):
                permutation[element] = members[(offset + 1) % len(members)]
    return tuple(permutation)


def _synthetic_geometry_json() -> tuple[bytes, str]:
    """Build a schema-exact group artifact without invoking the Rust compiler."""
    order = subject.PRODUCTION_GROUP_SIZE
    identity = order - 1
    table = _cyclic_table(order, identity)
    actions = [list(table[row * order : (row + 1) * order]) for row in range(order)]
    inverses = _inverse_table(table, order, identity)
    primes = subject._first_primes(subject.PRODUCTION_VOCAB_SIZE - 1)
    leaves = [identity, *(prime % order for prime in primes)]
    support = sorted(set(leaves))
    orders = subject._element_orders(table, order=order, identity=identity)
    permutation = _rotated_order_classes(orders, identity)
    self_moved = sum(index != value for index, value in enumerate(permutation))
    assert self_moved >= 100
    transport = [permutation[leaf] for leaf in leaves]
    witness = None
    for left in range(order):
        for right in range(order):
            product = table[left * order + right]
            permuted_product = permutation[product]
            product_of_permuted = table[
                permutation[left] * order + permutation[right]
            ]
            if permuted_product != product_of_permuted:
                witness = {
                    "left": left,
                    "right": right,
                    "true_product": product,
                    "permuted_product": permuted_product,
                    "product_of_permuted": product_of_permuted,
                }
                break
        if witness is not None:
            break
    assert witness is not None

    leaf = {
        "schema": subject.LEAF_SCHEMA,
        "domain": subject.LEAF_DOMAIN,
        "policy": subject.LEAF_POLICY,
        "max_token_id": subject.PRODUCTION_VOCAB_SIZE - 1,
        "leaf_indices": leaves,
        "direct_support_indices": support,
        "direct_support_count": len(support),
        "leaf_cid": "",
    }
    leaf["leaf_cid"] = subject.cid_bytes(subject._rust_json_bytes(leaf))
    scrambled_support = sorted(set(transport))
    h4_generated = subject._generated_subgroup(
        table, order=order, identity=identity, generators=support
    )
    cyclic_generated = subject._generated_subgroup(
        table, order=order, identity=identity, generators=support
    )
    scrambled_generated = subject._generated_subgroup(
        table, order=order, identity=identity, generators=scrambled_support
    )
    root = {
        "schema": subject.GEOMETRY_SCHEMA,
        "domain": subject.GEOMETRY_DOMAIN,
        "max_token_id": subject.PRODUCTION_VOCAB_SIZE - 1,
        "group_order": order,
        "h4_root_table_kappa": "synthetic-group-root-witness",
        "h4_multiplication_table_kappa": "synthetic-group-law-witness",
        "identity_index": identity,
        "inverse_indices": inverses,
        "h4_multiplication_indices": list(table),
        "c120_inverse_indices": list(inverses),
        "c120_multiplication_indices": list(table),
        "h4_left_regular_permutations": actions,
        "c120_left_regular_permutations": copy.deepcopy(actions),
        "leaf_map": leaf,
        "scramble": {
            "schema": subject.SCRAMBLE_SCHEMA,
            "domain": subject.SCRAMBLE_DOMAIN,
            "policy": subject.SCRAMBLE_POLICY,
            "permutation": list(permutation),
            "transport_leaf_indices": transport,
            "moved_count": self_moved,
            "element_orders": list(orders),
            "identity_fixed": True,
            "element_orders_preserved": True,
            "used_leaf_order_histogram": subject._histogram_records(support, orders),
            "scrambled_used_action_order_histogram": subject._histogram_records(
                scrambled_support, orders
            ),
            "nonhomomorphism_witness": witness,
            "used_action_generated_subgroup_count": len(scrambled_generated),
        },
        "censuses": {
            "direct_leaf_support_indices": support,
            "direct_leaf_support_count": len(support),
            "direct_nonidentity_leaf_support_count": len(set(support) - {identity}),
            "identity_token_count": sum(leaf_index == identity for leaf_index in leaves),
            "h4_generated_subgroup_indices": list(h4_generated),
            "h4_generated_subgroup_count": len(h4_generated),
            "c120_generated_subgroup_indices": list(cyclic_generated),
            "c120_generated_subgroup_count": len(cyclic_generated),
            "scrambled_h4_generated_subgroup_indices": list(scrambled_generated),
            "scrambled_h4_generated_subgroup_count": len(scrambled_generated),
        },
        "artifact_cid": "",
    }
    root["artifact_cid"] = subject.cid_bytes(subject._rust_json_bytes(root))
    return subject._rust_json_bytes(root), root["artifact_cid"]


def _s3_table() -> tuple[int, ...]:
    permutations = list(itertools.permutations(range(3)))
    index = {permutation: offset for offset, permutation in enumerate(permutations)}
    values = []
    for left in permutations:
        for right in permutations:
            product = tuple(left[right[position]] for position in range(3))
            values.append(index[product])
    return tuple(values)


def _small_arms() -> dict[str, GroupAddressArtifact]:
    order = 6
    leaves = torch.arange(order, dtype=torch.long)
    exact_table = _s3_table()
    exact = torch.tensor(exact_table, dtype=torch.long).view(order, order)
    cyclic_table = _cyclic_table(order, 0)
    cyclic = torch.tensor(cyclic_table, dtype=torch.long).view(order, order)
    permutation = (0, 2, 1, 4, 5, 3)
    scrambled = exact.index_select(0, torch.tensor(permutation, dtype=torch.long))
    return {
        "exact_h4": GroupAddressArtifact("exact_h4", 0, leaves, exact, "synthetic"),
        "cyclic_120": GroupAddressArtifact("cyclic_120", 0, leaves.clone(), cyclic, "synthetic"),
        "scrambled_h4": GroupAddressArtifact(
            "scrambled_h4", 0, leaves.clone(), scrambled, "synthetic"
        ),
    }


class _FakeTelemetry:
    def synchronize(self) -> None:
        return None

    def empty_cache(self) -> None:
        return None

    def recommended_memory(self) -> int:
        return 1_000_000_000

    def allocated_memory(self) -> int:
        return 1


class GroupGeometryLoaderTests(unittest.TestCase):
    def test_schema_exact_geometry_loads_three_raw_leaf_matched_arms(self) -> None:
        raw, expected_cid = _synthetic_geometry_json()
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "geometry.json"
            path.write_bytes(raw)
            geometry = subject.load_group_geometry_artifacts(path)
        self.assertEqual(geometry.artifact_cid, expected_cid)
        self.assertEqual(geometry.exact_h4.identity_offset, 119)
        self.assertEqual(geometry.exact_h4.token_leaves[:4].tolist(), [119, 2, 3, 5])
        self.assertEqual(len(geometry.direct_support), 35)
        self.assertEqual(
            (geometry.h4_generated_count, geometry.c120_generated_count, geometry.scrambled_generated_count),
            (120, 120, 120),
        )
        self.assertTrue(
            torch.equal(geometry.exact_h4.token_leaves, geometry.scrambled_h4.token_leaves)
        )
        self.assertFalse(
            torch.equal(geometry.exact_h4.left_actions, geometry.scrambled_h4.left_actions)
        )

    def test_leaf_tamper_fails_even_with_fresh_self_cids(self) -> None:
        raw, _ = _synthetic_geometry_json()
        value = subject.json.loads(raw)
        value["leaf_map"]["leaf_indices"][1] = 7
        value["leaf_map"]["direct_support_indices"] = sorted(
            set(value["leaf_map"]["leaf_indices"])
        )
        value["leaf_map"]["direct_support_count"] = len(
            value["leaf_map"]["direct_support_indices"]
        )
        value["leaf_map"]["leaf_cid"] = ""
        value["leaf_map"]["leaf_cid"] = subject.cid_bytes(
            subject._rust_json_bytes(value["leaf_map"])
        )
        value["artifact_cid"] = ""
        value["artifact_cid"] = subject.cid_bytes(subject._rust_json_bytes(value))
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "geometry.json"
            path.write_bytes(subject._rust_json_bytes(value))
            with self.assertRaisesRegex(ValueError, "zero-based"):
                subject.load_group_geometry_artifacts(path)

    def test_prepare_copies_validated_geometry_and_is_create_once(self) -> None:
        raw, expected_cid = _synthetic_geometry_json()
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            source = base / "source"
            source.mkdir()
            geometry_path = base / "geometry.json"
            geometry_path.write_bytes(raw)
            root = base / "prepared"

            def fake_population(
                output_root: Path,
                source_root: Path,
                geometry: subject.GroupGeometryBundle,
            ) -> dict[str, object]:
                self.assertEqual(source_root, source.resolve())
                self.assertEqual(geometry.artifact_cid, expected_cid)
                output_root.mkdir(parents=True)
                (output_root / subject.TRAINING_VIEW_MANIFEST_NAME).write_bytes(b"training-view\n")
                return {
                    "population": {"status": "TEST_ONLY"},
                    "training_view": {
                        "population_manifest_cid": "blake3:" + "1" * 64,
                        "manifest_cid": "blake3:" + "2" * 64,
                    },
                }

            with mock.patch.object(
                subject,
                "prepare_group_retention_population",
                side_effect=fake_population,
            ):
                result = subject.prepare_group_retention_data(root, source, geometry_path)
            self.assertEqual(result["preparation"]["geometry_artifact_cid"], expected_cid)
            self.assertEqual((root / subject.GEOMETRY_RELATIVE_PATH).read_bytes(), raw)
            self.assertTrue((root / subject.PREPARATION_MANIFEST_NAME).is_file())
            with self.assertRaisesRegex(FileExistsError, "create-once"):
                subject.prepare_group_retention_data(root, source, geometry_path)


class StructuralSignatureTests(unittest.TestCase):
    def test_r_action_uses_prior_writes_and_true_candidate_support_only(self) -> None:
        arms = _small_arms()
        story = [1, 2, 3, 4, 5, 1, 3, 2, 4]
        result = subject._partition_signature_census(
            [story],
            arms=arms,
            direct_support=range(6),
            expected_story_tokens=None,
        )
        self.assertEqual(result["rows"], len(story) - 1)
        self.assertGreater(result["r_action"], 0)
        self.assertEqual(result["stories_with_r_action"], 1)
        self.assertEqual(result["next_token_reads"], 0)

    def test_changed_candidate_leaf_map_is_rejected(self) -> None:
        arms = _small_arms()
        changed = arms["cyclic_120"].token_leaves.clone()
        changed[1], changed[2] = changed[2].clone(), changed[1].clone()
        arms["cyclic_120"] = GroupAddressArtifact(
            "cyclic_120",
            0,
            changed,
            arms["cyclic_120"].left_actions,
            "synthetic",
        )
        with self.assertRaisesRegex(ValueError, "raw candidate-leaf"):
            subject._partition_signature_census(
                [[1, 2, 3]],
                arms=arms,
                direct_support=range(6),
                expected_story_tokens=None,
            )


class PreflightOrchestrationTests(unittest.TestCase):
    def _small_bundle(self) -> subject.GroupGeometryBundle:
        arms = _small_arms()
        return subject.GroupGeometryBundle(
            exact_h4=arms["exact_h4"],
            cyclic_120=arms["cyclic_120"],
            scrambled_h4=arms["scrambled_h4"],
            artifact_cid="blake3:" + "1" * 64,
            geometry_file_cid="blake3:" + "2" * 64,
            direct_support=tuple(range(6)),
            h4_generated_count=120,
            c120_generated_count=120,
            scrambled_generated_count=120,
        )

    def _training_view(self, *, r_action: int = 41) -> dict[str, object]:
        return {
            "population_manifest_cid": "blake3:" + "3" * 64,
            "geometry": {
                "status": "COMPUTED",
                "summary": {
                    "passed": r_action >= 41,
                    "heldout": {
                        "r_action": r_action,
                        "stories_with_r_action": 64,
                    },
                    "generated_state_coverage": {
                        "exact_h4": 120,
                        "cyclic_120": 120,
                        "scrambled_h4": 120,
                    },
                    "next_token_reads": 0,
                },
            },
        }

    def _config(self) -> subject.PreflightExecutionConfig:
        return subject.PreflightExecutionConfig(
            model=GroupRetentionConfig(
                vocab_size=6,
                hidden_size=4,
                group_size=6,
                banks=2,
                max_sequence_length=3,
                checkpoint_chunk_size=1,
                initialization_seed=9736,
            ),
            batch_size=2,
            context=3,
            warmup_steps=0,
            measured_steps=1,
            smoke_stories=2,
            smoke_steps=1,
            required_loss_reduction=0.0,
            required_state_off_delta=0.0,
            eta_ceiling_seconds=1_000_000.0,
        )

    def test_pass_publishes_frozen_authorization_without_running_main(self) -> None:
        bundle = self._small_bundle()
        training_view = self._training_view()
        preparation = {"manifest_cid": "blake3:" + "4" * 64}
        fit = torch.tensor([[1, 2, 3, 4], [2, 3, 4, 5]], dtype=torch.long)

        def executor(*args: object, **kwargs: object) -> dict[str, object]:
            return {
                "execution_path": subject.PREFLIGHT_EXECUTION_PATH,
                "use_checkpoint": False,
                "direct_recurrence_parity": "REQUIRED",
                "timing": {"passed": True},
                "memory": {"passed": True},
                "gradients": {"passed": True},
                "smoke": {"passed": True},
                "equal_operation_and_read_ledgers": True,
                "passed": True,
            }

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            patches = (
                mock.patch.object(
                    subject, "_load_prepared", return_value=(preparation, training_view, bundle)
                ),
                mock.patch.object(subject, "_load_fit_sequences", return_value=fit),
                mock.patch.object(
                    subject,
                    "trainer_implementation_contract",
                    return_value={"tree_cid": "blake3:" + "5" * 64, "files": []},
                ),
            )
            with patches[0], patches[1], patches[2]:
                result = subject.run_group_retention_preflight(
                    root,
                    _executor=executor,
                    _device_provider=lambda backend: (torch.device("cpu"), _FakeTelemetry()),
                    _execution_config=self._config(),
                )
            self.assertEqual(result["result"]["verdict"], "PASS")
            self.assertEqual(result["result"]["heldout"], {"status": "NOT_RUN", "reads": 0})
            self.assertEqual(
                result["authorization"]["authorization"],
                "ONE_SHOT_MAIN_256_STEPS_PER_ARM",
            )
            self.assertEqual(result["authorization"]["heldout"]["status"], "NOT_RUN")
            self.assertTrue((root / subject.AUTHORIZATION_RELATIVE_PATH).is_file())
            with self.assertRaisesRegex(FileExistsError, "sole #973"):
                subject.run_group_retention_preflight(
                    root,
                    _executor=executor,
                    _device_provider=lambda backend: (torch.device("cpu"), _FakeTelemetry()),
                    _execution_config=self._config(),
                )

    def test_structural_miss_is_terminal_and_executor_never_runs(self) -> None:
        bundle = self._small_bundle()
        training_view = self._training_view(r_action=40)
        preparation = {"manifest_cid": "blake3:" + "4" * 64}
        fit = torch.tensor([[1, 2, 3, 4], [2, 3, 4, 5]], dtype=torch.long)
        executor = mock.Mock()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with (
                mock.patch.object(
                    subject, "_load_prepared", return_value=(preparation, training_view, bundle)
                ),
                mock.patch.object(subject, "_load_fit_sequences", return_value=fit),
                mock.patch.object(
                    subject,
                    "trainer_implementation_contract",
                    return_value={"tree_cid": "blake3:" + "5" * 64, "files": []},
                ),
            ):
                result = subject.run_group_retention_preflight(
                    root,
                    _executor=executor,
                    _device_provider=lambda backend: (torch.device("cpu"), _FakeTelemetry()),
                    _execution_config=self._config(),
                )
            executor.assert_not_called()
            self.assertEqual(result["result"]["verdict"], subject.TERMINAL_UNAVAILABLE)
            self.assertEqual(result["result"]["main"]["status"], subject.MAIN_NOT_RUN)
            self.assertIsNone(result["authorization"])
            self.assertFalse((root / subject.AUTHORIZATION_RELATIVE_PATH).exists())

    def test_non_mps_backend_is_rejected_before_a_run_marker(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with self.assertRaisesRegex(ValueError, "CUDA are forbidden"):
                subject.run_group_retention_preflight(root, backend="cpu")
            self.assertFalse((root / subject.STARTED_RELATIVE_PATH).exists())

    def test_production_contract_selects_uncheckpointed_stationary_frame(self) -> None:
        config = subject.PreflightExecutionConfig.production()
        contract = subject._production_contract(config)["construction"]
        self.assertEqual(
            contract["execution_path"], "exact_stationary_frame_closed_form"
        )
        self.assertIs(contract["use_checkpoint"], False)
        self.assertEqual(contract["direct_recurrence_parity"], "REQUIRED")
        self.assertNotIn("activation_checkpoint_chunk", contract)
        self.assertEqual(contract["reference_checkpoint_chunk"], 16)
        self.assertIn("excluded from preflight work", contract["reference_checkpoint_role"])
        rationale = contract["execution_selection_rationale"]
        self.assertEqual(rationale["observed_current_memory_bytes"], 1_597_398_528)
        self.assertEqual(rationale["observed_driver_memory_bytes"], 3_521_118_208)
        self.assertEqual(
            rationale["observed_recommended_memory_bytes"], 12_713_115_648
        )
        self.assertTrue(rationale["driver_below_recommended"])
        self.assertIn("query and record fresh", rationale["binding_run_requirement"])

    def test_training_and_loss_helpers_disable_checkpoint_execution(self) -> None:
        config = self._config()
        model = subject.R4GroupAddressedRetentionLMV1(
            config.model, _small_arms()["exact_h4"]
        )
        optimizer = subject._optimizer(model, config)
        inputs = torch.tensor([[1, 2, 3], [2, 3, 4]], dtype=torch.long)
        targets = torch.tensor([[2, 3, 4], [3, 4, 5]], dtype=torch.long)
        with mock.patch.object(model, "forward", wraps=model.forward) as forward:
            _, audit = subject._one_training_step(
                model, optimizer, inputs, targets, config
            )
            self.assertIs(forward.call_args.kwargs["use_checkpoint"], False)
        self.assertEqual(audit.checkpoint_chunks, 0)

        with mock.patch.object(model, "forward", wraps=model.forward) as forward:
            _, audit = subject._loss(model, inputs, targets)
            self.assertIs(forward.call_args.kwargs["use_checkpoint"], False)
        self.assertEqual(audit.checkpoint_chunks, 0)


class GroupRetentionCliTests(unittest.TestCase):
    def test_prepare_and_preflight_commands_parse_the_frozen_arguments(self) -> None:
        prepare = cli_subject.parser().parse_args(
            [
                "--root",
                "/tmp/group-retention",
                "prepare-group-retention",
                "--source-root",
                "/tmp/source",
                "--geometry",
                "/tmp/geometry.json",
            ]
        )
        self.assertEqual(prepare.command, "prepare-group-retention")
        self.assertEqual(prepare.root, Path("/tmp/group-retention").resolve())
        self.assertEqual(prepare.source_root, Path("/tmp/source").resolve())
        self.assertEqual(prepare.geometry, Path("/tmp/geometry.json").resolve())

        preflight = cli_subject.parser().parse_args(
            ["preflight-group-retention", "--backend", "mps"]
        )
        self.assertEqual(preflight.command, "preflight-group-retention")
        self.assertEqual(preflight.backend, "mps")


if __name__ == "__main__":
    unittest.main()
