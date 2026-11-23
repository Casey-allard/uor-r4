// Real in-browser UOR-R4 runtime bridge:
// - Primary: `uor-r4:qwen3-0.6b-foundry` (`UORR4V13` pure-integer Qwen3-0.6B Foundry via `./uor-v13-worker.mjs`)
// - Secondary: pinned M1 discrete trunk (`uor-r4:m1-discrete-trunk-seed-1`) + local `.uorlut`/`.d11` upload (`./uor/js/uor_model_worker.js`)
//
// Zero fabricated replies or dummy seed-5 models.

import {
  encodeTextWithArtifact,
  tokenizerLabel,
  validateTokenizerArtifact,
} from './uor/js/uor_tokenizer_artifact.js';

const BASE = new URL('./', import.meta.url);
const OPFS_DIR = 'uor-r4-models';
const OPFS_V13_DIR = 'uor-r4-v13-packs';
const IDB_NAME = 'uor-r4-model-cache';
const IDB_STORE = 'blobs';
const IDB_V13_NAME = 'uor-r4-v13-cache';
const DEFAULT_MODEL_SHORT = 'qwen3-0.6b-foundry';

const FALLBACK_V13_MODEL = Object.freeze({
  id: 'qwen3-0.6b-foundry',
  kind: 'stitch-packs',
  primary: true,
  label: 'UOR-R4 Qwen3-0.6B Foundry',
  format: 'UORR4V13',
  manifest_url:
    'https://raw.githubusercontent.com/Casey-allard/lexlean-r4/3b04cd460735af679d912ce749e6e4bb2bccc0b0/manifest.json',
  stitch_manifest_url:
    'https://raw.githubusercontent.com/Casey-allard/lexlean-r4/3b04cd460735af679d912ce749e6e4bb2bccc0b0/manifest.stitch.json',
  manifest_sha256: '9c71182fe5462b55990b7f139635d0183ce73b15210c9169d68326f65f704a27',
  stitch_manifest_sha256: '42154f44c064d364303b1695b6c941228719be953b0aa8f8b4254c2b3761cd61',
  container_sha256: '789333898aed75c3be0f1db85a4da5b50bb1067481c37a79af38c605b4488d61',
  size_bytes: 500760064,
  context_window: 4096,
  note: 'Pure-integer UORR4V13 Qwen3-0.6B fine-tuned for Foundry 16-tool webharness.',
});

const BUNDLED_FALLBACK_CATALOG = Object.freeze({
  schema: 'uor-r4.studio-models/1',
  models: [FALLBACK_V13_MODEL],
});

let worker = null;
let v13Worker = null;
let catalogCache = null;
let loadedModelId = null;
let loadedCapabilities = null;
let loadedTokenizerArtifact = null;
let localUploadRecord = null;
let opfsStatus = 'untested';
let pendingOp = Promise.resolve();
let nextMsgId = 1;
const pendingReplies = new Map();
const pendingV13Replies = new Map();
const progressListeners = new Set();

let hydrationState = {
  modelId: `uor-r4:${DEFAULT_MODEL_SHORT}`,
  phase: 'idle',
  percent: 0,
  packsDone: 0,
  packsTotal: 10,
  piecesDone: 0,
  piecesTotal: 452,
  bytesLoaded: 0,
  bytesTotal: 500760064,
  cached: false,
  cacheBackend: null,
  recoveredPieces: [],
  error: null,
};

function notifyProgress(patch) {
  hydrationState = { ...hydrationState, ...patch };
  for (const fn of progressListeners) {
    try {
      fn(hydrationState);
    } catch {}
  }
}

export function onProgress(fn) {
  progressListeners.add(fn);
  try {
    fn(hydrationState);
  } catch {}
  return () => progressListeners.delete(fn);
}

export function getHydrationState() {
  return { ...hydrationState };
}

