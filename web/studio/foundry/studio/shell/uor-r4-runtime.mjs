// Real in-browser UOR-R4 runtime bridge:
// - Built-in model: `uor-r4:qwen3-0.6b-foundry` (`UORR4V13` pure-integer Qwen3-0.6B Foundry via `./uor-v13-worker.mjs`)
// - Zero M1 dummy/seed models and zero weight-file upload controls.

const OPFS_V13_DIR = 'uor-r4-v13-packs';
const IDB_STORE = 'blobs';
const IDB_V13_NAME = 'uor-r4-v13-cache';
const DEFAULT_MODEL_SHORT = 'qwen3-0.6b-foundry';

const BASE = new URL('./', import.meta.url);

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

let v13Worker = null;
let catalogCache = null;
let loadedModelId = null;
let loadedCapabilities = null;
let opfsStatus = 'untested';
let pendingOp = Promise.resolve();
let nextMsgId = 1;
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

async function openFallbackIdb(name = IDB_V13_NAME) {
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
    if (entry.id === 'synthetic-seed-5' || entry.id === 'm1-discrete-trunk-seed-1') continue;
    if (entry.format !== 'UORR4V13' && entry.kind !== 'stitch-packs') continue;
    const cached = await isV13Cached(entry.container_sha256);
    models.push({
      ...entry,
      apiName: toFullModelId(entry.id),
      cached,
      loaded: loadedModelId === entry.id,
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
    const cat = await getCatalog();
    const entry = cat.models.find((m) => m.id === shortId);
    if (!entry) {
      throw new Error(`Unknown UOR-R4 model id: ${shortId}`);
    }
    return loadV13Entry(entry, { signal });
  });
}

export async function generate({
  modelId = `uor-r4:${DEFAULT_MODEL_SHORT}`,
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
