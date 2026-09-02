from __future__ import annotations

import copy
import math
import tempfile
import unittest
from pathlib import Path
from unittest.mock import Mock, patch

from r4_softmax_trainer import predictive_block_delta_campaign_v2 as campaign


def _score(
    intervention: str,
    *,
    gain: float,
    wins: int,
    own_nll: float = 2.0,
    work: tuple[int, ...] = (64, 64, 4096),
    forbidden_reads: int = 0,
) -> campaign.ProbeScore:
    return campaign.ProbeScore(
        intervention=intervention,
        directions=campaign.PROBE_DIRECTIONS,
        targets=campaign.PROBE_TARGETS,
        mean_gain_nats_per_token=gain,
        wins=wins,
        own_nll_nats_per_token=own_nll,
        foreign_nll_nats_per_token=own_nll + gain,
        maximum_head_logits=1.0,
        forbidden_reads=forbidden_reads,
        work_signature=work,
        trace_cid="blake3:" + "1" * 64,
    )


def _synthetic_selector() -> dict:
    unsigned = {
        "schema": campaign.SELECTOR_SCHEMA,
        "v4_population_cid": campaign.V4_POPULATION_CID,
        "pair_start": campaign.PAIR_START,
        "pair_end_exclusive": campaign.PAIR_STOP,
        "pairs": [
            {
                "pair_index": pair_index,
                "left_source_story_ordinal": 400_000 + pair_index * 2,
                "left_story_cid": "blake3:" + f"{pair_index + 1:064x}",
                "right_source_story_ordinal": 400_001 + pair_index * 2,
                "right_story_cid": "blake3:" + f"{pair_index + 101:064x}",
            }
            for pair_index in range(campaign.PAIR_START, campaign.PAIR_STOP)
        ],
    }
    return campaign._with_self_cid(unsigned, "selector_cid")


def _mechanics() -> dict:
    return {
        "all_frame_identity_maximum_delta": 0.0,
        "all_frame_step_connection_maximum_delta": 0.0,
        "transported_matrix_read_covariance_maximum_delta": 0.0,
        "full_delta_strict_causal_prefix_maximum_logits_delta": 0.0,
        "full_delta_unobserved_target_mutation_maximum_prefix_delta": 0.0,
        "additive_strict_causal_prefix_maximum_logits_delta": 0.0,
        "additive_unobserved_target_mutation_maximum_prefix_delta": 0.0,
        "state_off_v1_maximum_logits_delta": 0.0,
        "full_delta_artifact_replay_maximum_logits_delta": 0.0,
        "additive_artifact_replay_maximum_logits_delta": 0.0,
        "transport_permutation_head_effect": 0.1,
        "full_delta_binding_observable_maximum_head_logits": 0.1,
        "additive_binding_observable_maximum_head_logits": 0.1,
        "equal_runtime_intervention_work": True,
        "equal_probe_work": True,
        "forbidden_reads": 0,
        "probe_forbidden_reads": 0,
        "initial_binding_values_byte_identical": True,
        "initial_qualified_base_byte_identical": True,
        "equal_optimizer_batch_update_work": True,
        "full_delta_complete_gradient_coverage": True,
        "additive_complete_gradient_coverage": True,
        "both_qualified_bases_unchanged": True,
        "full_delta_replay_values_destroyed": campaign.TRAINABLE_PARAMETERS,
        "additive_replay_values_destroyed": campaign.TRAINABLE_PARAMETERS,
        "passed": True,
    }


def _fit(intervention: campaign.FitIntervention) -> dict:
    initial = "blake3:" + "2" * 64
    fitted_digit = "3" if intervention == "native" else "4"
    return {
        "intervention": intervention,
        "updates": campaign.MAXIMUM_UPDATES,
        "elapsed_seconds": 0.5,
        "final_loss": 2.0,
        "final_gradient_norm": 1.0,
        "gradient_values_seen": campaign.TRAINABLE_PARAMETERS,
        "gradient_values_required": campaign.TRAINABLE_PARAMETERS,
        "all_trainable_values_received_finite_nonzero_gradient": True,
        "qualified_base_unchanged": True,
        "initial_binding_cid": initial,
        "fitted_binding_cid": "blake3:" + fitted_digit * 64,
        "batch_schedule_cid": campaign._batch_schedule_cid(
            campaign.PROBE_DIRECTIONS, campaign.MAXIMUM_UPDATES
        ),
    }


