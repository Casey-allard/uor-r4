import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from tools import context_bootstrap


class ContextBootstrapTest(unittest.TestCase):
    def test_startup_group_includes_core_project_goals_near_front(self):
        startup = context_bootstrap.GROUPS["startup"]
        self.assertEqual(startup[1], context_bootstrap.ROOT / "CORE_PROJECT_GOALS.md")

    def test_startup_group_includes_canonical_state_files(self):
        startup = context_bootstrap.GROUPS["startup"]
        self.assertIn(
            context_bootstrap.ROOT / "docs/research/KILL_LIST_TRACKER.md", startup
        )
        self.assertIn(
            context_bootstrap.ROOT / "docs/research/ACTIVE_STATE.md", startup
        )
        self.assertIn(
            context_bootstrap.ROOT / "docs/research/SUPPORTING_EVIDENCE.md", startup
        )

    def test_theory_group_includes_core_project_goals(self):
        theory = context_bootstrap.GROUPS["theory"]
        self.assertIn(context_bootstrap.ROOT / "CORE_PROJECT_GOALS.md", theory)


if __name__ == "__main__":
    unittest.main()
