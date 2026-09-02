from __future__ import annotations

import copy
import math
import tempfile
import unittest
from pathlib import Path
from unittest.mock import Mock, patch

import torch

from r4_softmax_trainer import predictive_block_delta_campaign as campaign


def _score(
    intervention: str,
    *,
    gain: float,
    wins: int,
    work: tuple[int, ...] = (64, 64, 4096),
    forbidden_reads: int = 0,
) -> campaign.ProbeScore:
    return campaign.ProbeScore(
        intervention=intervention,
        directions=campaign.PROBE_DIRECTIONS,
        targets=campaign.PROBE_TARGETS,
        mean_gain_nats_per_token=gain,
        wins=wins,
        own_nll_nats_per_token=2.0,
        foreign_nll_nats_per_token=2.0 + gain,
        maximum_head_logits=1.0,
        forbidden_reads=forbidden_reads,
        work_signature=work,
        trace_cid="blake3:" + "1" * 64,
    )


class _DisposableModel:
    def __init__(self) -> None:
        self.values = [
            torch.nn.Parameter(torch.ones(campaign.TRAINABLE_PARAMETERS - 3)),
            torch.nn.Parameter(torch.ones(3)),
        ]

    def trainable_parameters(self):
        return tuple(self.values)


class _TransportModel:
    def __init__(self) -> None:
        order = 120
        angles = torch.arange(order, dtype=torch.float32) * (2.0 * math.pi / order)
        self.frame_matrices = torch.eye(4).repeat(order, 1, 1)
        self.frame_matrices[:, 0, 0] = torch.cos(angles)
        self.frame_matrices[:, 0, 1] = -torch.sin(angles)
        self.frame_matrices[:, 1, 0] = torch.sin(angles)
        self.frame_matrices[:, 1, 1] = torch.cos(angles)
        self.frame_multiplication = torch.tensor(
            [
                [(left + right) % order for right in range(order)]
                for left in range(order)
            ],
            dtype=torch.long,
        )

    def _step_transport(self, leaves, *, intervention):
        if intervention != "native":
            raise AssertionError("test transport accepts only native")
        return self.frame_matrices.index_select(0, leaves).transpose(-1, -2)


