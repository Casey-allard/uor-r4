"""Focused cross-language checks for the canonical H4 frame sidecar."""

from __future__ import annotations

import subprocess
import tempfile
import unittest
from pathlib import Path

import torch

from r4_softmax_trainer.h4_spin_frame_sidecar import (
    GROUP_SIZE,
    H4SpinFrameArtifactV1,
)


class H4SpinFrameSidecarTests(unittest.TestCase):
    def test_rust_export_loads_with_identity_and_all_products(self) -> None:
        repository = Path(__file__).resolve().parents[3]
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "h4-spin-frames.json"
            subprocess.run(
                [
                    "cargo",
                    "run",
                    "--quiet",
                    "--offline",
                    "-p",
                    "uor-r4-core",
                    "--bin",
                    "r4-h4-spin-frame-export",
                    "--",
                    str(path),
                ],
                cwd=repository,
                check=True,
                capture_output=True,
                text=True,
            )
            frames = H4SpinFrameArtifactV1.load(path)

        frames.validate(group_size=GROUP_SIZE)
        self.assertEqual(tuple(frames.frame_matrices.shape), (120, 4, 4))
        self.assertEqual(tuple(frames.multiplication_indices.shape), (120, 120))
        self.assertTrue(
            torch.equal(
                frames.frame_matrices[frames.identity_index],
                torch.eye(4, dtype=torch.float32),
            )
        )
        composed = torch.matmul(
            frames.frame_matrices[:, None], frames.frame_matrices[None, :]
        )
        expected = frames.frame_matrices[frames.multiplication_indices]
        self.assertTrue(torch.allclose(composed, expected, rtol=0.0, atol=2.0e-6))
        self.assertEqual(
            sorted(frames.transport_permutation.tolist()), list(range(GROUP_SIZE))
        )
        self.assertEqual(
            int(frames.transport_permutation[frames.identity_index]),
            frames.identity_index,
        )


if __name__ == "__main__":
    unittest.main()
