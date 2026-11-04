import os
from pathlib import Path
import subprocess
import unittest

REPO = Path(__file__).resolve().parents[2]
RUNNER = REPO / 'scripts/studio/native-runner.sh'


class NativeRunnerContractTests(unittest.TestCase):
    def test_pinned_downloads_and_secure_modes(self):
        text = RUNNER.read_text(encoding='utf-8')
        self.assertIn('uor-chat-arm64-apple-darwin', text)
        self.assertIn('studio-local.tar.gz', text)
        self.assertRegex(text, r"UOR_CHAT_SHA256='[0-9a-f]{64}'")
        self.assertRegex(text, r"STUDIO_BUNDLE_SHA256='[0-9a-f]{64}'")
        self.assertIn('dual-c131-seed-1-raw_both.uorlut', text)
        self.assertIn('tokenizer-seed-1.json', text)
        self.assertIn('shasum -a 256', text)
        self.assertIn('--token-stdin', text)
        self.assertIn('--studio-root', text)
        self.assertIn('synthetic-stack.lut', text)
        self.assertIn('127.0.0.1:8000', text)
        self.assertNotIn('PAGES_ROOT', text)
        self.assertNotIn('cloudflared', text)
        self.assertNotIn('--tunnel', text)
        self.assertNotIn('eval ', text)
        self.assertNotIn('echo "$UOR_RUNNER_TOKEN"', text)
        self.assertNotIn('echo "${UOR_RUNNER_TOKEN}"', text)

    def test_check_mode_is_side_effect_free(self):
        env = os.environ.copy()
        env['UOR_RUNNER_HOME'] = '/tmp/uor-runner-contract-check'
        result = subprocess.run(
            ['bash', str(RUNNER), '--check'],
            env=env,
            text=True,
            capture_output=True,
            check=True,
        )
        self.assertIn('uor-chat-arm64-apple-darwin', result.stdout)
        self.assertIn('dual-c131-seed-1-raw_both.uorlut', result.stdout)
        self.assertFalse(Path(env['UOR_RUNNER_HOME']).exists())


if __name__ == '__main__':
    unittest.main()
