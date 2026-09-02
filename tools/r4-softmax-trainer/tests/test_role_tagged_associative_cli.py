"""Focused CLI registration and dispatch tests for issue #1045."""

from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from r4_softmax_trainer import cli


class RoleTaggedAssociativeCliTests(unittest.TestCase):
    def test_cli_registers_the_open_development_lifecycle(self) -> None:
        command = cli.parser()
        for name in (
            "prepare-role-tagged-associative",
            "preflight-role-tagged-associative",
            "run-role-tagged-associative",
            "verify-role-tagged-associative",
        ):
            self.assertEqual(command.parse_args([name]).command, name)

    def test_prepare_passes_only_the_open_source_root(self) -> None:
        result = {"preparation_cid": "blake3:" + "1" * 64}
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "run"
            source = Path(directory) / "source"
            arguments = [
                "r4-softmax-trainer",
                "--root",
                str(root),
                "prepare-role-tagged-associative",
                "--source-root",
                str(source),
            ]
            with (
                patch.object(sys, "argv", arguments),
                patch.object(
                    cli,
                    "prepare_role_tagged_associative_development",
                    return_value=result,
                ) as invoked,
                patch.object(cli, "_print_result") as emit,
            ):
                cli.main()
        invoked.assert_called_once_with(root.resolve(), source_root=source.resolve())
        emit.assert_called_once_with(result)

    def test_remaining_commands_dispatch_to_one_run_root(self) -> None:
        functions = {
            "preflight-role-tagged-associative": (
                "preflight_role_tagged_associative_development"
            ),
            "run-role-tagged-associative": "run_role_tagged_associative_development",
            "verify-role-tagged-associative": (
                "verify_role_tagged_associative_development"
            ),
        }
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "run"
            for command, function_name in functions.items():
                result = {"command": command}
                arguments = [
                    "r4-softmax-trainer",
                    "--root",
                    str(root),
                    command,
                ]
                with (
                    patch.object(sys, "argv", arguments),
                    patch.object(cli, function_name, return_value=result) as invoked,
                    patch.object(cli, "_print_result") as emit,
                ):
                    cli.main()
                invoked.assert_called_once_with(root.resolve())
                emit.assert_called_once_with(result)


if __name__ == "__main__":
    unittest.main()
