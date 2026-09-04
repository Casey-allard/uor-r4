"""Focused publication check for #973's bounded fit output pair."""

from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from r4_softmax_trainer.quaternion_cube_fit import _write_output_bundle


class QuaternionCubeFitTests(unittest.TestCase):
    def test_model_and_result_publish_as_one_create_once_directory(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "fit"
            artifact = b"fitted-model"
            result = {"schema": "example/1", "status": "COMPLETE"}

            _write_output_bundle(output, artifact, result)

            self.assertEqual((output / "model.safetensors").read_bytes(), artifact)
            self.assertEqual(json.loads((output / "fit.json").read_text()), result)
            self.assertFalse(output.with_name(".fit.tmp").exists())
            with self.assertRaises(FileExistsError):
                _write_output_bundle(output, artifact, result)


if __name__ == "__main__":
    unittest.main()
