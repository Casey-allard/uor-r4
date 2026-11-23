// Real in-browser UORR4V13 pure-integer Qwen3-0.6B Foundry Web Worker.
// Downloads and verifies Run 5 Stitch 2-of-3 packs with OPFS/IndexedDB caching,
// self-heals damaged shards via `v13_stitch_recover_pair`, mounts into
// `uor_foundry_v13_wasm`, and streams non-blocking chunked turns.

const BASE = new URL('./', import.meta.url);
const OPFS_ROOT_DIR = 'uor-r4-v13-packs';
const IDB_NAME = 'uor-r4-v13-cache';
const IDB_STORE = 'blobs';

let wasmModulePromise = null;
let mountedContainerSha = null;
let mountedModelId = null;
let activeTurn = null; // { id, turnId: BigInt, cancelled: boolean }
let activeLoad = null; // { id, controller: AbortController }

async function getWasm() {
  if (!wasmModulePromise) {
    wasmModulePromise = (async () => {
      const mod = await import(new URL('./v13/uor_foundry_v13_wasm.js', BASE).href);
      if (self.__UOR_V13_WASM_BYTES__) {
        await mod.default({ module_or_path: self.__UOR_V13_WASM_BYTES__ });
      } else {
        const wasmUrl = new URL('./v13/uor_foundry_v13_wasm_bg.wasm', BASE);
        await mod.default({ module_or_path: wasmUrl });
      }
      return mod;
    })().catch((err) => {
      wasmModulePromise = null;
      throw err;
    });
  }
  return wasmModulePromise;
}

async function sha256Hex(bytes) {
  const u8 = bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes);
  const digest = await crypto.subtle.digest('SHA-256', u8);
  return Array.from(new Uint8Array(digest), (b) => b.toString(16).padStart(2, '0')).join('');
}

async function openIdb() {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(IDB_NAME, 1);
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

async function cacheRead(scopeKey, fileKey) {
  const fullKey = `${scopeKey}/${fileKey}`;
  try {
    if (self.navigator?.storage?.getDirectory) {
      const root = await self.navigator.storage.getDirectory();
      const baseDir = await root.getDirectoryHandle(OPFS_ROOT_DIR, { create: false });
      const scopeDir = await baseDir.getDirectoryHandle(scopeKey, { create: false });
      const fh = await scopeDir.getFileHandle(fileKey, { create: false });
      const f = await fh.getFile();
      return { bytes: new Uint8Array(await f.arrayBuffer()), backend: 'opfs' };
    }
  } catch {
    // Fall back to IndexedDB.
  }
  try {
    const db = await openIdb();
    const val = await new Promise((resolve, reject) => {
      const tx = db.transaction(IDB_STORE, 'readonly');
      const q = tx.objectStore(IDB_STORE).get(fullKey);
      q.onsuccess = () => resolve(q.result || null);
      q.onerror = () => reject(q.error);
    });
    if (val) {
      return {
        bytes: val instanceof Uint8Array ? val : new Uint8Array(val),
        backend: 'indexeddb',
      };
    }
  } catch {
    // Ignore IDB read failure.
  }
  return null;
}

async function cacheWrite(scopeKey, fileKey, bytes) {
  const u8 = bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes);
  const fullKey = `${scopeKey}/${fileKey}`;
  try {
    if (self.navigator?.storage?.getDirectory) {
      const root = await self.navigator.storage.getDirectory();
      const baseDir = await root.getDirectoryHandle(OPFS_ROOT_DIR, { create: true });
      const scopeDir = await baseDir.getDirectoryHandle(scopeKey, { create: true });
      const fh = await scopeDir.getFileHandle(fileKey, { create: true });
      const w = await fh.createWritable();
      await w.write(u8);
      await w.close();
      return 'opfs';
    }
  } catch {
    // Fall back to IndexedDB.
  }
  try {
    const db = await openIdb();
    await new Promise((resolve, reject) => {
      const tx = db.transaction(IDB_STORE, 'readwrite');
      tx.objectStore(IDB_STORE).put(u8, fullKey);
      tx.oncomplete = () => resolve();
      tx.onerror = () => reject(tx.error);
    });
    return 'indexeddb';
  } catch {
    return 'memory';
  }
}

