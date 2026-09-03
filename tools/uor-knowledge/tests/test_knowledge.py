"""Only the FTS, provenance, scope, import, and read-only behavior contract."""

import hashlib
import json
import os
from pathlib import Path
import sqlite3
import tempfile
import unittest
from unittest.mock import patch

import uor_knowledge as knowledge


def item(identity, visibility="public", **changes):
    return {
        "id": identity, "kind": "research", "title": "Transport evidence",
        "body": "Quaternion transport preserves this snapshot.",
        "origin": "https://example.org/research", "revision": "commit-abc",
        "visibility": visibility, "evidence_status": "SOURCE_REVIEWED",
        "collected_at": "2026-09-03T03:00:00Z", **changes,
    }


def edge(source, target, visibility="public", **changes):
    return {"source": source, "target": target, "relation": "references",
            "basis": "Explicit cited source", "visibility": visibility, **changes}


class KnowledgeBehavior(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory(prefix="uor-knowledge-")
        self.addCleanup(self.directory.cleanup)
        self.path = Path(self.directory.name) / "index ?# %.sqlite3"
        environment = patch.dict(os.environ, {"UOR_KNOWLEDGE_DB": str(self.path)})
        environment.start()
        self.addCleanup(environment.stop)

    def ingest(self, *records):
        path = Path(self.directory.name) / "batch.jsonl"
        path.write_text("\n".join(json.dumps(record) for record in records) + "\n", encoding="utf-8")
        return knowledge.ingest(path)

    def test_fts_and_source_provenance(self):
        record = item("one", body="κ quaternion transport: evidence with Café.")
        self.ingest(record)
        result = knowledge.search_knowledge("κ quaternion Café")
        self.assertEqual([row["id"] for row in result["results"]], ["one"])
        self.assertEqual(result["results"][0]["revision"], record["revision"])
        self.assertEqual(result["results"][0]["content_sha256"], hashlib.sha256(record["body"].encode()).hexdigest())
        source = knowledge.get_source("one", offset=2, max_chars=10)
        self.assertEqual(source["text"], record["body"][2:12])
        self.assertEqual(source["next_offset"], 12)
        self.assertEqual(source["origin"], record["origin"])
        self.assertEqual(knowledge.search_knowledge('" OR * : ()')["results"], [])

    def test_visibility_on_search_source_edges_and_status(self):
        self.ingest(item("public-a"), item("public-b"), item("private-a", "private"), item("private-b", "private"),
                    edge("public-a", "public-b"), edge("public-a", "private-a", "private"),
                    edge("private-a", "private-b", "private"),
                    edge("public-a", "public-b", "private", relation="private-note", basis="CONFIDENTIAL"))
        self.assertEqual(len(knowledge.search_knowledge("transport")["results"]), 2)
        self.assertEqual(len(knowledge.search_knowledge("transport", "private")["results"]), 2)
        self.assertEqual(len(knowledge.search_knowledge("transport", "all")["results"]), 4)
        self.assertEqual(knowledge.get_source("private-a"), knowledge.get_source("absent"))
        self.assertEqual(knowledge.related_sources("private-a"), knowledge.related_sources("absent"))
        public_edges = knowledge.related_sources("public-a")["edges"]
        self.assertEqual(len(public_edges), 1)
        self.assertNotIn("CONFIDENTIAL", json.dumps(public_edges))
        self.assertEqual(len(knowledge.related_sources("public-b")["edges"]), 1)
        self.assertEqual(len(knowledge.related_sources("public-a", scope="all")["edges"]), 3)
        self.assertEqual(len(knowledge.related_sources("private-a", scope="private")["edges"]), 1)
        self.assertEqual(knowledge.get_source("private-a", scope="private")["visibility"], "private")
        self.assertEqual(knowledge.knowledge_status()["edges"], 1)
        self.assertEqual(knowledge.knowledge_status("all")["edges"], 4)
        self.assertEqual(sum(row["count"] for row in knowledge.knowledge_status()["counts"]), 2)
        self.assertIn("error", knowledge.related_sources("public-a", scope="invalid"))

    def test_idempotence_and_immutable_identity_collisions(self):
        one, two, link = item("one"), item("two"), edge("one", "two")
        first = self.ingest(link, one, two)  # Forward references are accepted within the batch.
        self.assertEqual((first["added"], first["added_edges"]), (2, 1))
        before = self.path.read_bytes()
        repeated = self.ingest(one, two, link)
        self.assertEqual((repeated["added"], repeated["added_edges"]), (0, 0))
        self.assertEqual((repeated["duplicate_items"], repeated["duplicate_edges"]), (2, 1))
        self.assertEqual(self.path.read_bytes(), before)
        self.ingest({**one, "collected_at": "2026-09-04T03:00:00Z"})
        self.assertEqual(knowledge.get_source("one")["collected_at"], one["collected_at"])
        for changed in ({"body": "Changed content"}, {"origin": "https://example.org/other"},
                        {"revision": "new-commit"}, {"visibility": "private"}, {"evidence_status": "NEW_STATUS"}):
            with self.assertRaisesRegex(ValueError, "identity collision"):
                self.ingest(item("would-be-new"), {**one, **changed})
            self.assertIn("error", knowledge.get_source("would-be-new"))
        with self.assertRaisesRegex(ValueError, "identity collision"):
            self.ingest({**link, "basis": "Changed basis"})
        self.assertEqual(knowledge.make_source_id(one), knowledge.make_source_id({**one, "collected_at": "later"}))
        self.assertNotEqual(knowledge.make_source_id(one), knowledge.make_source_id({**one, "revision": "later"}))
        self.assertEqual(len(knowledge.search_knowledge("transport")["results"]), 2)

    def test_validation_and_atomic_import(self):
        for invalid in ([], {**item("one"), "body": None}, {**item("one"), "visibility": "all"},
                        {**item("one"), "content_sha256": "0" * 64},
                        {**item("one"), "collected_at": "2026-09-03"},
                        {**item("one"), "unexpected": "discarding unknown data is unsafe"}):
            with self.assertRaises(ValueError):
                self.ingest(item("first"), invalid)
            self.assertFalse(self.path.exists())
        self.ingest(item("existing"))
        with self.assertRaisesRegex(ValueError, "endpoint"):
            self.ingest(item("rolled-back"), edge("existing", "missing"))
        self.assertIn("error", knowledge.get_source("rolled-back"))
        with self.assertRaisesRegex(ValueError, "public endpoints"):
            self.ingest(item("private", "private"), edge("existing", "private"))
        self.assertIn("error", knowledge.get_source("private", scope="all"))
        self.assertEqual(len(knowledge.search_knowledge("transport")["results"]), 1)

    def test_read_only_connection_closes_and_never_creates_missing_index(self):
        with self.assertRaises(FileNotFoundError):
            knowledge.knowledge_status()
        self.assertFalse(self.path.exists())
        self.ingest(item("one"))
        before = self.path.read_bytes()
        other = Path(self.directory.name) / "attached.sqlite3"
        with knowledge.connect() as db:
            for sql, params in (("DELETE FROM items", ()), ("CREATE TEMP TABLE leak (secret TEXT)", ()),
                                ("PRAGMA query_only=OFF", ()), ("ATTACH DATABASE ? AS other", (str(other),))):
                with self.assertRaises(sqlite3.DatabaseError):
                    db.execute(sql, params)
        with self.assertRaises(sqlite3.ProgrammingError):
            db.execute("SELECT 1")
        self.assertFalse(other.exists())
        self.assertEqual(self.path.read_bytes(), before)
        self.assertEqual(self.path.stat().st_mode & 0o777, 0o600)


if __name__ == "__main__":
    unittest.main()
