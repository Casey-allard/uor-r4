"""Tests for the harness core: front matter, redaction, receipts, brief assembly.

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

import core  # noqa: E402
import receipts  # noqa: E402
import registries  # noqa: E402


class TestFrontMatter(unittest.TestCase):
    def test_scalars_lists_and_nested(self) -> None:
        text = (
            "---\n"
            "name: demo\n"
            "version: 2\n"
            "flag: true\n"
            "ratio: 0.5\n"
            "tags: [a, b, c]\n"
            "block:\n"
            "  - one\n"
            "  - two\n"
            "nested:\n"
            "  route:\n"
            "    provider: deepseek-account\n"
            "    model: deepseek-flash\n"
            "---\n"
            "body text\n"
        )
        fm, body = core.parse_front_matter(text)
        self.assertEqual(fm["name"], "demo")
        self.assertEqual(fm["version"], 2)
        self.assertIs(fm["flag"], True)
        self.assertEqual(fm["ratio"], 0.5)
        self.assertEqual(fm["tags"], ["a", "b", "c"])
        self.assertEqual(fm["block"], ["one", "two"])
        self.assertEqual(fm["nested"]["route"]["model"], "deepseek-flash")
        self.assertEqual(body.strip(), "body text")

    def test_no_front_matter_is_not_an_error(self) -> None:
        fm, body = core.parse_front_matter("# just markdown\n")
        self.assertEqual(fm, {})
        self.assertIn("just markdown", body)

    def test_quoted_values_keep_their_text(self) -> None:
        fm, _ = core.parse_front_matter('---\nwhen: ["a, b", c]\n---\nx\n')
        self.assertEqual(fm["when"], ["a, b", "c"])


class TestRedaction(unittest.TestCase):
    def test_token_shapes_are_redacted(self) -> None:
        text = "token ghp_abcdefghijklmnop and sk-abcdefghijkl"
        out = core.redact(text)
        self.assertNotIn("ghp_abcdefghijklmnop", out)
        self.assertNotIn("sk-abcdefghijkl", out)
        self.assertIn("<redacted>", out)

    def test_secret_paths_are_recognised(self) -> None:
        self.assertTrue(core.is_secret_path(Path("/home/u/.runpod/config.toml")))
        self.assertTrue(core.is_secret_path(Path("/home/u/.dsh/.credentials.yaml")))
        self.assertFalse(core.is_secret_path(Path("/repo/docs/harness/README.md")))


class TestReceipts(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        root = Path(self.tmp.name)
        (root / "docs" / "harness" / "ledger").mkdir(parents=True)
        self.paths = core.HarnessPaths(repo=root, harness=root / "docs" / "harness", state=root / ".harness")

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def test_record_and_read_roundtrip(self) -> None:
        rec = receipts.record(self.paths, "skill", skill="research-plan-cycle", milestone="m1")
        self.assertEqual(rec["kind"], "skill")
        self.assertEqual(rec["event"], "harness_skill_used")
        self.assertIn("ts", rec)
        back = receipts.read(self.paths)
        self.assertEqual(len(back), 1)
        self.assertEqual(back[0]["skill"], "research-plan-cycle")

    def test_unknown_kind_is_rejected(self) -> None:
        with self.assertRaises(ValueError):
            receipts.record(self.paths, "nonsense")

    def test_secret_like_fields_are_dropped(self) -> None:
        """No token-shaped value may reach any shard, whatever file it lands in."""
        rec = receipts.record(self.paths, "team", role="systems-cost", token="ghp_abcdefghijklmnop")
        self.assertNotIn("token", rec)
        shards = sorted(self.paths.ledger_dir.glob("*.jsonl"))
        self.assertTrue(shards, "record() wrote a shard")
        for shard in shards:
            self.assertNotIn("ghp_", shard.read_text(), f"{shard.name} carries a token")

    def test_an_unattributed_receipt_warns_loudly(self) -> None:
        """Thirteen receipts accumulated unattributed before anything said so. A record that
        cannot name its lab defeats the visibility the shards exist for, so the warning is
        part of the contract, not a nicety."""
        import contextlib
        import io
        import os

        os.environ.pop("UOR_LAB", None)
        receipts._UNATTRIBUTED_WARNED[0] = False
        err = io.StringIO()
        with contextlib.redirect_stderr(err):
            rec = receipts.record(self.paths, "skill", skill="a")
        self.assertFalse(rec.get("lab"), "no lab, so the record is honest about it")
        self.assertIn("no lab identity", err.getvalue())
        self.assertIn("UOR_LAB", err.getvalue(), "the warning must name the fix")

    def test_the_warning_is_not_repeated_all_session(self) -> None:
        import contextlib
        import io
        import os

        os.environ.pop("UOR_LAB", None)
        receipts._UNATTRIBUTED_WARNED[0] = False
        err = io.StringIO()
        with contextlib.redirect_stderr(err):
            receipts.record(self.paths, "skill", skill="a")
            receipts.record(self.paths, "skill", skill="b")
        self.assertEqual(err.getvalue().count("no lab identity"), 1, "one warning per process, not per record")

    def test_receipts_are_sharded_per_lab(self) -> None:
        """Two labs must never write the same ledger file; that is what sharding is for."""
        import os

        os.environ["UOR_LAB"] = "lab-a"
        try:
            receipts.record(self.paths, "skill", skill="a")
            os.environ["UOR_LAB"] = "lab-b"
            receipts.record(self.paths, "skill", skill="b")
        finally:
            os.environ.pop("UOR_LAB", None)
        names = {p.name for p in self.paths.ledger_dir.glob("*.jsonl")}
        self.assertIn(f"lab-a-{core.month()}.jsonl", names)
        self.assertIn(f"lab-b-{core.month()}.jsonl", names)
        # and both are readable through the one public reader
        back = receipts.read(self.paths)
        self.assertEqual({r.get("skill") for r in back}, {"a", "b"})

    def test_summarize_counts_by_kind(self) -> None:
        receipts.record(self.paths, "skill", skill="a")
        receipts.record(self.paths, "skill", skill="a")
        receipts.record(self.paths, "team", role="r")
        s = receipts.summarize(self.paths)
        self.assertEqual(s["total"], 3)
        self.assertEqual(s["by_kind"], {"skill": 2, "team": 1})
        self.assertEqual(s["skills_used"], {"a": 2})

    def test_read_tolerates_a_corrupt_line(self) -> None:
        receipts.record(self.paths, "skill", skill="a")
        path = self.paths.ledger_dir / f"{core.month()}.jsonl"
        with path.open("a", encoding="utf-8") as fh:
            fh.write("{not json\n")
        self.assertEqual(len(receipts.read(self.paths)), 1)


class TestRealRegistries(unittest.TestCase):
    """The committed registries must actually load; a broken registry is a hard error."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.paths = core.HarnessPaths.discover(repo=REPO)
        cls.reg = registries.Registry.load(cls.paths)

    def test_milestones_present(self) -> None:
        for key in ("m1", "m2", "m3", "m4"):
            self.assertIsNotNone(self.reg.milestones.get(key), key)
        self.assertEqual(self.reg.milestones.get("m1").issue, 2029)
        self.assertEqual(self.reg.milestones.get("2029").id, "M1")

    def test_owner_rules_present(self) -> None:
        ids = {r["id"] for r in self.reg.milestones.owner_rules}
        self.assertLessEqual({"D21", "D22", "D23", "D24"}, ids)

    def test_milestone_headline_line_is_readable(self) -> None:
        line = self.reg.milestones.get("m1").headline_line()
        self.assertIn("M1", line)
        self.assertIn("->", line)

    def test_decisions_parse_and_include_d23(self) -> None:
        self.assertTrue(len(self.reg.decisions) >= 20)
        from registries import find_decision

        self.assertIsNotNone(find_decision(self.reg.decisions, "D23"))