async function fetchBytes(url, expectedSha = null, signal = undefined) {
  const resp = await fetch(url, { signal, credentials: 'omit' });
  if (!resp.ok) {
    throw new Error(`HTTP ${resp.status} fetching ${url}`);
  }
  const bytes = new Uint8Array(await resp.arrayBuffer());
  const actualSha = await sha256Hex(bytes);
  if (expectedSha && actualSha !== expectedSha) {
    const err = new Error(`SHA-256 mismatch for ${url}: expected ${expectedSha}, got ${actualSha}`);
    err.actualSha = actualSha;
    err.bytes = bytes;
    throw err;
  }
  return { bytes, sha256: actualSha };
}

function resolveBase(manifestUrl, mirrorBaseUrl) {
  const raw = String(mirrorBaseUrl || '').trim();
  if (raw) {
    const parsed = new URL(raw, self.location?.href || manifestUrl);
    const sameOrigin = Boolean(self.location?.origin && parsed.origin === self.location.origin);
    const isLoopback =
      parsed.hostname === '127.0.0.1' ||
      parsed.hostname === 'localhost' ||
      parsed.hostname === '[::1]';
    if (!['http:', 'https:'].includes(parsed.protocol) || (!sameOrigin && !isLoopback)) {
      throw new Error(`Disallowed mirrorBaseUrl origin: ${parsed.origin}`);
    }
    const href = parsed.href;
    return href.endsWith('/') ? href : `${href}/`;
  }
  return new URL('./', manifestUrl).href;
}

async function mapLimit(items, limit, fn) {
  let cursor = 0;
  const workers = Array.from({ length: Math.min(limit, items.length) }, async () => {
    while (cursor < items.length) {
      const idx = cursor;
      cursor += 1;
      await fn(items[idx], idx);
    }
  });
  await Promise.all(workers);
}

