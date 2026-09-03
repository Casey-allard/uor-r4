"""Whole-source training-clock comparison with one declared input augmentation."""

from __future__ import annotations

import ast
import unittest
from pathlib import Path

PACKAGE = Path(__file__).resolve().parents[1] / "src/r4_softmax_trainer"
RENAMED_STRINGS = {
    "One fixed, resumable cyclic-fact fit; no development scoring.": (
        "One fixed, resumable query-token readout fit; no development scoring."
    ),
    "uor-r4.zoology-cyclic-facts-fit-start/1": "uor-r4.zoology-query-readout-fit-start/1",
    "uor-r4.zoology-cyclic-facts-fit-checkpoint/1": "uor-r4.zoology-query-readout-fit-checkpoint/1",
    "uor-r4.zoology-cyclic-facts-fit/1": "uor-r4.zoology-query-readout-fit/1",
}
AUGMENT_BATCH = ast.parse(
    "batch = augment_training_batch(batch, completed_updates=updates)"
).body[0]


def _dump(node: ast.AST) -> str:
    return ast.dump(node, include_attributes=False)


class _NormalizeAugmentation(ast.NodeTransformer):
    def __init__(self) -> None:
        self.renamed: list[str] = []
        self.imports = self.batch_calls = self.ledger_records = 0

    def visit_Constant(self, node: ast.Constant) -> ast.AST:
        if isinstance(node.value, str) and node.value in RENAMED_STRINGS:
            self.renamed.append(node.value)
            return ast.copy_location(ast.Constant(RENAMED_STRINGS[node.value]), node)
        return node

    def visit_ImportFrom(self, node: ast.ImportFrom) -> ast.AST | None:
        if (
            node.level == 1
            and node.module == "augmentation"
            and [(name.name, name.asname) for name in node.names]
            == [("augment_training_batch", None), ("rotation_ledger", None)]
        ):
            self.imports += 1
            return None
        return node

    def visit_Assign(self, node: ast.Assign) -> ast.AST | None:
        if _dump(node) == _dump(AUGMENT_BATCH):
            self.batch_calls += 1
            return None
        return self.generic_visit(node)

    def visit_Dict(self, node: ast.Dict) -> ast.AST:
        self.generic_visit(node)
        expected = ast.parse(
            "rotation_ledger(updates, unknown_presentations)", mode="eval"
        ).body
        kept = []
        for key, value in zip(node.keys, node.values, strict=True):
            if (
                isinstance(key, ast.Constant)
                and key.value == "augmentation"
                and _dump(value) == _dump(expected)
            ):
                self.ledger_records += 1
            else:
                kept.append((key, value))
        node.keys = [key for key, _ in kept]
        node.values = [value for _, value in kept]
        return node


class CyclicFactTrainingClockTests(unittest.TestCase):
    def test_plain_source_clock_matches_except_input_rotation_and_derived_ledger(
        self,
    ) -> None:
        old = ast.parse((PACKAGE / "zoology_query_readout/training.py").read_text())
        new = ast.parse((PACKAGE / "zoology_cyclic_facts/training.py").read_text())
        fit = next(
            node
            for node in new.body
            if isinstance(node, ast.FunctionDef) and node.name == "_fit_locked"
        )
        loop = next(node for node in fit.body if isinstance(node, ast.While))
        sampler_call = ast.parse("batch = sampler.next_batch()").body[0]
        index = next(
            index
            for index, node in enumerate(loop.body)
            if _dump(node) == _dump(sampler_call)
        )
        self.assertEqual(_dump(loop.body[index + 1]), _dump(AUGMENT_BATCH))
        normalize = _NormalizeAugmentation()
        normalized = normalize.visit(new)
        self.assertCountEqual(normalize.renamed, RENAMED_STRINGS)
        self.assertEqual(
            (normalize.imports, normalize.batch_calls, normalize.ledger_records),
            (1, 1, 1),
        )
        self.assertEqual(
            _dump(normalized),
            _dump(old),
            "Source constructor, RNG, sampler, scheduler, loss, admission, resume and work ledger must remain unchanged",
        )


if __name__ == "__main__":
    unittest.main()
