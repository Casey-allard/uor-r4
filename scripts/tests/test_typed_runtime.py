import subprocess
import tempfile
import unittest
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
RUNTIME = REPO / 'scripts/studio/runtime-files.sh'
RUNNER = REPO / 'scripts/studio/native-runner.sh'


def run(*args):
    return subprocess.run(args, cwd=REPO, text=True, capture_output=True)


class TypedRuntimeContractTests(unittest.TestCase):
    def test_typed_allowlist_copies_generated_assets_and_rejects_unexpected(self):
        with tempfile.TemporaryDirectory() as tmp:
            dest = Path(tmp) / 'studio'
            result = run('bash', str(RUNTIME), 'copy', str(REPO / 'web/studio/dist'), str(dest), '--typed')
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertTrue((dest / 'index.html').is_file())
            self.assertTrue((dest / 'models.json').is_file())
            self.assertTrue((dest / 'assets/js/lifecycle_persistence.js').is_file())
            self.assertTrue(any((dest / 'assets').glob('index-*.js')))
            self.assertTrue(any((dest / 'assets').glob('index-*.css')))
            self.assertTrue(any((dest / 'assets').glob('uor_r4_integer_bg-*.wasm')))
            for name in ['uor_r4_integer.js', 'uor_r4_integer_bg.wasm', 'synthetic-stack.lut', 'tokenizer.json']:
                self.assertTrue((dest / 'pkg' / name).is_file(), name)
            for forbidden in ['index.css', 'package.json', 'tsconfig.json', 'vite.config.ts', 'README.md', 'src', 'tests', 'node_modules', 'pkg/README.md', 'pkg/uor_r4_integer.d.ts', 'pkg/package.json']:
                self.assertFalse((dest / forbidden).exists(), forbidden)
            (dest / 'assets/example.map').write_text('unexpected', encoding='utf-8')
            rejected = run('bash', str(RUNTIME), '--verify', str(dest), '--typed')
            self.assertNotEqual(rejected.returncode, 0, rejected.stdout + rejected.stderr)

    def test_native_runner_local_uses_typed_dist_without_download(self):
        source = RUNNER.read_text(encoding='utf-8')
        self.assertIn('--local', source)
        self.assertIn('web/studio/dist', source)
        self.assertIn('build_serve_wasm.sh', source)
        self.assertIn('npm run build', source)
        self.assertIn('/?typed=1', source)
        self.assertIn('local_target="${CARGO_TARGET_DIR:-$repo_root/target}/native-runner"', source)
        self.assertIn('local_chat="$local_target/release/uor-chat"', source)
        result = run('bash', str(RUNNER), '--local', '--check')
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn('local: web/studio/dist', result.stdout)
        self.assertIn('download: none', result.stdout)

    def test_tauri_app_url_activates_the_mounted_application(self):
        source = (REPO / 'src-tauri/src/lib.rs').read_text(encoding='utf-8')
        self.assertIn('WebviewUrl::App("index.html?typed=1".into())', source)

    def test_tauri_preparer_declares_typed_dist(self):
        result = run('bash', str(REPO / 'scripts/studio/prepare-tauri-runtime.sh'), '--check')
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn('typed: web/studio/dist', result.stdout)


if __name__ == '__main__':
    unittest.main()
