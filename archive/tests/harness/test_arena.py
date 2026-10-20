"""Tests for the regression arena: lineage, direction-aware comparison, and its own limits.

Run from the repo root:  python3 -m unittest discover -s tests/harness -t . -v
"""

from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
HARNESS = REPO / "scripts" / "harness"
if str(HARNESS) not in sys.path:
    sys.path.insert(0, str(HARNESS))

import arena as A  # noqa: E402
import core  # noqa: E402
import registries  # noqa: E402


def _paths(tmp: Path) -> core.HarnessPaths:
    """A temp harness root with the minimum registries Registry.load needs."""
    harness = tmp / "docs" / "harness"
    harness.mkdir(parents=True, exist_ok=True)
    (harness / "milestones.json").write_text(json.dumps({
        "schema": 1,
        "tracker": {"issue": 2028},
        "milestones": {"m1": {"issue": 2029, "id": "M1", "title": "t", "lab": "l", "status": "in-progress",
                              "headline": {"label": "x", "current": 0, "target": 1, "of": 1},
                              "secondary": [], "panel": "v5-reply", "base_artifact": "", "mechanisms": [],
                              "lines": [], "next": ""}},
        "owner_rules": [],
    }), encoding="utf-8")
    return core.HarnessPaths(repo=tmp, harness=harness, state=tmp / ".harness")


def _write(tmp: Path, doc: dict) -> None:
    (tmp / "docs" / "harness" / "arena.json").write_text(json.dumps(doc), encoding="utf-8")


def _row(cid: str, **kw) -> dict:
    base = {
        "id": cid, "artifact": f"artifact-{cid}", "milestone": "m1", "date": "2026-10-12",
        "parent": None, "verdict": "KEEP", "confidence": "provisional", "mechanism": "m",
        "panel": "v5-reply", "seeds": 1, "rows": 10, "scope": "a scope", "scores": {}, "record": "",
    }
    base.update(kw)
    return base


BARS = {
    "recall": {"direction": "higher_is_better", "unit": "recall"},
    "cost": {"direction": "lower_is_better", "unit": "candidates/query"},
}


class TestDirectionAwareComparison(unittest.TestCase):
    """The bug this guards against: a cost increase reported as a gain."""

    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.paths = _paths(self.root)
        _write(self.root, {"schema": 1, "bars": BARS, "limits": "x" * 60, "candidates": [
            _row("p", scores={"recall": 0.3, "cost": 36}),
            _row("c", parent="p", scores={"recall": 0.4, "cost": 69}),
        ]})
        self.reg = registries.Registry.load(self.paths)

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def test_a_higher_cost_is_a_regression_not_a_gain(self) -> None:
        r = A.compare(self.paths, self.reg, "c")
        lost = {e["bar"] for e in r["lost"]}
        gained = {e["bar"] for e in r["gained"]}
        self.assertIn("cost", lost, "69 candidates/query is worse than 36, not better")
        self.assertNotIn("cost", gained)
        self.assertIn("recall", gained)

    def test_an_equal_value_is_neither_gain_nor_regression(self) -> None:
        _write(self.root, {"schema": 1, "bars": BARS, "limits": "x" * 60, "candidates": [
            _row("p", scores={"recall": 0.5, "cost": 36}),
            _row("c", parent="p", scores={"recall": 0.5, "cost": 36}),
        ]})
        r = A.compare(self.paths, self.reg, "c")
        self.assertEqual(r["gained"], [])
        self.assertEqual(r["lost"], [])
        self.assertEqual(len(r["unchanged"]), 2)

    def test_an_undeclared_direction_is_flagged_not_guessed(self) -> None:
        _write(self.root, {"schema": 1, "bars": {}, "limits": "x" * 60, "candidates": [
            _row("p", scores={"mystery": 1}),
            _row("c", parent="p", scores={"mystery": 9}),
        ]})
        r = A.compare(self.paths, self.reg, "c")
        self.assertEqual(r["gained"], [])
        self.assertEqual(r["lost"], [])
        self.assertIn("note", r["unchanged"][0])

    def test_a_missing_bar_is_unavailable_never_zero(self) -> None:
        _write(self.root, {"schema": 1, "bars": BARS, "limits": "x" * 60, "candidates": [
            _row("p", scores={"recall": 0.3, "cost": 36}),
            _row("c", parent="p", scores={"recall": 0.4}),
        ]})
        r = A.compare(self.paths, self.reg, "c")
        self.assertEqual([e["bar"] for e in r["unavailable"]], ["cost"])

    def test_a_root_row_is_not_comparable(self) -> None:
        r = A.compare(self.paths, self.reg, "p")
        self.assertFalse(r["comparable"])
        self.assertIn("root", r["reason"])