function sanitizeMirrorBase(raw) {
  const s = String(raw || '').trim();
  if (!s) return '';
  try {
    const parsed = new URL(s, globalThis.location?.href || 'http://127.0.0.1/');
    if (!['http:', 'https:'].includes(parsed.protocol)) return '';
    const sameOrigin = Boolean(globalThis.location?.origin && parsed.origin === globalThis.location.origin);
    const isLoopback =
      parsed.hostname === '127.0.0.1' ||
      parsed.hostname === 'localhost' ||
      parsed.hostname === '[::1]';
    if (!sameOrigin && !isLoopback) return '';
    return parsed.href;
  } catch {
    return '';
  }
}

function getV13MirrorBase() {
  try {
    if (typeof globalThis.__UOR_V13_MIRROR__ === 'string' && globalThis.__UOR_V13_MIRROR__.trim()) {
      return sanitizeMirrorBase(globalThis.__UOR_V13_MIRROR__);
    }
    const q = new URLSearchParams(globalThis.location?.search || '').get('v13Mirror');
    if (q && q.trim()) return sanitizeMirrorBase(q);
    const s = globalThis.localStorage?.getItem('uor.v13.mirror');
    if (s && s.trim()) return sanitizeMirrorBase(s);
  } catch {}
  return '';
}

function normalizeModelId(modelId) {
  const raw = String(modelId || `uor-r4:${DEFAULT_MODEL_SHORT}`).trim();
  return raw.startsWith('uor-r4:') ? raw.slice('uor-r4:'.length) : raw;
}

function toFullModelId(shortId) {
  return shortId.startsWith('uor-r4:') ? shortId : `uor-r4:${shortId}`;
}

async function sha256Hex(bytes) {
  const u8 = bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes);
  const digest = await crypto.subtle.digest('SHA-256', u8);
  return Array.from(new Uint8Array(digest), (b) => b.toString(16).padStart(2, '0')).join('');
}

async function openFallbackIdb(name = IDB_NAME) {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(name, 1);
    req.onupgradeneeded = () => {
      const db = req.result;
      if (!db.objectStoreNames.contains(IDB_STORE)) {
        db.createObjectStore(IDB_STORE);
      }
    };
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });
}

async function cacheReadBytes(key) {
  try {
    if (navigator.storage?.getDirectory) {
      const root = await navigator.storage.getDirectory();
      const dir = await root.getDirectoryHandle(OPFS_DIR, { create: true });
      const fh = await dir.getFileHandle(key);
      const file = await fh.getFile();
      opfsStatus = 'opfs';
      return new Uint8Array(await file.arrayBuffer());
    }
  } catch {
    // Fall through to IndexedDB cache.
  }
  try {
    const db = await openFallbackIdb(IDB_NAME);
    const val = await new Promise((resolve, reject) => {
      const tx = db.transaction(IDB_STORE, 'readonly');
      const req = tx.objectStore(IDB_STORE).get(key);
      req.onsuccess = () => resolve(req.result || null);
      req.onerror = () => reject(req.error);
    });
    if (val) {
      opfsStatus = opfsStatus === 'opfs' ? 'opfs' : 'indexeddb';
      return val instanceof Uint8Array ? val : new Uint8Array(val);
    }
  } catch {
    // Cache miss or unavailable storage.
  }
  return null;
}

async function cacheWriteBytes(key, bytes) {
  const u8 = bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes);
  try {
    if (navigator.storage?.getDirectory) {
      const root = await navigator.storage.getDirectory();
      const dir = await root.getDirectoryHandle(OPFS_DIR, { create: true });
      const fh = await dir.getFileHandle(key, { create: true });
      const writable = await fh.createWritable();
      await writable.write(u8);
      await writable.close();
      opfsStatus = 'opfs';
      return 'opfs';
    }
  } catch {
    // Fall through to IndexedDB cache.
  }
  try {
    const db = await openFallbackIdb(IDB_NAME);
    await new Promise((resolve, reject) => {
      const tx = db.transaction(IDB_STORE, 'readwrite');
      tx.objectStore(IDB_STORE).put(u8, key);
      tx.oncomplete = () => resolve();
      tx.onerror = () => reject(tx.error);
    });
    opfsStatus = 'indexeddb';
    return 'indexeddb';
  } catch {
    return 'memory';
  }
}

