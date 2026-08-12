import json
import math
import threading
import unittest
import urllib.error
import urllib.request
from http.server import ThreadingHTTPServer
from pathlib import Path

import numpy as np

import server


MODEL_INVARIANTS = {
    "identity_required": "All routing and chat requests require explicit identity.",
    "identity_determinism": "Identity maps deterministically to UOR profile and QIMC seed.",
    "uor_first_attestation": "Every routed result includes UOR payload and attestation metadata.",
    "uor_control_plane": "Identity-derived UOR control affects routing score and Hopf transport.",
    "route_contract": "Routing returns one winning route and one score-bearing route per window.",
    "api_verification": "HTTP endpoints verify attestations and expose supported UOR capabilities.",
    "state_isolation": "Per-identity state evolution is isolated and does not cross-contaminate.",
}

VECTORS_PATH = Path(__file__).with_name("vv_test_vectors.json")


class ValidationError(AssertionError):
    pass


class VvAssertsMixin:
    def require(self, condition: bool, msg: str):
        if not condition:
            raise ValidationError(msg)

    def assert_float_range(self, value: float, lo: float, hi: float, name: str):
        self.require(isinstance(value, (float, int)), f"{name} must be numeric")
        self.require(lo <= float(value) <= hi, f"{name} out of range [{lo}, {hi}]: {value}")


