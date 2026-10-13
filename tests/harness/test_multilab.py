"""Tests for multi-lab coordination: presence, committed claims, ledger shards.

The properties that matter at four labs: a claim must be visible to labs that were never
told about it, a stale presence file must never read as live, and two labs must never write
the same ledger file.

Run from the repo root:  python3 -m unittest discover -s tests/harness -t . -v
"""

from __future__ import annotations

import json
import os
import sys
import tempfile
import unittest
from datetime import datetime, timedelta, timezone
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
HARNESS = REPO / "scripts" / "harness"
if str(HARNESS) not in sys.path:
    sys.path.insert(0, str(HARNESS))

import core  # noqa: E402
import multilab  # noqa: E402
import receipts  # noqa: E402


def _paths(tmp: Path) -> core.HarnessPaths:
    harness = tmp / "docs" / "harness"
    (harness / "ledger").mkdir(parents=True, exist_ok=True)
    (harness / "milestones.json").write_text(json.dumps({
        "schema": 1, "tracker": {"issue": 2028}, "owner_rules": [],
        "milestones": {"m1": {"issue": 2029, "id": "M1", "title": "t", "lab": "l", "status": "in-progress",
                              "headline": {"label": "x", "current": 0, "target": 1, "of": 1},
                              "secondary": [], "panel": "p", "base_artifact": "", "mechanisms": [],
                              "lines": [], "next": ""}},
    }), encoding="utf-8")
    return core.HarnessPaths(repo=tmp, harness=harness, state=tmp / ".harness")