def _valid_cached_result() -> dict:
    mechanics = {
        "all_frame_identity_maximum_delta": 0.0,
        "all_frame_step_connection_maximum_delta": 0.0,
        "transported_matrix_read_covariance_maximum_delta": 0.0,
        "strict_causal_prefix_maximum_logits_delta": 0.0,
        "unobserved_target_mutation_maximum_prefix_delta": 0.0,
        "state_off_v1_maximum_logits_delta": 0.0,
        "artifact_replay_maximum_logits_delta": 0.0,
        "transport_permutation_head_effect": 0.1,
        "binding_observable_maximum_head_logits": 0.1,
        "equal_geometric_plain_intervention_work": True,
        "forbidden_reads": 0,
        "passed": True,
        "gradient_values_seen": campaign.TRAINABLE_PARAMETERS,
        "gradient_values_required": campaign.TRAINABLE_PARAMETERS,
        "all_trainable_values_received_finite_nonzero_gradient": True,
        "qualified_base_unchanged": True,
    }
    native = _score("native", gain=0.0, wins=0)
    additive = _score("no_delta", gain=0.0, wins=0)
    state_off = _score("state_off", gain=0.0, wins=0)
    decision = campaign.admission_decision(
        native=native,
        additive=additive,
        state_off=state_off,
        mechanics=mechanics,
    )
    return campaign._with_self_cid(
        {
            "schema": campaign.RESULT_SCHEMA,
            "issue": campaign.ISSUE,
            "policy": campaign.POLICY,
            "model_policy": campaign.MODEL_POLICY,
            "implementation": campaign.trainer_implementation_contract(),
            "execution": {
                "device": "cpu",
                "torch_intraop_threads": torch.get_num_threads(),
                "torch_interop_threads": torch.get_num_interop_threads(),
                "total_elapsed_seconds": 2.0,
            },
            "inputs": {
                "predecessor": {
                    "policy": campaign.PREDECESSOR_POLICY,
                    "result_cid": campaign.PREDECESSOR_RESULT_CID,
                    "artifact_cid": campaign.PREDECESSOR_ARTIFACT_CID,
                    "artifact_bytes": campaign.PREDECESSOR_ARTIFACT_BYTES,
                },
                "revealed_v4": {
                    "population_cid": campaign.V4_POPULATION_CID,
                    "commitment_cid": campaign.V4_COMMITMENT_CID,
                    "reveal_cid": campaign.V4_REVEAL_CID,
                    "pairs": campaign.PROBE_PAIRS,
                    "directions": campaign.PROBE_DIRECTIONS,
                    "targets": campaign.PROBE_TARGETS,
                },
                "h4_spin_frames": {
                    "artifact_cid": campaign.H4_FRAME_ARTIFACT_CID,
                    "file_cid": campaign.H4_FRAME_FILE_CID,
                    "root_table_kappa": campaign.ROOT_TABLE_KAPPA,
                    "multiplication_table_kappa": campaign.PRODUCT_TABLE_KAPPA,
                },
            },
            "dose": {
                "pairs": campaign.PROBE_PAIRS,
                "directions": campaign.PROBE_DIRECTIONS,
                "targets": campaign.PROBE_TARGETS,
                "maximum_updates": campaign.MAXIMUM_UPDATES,
                "completed_updates": 1,
                "cuda": "FORBIDDEN",
            },
            "mechanics": mechanics,
            "fit": {
                "updates": 1,
                "elapsed_seconds": 1.0,
                "final_loss": 2.0,
                "final_gradient_norm": 1.0,
                "gradient_values_seen": campaign.TRAINABLE_PARAMETERS,
                "gradient_values_required": campaign.TRAINABLE_PARAMETERS,
                "all_trainable_values_received_finite_nonzero_gradient": True,
                "qualified_base_unchanged": True,
            },
            "scores": {
                "full_delta": native.record(),
                "additive_no_overwrite": additive.record(),
                "state_off": state_off.record(),
            },
            "decision": decision,
            "verdict": decision["verdict"],
            "admitted": decision["admitted"],
            "disposable_weights": {
                "status": "DESTROYED_IN_MEMORY_NO_ARTIFACT",
                "values": campaign.TRAINABLE_PARAMETERS,
            },
            "production_v5": {
                "authorized": decision["admitted"],
                "created": False,
                "inspected": False,
                "selector": "NOT_IMPLEMENTED_IN_PREFLIGHT_MODULE",
            },
            "writer_process_id": 1,
        },
        "result_cid",
    )


def _resign(value: dict) -> dict:
    unsigned = copy.deepcopy(value)
    unsigned.pop("result_cid", None)
    return campaign._with_self_cid(unsigned, "result_cid")


