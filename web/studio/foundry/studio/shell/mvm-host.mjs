// Host-side manager for per-app RV64IMAC MVM Micro-Linux Web Workers, OPFS CoW persistence,
// and Foundry TerminalPanel IPC channels (`terminal:open/close/kill/write/resize/serialize`).

const BASE = new URL('./', import.meta.url);
const OPFS_MVM_DIR = 'uor-mvm-snapshots';
const IDB_NAME = 'uor-r4-mvm-cow';
const IDB_STORE = 'snapshots';

async function openSnapshotIdb() {
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

async function readCowSnapshot(appKey) {
  const key = `app-${appKey}.bin`;
  try {
    if (navigator.storage?.getDirectory) {
      const root = await navigator.storage.getDirectory();
      const dir = await root.getDirectoryHandle(OPFS_MVM_DIR, { create: true });
      const fh = await dir.getFileHandle(key);
      const file = await fh.getFile();
      return new Uint8Array(await file.arrayBuffer());
    }
  } catch {
    // Fall through to IndexedDB.
  }
  try {
    const db = await openSnapshotIdb();
    const val = await new Promise((resolve, reject) => {
      const tx = db.transaction(IDB_STORE, 'readonly');
      const req = tx.objectStore(IDB_STORE).get(key);
      req.onsuccess = () => resolve(req.result || null);
      req.onerror = () => reject(req.error);
    });
    return val ? new Uint8Array(val) : null;
  } catch {
    return null;
  }
}

async function writeCowSnapshot(appKey, bytes) {
  const key = `app-${appKey}.bin`;
  const u8 = bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes);
  try {
    if (navigator.storage?.getDirectory) {
      const root = await navigator.storage.getDirectory();
      const dir = await root.getDirectoryHandle(OPFS_MVM_DIR, { create: true });
      const fh = await dir.getFileHandle(key, { create: true });
      const writable = await fh.createWritable();
      await writable.write(u8);
      await writable.close();
      return 'opfs';
    }
  } catch {
    // Fall through to IndexedDB.
  }
  try {
    const db = await openSnapshotIdb();
    await new Promise((resolve, reject) => {
      const tx = db.transaction(IDB_STORE, 'readwrite');
      tx.objectStore(IDB_STORE).put(u8, key);
      tx.oncomplete = () => resolve();
      tx.onerror = () => reject(tx.error);
    });
    return 'indexeddb';
  } catch {
    return 'memory';
  }
}

function toCrLf(text) {
  return String(text || '').replace(/\r?\n/g, '\r\n');
}

