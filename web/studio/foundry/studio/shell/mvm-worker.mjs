// Dedicated Module Web Worker hosting WasmMvmSandbox from crates/uor-r4-mvm (wasm feature).

import initMvm, { WasmMvmSandbox } from './pkg-mvm/uor_r4_mvm.js';

let initPromise = null;
let sandbox = null;

async function ensureSandbox(initialSnapshot) {
  if (!initPromise) {
    initPromise = initMvm();
  }
  await initPromise;
  if (!sandbox) {
    sandbox = new WasmMvmSandbox();
    if (initialSnapshot && initialSnapshot.byteLength > 0) {
      sandbox.import_cow_snapshot(
        initialSnapshot instanceof Uint8Array ? initialSnapshot : new Uint8Array(initialSnapshot),
      );
    }
  }
  return sandbox;
}

self.onmessage = async ({ data }) => {
  const { id, action, payload = {} } = data || {};
  try {
    if (action === 'init') {
      const box = await ensureSandbox(payload.snapshot);
      self.postMessage({
        id,
        ok: true,
        identity: JSON.parse(box.guest_identity_json()),
        cwd: box.cwd(),
      });
      return;
    }

    const box = await ensureSandbox(null);

    if (action === 'exec') {
      const result = JSON.parse(box.exec_json(String(payload.cmd || ''), String(payload.cwd || box.cwd())));
      const snapshot = box.export_cow_snapshot();
      const identity = JSON.parse(box.guest_identity_json());
      const copy = new Uint8Array(snapshot);
      self.postMessage(
        {
          id,
          ok: true,
          result,
          identity,
          snapshot: copy,
        },
        [copy.buffer],
      );
      return;
    }

    if (action === 'read_file') {
      const bytes = box.read_file(String(payload.path || ''));
      self.postMessage({ id, ok: true, bytes: bytes || null });
      return;
    }

    if (action === 'write_file') {
      const bytes = payload.bytes instanceof Uint8Array ? payload.bytes : new Uint8Array(payload.bytes || []);
      box.write_file(String(payload.path || ''), bytes);
      const snapshot = box.export_cow_snapshot();
      const copy = new Uint8Array(snapshot);
      self.postMessage({ id, ok: true, snapshot: copy }, [copy.buffer]);
      return;
    }

    if (action === 'remove_file') {
      box.remove_file(String(payload.path || ''));
      const snapshot = box.export_cow_snapshot();
      const copy = new Uint8Array(snapshot);
      self.postMessage({ id, ok: true, snapshot: copy }, [copy.buffer]);
      return;
    }

    if (action === 'list_files') {
      const files = JSON.parse(box.list_files_json(String(payload.prefix || '/')));
      self.postMessage({ id, ok: true, files });
      return;
    }

    if (action === 'export_snapshot') {
      const snapshot = box.export_cow_snapshot();
      const copy = new Uint8Array(snapshot);
      self.postMessage({ id, ok: true, snapshot: copy }, [copy.buffer]);
      return;
    }

    if (action === 'import_snapshot') {
      const snap = payload.snapshot instanceof Uint8Array ? payload.snapshot : new Uint8Array(payload.snapshot || []);
      box.import_cow_snapshot(snap);
      self.postMessage({
        id,
        ok: true,
        identity: JSON.parse(box.guest_identity_json()),
        cwd: box.cwd(),
      });
      return;
    }

    throw new Error(`Unknown MVM worker action: ${action}`);
  } catch (error) {
    self.postMessage({
      id,
      ok: false,
      error: String(error?.message || error),
    });
  }
};
