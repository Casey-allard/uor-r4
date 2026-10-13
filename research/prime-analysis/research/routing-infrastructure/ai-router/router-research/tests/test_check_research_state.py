import tempfile
import textwrap
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from tools import check_research_state


def _write(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(textwrap.dedent(content).lstrip())


class CheckResearchStateTest(unittest.TestCase):
    def test_repo_state_validates(self):
        errors = check_research_state.validate_state(check_research_state.ROOT)
        self.assertEqual(errors, [])

    def test_reports_mismatched_primary_increment(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            root = Path(tmpdir)
            _write(
                root / "docs/research/ACTIVE_STATE.md",
                """
                - Current primary RR: `RR-061`
                - Current primary INC: `INC-0136`
                - Current primary increment doc:
                  `docs/research/increments/INC_0136.md`
                - Kill-list stage: `measure-consistent shell routing`
                ## Earlier-Stage Justification
                - Justification:
                  `fixed-harness route law is the live gate`
                ## Deferred Branches
                - `INC-0135`:
                  `docs/research/increments/INC_0135.md`
                """,
            )
            _write(
                root / "docs/research/KILL_LIST_TRACKER.md",
                """
                - Current primary RR: `RR-061`
                - Current primary INC: `INC-0136`
                ## 1. Hyperbolic Embedding Stability
                - Status: `partial`
                - Next branch:
                  - `deferred`
                ## 2. Measure-Consistent Shell Routing
                - Status: `open`
                - Next branch:
                  - `INC-0136`
                """,
            )
            _write(
                root / "docs/routes/ROUTE_MATRIX.md",
                """
                - Current primary RR: `RR-061`
                - Current primary INC: `INC-0135`
                """,
            )
            _write(
                root / "docs/program/PROJECT_BOARD.md",
                """
                - Current primary RR: `RR-061`
                - Current primary INC: `INC-0136`
                """,
            )
            _write(
                root / "docs/program/ISSUE_REGISTRY.md",
                """
                - Current primary RR: `RR-061`
                - Current primary INC: `INC-0136`
                """,
            )
            _write(
                root / "docs/research/increments/INC_0136.md",
                """
                ## Status
                Queued next.
                ## Kill-List Stage
                x
                ## Mathematical Object
                x
                ## Success Condition
                x
                ## Falsification Condition
                x
                """,
            )
            _write(
                root / "docs/research/increments/INC_0135.md",
                """
                ## Status
                Deferred after geometry return.
                ## Kill-List Stage
                x
                ## Mathematical Object
                x
                ## Success Condition
                x
                ## Falsification Condition
                x
                """,
            )

            errors = check_research_state.validate_state(root)
            self.assertTrue(any("primary INC mismatch" in error for error in errors))


if __name__ == "__main__":
    unittest.main()