async function isCachedInStorage(key) {
  if (!key) return false;
  try {
    if (navigator.storage?.getDirectory) {
      const root = await navigator.storage.getDirectory();
      const dir = await root.getDirectoryHandle(OPFS_DIR, { create: false });
      await dir.getFileHandle(key, { create: false });
      return true;
    }
  } catch {}
  try {
    const db = await openFallbackIdb(IDB_NAME);
    const existing = await new Promise((resolve) => {
      const tx = db.transaction(IDB_STORE, 'readonly');
      const req = tx.objectStore(IDB_STORE).getKey(key);
      req.onsuccess = () => resolve(req.result !== undefined);
      req.onerror = () => resolve(false);
    });
    return existing;
  } catch {
    return false;
  }
}

async function isV13Cached(containerSha) {
  if (!containerSha) return false;
  try {
    if (navigator.storage?.getDirectory) {
      const root = await navigator.storage.getDirectory();
      const base = await root.getDirectoryHandle(OPFS_V13_DIR, { create: false });
      const scoped = await base.getDirectoryHandle(containerSha, { create: false });
      await scoped.getFileHandle('pack_09.pack', { create: false });
      return true;
    }
  } catch {}
  try {
    const db = await openFallbackIdb(IDB_V13_NAME);
    const existing = await new Promise((resolve) => {
      const tx = db.transaction(IDB_STORE, 'readonly');
      const req = tx.objectStore(IDB_STORE).getKey(`${containerSha}/pack_09.pack`);
      req.onsuccess = () => resolve(req.result !== undefined);
      req.onerror = () => resolve(false);
    });
    return existing;
  } catch {
    return false;
  }
}

function ensureWorker() {
  if (worker) return worker;
  worker = new Worker(new URL('./uor/js/uor_model_worker.js', BASE), { type: 'module' });
  worker.addEventListener('message', ({ data }) => {
    if (!data || typeof data !== 'object') return;
    const entry = pendingReplies.get(data.id);
    if (!entry) return;
    if (data.action === 'stream_token') {
      entry.onToken?.(data);
      return;
    }
    if (data.action === 'generate_error' || data.action === 'operation_error') {
      pendingReplies.delete(data.id);
      entry.reject(new Error(data.error || 'UOR-R4 worker operation failed'));
      return;
    }
    if (entry.expectAction && data.action !== entry.expectAction) {
      return;
    }
    pendingReplies.delete(data.id);
    entry.resolve(data);
  });
  worker.addEventListener('error', (err) => {
    for (const [id, entry] of pendingReplies.entries()) {
      pendingReplies.delete(id);
      entry.reject(new Error(err?.message || 'UOR-R4 worker error'));
    }
  });
  return worker;
}

function ensureV13Worker() {
  if (v13Worker) return v13Worker;
  v13Worker = new Worker(new URL('./uor-v13-worker.mjs', BASE), { type: 'module' });
  v13Worker.addEventListener('message', ({ data }) => {
    if (!data || typeof data !== 'object') return;
    if (data.action === 'v13_progress') {
      if (data.cacheBackend && data.cacheBackend !== 'network') {
        opfsStatus = data.cacheBackend;
      }
      notifyProgress({
        phase: data.phase,
        percent: data.percent,
        packsDone: data.packsDone,
        packsTotal: data.packsTotal,
        piecesDone: data.piecesDone,
        piecesTotal: data.piecesTotal,
        bytesLoaded: data.bytesLoaded,
        bytesTotal: data.bytesTotal,
        cached: Boolean(data.cached),
        cacheBackend: data.cacheBackend || null,
        recoveredPieces: Array.isArray(data.recoveredPieces) ? data.recoveredPieces : [],
        error: null,
      });
      return;
    }
    const entry = pendingV13Replies.get(data.id);
    if (!entry) return;
    if (data.action === 'v13_token') {
      entry.onToken?.(data);
      return;
    }
    if (data.action === 'v13_error') {
      pendingV13Replies.delete(data.id);
      const err = new Error(data.error || 'UOR-R4 V13 operation failed');
      if (data.cancelled) err.name = 'AbortError';
      entry.reject(err);
      return;
    }
    if (entry.expectAction && data.action !== entry.expectAction) {
      return;
    }
    pendingV13Replies.delete(data.id);
    entry.resolve(data);
  });
  v13Worker.addEventListener('error', (err) => {
    for (const [id, entry] of pendingV13Replies.entries()) {
      pendingV13Replies.delete(id);
      entry.reject(new Error(err?.message || 'UOR-R4 V13 worker error'));
    }
  });
  return v13Worker;
}

