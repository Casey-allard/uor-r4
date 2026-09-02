"""Focused causal-data checks for issue #1045."""

from __future__ import annotations

import re
import unittest
from dataclasses import replace
from pathlib import Path
from types import SimpleNamespace
from unittest import mock

import torch

from r4_softmax_trainer import position_kv_binding_data as inherited
from r4_softmax_trainer import role_tagged_associative_data as subject


class _ToyTokenizer:
    """Tiny deterministic tokenizer sufficient to exercise the finite-state ABI."""

    def __init__(self, lexical_ids: dict[str, int]) -> None:
        self._ids = dict(lexical_ids)
        self._next = 10

    def encode(self, text: str, add_special_tokens: bool = False) -> SimpleNamespace:
        pieces = re.findall(r"[A-Za-z]+|[^A-Za-z\s]", text)
        ids: list[int] = []
        for piece in pieces:
            if piece not in self._ids:
                while self._next in self._ids.values():
                    self._next += 1
                self._ids[piece] = self._next
                self._next += 1
            ids.append(self._ids[piece])
        return SimpleNamespace(ids=ids)


def _lexical_ids() -> dict[str, int]:
    values = {
        value: 1_000 + index
        for index, value in enumerate(inherited.KEY_LEXICON)
    }
    values.update(
        {
            value: 2_000 + index
            for index, value in enumerate(inherited.VALUE_LEXICON)
        }
    )
    values["unknown"] = inherited.UNKNOWN_TOKEN_ID
    return values