class TestConceptualModelFunctionContracts(unittest.TestCase, VvAssertsMixin):
    @classmethod
    def setUpClass(cls):
        with VECTORS_PATH.open("r", encoding="utf-8") as f:
            cls.vectors = json.load(f)

    def test_invariants_declared(self):
        self.require(len(MODEL_INVARIANTS) >= 7, "Conceptual model invariants are incomplete")

    def test_identity_profile_and_qimc_determinism(self):
        identity = "tenant-gambia-water"

        p1 = server.identity_to_uor_profile(identity)
        p2 = server.identity_to_uor_profile(identity)

        self.require(p1["identity_uor_address"] == p2["identity_uor_address"], "Identity address not deterministic")
        self.require(p1["identity_uor_digest"] == p2["identity_uor_digest"], "Identity digest not deterministic")
        self.require(p1["identity_uor_hash_algorithm"] == "sha256", "Text identity primary hash must be sha256")

        prime1, idx1, meta1 = server.identity_to_qimc_prime(identity)
        prime2, idx2, meta2 = server.identity_to_qimc_prime(identity)
        self.require(prime1 == prime2 and idx1 == idx2, "QIMC mapping is not deterministic")
        self.require(meta1["identity_uor_digest"] == meta2["identity_uor_digest"], "QIMC digest mismatch")

    def test_uor_identity_passthrough(self):
        source_identity = "tenant-grid-ops"
        addr = server.identity_to_uor_profile(source_identity)["identity_uor_address"]

        profile = server.identity_to_uor_profile(addr)
        self.require(profile["identity_uor_address"] == addr, "UOR identity should pass through unchanged")
        self.require(profile["identity_uor_hash_algorithm"] == "sha256", "UOR identity hash algorithm mismatch")

    def test_attestation_contract_for_all_hash_algorithms(self):
        payload = {
            "event": "strict-vv",
            "window": 7,
            "severity": "high",
            "actors": ["router", "validator"],
        }

        for algo in server.SUPPORTED_UOR_HASH_ORDER:
            att = server.generate_uor_attestation(payload, hash_algorithm=algo, include_multihash=True)
            self.require(att["address"].startswith(f"{algo}:"), f"Address prefix mismatch for {algo}")
            self.require(att["hash_algorithm"] == algo, f"hash_algorithm mismatch for {algo}")
            self.require(att["multihash_addresses"].get(algo) == att["address"], f"multihash mismatch for {algo}")

            verify_result = str(att.get("verify_result", ""))
            self.require(
                verify_result == "witness-unavailable" or verify_result.startswith(f"{algo}:"),
                f"Unexpected verify result for {algo}: {verify_result}",
            )

    def test_route_contract_against_real_world_vectors(self):
        for vector in self.vectors:
            with self.subTest(vector=vector["name"]):
                routed_pack = server.route_query_to_manifold(
                    vector["query"],
                    include_eigenvalues=True,
                    identity=vector["identity"],
                )
                routed = routed_pack["routed"]
                all_routes = routed_pack["all_routes"]

                self.require(len(all_routes) == server.NUM_WINDOWS, "all_routes length must equal NUM_WINDOWS")

                best_window = int(routed["window_index"])
                self.require(1 <= best_window <= server.NUM_WINDOWS, "window_index out of range")

                self.require(len(routed["eigenvalues"]) == 8, "Expected top 8 eigenvalues")
                self.require(len(routed["state_vector"]) > 0, "State vector slice must be non-empty")

                qimc = routed["qimc"]
                hopf = routed["hopf"]
                uor = routed["uor"]

                self.require(qimc["identity"] == vector["identity"].lower(), "QIMC identity mismatch")
                self.require(qimc["identity_uor_address"].startswith("sha256:"), "Identity UOR address must be sha256")
                self.require("uor_control" in qimc, "Missing uor_control in QIMC payload")

                self.assert_float_range(qimc["uor_control"]["entropy_bias"], 0.0, 1.0, "entropy_bias")
                self.assert_float_range(hopf["phase_transport_lambda"], 0.70, 1.30, "phase_transport_lambda")
                self.require(2 <= int(hopf["hopf_chi_bins"]) <= 4, "hopf_chi_bins out of range")

                self.require("uor_payload" in routed and routed["uor_payload"], "Missing uor_payload")
                self.require(uor["address"].startswith("sha256:"), "Route attestation should default to sha256")

                recomputed = server.generate_uor_attestation(routed["uor_payload"], hash_algorithm="sha256")
                self.require(recomputed["address"] == routed["uor_address"], "UOR address does not match attested payload")

                scores = [float(item["routing_score"]) for item in all_routes]
                best_from_scores = int(all_routes[scores.index(max(scores))]["window_index"])
                self.require(best_window == best_from_scores, "Winning route does not match max routing score")

                rerun = server.route_query_to_manifold(
                    vector["query"],
                    include_eigenvalues=True,
                    identity=vector["identity"],
                )
                self.require(
                    rerun["routed"]["window_index"] == routed["window_index"],
                    "Routing is not deterministic for same identity/query",
                )
                self.require(
                    rerun["routed"]["qimc"]["identity_uor_digest"] == routed["qimc"]["identity_uor_digest"],
                    "Identity digest changed between equivalent runs",
                )

    def test_state_isolation_between_identities(self):
        id_a = "tenant-isolation-a"
        id_b = "tenant-isolation-b"

        server.reset_brain_state(identity=id_a)
        server.reset_brain_state(identity=id_b)

        baseline_b = np.copy(server.get_brain_state(id_b))
        _ = server.evolve_brain_state("a long evolving prompt about drought planning", gamma=0.5, identity=id_a)
        after_b = np.copy(server.get_brain_state(id_b))

        self.require(np.allclose(baseline_b, after_b), "State isolation failed: identity B changed after evolving identity A")