function callWorker(action, payload, expectAction, { onToken, signal } = {}) {
  const w = ensureWorker();
  const id = `uor-${nextMsgId++}`;
  return new Promise((resolve, reject) => {
    const onAbort = () => {
      try {
        w.postMessage({ action: 'cancel', id });
      } catch {}
    };
    if (signal) {
      if (signal.aborted) {
        reject(Object.assign(new Error('Generation cancelled'), { name: 'AbortError' }));
        return;
      }
      signal.addEventListener('abort', onAbort, { once: true });
    }
    pendingReplies.set(id, {
      expectAction,
      onToken,
      resolve: (value) => {
        signal?.removeEventListener('abort', onAbort);
        resolve(value);
      },
      reject: (err) => {
        signal?.removeEventListener('abort', onAbort);
        reject(err);
      },
    });
    w.postMessage({ action, id, payload });
  });
}

function callV13Worker(action, payload, expectAction, { onToken, signal } = {}) {
  const w = ensureV13Worker();
  const id = `v13-${nextMsgId++}`;
  return new Promise((resolve, reject) => {
    let settled = false;
    const onAbort = () => {
      try {
        w.postMessage({ action: 'cancel_v13', id });
      } catch {}
      setTimeout(() => {
        if (!settled && pendingV13Replies.has(id)) {
          pendingV13Replies.delete(id);
          settled = true;
          reject(Object.assign(new Error('V13 operation cancelled'), { name: 'AbortError' }));
        }
      }, 250);
    };
    if (signal) {
      if (signal.aborted) {
        reject(Object.assign(new Error('V13 operation cancelled'), { name: 'AbortError' }));
        return;
      }
      signal.addEventListener('abort', onAbort, { once: true });
    }
    pendingV13Replies.set(id, {
      expectAction,
      onToken,
      resolve: (value) => {
        if (settled) return;
        settled = true;
        signal?.removeEventListener('abort', onAbort);
        resolve(value);
      },
      reject: (err) => {
        if (settled) return;
        settled = true;
        signal?.removeEventListener('abort', onAbort);
        reject(err);
      },
    });
    w.postMessage({ action, id, payload });
  });
}

function serializeOp(fn) {
  const next = pendingOp.then(fn, fn);
  pendingOp = next.catch(() => {});
  return next;
}