async function handleLoadV13(id, payload) {
  const wasm = await getWasm();
  const modelId = String(payload.modelId || 'uor-r4:qwen3-0.6b-foundry');
  const expectedContainerSha = String(payload.containerSha256 || '').trim().toLowerCase();

  if (
    mountedContainerSha &&
    expectedContainerSha &&
    mountedContainerSha === expectedContainerSha &&
    mountedModelId === modelId
  ) {
    const status = JSON.parse(wasm.v13_status());
    if (status.mounted) {
      self.postMessage({
        id,
        action: 'v13_loaded',
        modelId,
        status,
        cacheBackend: payload.lastBackend || 'memory',
        recoveredPieces: [],
        reusedMount: true,
      });
      return;
    }
  }

  const baseHref = resolveBase(payload.manifestUrl, payload.mirrorBaseUrl);
  const manifestUrl = payload.mirrorBaseUrl ? new URL('manifest.json', baseHref).href : payload.manifestUrl;
  const stitchUrl = payload.mirrorBaseUrl
    ? new URL('manifest.stitch.json', baseHref).href
    : payload.stitchManifestUrl;
  const loadCtrl = new AbortController();
  activeLoad = { id, controller: loadCtrl };

  try {
    self.postMessage({
      id,
      action: 'v13_progress',
      phase: 'manifest',
      percent: 1,
      packsDone: 0,
      packsTotal: 0,
      piecesDone: 0,
      piecesTotal: 0,
      bytesLoaded: 0,
      bytesTotal: Number(payload.sizeBytes || 0),
      recoveredPieces: [],
    });

    const manifestRes = await fetchBytes(
      manifestUrl,
      payload.manifestSha256 ? String(payload.manifestSha256).toLowerCase() : null,
      loadCtrl.signal,
    );
    const manifestText = new TextDecoder('utf-8').decode(manifestRes.bytes);
    const manifest = JSON.parse(manifestText);
    if (manifest.format !== 'UORR4V13' || !Array.isArray(manifest.packs) || !Array.isArray(manifest.pieces)) {
      throw new Error('Invalid UORR4V13 pack manifest');
    }
    if (expectedContainerSha && manifest.container_sha256 !== expectedContainerSha) {
      throw new Error(`Container SHA-256 mismatch: expected ${expectedContainerSha}, got ${manifest.container_sha256}`);
    }

    let stitchManifest = null;
    if (stitchUrl) {
      const stitchRes = await fetchBytes(
        stitchUrl,
        payload.stitchManifestSha256 ? String(payload.stitchManifestSha256).toLowerCase() : null,
        loadCtrl.signal,
      );
      const stitchText = new TextDecoder('utf-8').decode(stitchRes.bytes);
      const vres = JSON.parse(wasm.v13_stitch_verify_manifest(stitchText));
      if (!vres.ok) {
        throw new Error(`Stitch manifest verification failed: ${vres.message || vres.error_code}`);
      }
      stitchManifest = JSON.parse(stitchText);
    }

    const containerSha = manifest.container_sha256;
    const packsTotal = manifest.packs.length;
    const piecesTotal = manifest.piece_count;
    const bytesTotal = manifest.packs.reduce((n, p) => n + p.byte_length, 0);
    const pieceBuffers = new Array(piecesTotal).fill(null);
    const failedPieces = new Set();
    const recoveredPieces = [];

    let packsDone = 0;
    let piecesDone = 0;
    let bytesLoaded = 0;
    let cacheBackend = 'network';
    let allFromCache = true;

    const emitProgress = (phase) => {
      if (loadCtrl.signal.aborted) return;
      const pct = bytesTotal > 0 ? Math.min(95, Math.round((bytesLoaded / bytesTotal) * 90) + 2) : 50;
      self.postMessage({
        id,
        action: 'v13_progress',
        phase,
        percent: phase === 'mounting' ? 96 : pct,
        packsDone,
        packsTotal,
        piecesDone,
        piecesTotal,
        bytesLoaded,
        bytesTotal,
        cached: allFromCache,
        cacheBackend,
        recoveredPieces: [...recoveredPieces],
      });
    };

    const throwIfCancelled = () => {
      if (loadCtrl.signal.aborted) {
        throw Object.assign(new Error('V13 download cancelled'), { name: 'AbortError' });
      }
    };

    await mapLimit(manifest.packs, 2, async (pack) => {
      throwIfCancelled();
      let packBytes = null;
      const hit = await cacheRead(containerSha, pack.file);
      if (hit && hit.bytes.byteLength === pack.byte_length && (await sha256Hex(hit.bytes)) === pack.sha256) {
        packBytes = hit.bytes;
        cacheBackend = hit.backend;
      } else {
        throwIfCancelled();
        allFromCache = false;
        const url = new URL(pack.file, baseHref).href;
        try {
          const fetched = await fetchBytes(url, pack.sha256, loadCtrl.signal);
          packBytes = fetched.bytes;
          cacheBackend = await cacheWrite(containerSha, pack.file, packBytes);
        } catch (err) {
          if (err?.name === 'AbortError' || loadCtrl.signal.aborted) throw err;
          // Attempt piece-level inspection if pack bytes were returned with SHA mismatch.
          if (err?.bytes && err.bytes.byteLength === pack.byte_length) {
            packBytes = err.bytes;
          }
        }
      }

    for (let p = pack.piece_start; p < pack.piece_start + pack.piece_count; p += 1) {
      const pieceMeta = manifest.pieces[p];
      if (packBytes) {
        const rel = pieceMeta.byte_offset - pack.byte_offset;
        const slice = new Uint8Array(packBytes.subarray(rel, rel + pieceMeta.byte_length));
        if (slice.byteLength === pieceMeta.byte_length && (await sha256Hex(slice)) === pieceMeta.sha256) {
          pieceBuffers[p] = slice;
          piecesDone += 1;
        } else {
          failedPieces.add(p);
        }
      } else {
        failedPieces.add(p);
      }
    }

    packsDone += 1;
    bytesLoaded += pack.byte_length;
    emitProgress('hydrating');
  });

  // Self-heal any missing or corrupted pieces via Stitch 2-of-3 XOR parity recovery.
  if (failedPieces.size > 0) {
    if (!stitchManifest || !Array.isArray(stitchManifest.groups)) {
      throw new Error(`Shard download failed for pieces [${[...failedPieces].slice(0, 6).join(', ')}] and no Stitch parity manifest is available`);
    }
      emitProgress('stitch-healing');
      for (const p of [...failedPieces].sort((a, b) => a - b)) {
        throwIfCancelled();
        const group = stitchManifest.groups.find((g) => g.left.index === p || g.right?.index === p);
        if (!group) {
          throw new Error(`No Stitch parity group covers piece ${p}`);
        }
        const recoverLeft = group.left.index === p;
        const partnerEntry = recoverLeft ? group.right : group.left;
        if (!partnerEntry) {
          throw new Error(`Piece ${p} has no partner shard in group ${group.group_index}`);
        }
        let partnerBytes = pieceBuffers[partnerEntry.index];
        if (!partnerBytes) {
          const fetchedPartner = await fetchBytes(new URL(partnerEntry.file, baseHref).href, partnerEntry.sha256, loadCtrl.signal);
          partnerBytes = fetchedPartner.bytes;
        }
        const fetchedParity = await fetchBytes(new URL(group.parity.file, baseHref).href, group.parity.sha256, loadCtrl.signal);
      const recovered = wasm.v13_stitch_recover_pair(
        JSON.stringify(group),
        recoverLeft,
        !recoverLeft,
        recoverLeft ? new Uint8Array(0) : partnerBytes,
        recoverLeft,
        recoverLeft ? partnerBytes : new Uint8Array(0),
        true,
        fetchedParity.bytes,
        manifest.pieces[p].sha256,
      );
      pieceBuffers[p] = new Uint8Array(recovered);
      recoveredPieces.push(p);
      piecesDone += 1;
    }
  }

    throwIfCancelled();
    emitProgress('mounting');
    const beginMount = JSON.parse(wasm.v13_begin_mount(modelId, piecesTotal, containerSha, true));
    if (!beginMount.ok) {
      throw new Error(beginMount.message || 'v13_begin_mount failed');
    }
    for (let p = 0; p < piecesTotal; p += 1) {
      const buf = pieceBuffers[p];
      pieceBuffers[p] = null; // Free JS copy incrementally while copying into WASM heap.
      if (!buf) {
        throw new Error(`Missing verified piece ${p} during V13 mount`);
      }
      const pushRes = JSON.parse(wasm.v13_push_piece(buf));
      if (!pushRes.ok) {
        throw new Error(pushRes.message || `v13_push_piece(${p}) failed`);
      }
    }
    const finalized = JSON.parse(wasm.v13_finalize_mount());
    if (!finalized.ok || finalized.container_sha256 !== containerSha) {
      throw new Error(finalized.message || 'v13_finalize_mount verification failed');
    }

    mountedContainerSha = containerSha;
    mountedModelId = modelId;
    self.postMessage({
      id,
      action: 'v13_progress',
      phase: 'ready',
      percent: 100,
      packsDone,
      packsTotal,
      piecesDone,
      piecesTotal,
      bytesLoaded,
      bytesTotal,
      cached: allFromCache,
      cacheBackend,
      recoveredPieces,
    });
    self.postMessage({
      id,
      action: 'v13_loaded',
      modelId,
      status: finalized,
      cacheBackend,
      recoveredPieces,
    });
  } finally {
    if (activeLoad && activeLoad.id === id) {
      activeLoad = null;
    }
  }
}