class TestConceptualModelHttpContracts(unittest.TestCase, VvAssertsMixin):
    @classmethod
    def setUpClass(cls):
        cls._original_save_manifold_cache = server.save_manifold_cache
        server.save_manifold_cache = lambda *args, **kwargs: None
        cls.httpd = ThreadingHTTPServer(("127.0.0.1", 0), server.RouterAPIHandler)
        cls.port = cls.httpd.server_address[1]
        cls.thread = threading.Thread(target=cls.httpd.serve_forever, daemon=True)
        cls.thread.start()

    @classmethod
    def tearDownClass(cls):
        cls.httpd.shutdown()
        cls.httpd.server_close()
        server.save_manifold_cache = cls._original_save_manifold_cache

    def request_json(self, method: str, path: str, body=None):
        url = f"http://127.0.0.1:{self.port}{path}"
        data = None
        headers = {}
        if body is not None:
            data = json.dumps(body).encode("utf-8")
            headers["Content-Type"] = "application/json"
        req = urllib.request.Request(url, data=data, headers=headers, method=method)

        try:
            with urllib.request.urlopen(req, timeout=30) as r:
                status = r.status
                payload = json.loads(r.read().decode("utf-8"))
                return status, payload
        except urllib.error.HTTPError as e:
            payload = json.loads(e.read().decode("utf-8"))
            return e.code, payload

    def test_uor_capabilities_endpoint(self):
        status, payload = self.request_json("GET", "/api/uor/capabilities")
        self.require(status == 200, "Capabilities endpoint failed")
        self.require(payload["content_codec"] == "json", "Unexpected content codec")
        self.require(payload["default_hash_algorithm"] == "sha256", "Default hash algorithm mismatch")

        names = [item["name"] for item in payload["supported_hash_algorithms"]]
        self.require(names == list(server.SUPPORTED_UOR_HASH_ORDER), "Supported hash algorithms mismatch")

    def test_attest_and_verify_endpoints_for_all_hash_algorithms(self):
        payload = {
            "deployment": "field-router",
            "region": "west-africa",
            "timestamp": "2026-05-23T00:00:00Z",
        }
        for algo in server.SUPPORTED_UOR_HASH_ORDER:
            with self.subTest(algo=algo):
                status_att, att = self.request_json(
                    "POST",
                    "/api/uor/attest",
                    {"payload": payload, "hash_algorithm": algo, "include_multihash": True},
                )
                self.require(status_att == 200, f"Attest endpoint failed for {algo}")
                addr = att["uor"]["address"]
                self.require(addr.startswith(f"{algo}:"), f"Address prefix mismatch for {algo}")

                status_ver, ver = self.request_json(
                    "POST",
                    "/api/uor/verify",
                    {"payload": payload, "address": addr},
                )
                self.require(status_ver == 200, f"Verify endpoint failed for {algo}")
                self.require(ver["verified"] is True, f"Verify endpoint mismatch for {algo}")
                self.require(ver["matched_hash_algorithm"] == algo, f"Matched algorithm mismatch for {algo}")

    def test_chat_identity_enforcement_and_response_contract(self):
        status_bad, payload_bad = self.request_json("POST", "/api/chat", {"text": "hello"})
        self.require(status_bad == 400, "Chat endpoint must reject missing identity")
        self.require("identity is required" in payload_bad.get("error", ""), "Missing expected chat rejection message")

        status_ok, payload_ok = self.request_json(
            "POST",
            "/api/chat",
            {"text": "Validate strict conceptual model behavior", "identity": "tenant-chat-vv"},
        )
        self.require(status_ok == 200, "Chat endpoint failed for valid identity")

        metrics = payload_ok.get("metrics", {})
        self.require("qimc" in metrics and "hopf" in metrics, "Chat response missing QIMC/Hopf metrics")
        self.require("auto_tuned" in metrics, "Chat response missing auto_tuned metrics")
        self.require("uor_entropy_bias" in metrics["auto_tuned"], "Missing uor_entropy_bias in auto_tuned metrics")

        q = metrics["qimc"]
        self.require("uor_control" in q, "Missing uor_control in chat QIMC payload")
        self.assert_float_range(q["uor_control"]["entropy_bias"], 0.0, 1.0, "chat entropy_bias")


if __name__ == "__main__":
    print("Strict V&V conceptual model invariants:")
    for key, value in MODEL_INVARIANTS.items():
        print(f"- {key}: {value}")
    print("")
    unittest.main(verbosity=2)
