import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

REPO = Path(__file__).resolve().parents[2]
RUNTIME = REPO / 'scripts/studio/runtime-files.sh'
PREPARE = REPO / 'scripts/studio/prepare-tauri-runtime.sh'
CONF = REPO / 'src-tauri/tauri.conf.json'
FIXTURE_SPEC = importlib.util.spec_from_file_location(
    'served_forward_fixture', Path(__file__).with_name('test_validate_served_forward_receipt.py')
)
assert FIXTURE_SPEC is not None and FIXTURE_SPEC.loader is not None
FIXTURE_MODULE = importlib.util.module_from_spec(FIXTURE_SPEC)
FIXTURE_SPEC.loader.exec_module(FIXTURE_MODULE)


class TauriRuntimeContractTests(unittest.TestCase):
    def test_allowlist_has_no_development_files(self):
        text = RUNTIME.read_text(encoding='utf-8')
        result = subprocess.run(['bash', str(RUNTIME), '--list'], text=True, capture_output=True, check=True)
        for required in ['index.html', 'index.css', 'models.json', 'assets/js/studio_ui.js', 'pkg/uor_r4_integer_bg.wasm']:
            self.assertIn(required, result.stdout)
        for forbidden in ['src/main.ts', 'package.json', 'tsconfig.json', 'vite.config.ts', 'model_sources.test.mjs', 'pkg/README.md', 'node_modules']:
            self.assertNotIn(forbidden, result.stdout)
        self.assertNotIn('https://', result.stdout)

    def test_copy_is_runtime_only(self):
        with tempfile.TemporaryDirectory() as tmp:
            dest = Path(tmp) / 'studio'
            subprocess.run(['bash', str(RUNTIME), 'copy', str(REPO / 'web/studio'), str(dest), '--allow-generated-missing'], check=True)
            for required in ['index.html', 'index.css', 'assets/js/studio_ui.js', 'assets/js/studio_api.js']:
                self.assertTrue((dest / required).is_file(), required)
            for forbidden in ['src/main.ts', 'package.json', 'tsconfig.json', 'vite.config.ts', 'assets/js/model_sources.test.mjs', 'pkg/README.md', 'node_modules']:
                self.assertFalse((dest / forbidden).exists(), forbidden)

    def test_packaging_contract(self):
        conf = json.loads(CONF.read_text(encoding='utf-8'))
        self.assertEqual(conf['build']['frontendDist'], './runtime/studio')
        self.assertEqual(conf['bundle']['externalBin'], ['binaries/uor-chat'])
        self.assertTrue(conf['bundle']['active'])
        csp = conf['app']['security']['csp']
        self.assertIn("'wasm-unsafe-eval'", csp)
        self.assertIn("worker-src 'self'", csp)
        self.assertNotIn('https://', csp)
        self.assertEqual(conf['bundle']['resources']['runtime/studio/pkg/'], 'pkg/')
        self.assertEqual(conf['bundle']['resources']['runtime/studio/models.json'], 'models.json')
        self.assertEqual(conf['bundle']['resources']['runtime/studio/SHA256SUMS'], 'SHA256SUMS')
        self.assertTrue((REPO / 'src-tauri/assets/uor-icon.svg').is_file())

    def test_prepare_verifies_pinned_runtime(self):
        with tempfile.TemporaryDirectory() as tmp:
            env = os.environ.copy()
            env['UOR_TAURI_RUNTIME_HOME'] = tmp
            result = subprocess.run(['bash', str(PREPARE)], env=env, text=True, capture_output=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertTrue((Path(tmp) / 'binaries/uor-chat-aarch64-apple-darwin').is_file())
            self.assertTrue((Path(tmp) / 'runtime/studio/pkg/synthetic-stack.lut').is_file())
            self.assertTrue((Path(tmp) / 'runtime/studio/pkg/tokenizer.json').is_file())
            self.assertTrue((Path(tmp) / 'runtime/studio/models.json').is_file())
            self.assertTrue((Path(tmp) / 'icons/icon.png').is_file())
            self.assertTrue((Path(tmp) / 'icons/icon.icns').is_file())

    def test_pages_publisher_uses_the_same_runtime_allowlist(self):
        with tempfile.TemporaryDirectory() as tmp:
            dest = Path(tmp) / 'studio'
            result = subprocess.run(['bash', str(REPO / 'scripts/stage-studio-site.sh'), str(REPO / 'web/studio'), str(dest)], text=True, capture_output=True)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            for required in ['index.html', 'assets/js/desktop_openai.js', 'assets/js/desktop_actions.js', 'assets/js/mcp_client.js']:
                self.assertTrue((dest / required).is_file(), required)
            for forbidden in ['src', 'package.json', 'package-lock.json', 'tsconfig.json', 'vite.config.ts', 'node_modules', 'README.md', 'assets/js/mcp_client.test.mjs', 'verify_desktop_client.mjs']:
                self.assertFalse((dest / forbidden).exists(), forbidden)

    def test_pages_stages_trained_models_and_rejects_unexpected_files(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            artifact = root / 'trained.uorlut'
            artifact_bytes = (REPO / 'web/studio/pkg/synthetic-stack.lut').read_bytes()
            artifact.write_bytes(artifact_bytes)
            artifact_sha = hashlib.sha256(artifact_bytes).hexdigest()
            tokenizer = root / 'trained.tokenizer.json'
            tokenizer.write_bytes((REPO / 'web/studio/pkg/tokenizer.json').read_bytes())
            report = FIXTURE_MODULE.report()
            report['artifact'] = {'sha256': artifact_sha, 'bytes': len(artifact_bytes)}
            audit = root / 'trained.audit.json'
            audit.write_text(json.dumps(report), encoding='utf-8')
            audit_sha = hashlib.sha256(audit.read_bytes()).hexdigest()
            manifest = root / 'trained.json'
            manifest.write_text(json.dumps([{
                'id': 'trained-fixture',
                'label': 'Trained manifest fixture',
                'file': str(artifact),
                'tokenizer': str(tokenizer),
                'audit': str(audit),
                'audit_sha256': audit_sha,
            }]), encoding='utf-8')
            dest = root / 'studio'
            result = subprocess.run([
                'bash', str(REPO / 'scripts/stage-studio-site.sh'),
                str(REPO / 'web/studio'), str(dest), '--trained-manifest', str(manifest),
            ], text=True, capture_output=True)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            for suffix in ['uorlut', 'tokenizer.json', 'uorlut.audit.json']:
                self.assertTrue((dest / 'models' / f'trained-fixture.{suffix}').is_file(), suffix)
            catalog = json.loads((dest / 'models.json').read_text(encoding='utf-8'))
            self.assertIn('trained-fixture', [model['id'] for model in catalog['models']])
            (dest / 'models/unexpected.txt').write_text('not generated', encoding='utf-8')
            rejected = subprocess.run(
                ['bash', str(RUNTIME), '--verify', str(dest)],
                text=True, capture_output=True,
            )
            self.assertNotEqual(rejected.returncode, 0)
            self.assertIn('Unexpected runtime file', rejected.stderr)
            (dest / 'models/unexpected.txt').unlink()
            for name in ['extra.uorlut', 'extra.tokenizer.json', 'extra.uorlut.audit.json']:
                extra = dest / 'models' / name
                extra.write_bytes(b'undeclared but allowed suffix')
                rejected = subprocess.run(
                    ['bash', str(RUNTIME), '--verify', str(dest)],
                    text=True, capture_output=True,
                )
                self.assertNotEqual(rejected.returncode, 0, name)
                extra.unlink()
            artifact = dest / 'models/trained-fixture.uorlut'
            artifact.write_bytes(b'tampered declared artifact')
            rejected = subprocess.run(
                ['bash', str(RUNTIME), '--verify', str(dest)],
                text=True, capture_output=True,
            )
            self.assertNotEqual(rejected.returncode, 0)
            self.assertIn('hash', rejected.stderr.lower())
            artifact.write_bytes(artifact_bytes)
            bundled = dest / 'pkg/synthetic-stack.lut'
            bundled_bytes = bundled.read_bytes()
            bundled.write_bytes(b'tampered bundled stack')
            rejected = subprocess.run(
                ['bash', str(RUNTIME), '--verify', str(dest)],
                text=True, capture_output=True,
            )
            self.assertNotEqual(rejected.returncode, 0)
            self.assertIn('hash', rejected.stderr.lower())
            bundled.write_bytes(bundled_bytes)
            models = dest / 'models'
            models.rename(dest / 'models-real')
            models.symlink_to(dest / 'models-real', target_is_directory=True)
            rejected = subprocess.run(
                ['bash', str(RUNTIME), '--verify', str(dest)],
                text=True, capture_output=True,
            )
            self.assertNotEqual(rejected.returncode, 0)
            self.assertIn('symlink', rejected.stderr.lower())
            models.unlink()
            (dest / 'models-real').rename(models)
            pkg = dest / 'pkg'
            pkg.rename(dest / 'pkg-real')
            pkg.symlink_to(dest / 'pkg-real', target_is_directory=True)
            rejected = subprocess.run(
                ['bash', str(RUNTIME), '--verify', str(dest)],
                text=True, capture_output=True,
            )
            self.assertNotEqual(rejected.returncode, 0)
            self.assertIn('symlink', rejected.stderr.lower())
            pkg.unlink()
            (dest / 'pkg-real').rename(pkg)
            extra = dest / 'assets/js/unexpected-link.js'
            extra.symlink_to(artifact)
            rejected = subprocess.run(
                ['bash', str(RUNTIME), '--verify', str(dest)],
                text=True, capture_output=True,
            )
            self.assertNotEqual(rejected.returncode, 0)
            self.assertIn('symlink', rejected.stderr.lower())
            extra.unlink()

    def test_prepare_check_is_side_effect_free(self):
        with tempfile.TemporaryDirectory() as tmp:
            env = os.environ.copy()
            env['UOR_TAURI_RUNTIME_HOME'] = str(Path(tmp) / 'not-created')
            result = subprocess.run(['bash', str(PREPARE), '--check'], env=env, text=True, capture_output=True, check=True)
            self.assertIn('uor-chat-aarch64-apple-darwin', result.stdout)
            self.assertIn('externalBin: ["binaries/uor-chat"]', result.stdout)
            self.assertIn('f4f36c72baea6b0fb76c72ff511cf8e9615fd328022d3b08937660b28cc05f7e', result.stdout)
            self.assertFalse(Path(env['UOR_TAURI_RUNTIME_HOME']).exists())


if __name__ == '__main__':
    unittest.main()