async function handleGenerateV13(id, payload) {
  const wasm = await getWasm();
  const status = JSON.parse(wasm.v13_status());
  if (!status.mounted) {
    throw new Error('UORR4V13 model is not mounted yet');
  }
  const begin = JSON.parse(
    wasm.v13_begin_turn(
      JSON.stringify({
        system_prompt: payload.systemPrompt || '',
        messages: Array.isArray(payload.messages) ? payload.messages : [],
        prompt: String(payload.prompt || ''),
        max_new_tokens: payload.maxNewTokens || 384,
        prefill_slice_tokens: payload.prefillSliceTokens || 16,
        slice_tokens: payload.sliceTokens || 2,
        stop_sequences: Array.isArray(payload.stopSequences) && payload.stopSequences.length
          ? payload.stopSequences
          : ['</tool_call>'],
      }),
    ),
  );
  if (!begin.ok) {
    throw new Error(begin.message || 'v13_begin_turn failed');
  }
  const turnId = BigInt(begin.turn_id);
  activeTurn = { id, turnId, cancelled: false };

  try {
    let last;
    for (;;) {
      if (activeTurn?.cancelled) {
        const cancelRes = JSON.parse(wasm.v13_cancel_turn(turnId));
        self.postMessage({
          id,
          action: 'v13_error',
          error: 'Generation cancelled',
          cancelled: true,
          cancelRes,
        });
        return;
      }
      last = JSON.parse(wasm.v13_step_turn(turnId));
      if (!last.ok) {
        throw new Error(last.message || 'v13_step_turn failed');
      }
      if (last.delta) {
        self.postMessage({
          id,
          action: 'v13_token',
          chunk: last.delta,
          fullText: last.full_text || '',
          tokensEmitted: last.tokens_emitted || 0,
        });
      }
      if (last.status === 'done') {
        self.postMessage({
          id,
          action: 'v13_complete',
          fullText: last.full_text || '',
          tokensEmitted: last.tokens_emitted || 0,
          stopReason: last.stop_reason || null,
          float_ops: last.float_ops ?? 0,
          dense_matmul_ops: last.dense_matmul_ops ?? 0,
        });
        return;
      }
      if (last.status === 'cancelled') {
        self.postMessage({
          id,
          action: 'v13_error',
          error: 'Generation cancelled',
          cancelled: true,
        });
        return;
      }
      await new Promise((r) => setTimeout(r, 0));
    }
  } finally {
    if (activeTurn && activeTurn.id === id) {
      activeTurn = null;
    }
  }
}