if __name__ == "__main__":
    unittest.main()


class TestClockInjection(unittest.TestCase):
    """`--at` gives Python the same clock injection Rust's `status --at` already had.

    `parity.status_receipts_agreement` passed the instant to Rust alone, so it compared **Rust at an
    injected instant** against **Python at the wall clock**. It agreed only while the machine's local
    date happened to equal the injected date: `main` passed at `23:57Z`, the next run failed at
    `00:19Z` when the UTC date rolled over. Measured directly: at different instants the two sides
    render `2026-10-13: 188 receipts …` against `2026-10-14: 0 receipts …`.
    """

    def tearDown(self) -> None:
        core.set_clock(None)

    def test_an_injected_instant_controls_today_and_month(self) -> None:
        core.set_clock("2026-10-13T02:52:00-04:00")
        self.assertEqual(core.today(), "2026-10-13")
        self.assertEqual(core.month(), "2026-10")

    def test_a_different_instant_changes_the_buckets(self) -> None:
        """**This is the sensitivity the parity check needs.** With the clock injected, asking for a
        later date must move the day bucket - which is what the old wall-clock comparison could not
        do, and why it disagreed near midnight."""
        core.set_clock("2026-10-13T02:52:00-04:00")
        first = core.today()
        core.set_clock("2026-10-14T02:52:00-04:00")
        self.assertNotEqual(core.today(), first)
        self.assertEqual(core.today(), "2026-10-14")

    def test_no_injection_uses_the_real_clock(self) -> None:
        core.set_clock(None)
        self.assertEqual(core.today(), core.datetime.now(core.timezone.utc)
                         .astimezone().strftime("%Y-%m-%d"))

    def test_a_malformed_instant_falls_back_rather_than_raising(self) -> None:
        core.set_clock("not-a-date")
        self.assertEqual(len(core.today()), 10, "a bad injection must not break the render")
        self.assertEqual(len(core.month()), 7)