export async function getCatalog({ refresh = false } = {}) {
  if (!catalogCache || refresh) {
    try {
      const resp = await fetch(new URL('./models.json', BASE), { cache: 'no-store' });
      if (resp.ok) {
        catalogCache = await resp.json();
      } else {
        catalogCache = BUNDLED_FALLBACK_CATALOG;
      }
    } catch {
      catalogCache = BUNDLED_FALLBACK_CATALOG;
    }
  }
  const models = [];
  for (const entry of catalogCache.models || []) {
    if (entry.id === 'synthetic-seed-5') continue;
    const cached =
      entry.format === 'UORR4V13'
        ? await isV13Cached(entry.container_sha256)
        : entry.sha256
          ? await isCachedInStorage(`${entry.sha256}.bin`)
          : false;
    models.push({
      ...entry,
      apiName: toFullModelId(entry.id),
      cached,
      loaded: loadedModelId === entry.id,
    });
  }
  if (localUploadRecord) {
    models.push({
      id: 'local-upload',
      apiName: 'uor-r4:local-upload',
      kind: 'file',
      label: `Local file: ${localUploadRecord.fileName}`,
      format: 'UORLUT01',
      sha256: localUploadRecord.sha256,
      size_bytes: localUploadRecord.byteLength,
      cached: true,
      loaded: loadedModelId === 'local-upload',
    });
  }
  return {
    schema: catalogCache.schema || 'uor-r4.studio-models/1',
    status: getStatus(),
    hydration: getHydrationState(),
    models,
  };
}

export function getStatus() {
  return {
    projectStatus: 'Pre-alpha · M1 complete (1 of 8 milestones) · D90 Studio-only V13 Foundry',
    engine: loadedCapabilities?.engine || 'uor-foundry-v13/wasm',
    loadedModelId: loadedModelId ? toFullModelId(loadedModelId) : null,
    capabilities: loadedCapabilities,
    opfsStatus,
    hydration: getHydrationState(),
    weightsMatched: Boolean(
      loadedCapabilities &&
        (loadedCapabilities.expected_sha256 == null || loadedCapabilities.sha256_verified === true),
    ),
  };
}

async function fetchVerifiedBytes(url, expectedSha, cacheKey, { signal } = {}) {
  if (cacheKey) {
    const cached = await cacheReadBytes(cacheKey);
    if (cached) {
      if (!expectedSha || (await sha256Hex(cached)) === expectedSha) {
        return { bytes: cached, fromCache: true };
      }
    }
  }
  const resolved = /^https?:\/\//i.test(url) ? url : new URL(url, BASE).href;
  const resp = await fetch(resolved, { signal });
  if (!resp.ok) {
    throw new Error(`Failed to fetch artifact ${url}: HTTP ${resp.status}`);
  }
  const bytes = new Uint8Array(await resp.arrayBuffer());
  const actualSha = await sha256Hex(bytes);
  if (expectedSha && actualSha !== expectedSha) {
    throw new Error(`Artifact SHA-256 mismatch for ${url}: expected ${expectedSha}, got ${actualSha}`);
  }
  const key = cacheKey || `${actualSha}.bin`;
  await cacheWriteBytes(key, bytes);
  return { bytes, sha256: actualSha, fromCache: false };
}

async function fetchTokenizerJson(url, expectedSha, modelSha, { signal } = {}) {
  if (!url) return null;
  const key = modelSha ? `${modelSha}.tokenizer.json` : null;
  if (key) {
    const cached = await cacheReadBytes(key);
    if (cached) {
      try {
        if (!expectedSha || (await sha256Hex(cached)) === expectedSha) {
          return JSON.parse(new TextDecoder('utf-8').decode(cached));
        }
      } catch {}
    }
  }
  const resolved = /^https?:\/\//i.test(url) ? url : new URL(url, BASE).href;
  const resp = await fetch(resolved, { signal });
  if (!resp.ok) {
    throw new Error(`Failed to fetch tokenizer ${url}: HTTP ${resp.status}`);
  }
  const bytes = new Uint8Array(await resp.arrayBuffer());
  const actualSha = await sha256Hex(bytes);
  if (expectedSha && actualSha !== expectedSha) {
    throw new Error(`Tokenizer SHA-256 mismatch: expected ${expectedSha}, got ${actualSha}`);
  }
  if (key) {
    await cacheWriteBytes(key, bytes);
  }
  return JSON.parse(new TextDecoder('utf-8').decode(bytes));
}