class TestPresence(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.paths = _paths(Path(self.tmp.name))

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def test_presence_is_committed_not_hidden_in_state(self) -> None:
        """A presence file a lab cannot see is not presence."""
        multilab.write_presence(self.paths, "kimi", milestone="m1", doing="OP2 kernel")
        path = multilab.presence_path(self.paths, "kimi")
        self.assertTrue(path.exists())
        self.assertTrue(str(path).startswith(str(self.paths.harness)), "presence lives under docs/harness")
        self.assertFalse(str(path).startswith(str(self.paths.state)), "not under the gitignored state dir")

    def test_a_fresh_presence_reads_live(self) -> None:
        multilab.write_presence(self.paths, "kimi", milestone="m1", doing="work")
        p = multilab.read_presence(self.paths, "kimi")
        self.assertIsNotNone(p)
        self.assertFalse(p.stale)
        self.assertIn("live", p.state)

    def test_a_stale_presence_never_reads_as_live(self) -> None:
        """The important one: an abandoned lab must not look like a running lab."""
        multilab.write_presence(self.paths, "claude", milestone="m1")
        path = multilab.presence_path(self.paths, "claude")
        data = json.loads(path.read_text())
        data["updated"] = (datetime.now(timezone.utc) - timedelta(hours=3)).isoformat(timespec="seconds")
        path.write_text(json.dumps(data), encoding="utf-8")
        p = multilab.read_presence(self.paths, "claude")
        self.assertTrue(p.stale)
        self.assertIn("STALE", p.state)

    def test_an_unparseable_timestamp_is_stale_not_live(self) -> None:
        multilab.write_presence(self.paths, "codex", milestone="m1")
        path = multilab.presence_path(self.paths, "codex")
        data = json.loads(path.read_text())
        data["updated"] = "not-a-timestamp"
        path.write_text(json.dumps(data), encoding="utf-8")
        self.assertTrue(multilab.read_presence(self.paths, "codex").stale)

    def test_presence_requires_a_lab(self) -> None:
        with self.assertRaises(ValueError):
            multilab.write_presence(self.paths, "", milestone="m1")

    def test_all_presence_lists_every_lab_stale_last(self) -> None:
        multilab.write_presence(self.paths, "kimi", milestone="m1")
        multilab.write_presence(self.paths, "claude", milestone="m1")
        path = multilab.presence_path(self.paths, "claude")
        data = json.loads(path.read_text())
        data["updated"] = (datetime.now(timezone.utc) - timedelta(hours=5)).isoformat(timespec="seconds")
        path.write_text(json.dumps(data), encoding="utf-8")
        rows = multilab.all_presence(self.paths)
        self.assertEqual([r.lab for r in rows], ["kimi", "claude"], "live labs first, stale last")


class TestCommittedClaims(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.paths = _paths(Path(self.tmp.name))

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def test_a_claim_is_written_to_the_committed_store(self) -> None:
        ok, rec = multilab.claim(self.paths, "branch-a", "kimi")
        self.assertTrue(ok)
        self.assertTrue(multilab.claims_file(self.paths).exists())
        self.assertIn("docs/harness", str(multilab.claims_file(self.paths)))

    def test_another_lab_sees_the_claim(self) -> None:
        """The whole point: lab B must be refused a branch lab A holds, with no prior contact."""
        multilab.claim(self.paths, "branch-a", "kimi")
        ok, held = multilab.claim(self.paths, "branch-a", "claude")
        self.assertFalse(ok)
        self.assertEqual(held["lab"], "kimi")

    def test_a_lab_can_reclaim_its_own_branch(self) -> None:
        multilab.claim(self.paths, "branch-a", "kimi")
        ok, _ = multilab.claim(self.paths, "branch-a", "kimi")
        self.assertTrue(ok)

    def test_a_stale_claim_does_not_block_forever(self) -> None:
        multilab.claim(self.paths, "branch-a", "kimi")
        path = multilab.claims_file(self.paths)
        data = json.loads(path.read_text())
        data["branch-a"]["claimed_at"] = (datetime.now(timezone.utc) - timedelta(days=2)).isoformat(timespec="seconds")
        path.write_text(json.dumps(data), encoding="utf-8")
        ok, _ = multilab.claim(self.paths, "branch-a", "claude")
        self.assertTrue(ok, "a two-day-old claim is suspect, not a permanent lock")

    def test_claims_require_an_owner(self) -> None:
        with self.assertRaises(ValueError):
            multilab.claim(self.paths, "branch-a", "")

    def test_release_frees_the_branch(self) -> None:
        multilab.claim(self.paths, "branch-a", "kimi")
        ok, _ = multilab.release(self.paths, "branch-a", "kimi")
        self.assertTrue(ok)
        self.assertIsNone(multilab.claim_of(self.paths, "branch-a"))

    def test_another_lab_cannot_release_your_claim(self) -> None:
        multilab.claim(self.paths, "branch-a", "kimi")
        ok, _ = multilab.release(self.paths, "branch-a", "claude")
        self.assertFalse(ok)
        self.assertIsNotNone(multilab.claim_of(self.paths, "branch-a"))


class TestShardedLedger(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.paths = _paths(Path(self.tmp.name))

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def test_two_labs_write_different_files(self) -> None:
        os.environ["UOR_LAB"] = "kimi"
        try:
            receipts.record(self.paths, "skill", skill="a")
        finally:
            os.environ["UOR_LAB"] = "claude"
        try:
            receipts.record(self.paths, "skill", skill="b")
        finally:
            os.environ.pop("UOR_LAB", None)
        shards = sorted(p.name for p in self.paths.ledger_dir.glob("*.jsonl"))
        self.assertEqual(len(shards), 2, shards)
        self.assertTrue(all("-" in s for s in shards), "every shard is lab-prefixed")

    def test_readers_see_every_lab(self) -> None:
        os.environ["UOR_LAB"] = "kimi"
        try:
            receipts.record(self.paths, "skill", skill="a")
        finally:
            os.environ.pop("UOR_LAB", None)
        os.environ["UOR_LAB"] = "claude"
        try:
            receipts.record(self.paths, "skill", skill="b")
        finally:
            os.environ.pop("UOR_LAB", None)
        labs = {r.get("lab") for r in receipts.read(self.paths)}
        self.assertEqual(labs, {"kimi", "claude"})

    def test_migration_preserves_every_receipt(self) -> None:
        legacy = self.paths.ledger_dir / f"{core.month()}.jsonl"
        rows = [
            {"ts": "2026-10-01T00:00:00+00:00", "kind": "skill", "skill": "x", "lab": "kimi"},
            {"ts": "2026-10-01T00:01:00+00:00", "kind": "skill", "skill": "y"},
            "{ not json at all",
        ]
        legacy.write_text("\n".join(json.dumps(r) if isinstance(r, dict) else r for r in rows) + "\n", encoding="utf-8")
        result = multilab.migrate_ledger(self.paths)
        self.assertFalse(legacy.exists(), "the unsharded file is gone")
        self.assertEqual(result["total"], 3, "including the unreadable line, which is never dropped")
        readable = multilab.read_shards(self.paths)
        self.assertEqual(len(readable), 2, "the two valid receipts moved into shards")
        self.assertEqual({r["lab"] for r in readable}, {"kimi", "unknown"}, "one attributed, one not")
        # the unparseable line is preserved in an explicit quarantine shard, never discarded
        quarantine = self.paths.ledger_dir / f"unreadable-{core.month()}.jsonl"
        self.assertTrue(quarantine.exists(), "an unreadable line must be quarantined, not dropped")
        self.assertIn("not json", quarantine.read_text())


if __name__ == "__main__":
    unittest.main()
