import test from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { cpSync, mkdirSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { writeStudioCatalog } from '../../../../scripts/studio/catalog.mjs';

function fixtureRoot() {
    const root = mkdtempSync(join(tmpdir(), 'uor-studio-catalog-'));
    const repo = join(import.meta.dirname, '..', '..', '..', '..');
    execFileSync('bash', ['scripts/studio/runtime-files.sh', 'copy', join(repo, 'web/studio'), root, '--allow-generated-missing'], { cwd: repo });
    mkdirSync(join(root, 'pkg'), { recursive: true });
    for (const name of ['uor_r4_integer.js', 'uor_r4_integer_bg.wasm', 'synthetic-stack.lut', 'tokenizer.json']) {
        writeFileSync(join(root, 'pkg', name), name);
    }
    return root;
}

test('dist catalog matches the pre-change publisher for identical synthetic input', () => {
    const repo = join(import.meta.dirname, '..', '..', '..', '..');
    const root = fixtureRoot();
    const dist = join(root, 'dist');
    mkdirSync(dist, { recursive: true });
    cpSync(join(root, 'pkg'), join(dist, 'pkg'), { recursive: true });
    writeStudioCatalog({ dest: dist, publicUrl: '', publicLabel: 'Public model', trainedManifestPath: '', receiptGate: '' });

    mkdirSync(join(root, 'scripts/studio'), { recursive: true });
    cpSync(join(repo, 'scripts/studio/runtime-files.sh'), join(root, 'scripts/studio/runtime-files.sh'));
    const basePublisher = join(root, 'scripts/publisher-base.sh');
    const baseSource = execFileSync('git', ['show', '000f4b457ee020013654ffbc65f9da5061c2317a:scripts/stage-studio-site.sh'], { cwd: repo, encoding: 'utf8' });
    writeFileSync(basePublisher, baseSource);
    const published = join(root, 'published');
    execFileSync('bash', [basePublisher, root, published], { cwd: repo, stdio: 'pipe' });
    assert.equal(readFileSync(join(dist, 'models.json'), 'utf8'), readFileSync(join(published, 'models.json'), 'utf8'));
});