async function loadIntoWorker({ modelId, bytes, expectedSha, source, sourceLabel, fileName, tokenizerArtifact }) {
  const resp = await callWorker(
    'load_model',
    {
      bytes,
      expectedSha256: expectedSha || undefined,
      source,
      sourceLabel,
      fileName,
      tokenizerArtifact: tokenizerArtifact ?? undefined,
    },
    'model_loaded',
  );
  loadedModelId = modelId;
  loadedCapabilities = resp.capabilities;
  loadedTokenizerArtifact = tokenizerArtifact || null;
  return {
    modelId: toFullModelId(modelId),
    capabilities: resp.capabilities,
    sha256: resp.sha256,
    bytes: resp.bytes,
    tokenizerLabel: tokenizerLabel(
      tokenizerArtifact,
      resp.capabilities?.shape?.vocab || 0,
    ),
  };
}

async function loadV13Entry(entry, { signal } = {}) {
  const mirrorBaseUrl = getV13MirrorBase();
  notifyProgress({
    modelId: toFullModelId(entry.id),
    phase: 'hydrating',
    error: null,
  });
  try {
    const resp = await callV13Worker(
      'load_v13',
      {
        modelId: toFullModelId(entry.id),
        manifestUrl: entry.manifest_url,
        stitchManifestUrl: entry.stitch_manifest_url,
        manifestSha256: entry.manifest_sha256,
        stitchManifestSha256: entry.stitch_manifest_sha256,
        containerSha256: entry.container_sha256,
        sizeBytes: entry.size_bytes,
        mirrorBaseUrl,
      },
      'v13_loaded',
      { signal },
    );
    loadedModelId = entry.id;
    loadedTokenizerArtifact = null;
    if (resp.cacheBackend && resp.cacheBackend !== 'network') {
      opfsStatus = resp.cacheBackend;
    }
    loadedCapabilities = {
      engine: 'uor-foundry-v13/wasm',
      format: 'UORR4V13',
      can_predict: false,
      can_generate: true,
      artifact_sha256: resp.status.container_sha256,
      expected_sha256: entry.container_sha256 || resp.status.container_sha256,
      sha256_verified: resp.status.container_sha256 === (entry.container_sha256 || resp.status.container_sha256),
      float_ops: 0,
      dense_matmul_ops: 0,
      n_layers: resp.status.n_layers,
      d_model: resp.status.d_model,
      vocab: resp.status.vocab,
      max_seq: resp.status.max_seq,
      cacheBackend: resp.cacheBackend,
      recoveredPieces: resp.recoveredPieces || [],
    };
    return {
      modelId: toFullModelId(entry.id),
      capabilities: loadedCapabilities,
      sha256: loadedCapabilities.artifact_sha256,
      bytes: entry.size_bytes,
      cacheBackend: resp.cacheBackend,
      recoveredPieces: resp.recoveredPieces || [],
      cached: Boolean(resp.reusedMount || resp.cacheBackend === 'opfs' || resp.cacheBackend === 'indexeddb'),
    };
  } catch (err) {
    notifyProgress({
      phase: 'error',
      error: String(err?.message || err),
    });
    throw err;
  }
}

