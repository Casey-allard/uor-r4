"""Focused CLI tests for ``python -m r4_softmax_trainer.zoology_control``."""

from __future__ import annotations

import unittest
from pathlib import Path
from unittest.mock import patch

from r4_softmax_trainer.zoology_control import __main__ as subject


class ZoologyControlCliTests(unittest.TestCase):
    def test_parser_registers_the_create_once_lifecycle(self) -> None:
        root = Path("/tmp/run")
        source = Path("/tmp/source")
        predecessor = Path("/tmp/predecessor")
        prepared = subject._parser().parse_args(
            [
                "prepare",
                str(root),
                "--source-root",
                str(source),
                "--predecessor-root",
                str(predecessor),
            ]
        )
        self.assertEqual(prepared.command, "prepare")
        for command in ("preflight", "run", "verify"):
            self.assertEqual(
                subject._parser().parse_args([command, str(root)]).command,
                command,
            )

    def test_prepare_dispatches_both_open_roots(self) -> None:
        expected = {"preparation_cid": "blake3:" + "1" * 64}
        with (
            patch.object(subject, "prepare_zoology_control", return_value=expected) as invoked,
            patch.object(subject, "_print") as emitted,
        ):
            subject.main(
                [
                    "prepare",
                    "/tmp/run",
                    "--source-root",
                    "/tmp/source",
                    "--predecessor-root",
                    "/tmp/predecessor",
                ]
            )
        invoked.assert_called_once_with(
            Path("/tmp/run"),
            source_root=Path("/tmp/source"),
            predecessor_root=Path("/tmp/predecessor"),
        )
        emitted.assert_called_once_with(expected)


if __name__ == "__main__":
    unittest.main()
