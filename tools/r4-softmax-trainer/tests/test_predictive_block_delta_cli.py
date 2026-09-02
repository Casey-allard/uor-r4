"""Focused path, parser, and dispatch checks for the predictive delta gate."""

from __future__ import annotations

import sys
import unittest
from contextlib import redirect_stderr
from io import StringIO
from pathlib import Path
from unittest import mock

from r4_softmax_trainer import cli
from r4_softmax_trainer.paths import (
    default_language_path_root,
    default_learned_associative_readout_root,
    default_predictive_block_delta_frame_sidecar,
    default_predictive_block_delta_predecessor,
    default_predictive_block_delta_revealed_v4_root,
    default_predictive_block_delta_root,
    default_predictive_block_delta_v1_result,
    default_predictive_block_delta_v2_root,
    default_predictive_block_delta_terminal_prior_populations,
    default_predictive_block_delta_terminal_root,
    default_predictive_block_delta_terminal_v2_result,
)
from r4_softmax_trainer.predictive_block_delta_campaign import MAXIMUM_UPDATES


class PredictiveBlockDeltaCliTests(unittest.TestCase):
    def test_defaults_bind_the_new_root_predecessor_v4_and_geometry(self) -> None:
        root = default_predictive_block_delta_root()
        self.assertEqual(root.name, "issue-973-predictive-block-delta-v1")
        self.assertEqual(
            default_predictive_block_delta_predecessor(), default_language_path_root()
        )
        self.assertEqual(
            default_predictive_block_delta_revealed_v4_root(),
            default_learned_associative_readout_root(),
        )
        self.assertEqual(
            default_predictive_block_delta_frame_sidecar(),
            root / "geometry" / "h4-spin-frames.json",
        )

        arguments = cli.parser().parse_args(["preflight-predictive-block-delta"])
        self.assertEqual(arguments.predecessor_root, default_language_path_root())
        self.assertEqual(
            arguments.revealed_v4_root, default_learned_associative_readout_root()
        )
        self.assertEqual(arguments.frame_sidecar, default_predictive_block_delta_frame_sidecar())
        self.assertEqual(arguments.maximum_updates, MAXIMUM_UPDATES)

    def test_explicit_overrides_dispatch_and_update_bound_fails_closed(self) -> None:
        root = Path("/tmp/predictive-root")
        predecessor = Path("/tmp/predictive-predecessor")
        revealed = Path("/tmp/predictive-v4")
        frames = Path("/tmp/predictive-frames.json")
        result = {"verdict": "TEST_ONLY"}
        arguments = [
            "r4-softmax-trainer",
            "--root",
            str(root),
            "preflight-predictive-block-delta",
            "--predecessor-root",
            str(predecessor),
            "--revealed-v4-root",
            str(revealed),
            "--frame-sidecar",
            str(frames),
            "--maximum-updates",
            "7",
        ]
        with (
            mock.patch.object(sys, "argv", arguments),
            mock.patch.object(
                cli, "run_predictive_block_delta_preflight", return_value=result
            ) as run,
            mock.patch.object(cli, "_print_result") as print_result,
        ):
            cli.main()
        run.assert_called_once_with(
            root=root.resolve(),
            predecessor_root=predecessor.resolve(),
            revealed_v4_root=revealed.resolve(),
            frame_sidecar_path=frames.resolve(),
            maximum_updates=7,
        )
        print_result.assert_called_once_with(result)

        for invalid in (0, MAXIMUM_UPDATES + 1):
            command = cli.parser()
            with redirect_stderr(StringIO()), self.assertRaises(SystemExit):
                command.parse_args(
                    [
                        "preflight-predictive-block-delta",
                        "--maximum-updates",
                        str(invalid),
                    ]
                )

    def test_v2_uses_a_separate_root_and_dispatches_the_historical_v1_binding(
        self,
    ) -> None:
        root = Path("/tmp/predictive-v2-root")
        predecessor = Path("/tmp/predictive-v2-predecessor")
        revealed = Path("/tmp/predictive-v2-v4")
        frames = Path("/tmp/predictive-v2-frames.json")
        v1_result = Path("/tmp/predictive-v1-result.json")
        result = {"verdict": "TEST_ONLY_V2"}
        defaults = cli.parser().parse_args(
            ["preflight-predictive-block-delta-v2"]
        )
        self.assertEqual(
            default_predictive_block_delta_v2_root().name,
            "issue-973-predictive-block-delta-v2",
        )
        self.assertEqual(defaults.v1_result, default_predictive_block_delta_v1_result())

        arguments = [
            "r4-softmax-trainer",
            "--root",
            str(root),
            "preflight-predictive-block-delta-v2",
            "--predecessor-root",
            str(predecessor),
            "--revealed-v4-root",
            str(revealed),
            "--frame-sidecar",
            str(frames),
            "--v1-result",
            str(v1_result),
        ]
        with (
            mock.patch.object(sys, "argv", arguments),
            mock.patch.object(
                cli, "run_predictive_block_delta_v2_preflight", return_value=result
            ) as run,
            mock.patch.object(cli, "_print_result") as print_result,
        ):
            cli.main()
        run.assert_called_once_with(
            root=root.resolve(),
            predecessor_root=predecessor.resolve(),
            revealed_v4_root=revealed.resolve(),
            frame_sidecar_path=frames.resolve(),
            v1_result_path=v1_result.resolve(),
        )
        print_result.assert_called_once_with(result)

        with redirect_stderr(StringIO()), self.assertRaises(SystemExit):
            cli.parser().parse_args(
                [
                    "preflight-predictive-block-delta-v2",
                    "--maximum-updates",
                    "255",
                ]
            )

    def test_terminal_commands_bind_v5_defaults_and_dispatch_without_opening_data(
        self,
    ) -> None:
        defaults = cli.parser().parse_args(
            ["prepare-predictive-block-delta-terminal"]
        )
        prior = default_predictive_block_delta_terminal_prior_populations()
        self.assertEqual(
            default_predictive_block_delta_terminal_root().name,
            "issue-973-predictive-block-delta-v5",
        )
        self.assertEqual(defaults.v2_result, default_predictive_block_delta_terminal_v2_result())
        self.assertEqual(
            (defaults.v1_population, defaults.v2_population, defaults.v3_population, defaults.v4_population),
            prior,
        )

        root = Path("/tmp/predictive-v5-root")
        result = {"verdict": "TEST_ONLY_V5"}
        with (
            mock.patch.object(
                sys,
                "argv",
                [
                    "r4-softmax-trainer",
                    "--root",
                    str(root),
                    "run-predictive-block-delta-terminal",
                    "--resume",
                ],
            ),
            mock.patch.object(
                cli, "run_predictive_block_delta_terminal", return_value=result
            ) as run,
            mock.patch.object(cli, "_print_result") as print_result,
        ):
            cli.main()
        run.assert_called_once_with(root.resolve(), resume=True)
        print_result.assert_called_once_with(result)


if __name__ == "__main__":
    unittest.main()
