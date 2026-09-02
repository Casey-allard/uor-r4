"""Focused CLI registration and dispatch tests for issue #1043."""

from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from r4_softmax_trainer import cli
from r4_softmax_trainer import position_kv_binding_campaign as campaign


class PositionKVBindingCliTests(unittest.TestCase):
    def test_cli_registers_only_the_four_frozen_lifecycle_commands(self) -> None:
        command = cli.parser()
        for name in (
            "prepare-position-kv-binding",
            "preflight-position-kv-binding",
            "run-position-kv-binding",
            "verify-position-kv-binding",
        ):
            self.assertEqual(command.parse_args([name]).command, name)
        with self.assertRaises(SystemExit):
            command.parse_args(["run-position-kv-binding", "--resume"])

    def test_prepare_derives_and_passes_the_complete_exclusion_union(self) -> None:
        result = {"preparation_cid": "blake3:" + "1" * 64}
        exclusions = ("blake3:" + "2" * 64,)
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "campaign"
            source_root = Path(directory) / "source"
            v5_root = Path(directory) / "v5"
            arguments = [
                "r4-softmax-trainer",
                "--root",
                str(root),
                "prepare-position-kv-binding",
                "--source-root",
                str(source_root),
                "--v5-root",
                str(v5_root),
            ]
            with (
                patch.object(sys, "argv", arguments),
                patch.object(
                    cli,
                    "collect_position_kv_story_exclusions",
                    return_value=exclusions,
                ) as collect,
                patch.object(
                    cli,
                    "prepare_position_kv_binding_campaign",
                    return_value=result,
                ) as prepare,
                patch.object(cli, "_print_result") as emit,
            ):
                cli.main()
        collect.assert_called_once_with(
            source_root=source_root.resolve(),
            v5_root=v5_root.resolve(),
        )
        self.assertEqual(prepare.call_args.kwargs["excluded_story_cids"], exclusions)
        self.assertEqual(prepare.call_args.args[0], root.resolve())
        emit.assert_called_once_with(result)

    def test_arbitrary_story_exclusion_union_is_rejected(self) -> None:
        with self.assertRaisesRegex(ValueError, "complete freeze"):
            campaign._validate_complete_story_exclusions(
                ("blake3:" + "2" * 64,)
            )

    def test_remaining_lifecycle_commands_dispatch_without_runner_injection(self) -> None:
        functions = {
            "preflight-position-kv-binding": "preflight_position_kv_binding_campaign",
            "run-position-kv-binding": "run_position_kv_binding_campaign",
            "verify-position-kv-binding": "validate_position_kv_binding_result",
        }
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "campaign"
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