export function createMvmHost({ emit, getAppName }) {
  const sandboxes = new Map(); // appKey -> { worker, queue, nextId, pending, identity }
  const sessions = new Map(); // sessionId -> sessionState
  const appSessions = new Map(); // appKey -> sessionId

  function appKeyOf(appId) {
    return appId == null ? 'global' : String(appId);
  }

  async function ensureSandbox(appId) {
    const appKey = appKeyOf(appId);
    if (sandboxes.has(appKey)) {
      return sandboxes.get(appKey);
    }
    const worker = new Worker(new URL('./mvm-worker.mjs', BASE), { type: 'module' });
    const pending = new Map();
    const state = {
      appKey,
      worker,
      pending,
      nextId: 1,
      queue: Promise.resolve(),
      identity: null,
      readyPromise: null,
    };
    worker.addEventListener('message', ({ data }) => {
      if (!data || typeof data !== 'object') return;
      const entry = pending.get(data.id);
      if (!entry) return;
      pending.delete(data.id);
      if (data.ok) {
        entry.resolve(data);
      } else {
        entry.reject(new Error(data.error || 'MVM worker error'));
      }
    });
    worker.addEventListener('error', (err) => {
      for (const [id, entry] of pending.entries()) {
        pending.delete(id);
        entry.reject(new Error(err?.message || 'MVM worker failed'));
      }
    });

    const rpc = (action, payload) => {
      const run = () =>
        new Promise((resolve, reject) => {
          const id = `mvm-${state.nextId++}`;
          pending.set(id, { resolve, reject });
          worker.postMessage({ id, action, payload });
        });
      const queued = state.queue.then(run, run);
      state.queue = queued.catch(() => {});
      return queued;
    };

    state.rpc = rpc;
    state.readyPromise = (async () => {
      const saved = await readCowSnapshot(appKey);
      const initRes = await rpc('init', { snapshot: saved });
      state.identity = initRes.identity;
      return state;
    })();
    sandboxes.set(appKey, state);
    return state.readyPromise;
  }

  function promptText(cwd) {
    return `\u001b[32muor@rv64\u001b[0m:\u001b[34m${cwd || '/workspace'}\u001b[0m$ `;
  }

  function emitChunk(session, chunk) {
    if (!chunk) return;
    const startOffset = session.scrollback.length;
    session.scrollback += chunk;
    const endOffset = session.scrollback.length;
    emit(`terminal:data:${session.sessionId}`, {
      sessionId: session.sessionId,
      chunk,
      startOffset,
      endOffset,
    });
  }

  function redrawLine(session, nextLine) {
    const back = session.lineBuffer.length;
    session.lineBuffer = nextLine;
    if (back > 0) {
      emitChunk(session, `\r${promptText(session.cwd)}\u001b[K${nextLine}`);
    } else {
      emitChunk(session, nextLine);
    }
  }

  async function execCommand(appId, cmd, { echoToSession = true } = {}) {
    const trimmed = String(cmd || '').trim();
    const box = await ensureSandbox(appId);
    await box.readyPromise;
    const appKey = appKeyOf(appId);
    const activeSid = appSessions.get(appKey);
    const session = activeSid ? sessions.get(activeSid) : null;
    const cwd = session?.cwd || box.identity?.cwd || '/workspace';
    const res = await box.rpc('exec', { cmd: trimmed, cwd });
    box.identity = res.identity;
    if (res.snapshot) {
      await writeCowSnapshot(appKey, res.snapshot);
    }
    const execResult = res.result;
    if (session) {
      session.cwd = execResult.cwd || session.cwd;
      if (echoToSession && trimmed) {
        const prefix = session.lineBuffer ? '\r\n' : '';
        emitChunk(session, `${prefix}[mvm] $ ${trimmed}\r\n`);
        if (execResult.stdout) emitChunk(session, toCrLf(execResult.stdout));
        if (execResult.stderr) emitChunk(session, `\u001b[31m${toCrLf(execResult.stderr)}\u001b[0m`);
        if ((execResult.stdout && !execResult.stdout.endsWith('\n')) || (execResult.stderr && !execResult.stderr.endsWith('\n'))) {
          emitChunk(session, '\r\n');
        }
        session.lineBuffer = '';
        emitChunk(session, promptText(session.cwd));
      }
    }
    return execResult;
  }

  async function openSession({ appId = null, cols = 80, rows = 24 } = {}) {
    const appKey = appKeyOf(appId);
    const existingId = appSessions.get(appKey);
    if (existingId && sessions.has(existingId)) {
      const existing = sessions.get(existingId);
      if (!existing.exited) {
        return {
          sessionId: existing.sessionId,
          shell: existing.shell,
          cwd: existing.cwd,
          appName: existing.appName,
          scrollback: existing.scrollback,
          created: false,
        };
      }
      sessions.delete(existingId);
      appSessions.delete(appKey);
    }

    const box = await ensureSandbox(appId);
    await box.readyPromise;
    const sessionId = `mvm-${appKey}-${crypto.randomUUID().slice(0, 8)}`;
    const appName = (await getAppName?.(appId)) || 'UOR-R4 Workspace';
    const cwd = box.identity?.cwd || '/workspace';
    const guestCid = box.identity?.guest_cid || 'local';
    const banner =
      `UOR-R4 RV64IMAC Micro-Linux MVM (uor-r4-mvm WASM · guest ${guestCid})\r\n` +
      `Workspace persisted via CoW snapshot. Type "help", "uname -a", or ELF shell commands.\r\n` +
      promptText(cwd);

    const session = {
      sessionId,
      appId,
      appKey,
      shell: 'uor-r4-mvm (rv64imac)',
      cwd,
      appName,
      cols,
      rows,
      scrollback: banner,
      lineBuffer: '',
      history: [],
      historyIdx: -1,
      running: false,
      exited: false,
      created: true,
    };
    sessions.set(sessionId, session);
    appSessions.set(appKey, sessionId);
    return {
      sessionId: session.sessionId,
      shell: session.shell,
      cwd: session.cwd,
      appName: session.appName,
      scrollback: session.scrollback,
      created: true,
    };
  }

  async function runInteractiveLine(session, rawCmd) {
    const cmd = rawCmd.trim();
    emitChunk(session, '\r\n');
    if (cmd) {
      session.history.push(cmd);
      session.historyIdx = session.history.length;
    }
    if (cmd === 'clear') {
      emitChunk(session, '\u001b[2J\u001b[H');
      emitChunk(session, promptText(session.cwd));
      return;
    }
    if (!cmd) {
      emitChunk(session, promptText(session.cwd));
      return;
    }
    session.running = true;
    try {
      const box = await ensureSandbox(session.appId);
      await box.readyPromise;
      const res = await box.rpc('exec', { cmd, cwd: session.cwd });
      box.identity = res.identity;
      if (res.snapshot) {
        await writeCowSnapshot(session.appKey, res.snapshot);
      }
      const out = res.result;
      session.cwd = out.cwd || session.cwd;
      if (out.stdout) {
        emitChunk(session, toCrLf(out.stdout));
        if (!out.stdout.endsWith('\n')) emitChunk(session, '\r\n');
      }
      if (out.stderr) {
        emitChunk(session, `\u001b[31m${toCrLf(out.stderr)}\u001b[0m`);
        if (!out.stderr.endsWith('\n')) emitChunk(session, '\r\n');
      }
    } catch (error) {
      emitChunk(session, `\u001b[31m${toCrLf(String(error?.message || error))}\u001b[0m\r\n`);
    } finally {
      session.running = false;
      emitChunk(session, promptText(session.cwd));
    }
  }

  async function writeInput({ sessionId, data }) {
    const session = sessions.get(sessionId);
    if (!session || session.exited) {
      throw new Error(`Terminal session not active: ${sessionId}`);
    }
    const input = String(data || '');
    for (let i = 0; i < input.length; i += 1) {
      if (input.startsWith('\u001b[A', i)) {
        i += 2;
        if (session.history.length > 0 && session.historyIdx > 0) {
          session.historyIdx -= 1;
          redrawLine(session, session.history[session.historyIdx]);
        }
        continue;
      }
      if (input.startsWith('\u001b[B', i)) {
        i += 2;
        if (session.historyIdx < session.history.length - 1) {
          session.historyIdx += 1;
          redrawLine(session, session.history[session.historyIdx]);
        } else {
          session.historyIdx = session.history.length;
          redrawLine(session, '');
        }
        continue;
      }
      if (input[i] === '\u001b' && input[i + 1] === '[') {
        while (i + 1 < input.length && !/[A-Za-z~]/.test(input[i])) i += 1;
        continue;
      }
      const ch = input[i];
      if (ch === '\r' || ch === '\n') {
        if (ch === '\r' && input[i + 1] === '\n') i += 1;
        const cmd = session.lineBuffer;
        session.lineBuffer = '';
        await runInteractiveLine(session, cmd);
        continue;
      }
      if (ch === '\u007f' || ch === '\b') {
        if (session.lineBuffer.length > 0) {
          session.lineBuffer = session.lineBuffer.slice(0, -1);
          emitChunk(session, '\b \b');
        }
        continue;
      }
      if (ch === '\u0003') {
        session.lineBuffer = '';
        emitChunk(session, `^C\r\n${promptText(session.cwd)}`);
        continue;
      }
      if (ch === '\u000c') {
        emitChunk(session, `\u001b[2J\u001b[H${promptText(session.cwd)}${session.lineBuffer}`);
        continue;
      }
      if (ch >= ' ') {
        session.lineBuffer += ch;
        emitChunk(session, ch);
      }
    }
    return { ok: true };
  }

  function resizeSession({ sessionId, cols, rows }) {
    const session = sessions.get(sessionId);
    if (session) {
      session.cols = cols;
      session.rows = rows;
    }
    return { ok: true };
  }

  function serializeSession({ sessionId }) {
    const session = sessions.get(sessionId);
    if (!session) {
      return { scrollback: '', scrollbackEndOffset: 0 };
    }
    return {
      scrollback: session.scrollback,
      scrollbackEndOffset: session.scrollback.length,
    };
  }

  function closeSession({ sessionId }) {
    const session = sessions.get(sessionId);
    if (session && appSessions.get(session.appKey) === sessionId) {
      // Keep CoW sandbox warm; UI re-attaches cleanly via terminal:open.
      return { ok: true };
    }
    return { ok: true };
  }

  function killSession({ sessionId }) {
    const session = sessions.get(sessionId);
    if (session) {
      session.exited = true;
      sessions.delete(sessionId);
      if (appSessions.get(session.appKey) === sessionId) {
        appSessions.delete(session.appKey);
      }
      emit(`terminal:exit:${sessionId}`, { sessionId, exitCode: 0, signal: null });
    }
    return { ok: true };
  }

  async function getIdentity(appId) {
    const box = await ensureSandbox(appId);
    await box.readyPromise;
    return box.identity;
  }

  return {
    openSession,
    closeSession,
    killSession,
    writeInput,
    resizeSession,
    serializeSession,
    execCommand,
    getIdentity,
  };
}