def _valid_cached_result() -> tuple[dict, str]:
    selector = _synthetic_selector()
    mechanics = _mechanics()
    full = _score(
        "native",
        gain=campaign.ABSOLUTE_GAIN_THRESHOLD + 0.02,
        wins=52,
    )
    additive = _score("no_delta", gain=0.0, wins=20)
    state_off = _score("state_off", gain=0.0, wins=20)
    native = campaign.native_capacity_decision(
        full=full, state_off=state_off, mechanics=mechanics
    )
    attribution = campaign.additive_attribution_decision(
        full=full, additive=additive, state_off=state_off
    )
    implementation_files = [
        {
            "path": "src/r4_softmax_trainer/predictive_block_delta_campaign_v2.py",
            "bytes": 1,
            "cid": "blake3:" + "9" * 64,
        }
    ]
    implementation = {
        "files": implementation_files,
        "tree_cid": campaign.tree_cid(implementation_files),
    }
    result = campaign._with_self_cid(
        {
            "schema": campaign.RESULT_SCHEMA,
            "issue": campaign.ISSUE,
            "policy": campaign.POLICY,
            "model_policy": campaign.MODEL_POLICY,
            "implementation": implementation,
            "execution": {
                "device": "cpu",
                "torch_intraop_threads": campaign.CPU_THREADS,
                "torch_interop_threads": 1,
                "total_elapsed_seconds": 2.0,
            },
            "inputs": {
                "predictive_v1": {
                    "result_cid": campaign.V1_RESULT_CID,
                    "verdict": campaign.V1_VERDICT,
                    "admitted": False,
                    "implementation_tree_cid": campaign.V1_IMPLEMENTATION_TREE_CID,
                },
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
                    "pair_start": campaign.PAIR_START,
                    "pair_stop_exclusive": campaign.PAIR_STOP,
                    "pairs": campaign.PROBE_PAIRS,
                    "directions": campaign.PROBE_DIRECTIONS,
                    "targets": campaign.PROBE_TARGETS,
                    "slice_records_cid": campaign.SLICE_RECORDS_CID,
                    "ordered_identities_cid": campaign.SLICE_IDENTITIES_CID,
                },
                "selector": selector,
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
                "maximum_updates_per_arm": campaign.MAXIMUM_UPDATES,
                "batch_directions": campaign.BATCH_DIRECTIONS,
                "optimizer": {
                    "name": "AdamW",
                    "learning_rate": campaign.LEARNING_RATE,
                    "betas": list(campaign.ADAM_BETAS),
                    "epsilon": campaign.ADAM_EPSILON,
                    "weight_decay": 0.0,
                    "gradient_clip": campaign.GRADIENT_CLIP,
                },
                "cuda": "FORBIDDEN",
            },
            "mechanics": mechanics,
            "fits": {
                "full_delta": _fit("native"),
                "additive_no_overwrite": _fit("no_delta"),
            },
            "scores": {
                "full_delta": full.record(),
                "additive_no_overwrite": additive.record(),
                "state_off": state_off.record(),
            },
            "native_capacity": native,
            "additive_attribution": attribution,
            "verdict": native["verdict"],
            "admitted": native["admitted"],
            "disposable_weights": {
                "status": "DESTROYED_IN_MEMORY_NO_ARTIFACT",
                "full_delta_values": campaign.TRAINABLE_PARAMETERS,
                "additive_no_overwrite_values": campaign.TRAINABLE_PARAMETERS,
                "artifacts_written": 0,
            },
            "production_v5": {
                "authorized": native["admitted"],
                "created": False,
                "inspected": False,
                "selector": "NOT_IMPLEMENTED_IN_V2_PREFLIGHT_MODULE",
            },
            "writer_process_id": 1,
        },
        "result_cid",
    )
    return result, selector["selector_cid"]