class PredictiveBlockDeltaCampaignTests(unittest.TestCase):
    def test_frozen_probe_dose_and_thresholds(self) -> None:
        self.assertEqual(campaign.PROBE_PAIRS, 32)
        self.assertEqual(campaign.PROBE_DIRECTIONS, 64)
        self.assertEqual(campaign.PROBE_TARGETS, 1_024)
        self.assertEqual(campaign.MAXIMUM_UPDATES, 256)
        self.assertEqual(campaign.TRAINABLE_PARAMETERS, 9_228)
        self.assertEqual(
            campaign.V1_IMPLEMENTATION_TREE_CID,
            "blake3:317aa564eb2041c37768186d81691666a19ce01c168e4ab74a0b966d761212d2",
        )
        self.assertAlmostEqual(
            campaign.ABSOLUTE_GAIN_THRESHOLD,
            math.log(2.0) / 16,
        )
        self.assertAlmostEqual(
            campaign.INTERVENTION_LOSS_THRESHOLD,
            math.log(1.5) / 16,
        )

    def test_admission_requires_absolute_wins_delta_and_state(self) -> None:
        native_gain = campaign.ABSOLUTE_GAIN_THRESHOLD + 0.01
        control_gain = native_gain - campaign.INTERVENTION_LOSS_THRESHOLD - 0.01
        decision = campaign.admission_decision(
            native=_score("native", gain=native_gain, wins=52),
            additive=_score("no_delta", gain=control_gain, wins=20),
            state_off=_score("state_off", gain=control_gain, wins=20),
            mechanics={"passed": True},
        )
        self.assertTrue(decision["admitted"])
        self.assertEqual(decision["verdict"], campaign.VERDICT_ADMIT)

        no_delta_loss_missing = campaign.admission_decision(
            native=_score("native", gain=native_gain, wins=52),
            additive=_score("no_delta", gain=native_gain, wins=20),
            state_off=_score("state_off", gain=control_gain, wins=20),
            mechanics={"passed": True},
        )
        self.assertFalse(no_delta_loss_missing["admitted"])
        self.assertEqual(no_delta_loss_missing["verdict"], campaign.VERDICT_REJECT)

    def test_integrity_failure_is_invalid_not_scientific_rejection(self) -> None:
        gain = campaign.ABSOLUTE_GAIN_THRESHOLD + 0.1
        decision = campaign.admission_decision(
            native=_score("native", gain=gain, wins=64),
            additive=_score("no_delta", gain=0.0, wins=0, work=(1,)),
            state_off=_score("state_off", gain=0.0, wins=0),
            mechanics={"passed": True},
        )
        self.assertFalse(decision["admitted"])
        self.assertEqual(decision["verdict"], campaign.VERDICT_INVALID)

    def test_disposable_weights_are_zeroed_and_counted(self) -> None:
        model = _DisposableModel()
        self.assertEqual(
            campaign.destroy_disposable_weights(model),
            campaign.TRAINABLE_PARAMETERS,
        )
        self.assertTrue(
            all(torch.count_nonzero(value).item() == 0 for value in model.values)
        )

    def test_wrong_weight_count_fails_closed(self) -> None:
        model = _DisposableModel()
        model.values = [torch.nn.Parameter(torch.ones(1))]
        with self.assertRaisesRegex(RuntimeError, "count differs"):
            campaign.destroy_disposable_weights(model)

    def test_all_frame_connection_and_matrix_read_covariance(self) -> None:
        model = _TransportModel()
        checks = campaign.transport_mechanics(model, device=torch.device("cpu"))
        self.assertLessEqual(checks["all_frame_identity_maximum_delta"], 2e-5)
        self.assertLessEqual(
            checks["all_frame_step_connection_maximum_delta"], 2e-5
        )
        self.assertLessEqual(
            checks["transported_matrix_read_covariance_maximum_delta"], 2e-5
        )

        model._step_transport = lambda leaves, *, intervention: (  # type: ignore[method-assign]
            model.frame_matrices.index_select(0, leaves)
        )
        broken = campaign.transport_mechanics(model, device=torch.device("cpu"))
        self.assertGreater(broken["all_frame_step_connection_maximum_delta"], 0.1)

    def test_cached_result_is_verified_without_reopening_v4(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            result_path = root / campaign.RESULT_RELATIVE_PATH
            result_path.parent.mkdir(parents=True)
            result = _valid_cached_result()
            result_path.write_bytes(campaign.canonical_json_bytes(result))
            loader = Mock(side_effect=AssertionError("V4 must not reopen"))
            with (
                patch.object(campaign, "load_frozen_probe_inputs", loader),
                patch.object(
                    campaign,
                    "V1_IMPLEMENTATION_TREE_CID",
                    result["implementation"]["tree_cid"],
                ),
                patch.object(
                    campaign,
                    "trainer_implementation_contract",
                    side_effect=AssertionError("historical result must not read current tree"),
                ),
            ):
                observed = campaign.run_predictive_block_delta_preflight(
                    root=root,
                    predecessor_root=root / "predecessor",
                    revealed_v4_root=root / "revealed-v4",
                    frame_sidecar_path=root / "h4-spin-frame.json",
                )
            self.assertEqual(
                campaign.canonical_json_bytes(observed),
                campaign.canonical_json_bytes(result),
            )
            loader.assert_not_called()

    def test_truncated_and_self_consistent_tampered_results_fail_closed(self) -> None:
        valid = _valid_cached_result()
        cases: list[tuple[str, dict]] = []

        truncated = copy.deepcopy(valid)
        truncated.pop("scores")
        cases.append(("truncated", _resign(truncated)))

        planted = copy.deepcopy(valid)
        planted["implementation"] = {"files": [], "tree_cid": "blake3:" + "c" * 64}
        cases.append(("planted implementation", _resign(planted)))

        score_tamper = copy.deepcopy(valid)
        score_tamper["scores"]["full_delta"]["mean_gain_nats_per_token"] = 0.1
        score_tamper["scores"]["full_delta"]["foreign_nll_nats_per_token"] = 2.1
        score_tamper["scores"]["full_delta"]["wins"] = 64
        cases.append(("stale decision", _resign(score_tamper)))

        fit_tamper = copy.deepcopy(valid)
        fit_tamper["fit"]["gradient_values_seen"] = 1
        cases.append(("fit ledger", _resign(fit_tamper)))

        production_tamper = copy.deepcopy(valid)
        production_tamper["production_v5"]["authorized"] = True
        cases.append(("production authorization", _resign(production_tamper)))

        execution_tamper = copy.deepcopy(valid)
        execution_tamper["execution"]["torch_intraop_threads"] = 0
        cases.append(("execution threads", _resign(execution_tamper)))

        wall_tamper = copy.deepcopy(valid)
        wall_tamper["execution"]["total_elapsed_seconds"] = (
            campaign.HARD_WALL_SECONDS + 0.1
        )
        cases.append(("whole-gate wall", _resign(wall_tamper)))

        h4_tamper = copy.deepcopy(valid)
        h4_tamper["inputs"]["h4_spin_frames"]["artifact_cid"] = (
            "blake3:" + "d" * 64
        )
        cases.append(("planted H4 CID", _resign(h4_tamper)))

        for label, result in cases:
            with self.subTest(label=label), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                result_path = root / campaign.RESULT_RELATIVE_PATH
                result_path.parent.mkdir(parents=True)
                result_path.write_bytes(campaign.canonical_json_bytes(result))
                loader = Mock(side_effect=AssertionError("V4 must not reopen"))
                factory = Mock(side_effect=AssertionError("model must not construct"))
                with (
                    patch.object(campaign, "load_frozen_probe_inputs", loader),
                    patch.object(
                        campaign,
                        "V1_IMPLEMENTATION_TREE_CID",
                        valid["implementation"]["tree_cid"],
                    ),
                ):
                    with self.assertRaises(ValueError):
                        campaign.run_predictive_block_delta_preflight(
                            root=root,
                            predecessor_root=root / "predecessor",
                            revealed_v4_root=root / "revealed-v4",
                            frame_sidecar_path=root / "h4-spin-frame.json",
                            model_factory=factory,
                        )
                loader.assert_not_called()
                factory.assert_not_called()

    def test_corrupt_cached_result_fails_before_any_model_construction(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            result_path = root / campaign.RESULT_RELATIVE_PATH
            result_path.parent.mkdir(parents=True)
            result_path.write_text("{}\n", encoding="utf-8")
            factory = Mock(side_effect=AssertionError("model must not construct"))
            with self.assertRaisesRegex(ValueError, "result_cid"):
                campaign.run_predictive_block_delta_preflight(
                    root=root,
                    predecessor_root=root / "predecessor",
                    revealed_v4_root=root / "revealed-v4",
                    frame_sidecar_path=root / "h4-spin-frame.json",
                    model_factory=factory,
                )
            factory.assert_not_called()


if __name__ == "__main__":
    unittest.main()
