import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

export const PUBLIC_M1_ID = 'm1-discrete-trunk-seed-1';
const revision = '1fec0d4c42d4d5929a0cf08b1a0f17291d1dbc55';
const release = 'https://huggingface.co/caseyallard/uor-r4-m1-model/resolve/' + revision;
const receiptPath = 'scripts/studio/public-m1-served-forward.json';
const receiptSha = 'ff370aefd6ba8062e2316c5478a8d428b8e220f8c5da2d47f7b0eeed24bfe479';
const decisionSha = 'a009e0cbdc2a485583803ba9c9cc92e00810a2497c9413fc4a40057331e4cbbe';
const exportSha = '2d94fb873be0b4d60e8b5b79c551917b9009d429862af33d9f26363dd6a1719a';
const paritySha = 'd93d61bf0ee25825812522f49f7d82552d95c4f9601f54f0baa4cf49a843c12c';
const devSha = '685226d637bc541237b230a89b370f24bf02129c0bef57fed33aa77c8fe88add';
const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');

function evidenceBytes(path, digest) {
  const bytes = readFileSync(new URL('../../' + path, import.meta.url));
  assert.equal(sha256(bytes), digest, 'M1 evidence digest differs: ' + path);
  return bytes;
}

// The CPU reader and native serving model expose different interfaces. Compare the
// same integer words, including candidate order, before checking the browser top-8.
function decisionDigest(prefix, decision) {
  assert.equal(decision.query_q12.length, 32);
  assert.equal(decision.admitted_cells.length, 8);
  assert.equal(decision.top8.length, 8);
  assert.equal(decision.token, decision.top8[0]);
  const words = [...prefix, ...decision.query_q12, ...decision.admitted_cells, ...decision.top8, decision.token];
  const bytes = Buffer.alloc(words.length * 4);
  words.forEach((word, index) => {
    assert.ok(Number.isInteger(word), 'decision word must be an integer');
    bytes.writeInt32LE(word, index * 4);
  });
  return sha256(bytes);
}

export function validateM1Receipt(receipt) {
  const exports = JSON.parse(evidenceBytes('docs/labs/m1-close-saved-model/evidence/t16-frozen-zero-seed1-trunk-export.json', exportSha));
  const parity = JSON.parse(evidenceBytes('docs/labs/one-model-end-to-end/evidence-deepseek-11-trunk-parity.json', paritySha));
  const dev = JSON.parse(evidenceBytes('docs/labs/m1-close-saved-model/evidence/t16-frozen-zero-seed1-trunk-dev-score.json', devSha));
  assert.equal(receipt.schema, 'uor-r4.m1-discrete-served-forward/1');
  assert.equal(receipt.release_revision, revision);
  assert.equal(receipt.artifact.schema, 'uor-r4.lut-discrete-trunk/1');
  assert.equal(receipt.artifact.bytes, exports.bytes);
  assert.equal(receipt.artifact.sha256, exports.file_sha256);
  assert.equal(receipt.artifact.sha256, parity.artifact.sha256);
  assert.equal(receipt.artifact.sha256, dev.artifact_sha256);
  assert.equal(receipt.tokenizer.sha256, parity.tokenizer.sha256);
  assert.equal(receipt.tokenizer.bytes, 265393);
  assert.equal(receipt.export_sha256, exportSha);
  assert.equal(receipt.gpu_dev_score_sha256, devSha);
  assert.equal(receipt.cpu_panel.parity_sha256, paritySha);
  assert.equal(receipt.cpu_panel.tokens_sha256, parity.panel.tokens_sha256);
  assert.equal(receipt.cpu_panel.mask_sha256, parity.panel.mask_sha256);
  assert.equal(receipt.cpu_panel.manifest_sha256, parity.panel.manifest_sha256);
  const cpu = parity.cpu_engine_parity;
  assert.equal(parity.artifact.tables, 19);
  assert.equal(cpu.rows, parity.panel.response_targets);
  assert.equal(cpu.rows, 616);
  assert.equal(cpu.query_coordinates_checked, cpu.rows * 32);
  assert.equal(cpu.query_coordinate_mismatches, 0);
  assert.equal(cpu.admitted_cell_mismatches, 0);
  assert.equal(cpu.top8_mismatches, 0);
  assert.equal(cpu.token_mismatches, 0);
  assert.equal(dev.reference.checked, 14287);
  assert.equal(dev.reference.agreed, dev.reference.checked);
  assert.equal(dev.planted_fault_b4.b4_pass, true);

  const { prompt, prefix, independent_cpu, native_serving, browser_top8 } = receipt.browser_workload;
  assert.equal(prompt, 'The capital of France is');
  assert.deepEqual(prefix, [413, 2743, 3695, 372, 3704, 1651, 435]);
  assert.deepEqual(independent_cpu, native_serving, 'CPU and native serving decisions differ');
  assert.equal(decisionDigest(prefix, native_serving), decisionSha, 'raw decision differs from measured oracle');
  assert.deepEqual(browser_top8, native_serving.top8, 'browser and native top-8 differ');
}

export function validateM1CardPins(card, receipt) {
  assert.equal(card.sha256, receipt.artifact.sha256, 'catalog model SHA-256 differs');
  assert.equal(card.tokenizer_sha256, receipt.tokenizer.sha256, 'catalog tokenizer SHA-256 differs');
  assert.equal(card.schema, receipt.artifact.schema, 'catalog model schema differs');
  assert.equal(card.size_bytes, receipt.artifact.bytes, 'catalog model byte count differs');
}

export function publicM1Card(selector, dest) {
  if (selector.public_release !== PUBLIC_M1_ID || Object.keys(selector).length !== 1) {
    throw new Error('Unknown public release or attempt to override its pins');
  }
  const bytes = evidenceBytes(receiptPath, receiptSha);
  const receipt = JSON.parse(bytes);
  validateM1Receipt(receipt);
  const auditName = PUBLIC_M1_ID + '.uorlut.audit.json';
  mkdirSync(join(dest, 'models'), { recursive: true });
  writeFileSync(join(dest, 'models', auditName), bytes);
  return {
    id: PUBLIC_M1_ID, kind: 'published', primary: true, label: 'M1 next-token predictor (seed 1)',
    url: release + '/t16-frozen-zero-seed1-trunk.d11', tokenizer_url: release + '/tokenizer-seed-1.json',
    format: 'UORLUT01', schema: receipt.artifact.schema, size_bytes: receipt.artifact.bytes,
    sha256: receipt.artifact.sha256, tokenizer_sha256: receipt.tokenizer.sha256,
    served_forward_audit_schema: receipt.schema, served_forward_audit_sha256: receiptSha, audit_url: './models/' + auditName,
    note: 'Downloads 86.5 MB of public model weights and a matching tokenizer on request. Ranks eight next tokens locally.',
  };
}

export function validateM1DownloadRequests(requests, permittedRoots, clickedAt, countBeforeOffline) {
  for (const request of requests) {
    assert.ok(Number.isFinite(request.startTime) && request.startTime >= clickedAt, 'external request began before the trusted load click');
    assert.equal(request.method, 'GET');
    assert.ok(permittedRoots.includes(request.root), 'Unexpected download root: ' + request.root);
    assert.ok(request.origin === 'https://huggingface.co' || /^https:\/\/(?:[a-z0-9-]+\.)+cdn\.hf\.co$/.test(request.origin), 'Unexpected download origin: ' + request.origin);
  }
  assert.deepEqual([...new Set(requests.map(request => request.root))].sort(), [...permittedRoots].sort());
  assert.equal(requests.length, countBeforeOffline, 'external request observed during offline prediction');
}
