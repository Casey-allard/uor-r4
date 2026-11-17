import test from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { cpSync, mkdirSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { writeStudioCatalog } from '../../../../scripts/studio/catalog.mjs';
import { publicM1Card, PUBLIC_M1_ID, validateM1CardPins, validateM1DownloadRequests, validateM1Receipt } from '../../../../scripts/studio/public-m1.mjs';

test('M1 download validation checks late requests and the offline boundary independently', () => {
    const origin = 'https://huggingface.co';
    const roots = [origin + '/model', origin + '/tokenizer'];
    const requests = roots.map(root => ({ method: 'GET', origin, root, startTime: 12 }));
    validateM1DownloadRequests(requests, roots, 10, 2);

    const late = { ...requests[1], origin: 'https://unlisted.example' };
    assert.throws(() => validateM1DownloadRequests([...requests, late], roots, 10, 3), /Unexpected download origin/);
    assert.throws(() => validateM1DownloadRequests([...requests, requests[1]], roots, 10, 2), /offline prediction/);
    assert.throws(() => validateM1DownloadRequests([...requests, { ...requests[1], startTime: 9 }], roots, 10, 3), /before the trusted load click/);
});

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

test('public M1 publication takes only its fixed release selector', () => {
    const root = fixtureRoot();
    const manifest = join(root, 'trained.json');
    const stage = (entry) => {
        writeFileSync(manifest, JSON.stringify([entry]));
        return writeStudioCatalog({ dest: root, trainedManifestPath: manifest });
    };
    const selector = { public_release: 'm1-discrete-trunk-seed-1' };
    const card = stage(selector).models[1];
    assert.match(card.url, /\/resolve\/1fec0d4c42d4d5929a0cf08b1a0f17291d1dbc55\//);
    assert.equal(card.sha256, '04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d');
    assert.equal(card.tokenizer_sha256, 'a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377');
    assert.throws(() => stage({ ...selector, hf_url: 'https://example.org/other.d11' }), /override/);
    assert.throws(() => stage({ public_release: 'fbs' }), /Unknown/);
});

test('the live verifier requires all four served card pins independently of the downloaded bytes', () => {
    const root = fixtureRoot();
    const card = publicM1Card({ public_release: PUBLIC_M1_ID }, root);
    const receipt = JSON.parse(readFileSync(join(root, card.audit_url), 'utf8'));
    validateM1CardPins(card, receipt);
    for (const field of ['sha256', 'tokenizer_sha256', 'schema', 'size_bytes']) {
        const omitted = { ...card };
        delete omitted[field];
        assert.throws(() => validateM1CardPins(omitted, receipt), /catalog/);
        assert.throws(() => validateM1CardPins({ ...card, [field]: 'wrong' }, receipt), /catalog/);
    }
});

test('the measured M1 workload and raw decisions are required even when the two claimed outputs agree', () => {
    const path = new URL('../../../../scripts/studio/public-m1-served-forward.json', import.meta.url);
    const receipt = JSON.parse(readFileSync(path, 'utf8'));
    validateM1Receipt(receipt);
    for (const mutate of [
        r => { r.artifact.bytes -= 1; },
        r => { r.artifact.sha256 = '0'.repeat(64); },
        r => { r.tokenizer.sha256 = '0'.repeat(64); },
        r => { r.cpu_panel.mask_sha256 = '0'.repeat(64); },
        r => { r.browser_workload.prefix[0] += 1; },
        r => { r.browser_workload.independent_cpu.admitted_cells.reverse(); },
        r => { r.browser_workload.native_serving.query_q12[0] += 1; },
        r => { r.browser_workload.browser_top8.reverse(); },
        r => {
            r.browser_workload.independent_cpu.query_q12[0] += 1;
            r.browser_workload.native_serving.query_q12[0] += 1;
        },
    ]) {
        const changed = structuredClone(receipt);
        mutate(changed);
        assert.throws(() => validateM1Receipt(changed));
    }
});