export async function ensureModelLoaded(modelId, { signal } = {}) {
  const shortId = normalizeModelId(modelId);
  if (loadedModelId === shortId && loadedCapabilities) {
    return {
      modelId: toFullModelId(shortId),
      capabilities: loadedCapabilities,
      sha256: loadedCapabilities.artifact_sha256,
      cached: true,
    };
  }
  return serializeOp(async () => {
    if (loadedModelId === shortId && loadedCapabilities) {
      return {
        modelId: toFullModelId(shortId),
        capabilities: loadedCapabilities,
        sha256: loadedCapabilities.artifact_sha256,
        cached: true,
      };
    }
    if (shortId === 'local-upload') {
      if (!localUploadRecord) {
        throw new Error('No local .uorlut / .d11 artifact has been uploaded yet.');
      }
      const bytes = await cacheReadBytes(`${localUploadRecord.sha256}.bin`);
      if (!bytes) {
        throw new Error('Cached local .uorlut bytes were evicted; please upload the file again.');
      }
      return loadIntoWorker({
        modelId: 'local-upload',
        bytes,
        expectedSha: localUploadRecord.sha256,
        source: 'file',
        sourceLabel: localUploadRecord.fileName,
        fileName: localUploadRecord.fileName,
        tokenizerArtifact: localUploadRecord.tokenizerArtifact,
      });
    }

    const cat = await getCatalog();
    const entry = cat.models.find((m) => m.id === shortId);
    if (!entry) {
      throw new Error(`Unknown UOR-R4 model id: ${shortId}`);
    }
    if (entry.format === 'UORR4V13' || entry.kind === 'stitch-packs') {
      return loadV13Entry(entry, { signal });
    }
    const cacheKey = entry.sha256 ? `${entry.sha256}.bin` : null;
    const { bytes } = await fetchVerifiedBytes(entry.url, entry.sha256 || null, cacheKey, { signal });
    const actualSha = entry.sha256 || (await sha256Hex(bytes));
    const tokenizerArtifact = entry.tokenizer_url
      ? await fetchTokenizerJson(entry.tokenizer_url, entry.tokenizer_sha256 || null, actualSha, { signal })
      : null;

    return loadIntoWorker({
      modelId: entry.id,
      bytes,
      expectedSha: entry.sha256 || actualSha,
      source: entry.kind === 'bundled' ? 'bundled' : 'url',
      sourceLabel: entry.label || entry.id,
      fileName: entry.url,
      tokenizerArtifact,
    });
  });
}

export async function loadLocalArtifact({ fileName, bytes, tokenizerBytes }) {
  const raw = bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes || []);
  if (!raw.byteLength) {
    throw new Error('Uploaded model artifact is empty.');
  }
  const sha = await sha256Hex(raw);
  let tokenizerArtifact = null;
  if (tokenizerBytes && tokenizerBytes.byteLength > 0) {
    tokenizerArtifact = JSON.parse(new TextDecoder('utf-8').decode(tokenizerBytes));
  }
  return serializeOp(async () => {
    const inspect = await callWorker('inspect_model', { bytes: raw }, 'model_inspected');
    await cacheWriteBytes(`${sha}.bin`, raw);
    localUploadRecord = {
      fileName: String(fileName || 'uploaded.uorlut'),
      sha256: inspect.artifact_sha256,
      byteLength: inspect.artifact_bytes,
      tokenizerArtifact,
    };
    return loadIntoWorker({
      modelId: 'local-upload',
      bytes: raw,
      expectedSha: sha,
      source: 'file',
      sourceLabel: localUploadRecord.fileName,
      fileName: localUploadRecord.fileName,
      tokenizerArtifact,
    });
  });
}

export async function predictDiscrete({ modelId = 'uor-r4:m1-discrete-trunk-seed-1', prompt, tokenIds, signal } = {}) {
  await ensureModelLoaded(modelId, { signal });
  if (!loadedCapabilities?.can_predict) {
    throw new Error(`Model ${loadedModelId} does not serve discrete prefix prediction.`);
  }
  return serializeOp(async () => {
    const vocab = Number(loadedCapabilities.shape?.vocab || 0);
    const context = Number(loadedCapabilities.shape?.context || 0);
    const tokArtifact =
      loadedTokenizerArtifact && validateTokenizerArtifact(loadedTokenizerArtifact, loadedCapabilities.artifact_sha256, vocab)
        ? loadedTokenizerArtifact
        : null;
    let ids = Array.isArray(tokenIds) && tokenIds.length
      ? tokenIds.map(Number)
      : encodeTextWithArtifact(String(prompt || '').trim(), vocab, tokArtifact);
    if (!ids.length) {
      throw new Error('Non-empty prompt or tokenIds array required for discrete prediction.');
    }
    if (context > 0 && ids.length > context) {
      ids = ids.slice(-context);
    }
    return callWorker('predict_discrete', { tokenIds: ids }, 'discrete_prediction_result', { signal });
  });
}