def _mqar_example(index: int) -> inherited.CausalBindingExample:
    pairs = tuple(
        (
            inherited.MQAR_KEY_MIN
            + ((index * inherited.MQAR_RECORDS + lane) % 1_792),
            inherited.MQAR_VALUE_MIN
            + (
                ((index * inherited.MQAR_RECORDS + lane) // 1_792)
                * inherited.MQAR_RECORDS
                + lane
            ),
        )
        for lane in range(inherited.MQAR_RECORDS)
    )
    keys = tuple(key for key, _value in pairs)
    values = tuple(value for _key, value in pairs)
    tokens = [inherited.MQAR_FILLER_MIN] * inherited.CONTEXT
    for lane, (key, value) in enumerate(pairs):
        position = lane * 4
        tokens[position] = key
        tokens[position + 1] = value
    query_positions = tuple(range(32, 32 + inherited.MQAR_QUERIES))
    for position, key in zip(query_positions, keys, strict=True):
        tokens[position] = key
    labels = [inherited.IGNORE_INDEX] * inherited.CONTEXT
    for position, value in zip(query_positions, values, strict=True):
        labels[position] = value
    assignment = inherited._assignment_cid(keys, values)
    return inherited.CausalBindingExample(
        population="mqar",
        split="construction",
        example_index=index,
        world_index=index,
        family_index=0,
        input_ids=tuple(tokens),
        label_ids=tuple(labels),
        query_positions=query_positions,
        query_keys=keys,
        answers=values,
        binding_keys=keys,
        binding_values=values,
        binding_names=(),
        assignment_cid=assignment,
        world_cid=assignment,
        sequence_cid=inherited._sequence_cid(tokens, labels),
    )


def _english_examples() -> tuple[
    _ToyTokenizer,
    dict[str, int],
    inherited.CausalBindingExample,
    inherited.CausalBindingExample,
]:
    token_ids = _lexical_ids()
    tokenizer = _ToyTokenizer(token_ids)
    bindings = (
        ("spoon", "garden"),
        ("rope", "attic"),
        ("bell", "cave"),
        ("doll", "beach"),
    )
    world_cid = "blake3:" + "1" * 64
    history = inherited._english_example(
        tokenizer=tokenizer,  # type: ignore[arg-type]
        token_ids=token_ids,
        split="construction",
        example_index=0,
        world_index=0,
        family=0,
        bindings=bindings,
        world_cid=world_cid,
        query_key="doll",
        history=True,
    )
    no_history = inherited._english_example(
        tokenizer=tokenizer,  # type: ignore[arg-type]
        token_ids=token_ids,
        split="construction",
        example_index=0,
        world_index=0,
        family=0,
        bindings=bindings,
        world_cid=world_cid,
        query_key="doll",
        history=False,
    )
    return tokenizer, token_ids, history, no_history


class RoleDerivationTests(unittest.TestCase):
    def test_mqar_roles_are_physical_prefix_causal_and_label_independent(self) -> None:
        row = _mqar_example(0)
        tagged = subject.tag_mqar_example(row)
        self.assertEqual(tagged.role_ids.count(subject.RoleCode.KEY), 8)
        self.assertEqual(tagged.role_ids.count(subject.RoleCode.VALUE), 8)
        self.assertEqual(tagged.role_ids.count(subject.RoleCode.QUERY), 8)
        result = subject.validate_role_oracle((tagged,))
        self.assertTrue(result.passed)
        self.assertEqual(result.prefix_checks, inherited.CONTEXT)

        changed_labels = list(row.label_ids)
        changed_answers = list(row.answers)
        changed_labels[row.query_positions[0]] += 1
        changed_answers[0] += 1
        mutated = replace(
            row,
            label_ids=tuple(changed_labels),
            answers=tuple(changed_answers),
            sequence_cid=inherited._sequence_cid(row.input_ids, changed_labels),
        )
        retagged = subject.tag_mqar_example(mutated)
        self.assertEqual(retagged.role_ids, tagged.role_ids)
        self.assertEqual(retagged.stable_id, tagged.stable_id)
        self.assertEqual(subject._mqar_rank(mutated), subject._mqar_rank(row))

    def test_english_roles_use_only_tokenized_construction_markers(self) -> None:
        tokenizer, token_ids, history, no_history = _english_examples()
        schema = subject.build_english_role_schema(tokenizer, token_ids)  # type: ignore[arg-type]
        tagged_history = subject.tag_english_example(history, schema)
        tagged_no_history = subject.tag_english_example(no_history, schema)
        self.assertEqual(tagged_history.role_ids.count(subject.RoleCode.KEY), 4)
        self.assertEqual(tagged_history.role_ids.count(subject.RoleCode.VALUE), 4)
        self.assertEqual(tagged_history.role_ids.count(subject.RoleCode.QUERY), 2)
        self.assertEqual(tagged_no_history.role_ids.count(subject.RoleCode.KEY), 0)
        self.assertEqual(tagged_no_history.role_ids.count(subject.RoleCode.VALUE), 0)
        self.assertEqual(tagged_no_history.role_ids.count(subject.RoleCode.QUERY), 2)
        result = subject.validate_role_oracle(
            (tagged_history, tagged_no_history), english_schema=schema
        )
        self.assertTrue(result.passed)
        self.assertEqual(result.label_reads, 0)
        self.assertEqual(result.metadata_reads, 0)

    def test_natural_rows_are_all_text(self) -> None:
        self.assertEqual(
            subject.natural_role_ids((12, 34, 56)),
            (subject.RoleCode.TEXT,) * 3,
        )


class OpenSplitAndBatchTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.rows = tuple(_mqar_example(index) for index in range(subject.MQAR_TOTAL_ROWS))
        cls.split = subject.split_mqar_construction(cls.rows)

    def test_split_is_exact_input_ranked_and_pair_assignment_disjoint(self) -> None:
        split = self.split
        self.assertEqual(len(split.train), subject.MQAR_TRAIN_ROWS)
        self.assertEqual(len(split.development), subject.MQAR_DEVELOPMENT_ROWS)
        self.assertEqual(len(split.controls), subject.MQAR_CONTROL_ROWS)
        reversed_split = subject.split_mqar_construction(tuple(reversed(self.rows)))
        self.assertEqual(reversed_split.split_cid, split.split_cid)
        self.assertEqual(
            tuple(row.stable_id for row in reversed_split.train),
            tuple(row.stable_id for row in split.train),
        )

        populations = (split.train, split.development, split.controls)
        assignments = [
            {subject._physical_mqar_pairs(row.input_ids) for row in population}
            for population in populations
        ]
        pairs = [
            {pair for assignment in values for pair in assignment}
            for values in assignments
        ]
        for left in range(3):
            for right in range(left + 1, 3):
                self.assertFalse(assignments[left].intersection(assignments[right]))
                self.assertFalse(pairs[left].intersection(pairs[right]))

    def test_batch_uses_uint8_roles_and_per_row_selected_targets(self) -> None:
        rows = self.split.train[:2]
        batch = subject.batch_role_tagged_examples(rows)
        self.assertEqual(batch.input_ids.shape, (2, inherited.CONTEXT))
        self.assertEqual(batch.role_ids.dtype, torch.uint8)
        self.assertEqual(batch.selected_positions.shape, (2, inherited.MQAR_QUERIES))
        self.assertEqual(batch.targets.shape, (2, inherited.MQAR_QUERIES))
        self.assertTrue(
            torch.equal(
                batch.targets,
                torch.gather(batch.labels, 1, batch.selected_positions),
            )
        )
        logits = torch.arange(2 * inherited.CONTEXT * 3, dtype=torch.float32).reshape(
            2, inherited.CONTEXT, 3
        )
        selected, targets = subject.select_labeled_logits(logits, batch)
        self.assertEqual(selected.shape, (2, inherited.MQAR_QUERIES, 3))
        self.assertTrue(torch.equal(targets, batch.targets))

    def test_loader_calls_only_the_frozen_construction_boundary(self) -> None:
        tokenizer, token_ids, history, no_history = _english_examples()
        fake = SimpleNamespace(
            tokenizer_path=Path("/open/tokenizer.json"),
            natural_windows=object(),
            mqar=self.rows,
            english_history=(history,),
            english_no_history=(no_history,),
        )
        with (
            mock.patch.object(
                subject, "load_position_kv_binding_construction", return_value=fake
            ) as construction_loader,
            mock.patch.object(
                subject,
                "validate_tokenizer",
                return_value=(tokenizer, token_ids),
            ),
        ):
            loaded = subject.load_role_tagged_construction(Path("/open/construction"))
        construction_loader.assert_called_once_with(Path("/open/construction"))
        self.assertEqual(loaded.split_cid, self.split.split_cid)
        self.assertEqual(len(loaded.english_history), 1)
        self.assertEqual(len(loaded.english_no_history), 1)


if __name__ == "__main__":
    unittest.main()