def _resign(value: dict) -> dict:
    unsigned = copy.deepcopy(value)
    unsigned.pop("result_cid", None)
    return campaign._with_self_cid(unsigned, "result_cid")


class _CheapModel:
    def export_binding_artifact(self) -> bytes:
        return b"byte-identical-binding"

    def export_qualified_base_artifact(self) -> bytes:
        return b"byte-identical-qualified-base"


class PredictiveBlockDeltaCampaignV2Tests(unittest.TestCase):
    def test_public_freeze_constants(self) -> None:
        self.assertEqual((campaign.PAIR_START, campaign.PAIR_STOP), (32, 64))
        self.assertEqual(campaign.PROBE_PAIRS, 32)
        self.assertEqual(campaign.PROBE_DIRECTIONS, 64)
        self.assertEqual(campaign.PROBE_TARGETS, 1_024)
        self.assertEqual(campaign.MAXIMUM_UPDATES, 256)
        self.assertEqual(campaign.CPU_THREADS, 8)
        self.assertEqual(
            campaign.V2_IMPLEMENTATION_TREE_CID,
            "blake3:603791e7b74a682855507797a6ab3533e36f963914fc2d307549e97e87bde366",
        )
        self.assertEqual(
            campaign.SELECTOR_CID,
            "blake3:285be20c9c41267dbf925ea7d24d198b41a9014653ff62b1bdb64c8e2ee4fd5a",
        )
        self.assertAlmostEqual(
            campaign.ABSOLUTE_GAIN_THRESHOLD, math.log(2.0) / 16
        )
        self.assertAlmostEqual(
            campaign.INTERVENTION_LOSS_THRESHOLD, math.log(1.5) / 16
        )

    def test_native_capacity_alone_authorizes_v5(self) -> None:
        full = _score(
            "native",
            gain=campaign.ABSOLUTE_GAIN_THRESHOLD + 0.02,
            wins=52,
        )
        state_off = _score("state_off", gain=0.0, wins=0)
        native = campaign.native_capacity_decision(
            full=full, state_off=state_off, mechanics={"passed": True}
        )
        self.assertTrue(native["admitted"])
        self.assertEqual(native["verdict"], campaign.NATIVE_ADMIT)

        unstable_additive = campaign.additive_attribution_decision(
            full=full,
            additive=_score("no_delta", gain=0.1, wins=64, own_nll=3.0),
            state_off=state_off,
        )
        self.assertEqual(
            unstable_additive["verdict"], campaign.ADDITIVE_NO_STABLE_CAPACITY
        )
        self.assertFalse(
            unstable_additive["delta_prompt_specific_superiority"]
        )

    def test_additive_attribution_requires_valid_language_and_both_gates(self) -> None:
        state_off = _score("state_off", gain=0.0, wins=0)
        full = _score("native", gain=0.06, wins=64, own_nll=2.0)
        attributed = campaign.additive_attribution_decision(
            full=full,
            additive=_score("no_delta", gain=0.0, wins=0, own_nll=2.0),
            state_off=state_off,
        )
        self.assertEqual(attributed["verdict"], campaign.DELTA_SUPERIORITY)

        worse_native_nll = campaign.additive_attribution_decision(
            full=_score("native", gain=0.06, wins=64, own_nll=2.01),
            additive=_score("no_delta", gain=0.0, wins=0, own_nll=2.0),
            state_off=state_off,
        )
        self.assertEqual(
            worse_native_nll["verdict"],
            campaign.DELTA_SUPERIORITY_NOT_ESTABLISHED,
        )

    def test_strict_cached_result_reproduces_both_decisions(self) -> None:
        result, selector_cid = _valid_cached_result()
        with (
            patch.object(campaign, "SELECTOR_CID", selector_cid),
            patch.object(
                campaign,
                "V2_IMPLEMENTATION_TREE_CID",
                result["implementation"]["tree_cid"],
            ),
        ):
            campaign._validate_cached_result(result)

    def test_historical_v2_tree_ignores_later_package_files_but_tampering_fails(self) -> None:
        result, selector_cid = _valid_cached_result()
        historical_tree = result["implementation"]["tree_cid"]
        with (
            patch.object(campaign, "SELECTOR_CID", selector_cid),
            patch.object(campaign, "V2_IMPLEMENTATION_TREE_CID", historical_tree),
            patch.object(
                campaign,
                "trainer_implementation_contract",
                side_effect=AssertionError("current post-V2 tree must not be consulted"),
            ),
        ):
            campaign._validate_cached_result(result)

        tampered = copy.deepcopy(result)
        tampered["implementation"]["files"][0]["bytes"] = 2
        tampered = _resign(tampered)
        with (
            patch.object(campaign, "SELECTOR_CID", selector_cid),
            patch.object(campaign, "V2_IMPLEMENTATION_TREE_CID", historical_tree),
            self.assertRaises(ValueError),
        ):
            campaign._validate_cached_result(tampered)

    def test_structural_v2_validator_accepts_current_tree_for_clean_root_runs(self) -> None:
        campaign._validate_historical_v2_implementation(
            campaign.trainer_implementation_contract()
        )

    def test_tampered_selector_fit_and_authorization_fail_closed(self) -> None:
        valid, selector_cid = _valid_cached_result()
        mutations = []

        selector = copy.deepcopy(valid)
        unsigned_selector = copy.deepcopy(selector["inputs"]["selector"])
        unsigned_selector.pop("selector_cid")
        unsigned_selector["pairs"][0]["left_story_cid"] = "blake3:" + "a" * 64
        selector["inputs"]["selector"] = campaign._with_self_cid(
            unsigned_selector, "selector_cid"
        )
        mutations.append(_resign(selector))

        for updates in (1, 255):
            unmatched_fit = copy.deepcopy(valid)
            unmatched_fit["fits"]["additive_no_overwrite"]["updates"] = updates
            unmatched_fit["fits"]["additive_no_overwrite"][
                "batch_schedule_cid"
            ] = campaign._batch_schedule_cid(campaign.PROBE_DIRECTIONS, updates)
            mutations.append(_resign(unmatched_fit))

        unauthorized = copy.deepcopy(valid)
        unauthorized["production_v5"]["authorized"] = not valid["admitted"]
        mutations.append(_resign(unauthorized))

        with (
            patch.object(campaign, "SELECTOR_CID", selector_cid),
            patch.object(
                campaign,
                "V2_IMPLEMENTATION_TREE_CID",
                valid["implementation"]["tree_cid"],
            ),
        ):
            for mutation in mutations:
                with self.subTest(mutation=mutation["result_cid"]):
                    with self.assertRaises(ValueError):
                        campaign._validate_cached_result(mutation)

    def test_valid_cached_result_does_not_reopen_any_input(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            result, selector_cid = _valid_cached_result()
            result_path = root / campaign.RESULT_RELATIVE_PATH
            result_path.parent.mkdir(parents=True)
            result_path.write_bytes(campaign.canonical_json_bytes(result))
            loader = Mock(side_effect=AssertionError("frozen inputs must not reopen"))
            with (
                patch.object(campaign, "SELECTOR_CID", selector_cid),
                patch.object(
                    campaign,
                    "V2_IMPLEMENTATION_TREE_CID",
                    result["implementation"]["tree_cid"],
                ),
                patch.object(campaign, "load_frozen_v2_inputs", loader),
            ):
                observed = campaign.run_predictive_block_delta_v2_preflight(
                    root=root,
                    predecessor_root=root / "predecessor",
                    revealed_v4_root=root / "V4",
                    frame_sidecar_path=root / "H4.json",
                    v1_result_path=root / "V1.json",
                )
            self.assertEqual(observed["result_cid"], result["result_cid"])
            loader.assert_not_called()

    def test_wrong_selector_cannot_be_relabelled_with_a_valid_cid(self) -> None:
        selector = _synthetic_selector()
        self.assertRegex(selector["selector_cid"], r"^blake3:[0-9a-f]{64}$")
        with self.assertRaisesRegex(ValueError, "public freeze"):
            # These structurally valid pairs are not the frozen public selector.
            class Side:
                def __init__(self, ordinal: int, cid: str) -> None:
                    self.source_story_ordinal = ordinal
                    self.story_cid = cid

            class Pair:
                def __init__(self, index: int) -> None:
                    self.pair_index = index
                    self.left = Side(index * 2, "blake3:" + f"{index + 1:064x}")
                    self.right = Side(index * 2 + 1, "blake3:" + f"{index + 2:064x}")

            campaign._selector([Pair(index) for index in range(32, 64)])

    def test_submaximum_runner_dose_fails_before_result_input_or_model_access(
        self,
    ) -> None:
        for updates in (1, 255):
            with (
                self.subTest(updates=updates),
                tempfile.TemporaryDirectory() as directory,
            ):
                root = Path(directory)
                loader = Mock(side_effect=AssertionError("inputs must not open"))
                factory = Mock(side_effect=AssertionError("model must not construct"))
                reader = Mock(side_effect=AssertionError("result must not open"))
                with (
                    patch.object(campaign, "load_frozen_v2_inputs", loader),
                    patch.object(campaign, "_read_canonical_json", reader),
                ):
                    with self.assertRaisesRegex(ValueError, "exactly 256"):
                        campaign.run_predictive_block_delta_v2_preflight(
                            root=root,
                            predecessor_root=root / "predecessor",
                            revealed_v4_root=root / "V4",
                            frame_sidecar_path=root / "H4.json",
                            v1_result_path=root / "V1.json",
                            maximum_updates=updates,
                            model_factory=factory,
                        )
                loader.assert_not_called()
                factory.assert_not_called()
                reader.assert_not_called()
                self.assertFalse((root / campaign.RESULT_RELATIVE_PATH).exists())

    def test_exact_256_runner_dose_reaches_create_once_result_with_cheap_mocks(
        self,
    ) -> None:
        full = _score(
            "native",
            gain=campaign.ABSOLUTE_GAIN_THRESHOLD + 0.02,
            wins=52,
        )
        additive = _score("no_delta", gain=0.0, wins=0)
        state_off = _score("state_off", gain=0.0, wins=0)
        frozen = campaign.FrozenV2Inputs(
            predecessor=None,
            predecessor_artifact_path=Path("/not-opened"),
            frames=None,  # type: ignore[arg-type]
            pairs=(),
            records={},
        )

        def fitted(*_args, **kwargs):
            return _fit(kwargs["intervention"])

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            factory = Mock(side_effect=(_CheapModel(), _CheapModel()))
            validator = Mock()
            with (
                patch.object(campaign, "load_frozen_v2_inputs", return_value=frozen),
                patch.object(campaign.torch, "get_num_threads", return_value=8),
                patch.object(
                    campaign.torch, "get_num_interop_threads", return_value=8
                ),
                patch.object(campaign, "fit_independent_arm", side_effect=fitted) as fit,
                patch.object(campaign, "fitted_mechanics", return_value=_mechanics()),
                patch.object(
                    campaign,
                    "score_probe",
                    side_effect=(full, additive, state_off),
                ),
                patch.object(
                    campaign,
                    "destroy_disposable_weights",
                    return_value=campaign.TRAINABLE_PARAMETERS,
                ),
                patch.object(campaign, "_validate_cached_result", validator),
            ):
                result = campaign.run_predictive_block_delta_v2_preflight(
                    root=root,
                    predecessor_root=root / "predecessor",
                    revealed_v4_root=root / "V4",
                    frame_sidecar_path=root / "H4.json",
                    v1_result_path=root / "V1.json",
                    maximum_updates=campaign.MAXIMUM_UPDATES,
                    model_factory=factory,
                )
            self.assertEqual(fit.call_count, 2)
            self.assertEqual(
                result["fits"]["full_delta"]["updates"], campaign.MAXIMUM_UPDATES
            )
            self.assertEqual(
                result["fits"]["additive_no_overwrite"]["updates"],
                campaign.MAXIMUM_UPDATES,
            )
            self.assertTrue((root / campaign.RESULT_RELATIVE_PATH).is_file())
            validator.assert_called_once_with(result)


if __name__ == "__main__":
    unittest.main()
