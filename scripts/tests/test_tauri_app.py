import json
import os
from pathlib import Path
import subprocess
import unittest

REPO = Path(__file__).resolve().parents[2]
CONF = REPO / 'src-tauri/tauri.conf.json'
VERIFY = REPO / 'scripts/studio/verify-tauri-app.sh'


class TauriAppAcceptanceTests(unittest.TestCase):
    def test_packaged_resource_declaration(self):
        conf = json.loads(CONF.read_text(encoding='utf-8'))
        resources = conf['bundle']['resources']
        self.assertIn('runtime/studio/pkg/', resources)
        self.assertIn('runtime/studio/models.json', resources)
        self.assertEqual(conf['bundle']['targets'], 'app')

    def test_real_packaged_app(self):
        app = os.environ.get('UOR_TAURI_APP')
        if not app:
            self.skipTest('Set UOR_TAURI_APP to run the real .app acceptance check')
        if not os.environ.get('UOR_DESKTOP_SHA256'):
            self.fail('UOR_DESKTOP_SHA256 is required for executable identity verification')
        result = subprocess.run(['bash', str(VERIFY), app], text=True, capture_output=True)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn('PACKAGED APP CONTRACT PASSED', result.stdout)


if __name__ == '__main__':
    unittest.main()
