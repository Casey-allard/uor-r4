import json
import os
import shutil
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
            for req in [
                'index.html',
                '404.html',
                'holo.html',
                'manifest.json',
                'sw.js',
                'models.json',
                'host.bundle.js',
                'core.wasm',
                'assets/index.js',
                'assets/index.css',
                'kernel/kernel.holo',
                'pkg/uor_r4_integer.js',
                'pkg/uor_r4_integer_bg.wasm',
                'pkg/synthetic-stack.lut',
                'pkg/tokenizer.json',
                'pkg-mvm/uor_r4_mvm.js',
                'pkg-mvm/uor_r4_mvm_bg.wasm',
                'uor/js/uor_model_worker.js',
                'uor/js/uor_tokenizer_artifact.js',
                'app.json',
                'scaffold/files.json',
                'scaffold/src/vite-env.d.ts',
                'v86/v86.wasm',
                'images/buildroot-bzimage.bin',
            ]:
                self.assertTrue((dest / req).is_file(), req)
            for rel in json.loads((dest / 'scaffold/files.json').read_text(encoding='utf-8')):
                self.assertTrue((dest / 'scaffold' / rel).is_file(), f'scaffold/{rel}')
            for forbidden in ['sw.template.js', 'warmup.json', 'node_modules', 'pkg/README.md', 'pkg/uor_r4_integer.d.ts', 'pkg/package.json']:
                self.assertFalse((dest / forbidden).exists(), forbidden)
            (dest / 'assets/example.map').write_text('unexpected', encoding='utf-8')
            rejected = run('bash', str(RUNTIME), '--verify', str(dest), '--typed')
            self.assertNotEqual(rejected.returncode, 0, rejected.stdout + rejected.stderr)

    def test_typed_pages_staging_uses_dist(self):
        with tempfile.TemporaryDirectory() as tmp:
            typed_dest = Path(tmp) / 'typed'
            result = run('bash', str(REPO / 'scripts/stage-studio-site.sh'), '--typed', str(REPO / 'web/studio'), str(typed_dest))
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertTrue((typed_dest / 'index.html').is_file())
            self.assertTrue((typed_dest / 'models.json').is_file())
            self.assertTrue((typed_dest / 'core.wasm').is_file())
            self.assertTrue((typed_dest / 'assets/index.js').is_file())
            self.assertTrue((typed_dest / 'assets/index.css').is_file())
            for name in ['uor_r4_integer.js', 'uor_r4_integer_bg.wasm', 'synthetic-stack.lut', 'tokenizer.json']:
                self.assertTrue((typed_dest / 'pkg' / name).is_file(), name)
            for name in ['uor_r4_mvm.js', 'uor_r4_mvm_bg.wasm']:
                self.assertTrue((typed_dest / 'pkg-mvm' / name).is_file(), name)
            for rel in json.loads((typed_dest / 'scaffold/files.json').read_text(encoding='utf-8')):
                self.assertTrue((typed_dest / 'scaffold' / rel).is_file(), f'scaffold/{rel}')
            models_doc = json.loads((typed_dest / 'models.json').read_text(encoding='utf-8'))
            self.assertIs(models_doc['sources']['local_file'], False)
            model_ids = [m['id'] for m in models_doc['models']]
            self.assertEqual(model_ids, ['qwen3-0.6b-foundry'])
            for forbidden in ['sw.template.js', 'warmup.json', 'node_modules', 'pkg/README.md', 'pkg/uor_r4_integer.d.ts', 'pkg/package.json']:
                self.assertFalse((typed_dest / forbidden).exists(), forbidden)
            self.assertEqual(list(typed_dest.rglob('*.map')), [])

            default_dest = Path(tmp) / 'default'
            default_res = run('bash', str(REPO / 'scripts/stage-studio-site.sh'), str(REPO / 'web/studio'), str(default_dest))
            self.assertEqual(default_res.returncode, 0, default_res.stdout + default_res.stderr)
            self.assertTrue((default_dest / 'core.wasm').is_file())

    def test_publisher_exposes_dry_run_mode(self):
        publisher = REPO / 'scripts/studio/build-and-publish.sh'
        help_result = run('bash', str(publisher), '--help')
        self.assertEqual(help_result.returncode, 0, help_result.stdout + help_result.stderr)
        self.assertIn('--target', help_result.stdout)

        with tempfile.TemporaryDirectory() as tmp_name:
            tmp = Path(tmp_name)
            try:
                result = subprocess.run([
                    'bash', str(publisher), '--typed', '--no-build', '--skip-verify',
                    '--stage-dir', str(tmp / 'staged'), '--keep-worktree',
                ], cwd=REPO, text=True, capture_output=True, env={**os.environ, 'TMPDIR': tmp_name})
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                self.assertIn('Foundry Studio + UOR-R4 WASM + RV64IMAC MVM (dist)', result.stdout)
                self.assertIn('target:  gh-pages (dry run, nothing pushed)', result.stdout)
                staged = tmp / 'staged'
                self.assertTrue((staged / 'index.html').is_file())
                self.assertTrue((staged / 'models.json').is_file())
                self.assertTrue((staged / 'core.wasm').is_file())
                self.assertTrue((staged / 'pkg-mvm/uor_r4_mvm_bg.wasm').is_file())
                self.assertEqual(list(staged.rglob('*.map')), [])
            finally:
                for site in tmp.glob('studio-publish.*/site'):
                    if site.exists():
                        run('git', 'worktree', 'remove', '--force', str(site))
                shutil.rmtree(tmp, ignore_errors=True)

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

    def test_cowork_identity_separates_recovered_workbench_state(self):
        config = (REPO / 'src-tauri/tauri.conf.json').read_text(encoding='utf-8')
        self.assertIn('"identifier": "org.uor.r4.desktop.cowork"', config)
        self.assertNotIn('"identifier": "org.uor.r4.desktop"', config)

    def test_canonical_launcher_builds_and_opens_the_cowork_workbench(self):
        launcher = REPO / 'scripts/studio/open-cowork.sh'
        self.assertTrue(launcher.is_file(), 'canonical cowork launcher is missing')
        source = launcher.read_text(encoding='utf-8')
        self.assertIn('prepare-tauri-runtime.sh', source)
        self.assertIn('cargo tauri build --bundles app --features gui', source)
        self.assertIn('org.uor.r4.desktop.cowork', source)
        self.assertIn('${CARGO_TARGET_DIR:-$repo_root/target/cowork}', source)
        self.assertIn('exec "$app/Contents/MacOS/uor-desktop-client"', source)
        self.assertNotIn('open -n "$app"', source)
        self.assertNotIn('open -n -g', source)
        self.assertNotIn('studio-shared-shell-recovered', source)
        result = run('bash', str(launcher), '--check')
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn('identity: org.uor.r4.desktop.cowork', result.stdout)


if __name__ == '__main__':
    unittest.main()