class TestScopeGates(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.paths = _paths(self.root)
        self.reg = registries.Registry.load(self.paths)

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def test_an_uninterpretable_row_is_not_comparable(self) -> None:
        _write(self.root, {"schema": 1, "bars": BARS, "limits": "x" * 60, "candidates": [
            _row("p"), _row("c", parent="p", confidence="uninterpretable"),
        ]})
        r = A.compare(self.paths, self.reg, "c")
        self.assertFalse(r["comparable"])

    def test_a_decisive_claim_without_power_is_blocked(self) -> None:
        _write(self.root, {"schema": 1, "bars": BARS, "limits": "x" * 60, "candidates": [
            _row("p"), _row("c", parent="p", confidence="decisive", seeds=1, rows=10),
        ]})
        r = A.compare(self.paths, self.reg, "c")
        self.assertTrue(any("2 seeds" in b or "200 fresh rows" in b for b in r["blockers"]), r["blockers"])

    def test_a_keep_without_transfer_is_blocked(self) -> None:
        _write(self.root, {"schema": 1, "bars": BARS, "limits": "x" * 60, "candidates": [
            _row("p"), _row("c", parent="p", verdict="KEEP", scope="no transfer reported", scores={"recall": 1}),
        ]})
        r = A.compare(self.paths, self.reg, "c")
        self.assertTrue(any("transfer" in b for b in r["blockers"]), r["blockers"])

    def test_a_transfer_number_satisfies_the_requirement(self) -> None:
        _write(self.root, {"schema": 1, "bars": BARS, "limits": "x" * 60, "candidates": [
            _row("p"),
            _row("c", parent="p", verdict="KEEP", scope="fresh transfer: 0.41 on 200 fresh rows",
                 rows=200, scores={"recall": 1}),
        ]})
        r = A.compare(self.paths, self.reg, "c")
        self.assertFalse(any("transfer" in b for b in r["blockers"]), r["blockers"])

    def test_the_word_transfer_alone_is_not_evidence(self) -> None:
        """'no transfer reported' contains the word; it must not satisfy the requirement."""
        _write(self.root, {"schema": 1, "bars": BARS, "limits": "x" * 60, "candidates": [
            _row("p"), _row("c", parent="p", verdict="KEEP", scope="transfer not measured", scores={"recall": 1}),
        ]})
        r = A.compare(self.paths, self.reg, "c")
        self.assertTrue(any("fresh transfer" in b for b in r["blockers"]), r["blockers"])

    def test_a_panel_change_stops_the_comparison(self) -> None:
        _write(self.root, {"schema": 1, "bars": BARS, "limits": "x" * 60, "candidates": [
            _row("p", panel="v5-reply"), _row("c", parent="p", panel="m4-served"),
        ]})
        r = A.compare(self.paths, self.reg, "c")
        self.assertFalse(r["comparable"])
        self.assertTrue(any("panel changed" in b for b in r["blockers"]))


class TestLineage(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.paths = _paths(self.root)
        _write(self.root, {"schema": 1, "bars": BARS, "limits": "x" * 60, "candidates": [
            _row("a"), _row("b", parent="a"), _row("c", parent="b"),
        ]})

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def test_lineage_walks_from_the_root_down(self) -> None:
        self.assertEqual([c.id for c in A.load(self.paths).lineage("c")], ["a", "b", "c"])

    def test_a_cycle_does_not_hang(self) -> None:
        _write(self.root, {"schema": 1, "bars": BARS, "limits": "x" * 60, "candidates": [
            _row("a", parent="b"), _row("b", parent="a"),
        ]})
        self.assertTrue(A.load(self.paths).lineage("a"))


class TestCommittedArena(unittest.TestCase):
    """The committed registry must be valid and must state what it cannot measure."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.paths = core.HarnessPaths.discover(repo=REPO)
        cls.reg = registries.Registry.load(cls.paths)
        cls.arena = A.load(cls.paths)

    def test_rows_exist_and_are_valid(self) -> None:
        self.assertTrue(self.arena.rows, "the committed arena has rows")
        for c in self.arena.rows:
            self.assertTrue(c.id)
            self.assertTrue(c.verdict in A.VERDICTS, f"{c.id}: {c.verdict}")
            self.assertTrue(c.confidence in A.CONFIDENCE, f"{c.id}: {c.confidence}")

    def test_every_cited_record_exists(self) -> None:
        for c in self.arena.rows:
            if c.record:
                self.assertTrue((REPO / c.record).exists(), f"{c.id} cites {c.record}")

    def test_parents_resolve(self) -> None:
        ids = {c.id for c in self.arena.rows}
        for c in self.arena.rows:
            if c.parent:
                self.assertIn(c.parent, ids, c.id)

    def test_it_states_its_own_limits(self) -> None:
        limits = str(self.arena.raw.get("limits", ""))
        self.assertGreater(len(limits), 40)
        self.assertIn("cannot", limits.lower())

    def test_every_used_bar_declares_a_direction(self) -> None:
        declared = set(self.arena.raw.get("bars", {}))
        used = {k for c in self.arena.rows for k in c.scores}
        self.assertTrue(used <= declared, f"undeclared bars in use: {sorted(used - declared)}")


if __name__ == "__main__":
    unittest.main()
