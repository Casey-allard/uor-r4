import argparse
import unittest
from unittest import mock
import tempfile

import numpy as np

import hyperbolic_router_so8 as hr
from tasks.router_retrieval_eval import (
    _load_chart_cache,
    _load_route_cache,
    _save_chart_cache,
    _save_route_cache,
    augment_route_keys_with_complex,
    build_event_gate_train_score_bias,
    build_event_gate_train_mask,
    compute_amortized_retrieval_metrics,
    dense_retrieval,
    event_gate_changes_retrieval_surface,
    event_gate_stats,
    event_gate_value_from_error,
    parse_args,
    prepare_grouped_same_bucket_plan,
    routed_retrieval,
    routed_retrieval_grouped_same_bucket,
    save_retrieval_query_audit,
    sparse_event_training_audit,
)


class RouterRetrievalEvalTest(unittest.TestCase):
    def test_event_gate_is_audit_only_for_retrieval_surface(self):
        args = argparse.Namespace(
            retrieval_backend="routed_probe",
            train_route_mode="final_static",
            event_gate_mode="soft_error",
            event_gate_threshold=0.07,
            event_gate_tau=0.002,
            event_gate_translation_coupling="off",
        )

        self.assertFalse(event_gate_changes_retrieval_surface(args))

    def test_event_gate_prune_marks_retrieval_surface_active(self):
        args = argparse.Namespace(
            retrieval_backend="routed_probe",
            train_route_mode="final_static",
            event_gate_mode="soft_error",
            event_gate_threshold=0.07,
            event_gate_tau=0.002,
            event_gate_translation_coupling="train_gate_prune",
        )

        self.assertTrue(event_gate_changes_retrieval_surface(args))

    def test_event_gate_score_bias_marks_retrieval_surface_active(self):
        args = argparse.Namespace(
            retrieval_backend="routed_probe",
            train_route_mode="final_static",
            event_gate_mode="soft_error",
            event_gate_threshold=0.07,
            event_gate_tau=0.002,
            event_gate_translation_coupling="train_gate_score_bias",
        )

        self.assertTrue(event_gate_changes_retrieval_surface(args))

    def test_dense_retrieval_uses_full_candidate_set(self):
        train_z = np.array([[1.0, 0.0], [0.9, 0.1], [0.0, 1.0]], dtype=np.float64)
        train_y = np.array([[1.0, 0.0], [1.0, 0.0], [0.0, 1.0]], dtype=np.float64)
        train_tok = np.array([10, 10, 20], dtype=np.int32)
        eval_z = np.array([[1.0, 0.0]], dtype=np.float64)

        yhat, pred_tok, cand_mean, cand_frac = dense_retrieval(
            train_z, train_y, train_tok, eval_z, topk=2, block_size=1
        )

        self.assertEqual(pred_tok.tolist(), [10])
        self.assertAlmostEqual(cand_mean, 3.0)
        self.assertAlmostEqual(cand_frac, 1.0)
        self.assertGreater(yhat[0, 0], yhat[0, 1])

    def test_routed_retrieval_reduces_candidates(self):
        train_keys = [(0, 0), (0, 0), (0, 1), (0, 1)]
        eval_keys = [(0, 0)]
        train_z = np.array([
            [1.0, 0.0],
            [0.9, 0.1],
            [0.0, 1.0],
            [0.1, 0.9],
        ], dtype=np.float64)
        train_y = np.array([
            [1.0, 0.0],
            [1.0, 0.0],
            [0.0, 1.0],
            [0.0, 1.0],
        ], dtype=np.float64)
        train_tok = np.array([10, 10, 20, 20], dtype=np.int32)
        eval_z = np.array([[1.0, 0.0]], dtype=np.float64)

        yhat, pred_tok, cand_mean, cand_frac, probe_mean, fallback, backfill_trigger, backfill_added = routed_retrieval(
            train_keys, eval_keys, train_z, train_y, train_tok, eval_z, topk=1, probe_buckets=1
        )

        self.assertEqual(pred_tok.tolist(), [10])
        self.assertAlmostEqual(cand_mean, 2.0)
        self.assertAlmostEqual(cand_frac, 0.5)
        self.assertAlmostEqual(probe_mean, 1.0)
        self.assertAlmostEqual(fallback, 0.0)
        self.assertAlmostEqual(backfill_trigger, 0.0)
        self.assertAlmostEqual(backfill_added, 0.0)
        self.assertGreater(yhat[0, 0], yhat[0, 1])

    def test_grouped_same_bucket_matches_probe1_path(self):
        train_keys = [(0, 0), (0, 0), (0, 1), (0, 1)]
        eval_keys = [(0, 0), (0, 1)]
        train_z = np.array([
            [1.0, 0.0],
            [0.9, 0.1],
            [0.0, 1.0],
            [0.1, 0.9],
        ], dtype=np.float64)
        train_y = np.array([
            [1.0, 0.0],
            [1.0, 0.0],
            [0.0, 1.0],
            [0.0, 1.0],
        ], dtype=np.float64)
        train_tok = np.array([10, 10, 20, 20], dtype=np.int32)
        eval_z = np.array([[1.0, 0.0], [0.0, 1.0]], dtype=np.float64)

        grouped = routed_retrieval_grouped_same_bucket(
            train_keys, eval_keys, train_z, train_y, train_tok, eval_z, topk=1
        )
        regular = routed_retrieval(
            train_keys, eval_keys, train_z, train_y, train_tok, eval_z, topk=1, probe_buckets=1
        )

        for a, b in zip(grouped[:2], regular[:2]):
            np.testing.assert_allclose(a, b)
        self.assertAlmostEqual(grouped[2], regular[2])
        self.assertAlmostEqual(grouped[3], regular[3])
        self.assertAlmostEqual(grouped[4], regular[4])
        self.assertAlmostEqual(grouped[5], regular[5])
        self.assertAlmostEqual(grouped[6], regular[6])
        self.assertAlmostEqual(grouped[7], regular[7])

    def test_prepared_grouped_same_bucket_plan_matches_direct_path(self):
        train_keys = [(0, 0, 0), (0, 0, 0), (0, 1, 1), (0, 1, 1)]
        eval_keys = [(0, 0, 0), (0, 1, 1)]
        train_z = np.array([
            [1.0, 0.0, 0.2, 0.0],
            [0.9, 0.1, 0.1, 0.0],
            [0.0, 1.0, 0.0, 0.2],
            [0.1, 0.9, 0.0, 0.1],
        ], dtype=np.float64)
        train_y = np.array([
            [1.0, 0.0],
            [1.0, 0.0],
            [0.0, 1.0],
            [0.0, 1.0],
        ], dtype=np.float64)
        train_tok = np.array([10, 10, 20, 20], dtype=np.int32)
        eval_z = np.array([[1.0, 0.0, 0.2, 0.0], [0.0, 1.0, 0.0, 0.2]], dtype=np.float64)

        plan = prepare_grouped_same_bucket_plan(
            train_keys=train_keys,
            eval_keys=eval_keys,
            train_z=train_z,
            train_y=train_y,
            train_tok=train_tok,
            eval_z=eval_z,
        )
        direct = routed_retrieval_grouped_same_bucket(
            train_keys, eval_keys, train_z, train_y, train_tok, eval_z, topk=1
        )
        prepared = routed_retrieval_grouped_same_bucket(
            train_keys,
            eval_keys,
            train_z,
            train_y,
            train_tok,
            eval_z,
            topk=1,
            prepared_plan=plan,
        )

        for a, b in zip(direct[:2], prepared[:2]):
            np.testing.assert_allclose(a, b)
        self.assertAlmostEqual(direct[2], prepared[2])
        self.assertAlmostEqual(direct[3], prepared[3])
        self.assertAlmostEqual(direct[4], prepared[4])
        self.assertAlmostEqual(direct[5], prepared[5])
        self.assertAlmostEqual(direct[6], prepared[6])
        self.assertAlmostEqual(direct[7], prepared[7])

    def test_complex_route_keys_add_secondary_component(self):
        base_keys = [(0, 1), (0, 1), (1, 0)]
        field = np.array([
            [1.0, 0.0],
            [0.0, 1.0],
            [-1.0, 0.0],
        ], dtype=np.float64)
        keys, secondary_count = augment_route_keys_with_complex(
            base_keys=base_keys,
            field=field,
            dim_i=0,
            dim_j=1,
            roots=4,
            radius_bins=1,
        )

        self.assertEqual(len(keys[0]), 3)
        self.assertEqual(keys[0][:2], base_keys[0])
        self.assertGreaterEqual(secondary_count, 2)

    def test_complex_route_keys_reduce_candidates_when_secondary_matches(self):
        train_keys = [(0, 0, 0), (0, 0, 0), (0, 0, 1), (0, 1, 0)]
        eval_keys = [(0, 0, 1)]
        train_z = np.array([
            [1.0, 0.0],
            [0.9, 0.1],
            [0.0, 1.0],
            [0.1, 0.9],
        ], dtype=np.float64)
        train_y = np.array([
            [1.0, 0.0],
            [1.0, 0.0],
            [0.0, 1.0],
            [0.0, 1.0],
        ], dtype=np.float64)
        train_tok = np.array([10, 10, 20, 20], dtype=np.int32)
        eval_z = np.array([[0.0, 1.0]], dtype=np.float64)

        yhat, pred_tok, cand_mean, cand_frac, probe_mean, fallback, backfill_trigger, backfill_added = routed_retrieval(
            train_keys, eval_keys, train_z, train_y, train_tok, eval_z, topk=1, probe_buckets=1
        )

        self.assertEqual(pred_tok.tolist(), [20])
        self.assertAlmostEqual(cand_mean, 1.0)
        self.assertAlmostEqual(cand_frac, 0.25)
        self.assertAlmostEqual(probe_mean, 1.0)
        self.assertAlmostEqual(fallback, 0.0)
        self.assertAlmostEqual(backfill_trigger, 0.0)
        self.assertAlmostEqual(backfill_added, 0.0)
        self.assertGreater(yhat[0, 1], yhat[0, 0])

    def test_complex_backfill_can_recover_coarse_neighbor(self):
        train_keys = [(0, 0, 0), (0, 0, 1), (0, 0, 1)]
        eval_keys = [(0, 0, 0)]
        train_z = np.array([
            [1.0, 0.0],
            [0.95, 0.05],
            [0.90, 0.10],
        ], dtype=np.float64)
        train_y = np.array([
            [1.0, 0.0],
            [0.0, 1.0],
            [0.0, 1.0],
        ], dtype=np.float64)
        train_tok = np.array([10, 20, 20], dtype=np.int32)
        eval_z = np.array([[0.97, 0.03]], dtype=np.float64)

        no_backfill = routed_retrieval(
            train_keys, eval_keys, train_z, train_y, train_tok, eval_z, topk=1, probe_buckets=1, complex_backfill_items=0
        )
        with_backfill = routed_retrieval(
            train_keys, eval_keys, train_z, train_y, train_tok, eval_z, topk=1, probe_buckets=1, complex_backfill_items=1
        )

        self.assertEqual(no_backfill[1].tolist(), [10])
        self.assertEqual(with_backfill[1].tolist(), [20])
        self.assertGreater(with_backfill[2], no_backfill[2])
        self.assertAlmostEqual(with_backfill[5], 0.0)
        self.assertAlmostEqual(no_backfill[6], 0.0)
        self.assertAlmostEqual(no_backfill[7], 0.0)
        self.assertAlmostEqual(with_backfill[6], 1.0)
        self.assertGreater(with_backfill[7], 0.0)

    def test_complex_backfill_small_bucket_only_triggers_for_tiny_exact_bucket(self):
        train_keys = [(0, 0, 0), (0, 0, 1), (0, 0, 1), (0, 1, 0)]
        eval_keys = [(0, 0, 0), (0, 0, 1)]
        train_z = np.array([
            [1.0, 0.0],
            [0.95, 0.05],
            [0.90, 0.10],
            [0.0, 1.0],
        ], dtype=np.float64)
        train_y = np.array([
            [1.0, 0.0],
            [0.0, 1.0],
            [0.0, 1.0],
            [0.0, 1.0],
        ], dtype=np.float64)
        train_tok = np.array([10, 20, 20, 20], dtype=np.int32)
        eval_z = np.array([[0.97, 0.03], [0.92, 0.08]], dtype=np.float64)

        with_backfill = routed_retrieval(
            train_keys,
            eval_keys,
            train_z,
            train_y,
            train_tok,
            eval_z,
            topk=1,
            probe_buckets=1,
            complex_backfill_items=1,
            complex_backfill_mode="small_bucket",
            complex_backfill_max_exact=1,
        )

        self.assertAlmostEqual(with_backfill[6], 0.5)
        self.assertGreater(with_backfill[7], 0.0)

    def test_complex_backfill_low_margin_only_triggers_for_ambiguous_exact_bucket(self):
        train_keys = [(0, 0, 0), (0, 0, 0), (0, 0, 1), (0, 0, 1)]
        eval_keys = [(0, 0, 0), (0, 0, 1)]
        train_z = np.array([
            [1.0, 0.0],
            [0.99, 0.01],
            [0.80, 0.20],
            [0.0, 1.0],
        ], dtype=np.float64)
        train_y = np.array([
            [1.0, 0.0],
            [1.0, 0.0],
            [0.0, 1.0],
            [0.0, 1.0],
        ], dtype=np.float64)
        train_tok = np.array([10, 10, 20, 20], dtype=np.int32)
        eval_z = np.array([[0.995, 0.005], [0.79, 0.21]], dtype=np.float64)

        with_backfill = routed_retrieval(
            train_keys,
            eval_keys,
            train_z,
            train_y,
            train_tok,
            eval_z,
            topk=1,
            probe_buckets=1,
            complex_backfill_items=1,
            complex_backfill_mode="low_margin",
            complex_backfill_margin_threshold=0.02,
        )

        self.assertAlmostEqual(with_backfill[6], 0.5)
        self.assertGreater(with_backfill[7], 0.0)

    def test_complex_rerank_can_change_in_bucket_order_without_expanding_candidates(self):
        train_keys = [(0, 0, 0), (0, 0, 0)]
        eval_keys = [(0, 0, 0)]
        train_z = np.array([
            [1.0, 0.0, 0.1, 0.1],
            [0.7, 0.0, 0.0, 1.0],
        ], dtype=np.float64)
        train_y = np.array([
            [1.0, 0.0],
            [0.0, 1.0],
        ], dtype=np.float64)
        train_tok = np.array([10, 20], dtype=np.int32)
        eval_z = np.array([[1.0, 0.0, 0.0, 0.4]], dtype=np.float64)

        base = routed_retrieval(
            train_keys,
            eval_keys,
            train_z,
            train_y,
            train_tok,
            eval_z,
            topk=1,
            probe_buckets=1,
            complex_rerank_mode="none",
            complex_dim_i=2,
            complex_dim_j=3,
        )
        reranked = routed_retrieval(
            train_keys,
            eval_keys,
            train_z,
            train_y,
            train_tok,
            eval_z,
            topk=1,
            probe_buckets=1,
            complex_rerank_mode="complex_plane",
            complex_rerank_lambda=0.5,
            complex_dim_i=2,
            complex_dim_j=3,
        )

        self.assertEqual(base[1].tolist(), [10])
        self.assertEqual(reranked[1].tolist(), [20])
        self.assertAlmostEqual(base[2], reranked[2])
        self.assertAlmostEqual(base[3], reranked[3])

    def test_complex_low_margin_rerank_only_triggers_on_ambiguous_rows(self):
        train_keys = [(0, 0), (0, 0)]
        eval_keys = [(0, 0), (0, 0)]
        train_z = np.array([
            [1.0, 0.0, 0.1, 0.1],
            [0.7, 0.0, 0.0, 1.0],
        ], dtype=np.float64)
        train_y = np.array([
            [1.0, 0.0],
            [0.0, 1.0],
        ], dtype=np.float64)
        train_tok = np.array([10, 20], dtype=np.int32)
        eval_z = np.array([
            [0.8, 0.0, 0.0, 0.4],
            [1.0, 0.0, 0.0, 0.4],
        ], dtype=np.float64)

        base = routed_retrieval(
            train_keys,
            eval_keys,
            train_z,
            train_y,
            train_tok,
            eval_z,
            topk=1,
            probe_buckets=1,
            complex_rerank_mode="none",
            complex_dim_i=2,
            complex_dim_j=3,
        )
        reranked = routed_retrieval(
            train_keys,
            eval_keys,
            train_z,
            train_y,
            train_tok,
            eval_z,
            topk=1,
            probe_buckets=1,
            complex_rerank_mode="complex_plane_low_margin",
            complex_rerank_lambda=0.5,
            complex_rerank_margin_threshold=0.08,
            complex_dim_i=2,
            complex_dim_j=3,
        )

        self.assertEqual(base[1].tolist(), [10, 10])
        self.assertEqual(reranked[1].tolist(), [20, 10])
        self.assertAlmostEqual(base[2], reranked[2])
        self.assertAlmostEqual(base[3], reranked[3])

    def test_amortized_retrieval_metrics_scale_with_repeat_count(self):
        online_per_repeat, amortized = compute_amortized_retrieval_metrics(offline_total=8.0, online_total=4.0, query_repeats=4)
        self.assertAlmostEqual(online_per_repeat, 1.0)
        self.assertAlmostEqual(amortized, 3.0)

    def test_sparse_event_gate_helpers_match_proxy_surface(self):
        off_gate = event_gate_value_from_error(
            error_mag=0.01,
            mode="off",
            threshold=0.05,
            tau=0.01,
        )
        on_gate = event_gate_value_from_error(
            error_mag=0.01,
            mode="soft_error",
            threshold=0.05,
            tau=0.01,
        )
        error_mag, gate, active = event_gate_stats(
            yhat=[0.0, 0.0],
            y=[0.1, 0.0],
            mode="soft_error",
            threshold=0.05,
            tau=0.01,
        )

        self.assertAlmostEqual(off_gate, 1.0)
        self.assertLess(on_gate, 0.5)
        self.assertGreater(error_mag, 0.0)
        self.assertGreater(gate, 0.5)
        self.assertEqual(active, 1.0)

    def test_sparse_event_training_audit_reports_reduced_update_mass(self):
        train_keys = [(0, 0), (0, 0)]
        v_tr = np.array([[1.0, 0.0], [1.0, 0.0]], dtype=np.float64)
        z_tr = np.array([[1.0, 0.0], [1.0, 0.0]], dtype=np.float64)
        y_tr = np.array([[0.1, 0.0], [0.1, 0.0]], dtype=np.float64)

        dense = sparse_event_training_audit(
            train_keys_static=train_keys,
            z_tr_static=z_tr,
            v_tr=v_tr,
            y_tr=y_tr,
            d=2,
            dy=2,
            eta_p=0.04,
            eta_m=0.08,
            epochs=1,
            seed=0,
            event_gate_mode="off",
            event_gate_threshold=0.07,
            event_gate_tau=0.01,
        )
        sparse = sparse_event_training_audit(
            train_keys_static=train_keys,
            z_tr_static=z_tr,
            v_tr=v_tr,
            y_tr=y_tr,
            d=2,
            dy=2,
            eta_p=0.04,
            eta_m=0.08,
            epochs=1,
            seed=0,
            event_gate_mode="soft_error",
            event_gate_threshold=0.07,
            event_gate_tau=0.01,
        )

        self.assertAlmostEqual(dense["event_gate_mean"], 1.0)
        self.assertAlmostEqual(dense["event_gate_cost_proxy"], 1.0)
        self.assertLess(sparse["event_gate_mean"], 1.0)
        self.assertAlmostEqual(sparse["event_gate_cost_proxy"], sparse["event_gate_mean"])
        self.assertGreaterEqual(sparse["training_total_sec"], 0.0)
        self.assertEqual(sparse["sample_gate_mean"].shape[0], 2)

    def test_event_gate_train_mask_preserves_min_keep_per_bucket(self):
        train_keys = [(0, 0), (0, 0), (1, 0), (1, 0)]
        sample_gate_mean = np.array([0.01, 0.20, 0.00, 0.03], dtype=np.float64)

        mask = build_event_gate_train_mask(
            train_keys=train_keys,
            sample_gate_mean=sample_gate_mean,
            threshold=0.05,
            min_keep_per_bucket=1,
        )

        self.assertEqual(mask.tolist(), [False, True, False, True])

    def test_event_gate_train_score_bias_prefers_lower_gate_samples(self):
        sample_gate_mean = np.array([0.01, 0.02, 0.20], dtype=np.float64)

        bias = build_event_gate_train_score_bias(sample_gate_mean)

        self.assertGreater(bias[0], bias[1])
        self.assertGreater(bias[1], bias[2])
        self.assertAlmostEqual(float(np.mean(bias)), 0.0, places=7)

    def test_event_gate_score_bias_can_change_in_bucket_order(self):
        train_keys = [(0, 0), (0, 0)]
        eval_keys = [(0, 0)]
        train_z = np.array([
            [1.0, 0.0],
            [0.98, 0.02],
        ], dtype=np.float64)
        train_y = np.array([
            [1.0, 0.0],
            [0.0, 1.0],
        ], dtype=np.float64)
        train_tok = np.array([10, 20], dtype=np.int32)
        eval_z = np.array([[0.99, 0.01]], dtype=np.float64)
        train_score_bias = np.array([-0.5, 0.5], dtype=np.float64)

        base = routed_retrieval(
            train_keys,
            eval_keys,
            train_z,
            train_y,
            train_tok,
            eval_z,
            topk=1,
            probe_buckets=1,
        )
        biased = routed_retrieval(
            train_keys,
            eval_keys,
            train_z,
            train_y,
            train_tok,
            eval_z,
            topk=1,
            probe_buckets=1,
            train_score_bias=train_score_bias,
            score_bias_lambda=0.1,
        )

        self.assertEqual(base[1].tolist(), [10])
        self.assertEqual(biased[1].tolist(), [20])
        self.assertAlmostEqual(base[2], biased[2])
        self.assertAlmostEqual(base[3], biased[3])

    def test_parse_args_exposes_phase_field_retrieval_surface(self):
        argv = [
            "router_retrieval_eval.py",
            "--sector_mode", "phase4d_hopf_product_phase",
            "--field4_dims", "1,3,5,7",
            "--phase_transport_lambda", "0.0",
            "--phase_field_lambda", "1.0",
            "--product_shell_control_mode", "banded",
            "--product_shell_gate_threshold", "0.35",
            "--complex_rerank_mode", "complex_plane_low_margin",
            "--complex_rerank_margin_threshold", "0.08",
            "--event_gate_mode", "soft_error",
            "--event_gate_threshold", "0.07",
            "--event_gate_tau", "0.01",
            "--event_gate_translation_coupling", "train_gate_prune",
            "--event_gate_translation_prune_threshold", "0.02",
            "--event_gate_translation_min_keep_per_bucket", "1",
            "--retrieval_audit_dir", "results/analysis/inc0113_query_audit",
        ]
        with mock.patch("sys.argv", argv):
            args = parse_args()

        self.assertEqual(args.sector_mode, "phase4d_hopf_product_phase")
        self.assertEqual(args.field4_dims, "1,3,5,7")
        self.assertAlmostEqual(args.phase_transport_lambda, 0.0)
        self.assertAlmostEqual(args.phase_field_lambda, 1.0)
        self.assertEqual(args.product_shell_control_mode, "banded")
        self.assertAlmostEqual(args.product_shell_gate_threshold, 0.35)
        self.assertEqual(args.complex_rerank_mode, "complex_plane_low_margin")
        self.assertAlmostEqual(args.complex_rerank_margin_threshold, 0.08)
        self.assertEqual(args.event_gate_mode, "soft_error")
        self.assertAlmostEqual(args.event_gate_threshold, 0.07)
        self.assertAlmostEqual(args.event_gate_tau, 0.01)
        self.assertEqual(args.event_gate_translation_coupling, "train_gate_prune")
        self.assertAlmostEqual(args.event_gate_translation_prune_threshold, 0.02)
        self.assertEqual(args.event_gate_translation_min_keep_per_bucket, 1)
        self.assertEqual(args.retrieval_audit_dir, "results/analysis/inc0113_query_audit")

    def test_parse_args_exposes_event_gate_score_bias_surface(self):
        argv = [
            "router_retrieval_eval.py",
            "--event_gate_mode", "soft_error",
            "--event_gate_threshold", "0.07",
            "--event_gate_tau", "0.002",
            "--event_gate_translation_coupling", "train_gate_score_bias",
            "--event_gate_translation_score_bias_lambda", "0.03",
        ]
        with mock.patch("sys.argv", argv):
            args = parse_args()

        self.assertEqual(args.event_gate_translation_coupling, "train_gate_score_bias")
        self.assertAlmostEqual(args.event_gate_translation_score_bias_lambda, 0.03)

    def test_candidate_fraction_denominator_can_stay_at_full_train_size(self):
        train_keys = [(0, 0), (0, 0)]
        eval_keys = [(0, 0)]
        train_z = np.array([[1.0, 0.0], [0.9, 0.1]], dtype=np.float64)
        train_y = np.array([[1.0, 0.0], [1.0, 0.0]], dtype=np.float64)
        train_tok = np.array([10, 10], dtype=np.int32)
        eval_z = np.array([[1.0, 0.0]], dtype=np.float64)

        _, _, cand_mean, cand_frac, *_ = routed_retrieval(
            train_keys,
            eval_keys,
            train_z,
            train_y,
            train_tok,
            eval_z,
            topk=1,
            probe_buckets=1,
            candidate_fraction_denominator=4,
        )

        self.assertAlmostEqual(cand_mean, 2.0)
        self.assertAlmostEqual(cand_frac, 0.5)

    def test_routed_retrieval_query_audit_distinguishes_omission_from_in_candidate_loss(self):
        train_keys = [(0, 0), (0, 0), (1, 0)]
        eval_keys = [(0, 0), (1, 0)]
        train_z = np.array([
            [1.0, 0.0],
            [0.0, 1.0],
            [0.95, 0.05],
        ], dtype=np.float64)
        train_y = np.array([
            [1.0, 0.0],
            [0.0, 1.0],
            [1.0, 0.0],
        ], dtype=np.float64)
        train_tok = np.array([10, 20, 10], dtype=np.int32)
        eval_z = np.array([
            [0.98, 0.02],
            [0.99, 0.01],
        ], dtype=np.float64)
        eval_target_tok = np.array([20, 20], dtype=np.int32)

        result = routed_retrieval(
            train_keys,
            eval_keys,
            train_z,
            train_y,
            train_tok,
            eval_z,
            topk=1,
            probe_buckets=1,
            return_query_audit=True,
            eval_target_tok=eval_target_tok,
        )
        query_audit = result[-1]

        self.assertEqual(query_audit["candidate_count"].tolist(), [2, 1])
        self.assertEqual(query_audit["target_present"].tolist(), [1, 0])
        self.assertEqual(query_audit["best_target_rank"].tolist(), [2, 0])
        self.assertEqual(query_audit["topk_target_present"].tolist(), [0, 0])
        self.assertEqual(query_audit["correct"].tolist(), [0, 0])
        self.assertEqual(query_audit["pred_tok"].tolist(), [10, 10])

    def test_save_retrieval_query_audit_writes_expected_fields(self):
        query_audit = {
            "pred_tok": np.array([10, 20], dtype=np.int32),
            "correct": np.array([1, 0], dtype=np.int8),
            "candidate_count": np.array([3, 2], dtype=np.int32),
            "candidate_fraction": np.array([1.0, 0.5], dtype=np.float64),
            "target_present": np.array([1, 1], dtype=np.int8),
            "best_target_rank": np.array([1, 2], dtype=np.int32),
            "topk_target_present": np.array([1, 0], dtype=np.int8),
        }

        with tempfile.TemporaryDirectory() as tmpdir:
            path = f"{tmpdir}/audit.npz"
            save_retrieval_query_audit(
                path,
                eval_source_idx=np.array([7, 9], dtype=np.int64),
                target_tok=np.array([10, 10], dtype=np.int32),
                query_audit=query_audit,
            )
            payload = np.load(path)

            self.assertEqual(payload["eval_source_idx"].tolist(), [7, 9])
            self.assertEqual(payload["target_tok"].tolist(), [10, 10])
            self.assertEqual(payload["pred_tok"].tolist(), [10, 20])
            self.assertEqual(payload["best_target_rank"].tolist(), [1, 2])
            self.assertEqual(payload["topk_target_present"].tolist(), [1, 0])

    def test_chart_cache_round_trip_preserves_chart(self):
        chart = hr.Chart(
            R=np.array([[0.0, -1.0], [1.0, 0.0]], dtype=np.float64),
            s_global=np.array([1.1], dtype=np.float64),
            S_radial=np.array([[0.9, 1.0]], dtype=np.float64),
            scale_mode="radial",
            radial_rmax=2.5,
            radial_bins=2,
        )
        with tempfile.TemporaryDirectory() as td:
            path = f"{td}/chart.npz"
            _save_chart_cache(path, chart)
            loaded = _load_chart_cache(path)

        np.testing.assert_allclose(loaded.R, chart.R)
        np.testing.assert_allclose(loaded.s_global, chart.s_global)
        np.testing.assert_allclose(loaded.S_radial, chart.S_radial)
        self.assertEqual(loaded.scale_mode, chart.scale_mode)
        self.assertEqual(loaded.radial_bins, chart.radial_bins)
        self.assertAlmostEqual(float(loaded.radial_rmax), float(chart.radial_rmax))

    def test_route_cache_round_trip_preserves_train_routes(self):
        shell_tr = np.array([0, 1, 1], dtype=np.int64)
        sector_tr = np.array([2, 3, 4], dtype=np.int64)
        z_tr = np.array([[1.0, 0.0], [0.0, 1.0], [0.5, 0.5]], dtype=np.float64)
        with tempfile.TemporaryDirectory() as td:
            path = f"{td}/routes.npz"
            _save_route_cache(path, shell_tr=shell_tr, sector_tr=sector_tr, z_tr=z_tr)
            loaded_shell, loaded_sector, loaded_z = _load_route_cache(path)

        np.testing.assert_array_equal(loaded_shell, shell_tr)
        np.testing.assert_array_equal(loaded_sector, sector_tr)
        np.testing.assert_allclose(loaded_z, z_tr)


if __name__ == "__main__":
    unittest.main()