async function handleTestStitchRecover(id, payload) {
  const wasm = await getWasm();
  const baseHref = resolveBase(payload.manifestUrl, payload.mirrorBaseUrl);
  const stitchUrl = payload.mirrorBaseUrl
    ? new URL('manifest.stitch.json', baseHref).href
    : payload.stitchManifestUrl || new URL('manifest.stitch.json', baseHref).href;
  const manifestUrl = payload.mirrorBaseUrl
    ? new URL('manifest.json', baseHref).href
    : payload.manifestUrl;
  const expectedStitchSha = payload.stitchManifestSha256 ? String(payload.stitchManifestSha256).toLowerCase() : null;
  const expectedManifestSha = payload.manifestSha256 ? String(payload.manifestSha256).toLowerCase() : null;
  const stitchBytes = (await fetchBytes(stitchUrl, expectedStitchSha)).bytes;
  const stitchText = new TextDecoder('utf-8').decode(stitchBytes);
  const verify = JSON.parse(wasm.v13_stitch_verify_manifest(stitchText));
  if (!verify.ok) {
    throw new Error(verify.message || 'v13_stitch_verify_manifest failed');
  }
  const stitch = JSON.parse(stitchText);
  const groupIdx = Number(payload.groupIndex || 0);
  const group = stitch.groups[groupIdx];
  if (!group) {
    throw new Error(`Stitch group ${groupIdx} not found`);
  }
  if (!group.right) {
    throw new Error(`Stitch group ${groupIdx} is an unpaired singleton`);
  }
  const manifest = JSON.parse(new TextDecoder('utf-8').decode((await fetchBytes(manifestUrl, expectedManifestSha)).bytes));
  const findPackForPiece = (idx) =>
    manifest.packs.find((p) => idx >= p.piece_start && idx < p.piece_start + p.piece_count);
  const rightPack = findPackForPiece(group.right.index);
  const leftPack = findPackForPiece(group.left.index);
  if (!rightPack || !leftPack) {
    throw new Error(`Pack not found for stitch group ${groupIdx}`);
  }
  const rightPackBytes = (await fetchBytes(new URL(rightPack.file, baseHref).href, rightPack.sha256)).bytes;
  const rightMeta = manifest.pieces[group.right.index];
  const rightSlice = rightPackBytes.subarray(
    rightMeta.byte_offset - rightPack.byte_offset,
    rightMeta.byte_offset - rightPack.byte_offset + rightMeta.byte_length,
  );
  const leftPackBytes =
    leftPack.file === rightPack.file
      ? rightPackBytes
      : (await fetchBytes(new URL(leftPack.file, baseHref).href, leftPack.sha256)).bytes;
  const leftMeta = manifest.pieces[group.left.index];
  const origLeft = leftPackBytes.subarray(
    leftMeta.byte_offset - leftPack.byte_offset,
    leftMeta.byte_offset - leftPack.byte_offset + leftMeta.byte_length,
  );
  const parityBytes = (await fetchBytes(new URL(group.parity.file, baseHref).href, group.parity.sha256)).bytes;
  const recovered = wasm.v13_stitch_recover_pair(
    JSON.stringify(group),
    true,
    false,
    new Uint8Array(0),
    true,
    rightSlice,
    true,
    parityBytes,
    group.left.sha256,
  );
  const recoveredSha = await sha256Hex(recovered);
  self.postMessage({
    id,
    action: 'stitch_recover_result',
    ok: recoveredSha === group.left.sha256 && recovered.length === origLeft.length,
    groupIndex: groupIdx,
    pieceIndex: group.left.index,
    expectedSha256: group.left.sha256,
    recoveredSha256: recoveredSha,
    bytes: recovered.length,
    groupCount: verify.group_count,
  });
}

self.onmessage = async ({ data }) => {
  if (!data || typeof data !== 'object') return;
  const { id, action, payload } = data;
  try {
    if (action === 'cancel_v13') {
      if (activeTurn && (!id || activeTurn.id === id)) {
        activeTurn.cancelled = true;
      }
      if (activeLoad && (!id || activeLoad.id === id)) {
        activeLoad.controller.abort(Object.assign(new Error('V13 download cancelled'), { name: 'AbortError' }));
      }
      self.postMessage({ id, action: 'v13_cancel_ack', ok: true });
      return;
    }
    if (action === 'status_v13') {
      const wasm = await getWasm();
      self.postMessage({ id, action: 'v13_status_result', status: JSON.parse(wasm.v13_status()) });
      return;
    }
    if (action === 'load_v13') {
      await handleLoadV13(id, payload || {});
      return;
    }
    if (action === 'generate_v13') {
      await handleGenerateV13(id, payload || {});
      return;
    }
    if (action === 'test_stitch_recover') {
      await handleTestStitchRecover(id, payload || {});
      return;
    }
    throw new Error(`Unknown V13 worker action: ${action}`);
  } catch (err) {
    self.postMessage({
      id,
      action: 'v13_error',
      error: String(err?.message || err),
    });
  }
};
