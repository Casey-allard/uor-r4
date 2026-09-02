"""The complete #1067 fit retains the #1063 clock without executing a model."""

from __future__ import annotations

import ast
import unittest
from pathlib import Path

PACKAGE = Path(__file__).resolve().parents[1] / "src/r4_softmax_trainer"
RENAMED_STRINGS = {
    "One fixed, resumable query-token readout fit; no development scoring.": (
        "One fixed, resumable English-binding fit; no development scoring."
    ),
    "uor-r4.zoology-query-readout-fit-start/1": "uor-r4.zoology-english-fit-start/1",
    "uor-r4.zoology-query-readout-fit-checkpoint/1": (
        "uor-r4.zoology-english-fit-checkpoint/1"
    ),
    "uor-r4.zoology-query-readout-fit/1": "uor-r4.zoology-english-fit/1",
}


class _NormalizeIdentity(ast.NodeTransformer):
    def __init__(self) -> None:
        self.renamed: list[str] = []
        self.issue_fields = 0

    def visit_Constant(self, node: ast.Constant) -> ast.AST:
        if isinstance(node.value, str) and node.value in RENAMED_STRINGS:
            self.renamed.append(node.value)
            return ast.copy_location(ast.Constant(RENAMED_STRINGS[node.value]), node)
        return node

    def visit_Dict(self, node: ast.Dict) -> ast.AST:
        self.generic_visit(node)
        for index, (key, value) in enumerate(zip(node.keys, node.values, strict=True)):
            if (
                isinstance(key, ast.Constant)
                and key.value == "issue"
                and isinstance(value, ast.Attribute)
                and isinstance(value.value, ast.Name)
                and value.value.id == "contract"
                and value.attr == "ISSUE"
            ):
                self.issue_fields += 1
                node.values[index] = ast.copy_location(ast.Constant(1063), value)
        return node


class TrainingClockTests(unittest.TestCase):
    def test_complete_trainer_matches_source_except_declared_output_identity(
        self,
    ) -> None:
        old = ast.parse((PACKAGE / "zoology_english_binding/training.py").read_text())
        new = ast.parse((PACKAGE / "zoology_query_readout/training.py").read_text())
        normalize = _NormalizeIdentity()
        normalized = normalize.visit(new)
        self.assertCountEqual(normalize.renamed, RENAMED_STRINGS)
        self.assertEqual(normalize.issue_fields, 1)
        self.assertEqual(
            ast.dump(normalized, include_attributes=False),
            ast.dump(old, include_attributes=False),
            "Any fit, RNG, sampler, schedule, admission, resume or export change "
            "beyond the declared output identity breaks the frozen comparison",
        )


if __name__ == "__main__":
    unittest.main()
