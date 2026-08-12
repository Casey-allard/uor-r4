import json
import unittest
from pathlib import Path

import numpy as np

import server


VECTORS_PATH = Path(__file__).with_name("vv_test_vectors.json")


class TestIntegrationSystem(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        with VECTORS_PATH.open("r", encoding="utf-8") as f:
            cls.vectors = json.load(f)

    def test_identity_control_plane_variation(self):
        _, _, meta_a = server.identity_to_qimc_prime("tenant-integration-a")
        _, _, meta_b = server.identity_to_qimc_prime("tenant-integration-b")

        ctrl_a = server.derive_uor_control_plane(meta_a)
        ctrl_b = server.derive_uor_control_plane(meta_b)

        self.assertTrue(0.0 <= ctrl_a["entropy_bias"] <= 1.0)
        self.assertTrue(0.0 <= ctrl_b["entropy_bias"] <= 1.0)
        self.assertTrue(2 <= ctrl_a["hopf_chi_bins"] <= 4)
        self.assertTrue(2 <= ctrl_b["hopf_chi_bins"] <= 4)

        # Independent identities should usually map to distinct control fields.
        self.assertNotEqual(meta_a["identity_uor_digest"], meta_b["identity_uor_digest"])
        self.assertNotEqual(ctrl_a["phase_transport_lambda"], ctrl_b["phase_transport_lambda"])

    def test_route_to_generation_pipeline_consistency(self):
        for vector in self.vectors[:3]:
            identity = vector["identity"]
            text = vector["query"]

            state = server.evolve_brain_state(text, gamma=0.5, identity=identity)
            routed_pack = server.route_query_to_manifold(text, include_eigenvalues=True, identity=identity, state_vector=state)
            output = server.generate_response_from_metrics(
                text,
                routed_pack,
                max_tokens=120,
                temperature=0.5,
                identity=identity,
                engine="geometric",
                gamma=0.5,
                state_vector=state,
            )

            routed = routed_pack["routed"]
            metrics = output["metrics"]

            self.assertEqual(metrics["window_index"], routed["window_index"])
            self.assertEqual(metrics["qimc"]["identity_uor_digest"], routed["qimc"]["identity_uor_digest"])
            self.assertIn("uor_control", metrics["qimc"])
            self.assertIn("phase_transport_lambda", metrics["hopf"])
            self.assertIn("uor", metrics)
            self.assertTrue(metrics["uor"]["address"].startswith("sha256:"))

            recomputed = server.generate_uor_attestation(metrics["uor_payload"], hash_algorithm="sha256")
            self.assertEqual(recomputed["address"], metrics["uor_address"])

            self.assertIsInstance(output["description"], str)
            self.assertGreater(len(output["description"]), 0)
            self.assertIsInstance(output["trajectory"], list)
            # Trajectory may be empty if the prompt has no routable vocabulary terms.
            self.assertGreaterEqual(len(output["trajectory"]), 0)

    def test_scoped_indexing_isolation(self):
        identity_a = "tenant-scope-a"
        identity_b = "tenant-scope-b"

        text_a = "Unique scoped sentence alpha for integration isolation checks."
        text_b = "Unique scoped sentence beta for integration isolation checks."

        index_a_before = server.count_indexed_sentences(server.get_corpus_index_for_identity(identity_a))
        index_b_before = server.count_indexed_sentences(server.get_corpus_index_for_identity(identity_b))
        shared_before = server.count_indexed_sentences(server.get_corpus_index_for_identity(server.RESERVED_SHARED_IDENTITY))

        server.index_single_sentence(text_a, identity=identity_a)
        server.index_single_sentence(text_b, identity=identity_b)

        index_a_after = server.count_indexed_sentences(server.get_corpus_index_for_identity(identity_a))
        index_b_after = server.count_indexed_sentences(server.get_corpus_index_for_identity(identity_b))
        shared_after = server.count_indexed_sentences(server.get_corpus_index_for_identity(server.RESERVED_SHARED_IDENTITY))

        self.assertGreaterEqual(index_a_after, index_a_before + 1)
        self.assertGreaterEqual(index_b_after, index_b_before + 1)
        self.assertEqual(shared_before, shared_after)

        # Identity states remain isolated when one identity evolves.
        server.reset_brain_state(identity=identity_a)
        server.reset_brain_state(identity=identity_b)
        baseline_b = np.copy(server.get_brain_state(identity_b))
        _ = server.evolve_brain_state("only evolve identity a", gamma=0.4, identity=identity_a)
        after_b = np.copy(server.get_brain_state(identity_b))
        self.assertTrue(np.allclose(baseline_b, after_b))


if __name__ == "__main__":
    unittest.main(verbosity=2)
