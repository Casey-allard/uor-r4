"""Whole-source clock comparison; no model initialization or fit execution."""

from __future__ import annotations

import ast
import unittest
from pathlib import Path

PACKAGE = Path(__file__).resolve().parents[1] / "src/r4_softmax_trainer"
RENAMED_STRINGS = {
    "One fixed, resumable joint-query embedding fit; no development scoring.": (
        "One fixed, resumable query-token readout fit; no development scoring."
    ),
    "uor-r4.zoology-joint-query-fit-start/1": "uor-r4.zoology-query-readout-fit-start/1",
    "uor-r4.zoology-joint-query-fit-checkpoint/1": "uor-r4.zoology-query-readout-fit-checkpoint/1",
    "uor-r4.zoology-joint-query-fit/1": "uor-r4.zoology-query-readout-fit/1",
}


def _expression(source: str) -> str:
    return ast.dump(ast.parse(source, mode="eval").body, include_attributes=False)


class _NormalizeAdapter(ast.NodeTransformer):
    def __init__(self) -> None:
        self.renamed: list[str] = []
        self.source_imports = self.adapter_imports = 0
        self.metadata_checks = self.metadata_records = 0

    def visit_Constant(self, node: ast.Constant) -> ast.AST:
        if isinstance(node.value, str) and node.value in RENAMED_STRINGS:
            self.renamed.append(node.value)
            return ast.copy_location(ast.Constant(RENAMED_STRINGS[node.value]), node)
        return node

    def visit_ImportFrom(self, node: ast.ImportFrom) -> ast.AST | None:
        names = [(name.name, name.asname) for name in node.names]
        if (
            node.level == 2
            and node.module == "zoology_control.model"
            and names == [("ZoologyFigure2Config", None), ("set_zoology_seed", None)]
        ):
            self.source_imports += 1
            node.names.insert(1, ast.alias(name="ZoologyFigure2Model"))
        elif (
            node.level == 1
            and node.module == "model"
            and names
            in (
                [("QUERY_ENCODING", None)],
                [("ZoologyJointQueryModel", "ZoologyFigure2Model")],
            )
        ):
            self.adapter_imports += 1
            return None
        return node

    def visit_BoolOp(self, node: ast.BoolOp) -> ast.AST:
        self.generic_visit(node)
        kept = []
        for value in node.values:
            if isinstance(node.op, ast.Or) and ast.dump(
                value, include_attributes=False
            ) == _expression(
                'result["artifact"].get("query_encoding") != QUERY_ENCODING'
            ):
                self.metadata_checks += 1
            else:
                kept.append(value)
        node.values = kept
        return node

    def visit_Dict(self, node: ast.Dict) -> ast.AST:
        self.generic_visit(node)
        kept = []
        for key, value in zip(node.keys, node.values, strict=True):
            if (
                isinstance(key, ast.Constant)
                and key.value == "query_encoding"
                and ast.dump(value, include_attributes=False)
                == _expression("dict(QUERY_ENCODING)")
            ):
                self.metadata_records += 1
            else:
                kept.append((key, value))
        node.keys = [key for key, _ in kept]
        node.values = [value for _, value in kept]
        return node


class JointQueryTrainingClockTests(unittest.TestCase):
    def test_complete_trainer_matches_source_except_adapter_and_bound_identity(
        self,
    ) -> None:
        old = ast.parse((PACKAGE / "zoology_query_readout/training.py").read_text())
        new = ast.parse((PACKAGE / "zoology_joint_query/training.py").read_text())
        normalize = _NormalizeAdapter()
        normalized = normalize.visit(new)
        self.assertCountEqual(normalize.renamed, RENAMED_STRINGS)
        self.assertEqual((normalize.source_imports, normalize.adapter_imports), (1, 2))
        self.assertEqual(
            (normalize.metadata_checks, normalize.metadata_records), (1, 1)
        )
        self.assertEqual(
            ast.dump(normalized, include_attributes=False),
            ast.dump(old, include_attributes=False),
            "Only the declared constructor, encoding metadata, and output identities may change; "
            "all numerical, RNG, sampler, schedule, admission, resume and checkpoint paths must match",
        )


if __name__ == "__main__":
    unittest.main()
