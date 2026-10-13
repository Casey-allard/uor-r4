import unittest

from tools import sparse_event_proxy_translation_gap_audit as audit


class SparseEventProxyTranslationGapAuditTest(unittest.TestCase):
    def _proxy_analysis(self):
        return {
            "experiment_id": "inc0125_confirm",
            "route_stats": [
                {
                    "route_id": "H4XH4_FIELD_A150",
                    "mean_test_mse_after": 0.003884,
                    "mean_total_sec": 13.228,
                    "mean_event_gate_mean": 1.0,
                    "mean_event_gate_active_frac": 1.0,
                    "mean_shell_pmax": 0.576,
                    "mean_unseen_rate": 0.00095,
                },
                {
                    "route_id": "H4XH4_FIELD_A150_EVT_T070",
                    "mean_test_mse_after": 0.003895,
                    "mean_total_sec": 10.184,
                    "mean_event_gate_mean": 0.319,
                    "mean_event_gate_active_frac": 0.0,
                    "mean_shell_pmax": 0.576,
                    "mean_unseen_rate": 0.00095,
                },
                {
                    "route_id": "H4XH4_FIELD_A150_EVT_T070_TAU002",
                    "mean_test_mse_after": 0.003859,
                    "mean_total_sec": 10.213,
                    "mean_event_gate_mean": 0.020,
                    "mean_event_gate_active_frac": 0.0,
                    "mean_shell_pmax": 0.576,
                    "mean_unseen_rate": 0.00095,
                },
                {
                    "route_id": "H4XH4_FIELD_A150_HARD_T062",
                    "mean_test_mse_after": 0.003892,
                    "mean_total_sec": 11.919,
                    "mean_event_gate_mean": 0.840,
                    "mean_event_gate_active_frac": 0.840,
                    "mean_shell_pmax": 0.576,
                    "mean_unseen_rate": 0.00095,
                },
            ],
        }

    def _soft_translated_analysis(self):
        return {
            "experiment_id": "inc0100_confirm",
            "route_stats": [
                {
                    "route_id": "CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500",
                    "mean_test_top1_after": 0.0446,
                    "mean_retrieval_candidate_fraction": 0.19332824,
                    "mean_online_total_per_repeat_sec": 0.275914,
                    "mean_amortized_total_per_repeat_sec": 0.338014,
                    "mean_event_gate_mean": 1.0,
                    "mean_event_gate_active_frac": 1.0,
                    "mean_query_route_sec": 0.058011,
                    "mean_route_index_build_sec": 0.061027,
                    "mean_retrieval_search_sec": 0.217903,
                },
                {
                    "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500",
                    "mean_test_top1_after": 0.0446,
                    "mean_retrieval_candidate_fraction": 0.19332824,
                    "mean_online_total_per_repeat_sec": 0.117741,
                    "mean_amortized_total_per_repeat_sec": 0.168611,
                    "mean_event_gate_mean": 0.319065,
                    "mean_event_gate_active_frac": 0.0,
                    "mean_query_route_sec": 0.023339,
                    "mean_route_index_build_sec": 0.049970,
                    "mean_retrieval_search_sec": 0.094402,
                },
            ],
        }

    def _near_hard_translated_analysis(self):
        return {
            "experiment_id": "inc0102_screen",
            "route_stats": [
                {
                    "route_id": "DENSE_Q01_T2500",
                    "mean_test_top1_after": 0.0516,
                    "mean_retrieval_candidate_fraction": 1.0,
                    "mean_online_total_per_repeat_sec": 0.251108,
                    "mean_amortized_total_per_repeat_sec": 0.251108,
                    "mean_event_gate_mean": 1.0,
                    "mean_event_gate_active_frac": 1.0,
                    "mean_query_route_sec": 0.0,
                    "mean_route_index_build_sec": 0.0,
                    "mean_retrieval_search_sec": 0.251108,
                },
                {
                    "route_id": "CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500",
                    "mean_test_top1_after": 0.0444,
                    "mean_retrieval_candidate_fraction": 0.189016,
                    "mean_online_total_per_repeat_sec": 0.081392,
                    "mean_amortized_total_per_repeat_sec": 0.118415,
                    "mean_event_gate_mean": 1.0,
                    "mean_event_gate_active_frac": 1.0,
                    "mean_query_route_sec": 0.012768,
                    "mean_route_index_build_sec": 0.036070,
                    "mean_retrieval_search_sec": 0.068624,
                },
                {
                    "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500",
                    "mean_test_top1_after": 0.0444,
                    "mean_retrieval_candidate_fraction": 0.189016,
                    "mean_online_total_per_repeat_sec": 0.117009,
                    "mean_amortized_total_per_repeat_sec": 0.145747,
                    "mean_event_gate_mean": 0.319212,
                    "mean_event_gate_active_frac": 0.0,
                    "mean_query_route_sec": 0.021356,
                    "mean_route_index_build_sec": 0.027748,
                    "mean_retrieval_search_sec": 0.095653,
                },
                {
                    "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500",
                    "mean_test_top1_after": 0.0444,
                    "mean_retrieval_candidate_fraction": 0.189016,
                    "mean_online_total_per_repeat_sec": 0.215612,
                    "mean_amortized_total_per_repeat_sec": 0.263435,
                    "mean_event_gate_mean": 0.020973,
                    "mean_event_gate_active_frac": 0.0,
                    "mean_query_route_sec": 0.032464,
                    "mean_route_index_build_sec": 0.046637,
                    "mean_retrieval_search_sec": 0.183148,
                },
            ],
        }

    def test_build_payload_classifies_gap_as_translated_systems_cost_branch(self):
        payload = audit.build_payload(
            self._proxy_analysis(),
            self._soft_translated_analysis(),
            self._near_hard_translated_analysis(),
        )

        self.assertEqual(payload["branch_outcome"], "translated_systems_cost_branch")
        self.assertTrue(payload["quality_preserved_vs_soft_sparse"])
        self.assertFalse(payload["candidate_omission_supported"])
        self.assertFalse(payload["in_candidate_ordering_loss_supported"])
        self.assertTrue(payload["event_gate_overhead_supported"])
        self.assertTrue(payload["route_search_interaction_cost_supported"])
        self.assertEqual(
            payload["translated_near_hard_primary_runtime_driver_vs_soft_sparse"],
            "retrieval_search_sec",
        )

    def test_build_payload_can_classify_quality_loss_branch(self):
        near_hard = self._near_hard_translated_analysis()
        for row in near_hard["route_stats"]:
            if row["route_id"] == "CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500":
                row["mean_test_top1_after"] = 0.0430
        payload = audit.build_payload(
            self._proxy_analysis(),
            self._soft_translated_analysis(),
            near_hard,
        )

        self.assertEqual(payload["branch_outcome"], "translated_ordering_branch")
        self.assertFalse(payload["quality_preserved_vs_soft_sparse"])
        self.assertTrue(payload["in_candidate_ordering_loss_supported"])


if __name__ == "__main__":
    unittest.main()