export async function generate({
  modelId = `uor-r4:${DEFAULT_MODEL_SHORT}`,
  chatId = 'default',
  prompt,
  systemPrompt = '',
  messages,
  maxNewTokens = 384,
  stopSequences = ['</tool_call>'],
  sliceTokens = 2,
  prefillSliceTokens = 16,
  signal,
  onToken,
} = {}) {
  await ensureModelLoaded(modelId, { signal });
  if (loadedCapabilities?.format === 'UORR4V13') {
    return serializeOp(async () => {
      const result = await callV13Worker(
        'generate_v13',
        {
          systemPrompt,
          messages,
          prompt,
          maxNewTokens,
          stopSequences,
          sliceTokens,
          prefillSliceTokens,
        },
        'v13_complete',
        { onToken, signal },
      );
      return {
        fullText: result.fullText,
        tokenCount: result.tokensEmitted,
        stopReason: result.stopReason,
        float_ops: result.float_ops ?? 0,
        dense_matmul_ops: result.dense_matmul_ops ?? 0,
        capabilities: loadedCapabilities,
        weightsMatched:
          loadedCapabilities?.expected_sha256 == null || loadedCapabilities?.sha256_verified === true,
      };
    });
  }
  if (loadedCapabilities?.can_predict) {
    const pred = await predictDiscrete({ modelId, prompt, signal });
    const topFormatted = (pred.decoded || [])
      .map((item, idx) => `${idx + 1}. token ${item.id} (${JSON.stringify(item.text)})`)
      .join('\n');
    const summary = [
      `M1 discrete next-token prediction (\`${loadedModelId}\`, sha256 \`${pred.artifact_sha256.slice(0, 12)}…\`, ${pred.durationMs.toFixed(1)} ms):`,
      `Predicted next token ID: \`${pred.prediction}\` | Admitted cells: [${(pred.admitted_cells || []).join(', ')}]`,
      topFormatted ? `Top-8 candidates:\n${topFormatted}` : '',
    ]
      .filter(Boolean)
      .join('\n\n');
    onToken?.({ chunk: summary, fullText: summary, tokenCount: 1 });
    return {
      fullText: summary,
      tokenIds: [pred.prediction],
      tokenCount: 1,
      capabilities: loadedCapabilities,
      discretePrediction: pred,
      weightsMatched: true,
    };
  }
  return serializeOp(async () => {
    const result = await callWorker(
      'generate',
      {
        conversationId: String(chatId),
        prompt,
        messages,
        options: { max_new_tokens: maxNewTokens },
      },
      'generate_complete',
      { onToken, signal },
    );
    return {
      ...result,
      capabilities: loadedCapabilities,
      weightsMatched:
        loadedCapabilities?.expected_sha256 == null || loadedCapabilities?.sha256_verified === true,
    };
  });
}

export async function testStitchRecover({ mirrorBaseUrl = '', groupIndex = 0, signal } = {}) {
  return callV13Worker(
    'test_stitch_recover',
    {
      manifestUrl: FALLBACK_V13_MODEL.manifest_url,
      stitchManifestUrl: FALLBACK_V13_MODEL.stitch_manifest_url,
      manifestSha256: FALLBACK_V13_MODEL.manifest_sha256,
      stitchManifestSha256: FALLBACK_V13_MODEL.stitch_manifest_sha256,
      mirrorBaseUrl: mirrorBaseUrl ? sanitizeMirrorBase(mirrorBaseUrl) : getV13MirrorBase(),
      groupIndex,
    },
    'stitch_recover_result',
    { signal },
  );
}

export async function resetConversation(conversationId) {
  if (!worker || !loadedCapabilities || loadedCapabilities.can_predict || loadedCapabilities.format === 'UORR4V13') {
    return;
  }
  return serializeOp(() =>
    callWorker('reset_session', { conversationId: String(conversationId) }, 'session_reset'),
  );
}
