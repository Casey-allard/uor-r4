import json
import threading
import time
import unittest
import urllib.error
import urllib.request
from http.server import ThreadingHTTPServer

import server


class TestE2eApiJourney(unittest.TestCase):
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
            with urllib.request.urlopen(req, timeout=45) as r:
                return r.status, json.loads(r.read().decode("utf-8"))
        except urllib.error.HTTPError as e:
            payload = json.loads(e.read().decode("utf-8"))
            return e.code, payload

    def test_end_to_end_chat_to_verify_journey(self):
        identity = "tenant-e2e-main"
        text = "Plan resilient water and logistics coordination under seasonal stress."

        status_chat, chat = self.request_json("POST", "/api/chat", {"text": text, "identity": identity})
        self.assertEqual(status_chat, 200)

        metrics = chat["metrics"]
        self.assertIn("uor_payload", metrics)
        self.assertIn("uor_address", metrics)

        status_verify, verify = self.request_json(
            "POST",
            "/api/uor/verify",
            {"payload": metrics["uor_payload"], "address": metrics["uor_address"]},
        )
        self.assertEqual(status_verify, 200)
        self.assertTrue(verify["verified"])

    def test_end_to_end_identity_lifecycle_and_map_visibility(self):
        identity_a = "tenant-e2e-a"
        identity_b = "tenant-e2e-b"

        s1, _ = self.request_json("POST", "/api/chat", {"text": "alpha journey", "identity": identity_a})
        s2, _ = self.request_json("POST", "/api/chat", {"text": "beta journey", "identity": identity_b})
        self.assertEqual(s1, 200)
        self.assertEqual(s2, 200)

        # Allow background indexing thread to complete.
        time.sleep(0.25)

        s_sys, sysinfo = self.request_json("GET", "/api/sysinfo")
        self.assertEqual(s_sys, 200)
        self.assertGreaterEqual(int(sysinfo["session_states"]), 2)

        s_map, map_data = self.request_json("GET", "/api/map")
        self.assertEqual(s_map, 200)
        self.assertIn("points", map_data)
        self.assertIn("total", map_data)
        self.assertGreaterEqual(int(map_data["total"]), 0)

        scopes = {p.get("scope", "") for p in map_data.get("points", [])}
        self.assertIn(server.identity_key(identity_a), scopes)
        self.assertIn(server.identity_key(identity_b), scopes)

        s_reset, reset_payload = self.request_json("POST", "/api/reset", {"identity": identity_a})
        self.assertEqual(s_reset, 200)
        self.assertTrue(reset_payload.get("success"))

        # Identity still usable after reset.
        s_chat_after, chat_after = self.request_json("POST", "/api/chat", {"text": "post reset verification", "identity": identity_a})
        self.assertEqual(s_chat_after, 200)
        self.assertIn("metrics", chat_after)

    def test_end_to_end_capabilities_and_multialgo_attest(self):
        s_caps, caps = self.request_json("GET", "/api/uor/capabilities")
        self.assertEqual(s_caps, 200)

        algos = [entry["name"] for entry in caps.get("supported_hash_algorithms", [])]
        self.assertGreaterEqual(len(algos), 5)

        payload = {"entity": "e2e", "mode": "multihash"}
        for algo in algos:
            with self.subTest(algo=algo):
                s_att, att = self.request_json(
                    "POST",
                    "/api/uor/attest",
                    {"payload": payload, "hash_algorithm": algo, "include_multihash": True},
                )
                self.assertEqual(s_att, 200)
                address = att["uor"]["address"]
                self.assertTrue(address.startswith(f"{algo}:"))

                s_ver, ver = self.request_json("POST", "/api/uor/verify", {"payload": payload, "address": address})
                self.assertEqual(s_ver, 200)
                self.assertTrue(ver["verified"])
                self.assertEqual(ver["matched_hash_algorithm"], algo)


if __name__ == "__main__":
    unittest.main(verbosity=2)
