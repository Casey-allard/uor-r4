import unittest

from tools import sparse_event_translation_systems_cost_rescue as audit


class SparseEventTranslationSystemsCostRescueTest(unittest.TestCase):
    def _run(self, seed, top1, cand_frac, online, amortized, route_index, route_query, search, gate_mean=1.0, gate_active=1.0):
        return {
            "args": {"seed": seed},
            "metrics": {
                "test_top1_after": top1,
                "retrieval_candidate_fraction_mean": cand_frac,
                "retrieval_online_total_per_repeat_sec": online,
                "retrieval_total_amortized_per_repeat_sec": amortized,
                "event_gate_mean": gate_mean,
                "event_gate_active_frac": gate_active,
            },
            "timings_sec": {
                "route_index_build": route_index,
                "query_route": route_query,
                "retrieval_search": search,
            },
        }

    def test_build_payload_closes_rescue_when_only_event_gate_knobs_differ(self):
        translation_config = {
            "experiment_id": "translation_cfg",
            "common_args": {
                "retrieval_backend": "routed_probe",
                "train_route_mode": "final_static",
                "event_gate_mode": "soft_error",
                "event_gate_threshold": 0.07,
                "event_gate_tau": 0.01,
            },
            "routes": [
                {"route_id": "SOFT", "args": {}},
                {"route_id": "NEAR", "args": {"event_gate_tau": 0.002}},
            ],
        }
        translation_analysis = {
            "experiment_id": "translation",
            "results": {
                "SOFT": [
                    self._run(0, 0.44, 0.19, 0.12, 0.15, 0.03, 0.02, 0.10),
                    self._run(1, 0.44, 0.19, 0.18, 0.22, 0.04, 0.03, 0.15),
                ],
                "NEAR": [
                    self._run(0, 0.44, 0.19, 0.20, 0.26, 0.05, 0.03, 0.17),
                    self._run(1, 0.44, 0.19, 0.31, 0.39, 0.06, 0.05, 0.26),
                ],
            },
        }
        proxy_analysis = {
            "experiment_id": "proxy",
            "results": {
                "SOFT_PROXY": [
                    self._run(0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, gate_mean=0.32, gate_active=0.0),
                ],
                "NEAR_PROXY": [
                    self._run(0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, gate_mean=0.02, gate_active=0.0),
                ],
            },
        }

        payload = audit.build_payload(
            translation_config=translation_config,
            translation_analysis=translation_analysis,
            proxy_analysis=proxy_analysis,
            soft_route_id="SOFT",
            near_hard_route_id="NEAR",
            proxy_soft_route_id="SOFT_PROXY",
            proxy_near_hard_route_id="NEAR_PROXY",
        )

        self.assertEqual(payload["branch_outcome"], "no_selective_retrieval_rescue_surface")
        self.assertTrue(payload["contract_check"]["difference_is_event_gate_only"])
        self.assertFalse(payload["contract_check"]["event_gate_retrieval_surface_active"])
        self.assertIn("not wired into the translated retrieval surface", payload["interpretation"])

    def test_report_mentions_decision_surface(self):
        payload = {
            "branch_outcome": "no_selective_retrieval_rescue_surface",
            "interpretation": "The compared routes do not differ on retrieval surface.",
            "contract_check": {
                "route_arg_difference_keys": ["event_gate_tau"],
                "difference_is_event_gate_only": True,
                "event_gate_retrieval_surface_active": False,
                "event_gate_retrieval_surface_reason": "audit-only knobs",
            },
            "translation": {
                "deltas_candidate_minus_baseline": {
                    "top1_delta_candidate_minus_baseline": 0.0,
                    "candidate_fraction_delta_candidate_minus_baseline": 0.0,
                    "online_delta_candidate_minus_baseline_sec": 0.1,
                    "amortized_delta_candidate_minus_baseline_sec": 0.12,
                    "route_query_delta_candidate_minus_baseline_sec": 0.01,
                    "route_index_build_delta_candidate_minus_baseline_sec": 0.02,
                    "retrieval_search_delta_candidate_minus_baseline_sec": 0.09,
                }
            },
        }

        report = audit._report(payload)
        self.assertIn("Branch outcome", report)
        self.assertIn("event_gate_tau", report)
        self.assertIn("route-coupled sparse-event translated pilot", report)


if __name__ == "__main__":
    unittest.main()
