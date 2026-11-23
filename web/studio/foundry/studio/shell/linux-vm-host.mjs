// Host-side manager for the real x86 Linux 6.18.7 VM (afflom/stitch v86 WASM + Buildroot),
// bidirectional /workspace host9p <-> Studio IndexedDB sync, ttyS0 <-> xterm.js serial bridge,
// and NE2000 frame forwarding into Stitch networking.

const BASE = new URL('./', import.meta.url);
const enc = new TextEncoder();
const dec = new TextDecoder();

const INTERNAL_FILE_RE = /^\.uor_(boot|cmd|out|err|rc|pwd)_/;
const BINARY_EXT_RE = /\.(png|jpe?g|gif|webp|ico|woff2?|ttf|otf|wasm|bin|lut|d11|holo|zip|gz|xz|tar|pdf)$/i;
const MAX_SYNC_FILE_BYTES = 512 * 1024;
const MAX_SYNC_FILES = 256;

let libV86Promise = null;

function loadLibV86() {
  if (globalThis.V86 || globalThis.V86Starter) {
    return Promise.resolve(globalThis.V86 || globalThis.V86Starter);
  }
  if (libV86Promise) return libV86Promise;
  libV86Promise = new Promise((resolve, reject) => {
    const script = document.createElement('script');
    script.src = new URL('v86/libv86.js', BASE).href;
    script.async = false;
    script.onload = () => {
      const Ctor = globalThis.V86 || globalThis.V86Starter;
      if (!Ctor) {
        libV86Promise = null;
        reject(new Error('libv86.js loaded but V86 constructor is unavailable'));
        return;
      }
      resolve(Ctor);
    };
    script.onerror = () => {
      libV86Promise = null;
      reject(new Error('Failed to load ./v86/libv86.js'));
    };
    document.head.appendChild(script);
  });
  return libV86Promise;
}

function toCrLf(text) {
  return String(text || '').replace(/\r?\n/g, '\r\n');
}

function normalizeRelPath(raw) {
  return String(raw || '')
    .replaceAll('\\', '/')
    .replace(/^\/+/, '')
    .replace(/\/+/g, '/')
    .replace(/\/$/, '');
}

async function sha256Hex(bytes) {
  const digest = await crypto.subtle.digest('SHA-256', bytes);
  return Array.from(new Uint8Array(digest), (b) => b.toString(16).padStart(2, '0')).join('');
}

function createHiddenScreenContainer() {
  if (typeof document === 'undefined') return null;
  const host = document.createElement('div');
  host.setAttribute('aria-hidden', 'true');
  host.style.cssText =
    'position:fixed;left:-10000px;top:-10000px;width:640px;height:400px;overflow:hidden;opacity:0;pointer-events:none;';
  const textScreen = document.createElement('div');
  textScreen.style.cssText = 'white-space:pre;font:14px monospace;line-height:14px;';
  const canvas = document.createElement('canvas');
  host.appendChild(textScreen);
  host.appendChild(canvas);
  (document.body || document.documentElement).appendChild(host);
  return host;
}

function readScreenText(container) {
  if (!container) return '';
  return Array.from(container.querySelectorAll('span,div'))
    .map((el) => el.textContent || '')
    .join('\n');
}

function ensureFs9pDir(fs9p, dirPath) {
  const clean = normalizeRelPath(dirPath);
  if (!clean) return 0;
  let parentId = 0;
  for (const seg of clean.split('/')) {
    if (!seg || seg === '.') continue;
    let nextId = fs9p.Search(parentId, seg);
    if (nextId === -1) {
      fs9p.CreateDirectory(seg, parentId);
      nextId = fs9p.Search(parentId, seg);
    }
    if (nextId === -1) {
      throw new Error(`Unable to create 9p directory segment: ${seg}`);
    }
    parentId = nextId;
  }
  return parentId;
}

function delete9pEntry(fs9p, relPath) {
  const clean = normalizeRelPath(relPath);
  if (!clean) return false;
  const info = fs9p.SearchPath('/' + clean);
  if (info.id === -1 || info.parentid === -1) return false;
  const fileName = clean.split('/').pop();
  if (fs9p.IsDirectory(info.id)) {
    // libv86 RecursiveDelete unlinks descendants only, leaving the directory node linked;
    // unlink the directory node itself explicitly afterward (avoiding SearchPath().name === undefined).
    fs9p.RecursiveDelete('/' + clean);
  }
  fs9p.Unlink(info.parentid, fileName);
  return true;
}

async function write9pFile(fs9p, relPath, bytes) {
  const clean = normalizeRelPath(relPath);
  if (!clean) return;
  const parts = clean.split('/');
  const fileName = parts.pop();
  const parentId = ensureFs9pDir(fs9p, parts.join('/'));
  const existingId = fs9p.Search(parentId, fileName);
  if (existingId !== -1) {
    delete9pEntry(fs9p, clean);
  }
  const u8 = bytes instanceof Uint8Array ? bytes : enc.encode(String(bytes ?? ''));
  await fs9p.CreateBinaryFile(fileName, parentId, u8);
}

function delete9pFile(fs9p, relPath) {
  return delete9pEntry(fs9p, relPath);
}

async function read9pFileBytes(fs9p, relPath) {
  const clean = normalizeRelPath(relPath);
  if (!clean) return null;
  const info = fs9p.SearchPath('/' + clean);
  if (info.id === -1 || fs9p.IsDirectory(info.id)) return null;
  const data = await fs9p.read_file('/' + clean);
  return data ? new Uint8Array(data) : null;
}

async function walk9pFiles(fs9p, prefix = '', out = new Map(), depth = 0) {
  if (!fs9p || depth > 12 || out.size >= MAX_SYNC_FILES) return out;
  const dirQuery = prefix ? '/' + prefix + '/' : '/';
  const names = fs9p.read_dir(dirQuery) || [];
  for (const name of names) {
    if (!name || name === '.' || name === '..') continue;
    if (depth === 0 && INTERNAL_FILE_RE.test(name)) continue;
    if (name === '.git' || name === 'node_modules') continue;
    const rel = prefix ? `${prefix}/${name}` : name;
    const info = fs9p.SearchPath('/' + rel);
    if (info.id === -1) continue;
    if (fs9p.IsDirectory(info.id)) {
      await walk9pFiles(fs9p, rel, out, depth + 1);
      continue;
    }
    const inode = fs9p.GetInode(info.id);
    if (!inode || inode.size > MAX_SYNC_FILE_BYTES) continue;
    const bytes = await fs9p.read_file('/' + rel);
    if (bytes) {
      out.set(rel, new Uint8Array(bytes));
      if (out.size >= MAX_SYNC_FILES) break;
    }
  }
  return out;
}

function shellSingleQuote(value) {
  return `'${String(value).replace(/'/g, `'"'"'`)}'`;
}

function clear9pWorkspace(fs9p) {
  if (!fs9p) return;
  const names = fs9p.read_dir('/') || [];
  for (const name of names) {
    if (!name || name === '.' || name === '..' || INTERNAL_FILE_RE.test(name)) continue;
    delete9pEntry(fs9p, name);
  }
}

export async function reconcileVmWorkspaceForApp({
  fs9p,
  mountedAppId,
  targetAppId,
  lastSyncedHashes,
  listAppFiles,
  readAppFile,
}) {
  const appKey = targetAppId == null ? 'global' : String(targetAppId);
  let purged = false;
  if (mountedAppId !== undefined && mountedAppId !== targetAppId) {
    clear9pWorkspace(fs9p);
    if (mountedAppId != null) {
      lastSyncedHashes.delete(String(mountedAppId));
    }
    purged = true;
  }
  if (targetAppId == null) {
    return { ok: true, appId: null, purged, written: 0, deleted: 0 };
  }
  const prevMap = lastSyncedHashes.get(appKey) || new Map();
  const nextMap = new Map();
  const paths = ((await listAppFiles?.(targetAppId)) || [])
    .map(normalizeRelPath)
    .filter((p) => p && !INTERNAL_FILE_RE.test(p));

  let written = 0;
  let deleted = 0;
  for (const rel of paths.slice(0, MAX_SYNC_FILES)) {
    const text = await readAppFile?.(targetAppId, rel);
    if (typeof text !== 'string') continue;
    const bytes = enc.encode(text);
    if (bytes.byteLength > MAX_SYNC_FILE_BYTES) continue;
    const h = await sha256Hex(bytes);
    nextMap.set(rel, h);
    const existingBytes = await read9pFileBytes(fs9p, rel);
    const existingHash = existingBytes ? await sha256Hex(existingBytes) : null;
    if (existingHash !== h) {
      await write9pFile(fs9p, rel, bytes);
      written += 1;
    }
  }
  for (const oldPath of prevMap.keys()) {
    if (!nextMap.has(oldPath)) {
      if (delete9pFile(fs9p, oldPath)) deleted += 1;
    }
  }
  lastSyncedHashes.set(appKey, nextMap);
  return { ok: true, appId: targetAppId, purged, written, deleted };
}

export async function syncVmSnapshotToWorkspace({
  fs9p,
  appId,
  appKey,
  lastSyncedHashes,
  listAppFiles,
  readAppFile,
  applyWorkspaceChanges,
}) {
  if (!fs9p || appId == null) {
    return { ok: false, skipped: true };
  }
  const key = appKey ?? String(appId);
  const prevMap = lastSyncedHashes.get(key) || new Map();
  const vmFiles = await walk9pFiles(fs9p);
  const idbPaths = new Set(((await listAppFiles?.(appId)) || []).map(normalizeRelPath).filter(Boolean));
  const nextMap = new Map();
  const writes = [];
  const deletes = [];

  for (const [rel, bytes] of vmFiles.entries()) {
    if (BINARY_EXT_RE.test(rel)) continue;
    let text;
    try {
      text = dec.decode(bytes);
    } catch {
      continue;
    }
    const h = await sha256Hex(bytes);
    nextMap.set(rel, h);
    const curText = idbPaths.has(rel) ? await readAppFile?.(appId, rel) : undefined;
    const curHash = typeof curText === 'string' ? await sha256Hex(enc.encode(curText)) : null;
    if (curHash !== h) {
      writes.push({ path: rel, description: 'linux-vm-9p-sync', content: text });
    }
  }

  for (const oldPath of prevMap.keys()) {
    if (!vmFiles.has(oldPath) && idbPaths.has(oldPath)) {
      deletes.push({ path: oldPath });
    }
  }

  if (writes.length || deletes.length) {
    await applyWorkspaceChanges?.(appId, { writes, deletes, renames: [], deps: [] });
  }
  lastSyncedHashes.set(key, nextMap);
  return { ok: true, appId, writes: writes.length, deletes: deletes.length };
}

export function createLinuxVmHost({
  emit,
  getAppName,
  listAppFiles,
  readAppFile,
  applyWorkspaceChanges,
  getEthBridge,
}) {
  const sessions = new Map(); // sessionId -> session state
  const appSessions = new Map(); // appKey -> sessionId
  const lastSyncedHashes = new Map(); // appKey -> Map<relPath, sha256>

  let vm = null;
  let screenContainer = null;
  let bootPromise = null;
  let vmState = 'idle'; // idle | booting | online | error
  let vmError = null;
  let activeAppId = null;
  let mountedAppId = undefined;
  let bootLogTail = '';
  let serialPartial = [];
  let commandTail = Promise.resolve();
  let syncTail = Promise.resolve();
  let writeSyncTimer = null;
  let suppressVmWriteSyncUntil = 0;

  function appKeyOf(appId) {
    return appId == null ? 'global' : String(appId);
  }

  function emitChunkToActiveSessions(chunk) {
    if (!chunk) return;
    for (const session of sessions.values()) {
      if (session.exited) continue;
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
  }

  function emitStatus() {
    emit('linux:status-changed', getStatus());
  }

  function getStatus() {
    return {
      runtime: 'linux-6.18.7-v86',
      kernel: 'Linux 6.18.7',
      state: vmState,
      error: vmError,
      activeAppId,
      mountedAppId: mountedAppId === undefined ? null : mountedAppId,
      sessions: sessions.size,
      ethAttached: Boolean(getEthBridge?.()),
    };
  }

  async function syncWorkspaceToVm(appId = undefined) {
    if (!vm?.fs9p || vmState === 'idle') return { ok: false, skipped: true };
    const targetAppId = appId === undefined ? activeAppId : appId;
    clearTimeout(writeSyncTimer);
    const work = syncTail.then(async () => {
      suppressVmWriteSyncUntil = Date.now() + 500;
      const result = await reconcileVmWorkspaceForApp({
        fs9p: vm.fs9p,
        mountedAppId,
        targetAppId,
        lastSyncedHashes,
        listAppFiles,
        readAppFile,
      });
      mountedAppId = targetAppId;
      activeAppId = targetAppId;
      return result;
    });
    syncTail = work.catch(() => {});
    return work;
  }

  async function syncVmToWorkspace(appId = activeAppId) {
    if (!vm?.fs9p || vmState !== 'online') return { ok: false, skipped: true };
    if (appId == null) return { ok: true, appId: null, writes: 0, deletes: 0 };
    const work = syncTail.then(async () => {
      if (mountedAppId !== appId) {
        return { ok: false, skipped: true, reason: 'mounted-app-mismatch', mountedAppId, appId };
      }
      return syncVmSnapshotToWorkspace({
        fs9p: vm.fs9p,
        appId,
        appKey: appKeyOf(appId),
        lastSyncedHashes,
        listAppFiles,
        readAppFile,
        applyWorkspaceChanges,
      });
    });
    syncTail = work.catch(() => {});
    return work;
  }

  function scheduleVmToWorkspaceSync() {
    if (vmState !== 'online' || activeAppId == null || mountedAppId !== activeAppId) return;
    if (Date.now() < suppressVmWriteSyncUntil) return;
    clearTimeout(writeSyncTimer);
    const targetAppId = activeAppId;
    writeSyncTimer = setTimeout(() => {
      void syncVmToWorkspace(targetAppId).catch(() => {});
    }, 300);
  }

  async function ensureBooted(appId = undefined) {
    if (appId !== undefined) {
      activeAppId = appId;
    }
    if (vm && vmState === 'online') {
      return vm;
    }
    if (bootPromise) {
      return bootPromise;
    }
    vmState = 'booting';
    vmError = null;
    bootLogTail = '';
    emitStatus();

    bootPromise = (async () => {
      const V86Ctor = await loadLibV86();
      if (!screenContainer) {
        screenContainer = createHiddenScreenContainer();
      }
      const instance = new V86Ctor({
        wasm_path: new URL('v86/v86.wasm', BASE).href,
        memory_size: 64 * 1024 * 1024,
        vga_memory_size: 2 * 1024 * 1024,
        screen_container: screenContainer || undefined,
        bios: { url: new URL('v86/seabios.bin', BASE).href },
        vga_bios: { url: new URL('v86/vgabios.bin', BASE).href },
        bzimage: { url: new URL('images/buildroot-bzimage.bin', BASE).href },
        cmdline: 'console=tty0 console=ttyS0,115200 rdinit=/init nomodeset',
        filesystem: {},
        net_device: { type: 'ne2k' },
        autostart: false,
      });
      vm = instance;

      const utf8Decoder = new TextDecoder('utf-8', { fatal: false });
      instance.add_listener('serial0-output-byte', (byte) => {
        serialPartial.push(byte & 0xff);
        // Decode once multi-byte UTF-8 sequence is complete or single ASCII byte arrives.
        const last = byte & 0xff;
        if (last < 0x80 || serialPartial.length >= 4) {
          const text = utf8Decoder.decode(new Uint8Array(serialPartial), { stream: true });
          serialPartial = [];
          if (text) {
            bootLogTail = (bootLogTail + text).slice(-8192);
            emitChunkToActiveSessions(text);
          }
        }
      });

      instance.bus?.register?.('9p-write-end', () => {
        scheduleVmToWorkspaceSync();
      });

      instance.add_listener('net0-send', (frameBytes) => {
        const bridge = getEthBridge?.();
        if (bridge && frameBytes) {
          bridge.onOutboundFrame(frameBytes);
        }
      });

      await new Promise((resolve, reject) => {
        let settled = false;
        let pollTimer = null;
        let deadlineTimer = null;
        let started = false;
        let probeSent = false;
        let tty0RedirectSent = false;
        let vgaReadySince = 0;
        const nonce = crypto.randomUUID().replaceAll('-', '');
        const inName = `.uor_boot_in_${nonce}`;
        const outName = `.uor_boot_ok_${nonce}`;

        const cleanup = () => {
          if (pollTimer) clearInterval(pollTimer);
          if (deadlineTimer) clearTimeout(deadlineTimer);
        };

        const fail = (message) => {
          if (settled) return;
          settled = true;
          cleanup();
          vmState = 'error';
          vmError = message;
          emitStatus();
          reject(new Error(message));
        };

        instance.add_listener('download-error', () => {
          fail('v86 failed to load Linux kernel or BIOS assets');
        });

        instance.add_listener('emulator-loaded', () => {
          if (started || settled) return;
          started = true;
          try {
            instance.run();
          } catch (err) {
            fail(String(err?.message || err));
          }
        });

        deadlineTimer = setTimeout(() => {
          fail(`Linux 6.18.7 boot timed out after 45000ms (tail: ${bootLogTail.slice(-240)})`);
        }, 45000);

        pollTimer = setInterval(async () => {
          if (settled || !started) return;
          try {
            const serialReady = /(?:~%|\/ #|#\s*$|login:|Please press Enter)/m.test(bootLogTail);
            const vgaText = readScreenText(screenContainer);
            const vgaReady = /(?:~%|\/ #|#\s*$|Please press Enter)/.test(vgaText);
            if (vgaReady && !vgaReadySince) vgaReadySince = Date.now();

            if (!tty0RedirectSent && !serialReady && !probeSent && vgaReady && Date.now() - vgaReadySince > 1200) {
              tty0RedirectSent = true;
              if (typeof instance.keyboard_send_text === 'function') {
                if (/Please press Enter/.test(vgaText)) {
                  await instance.keyboard_send_text('\n', 10);
                }
                await instance.keyboard_send_text(
                  'setsid sh -c "exec /bin/sh </dev/ttyS0 >/dev/ttyS0 2>&1" &\n',
                  10,
                );
              }
            }

            if (!probeSent && (serialReady || tty0RedirectSent)) {
              if (/Please press Enter/.test(bootLogTail)) {
                instance.serial0_send('\n');
              }
              probeSent = true;
              await write9pFile(instance.fs9p, inName, enc.encode(nonce));
              const setupCmd =
                `mkdir -p /mnt; ` +
                `{ grep -q ' /mnt 9p ' /proc/mounts || mount -t 9p -o trans=virtio host9p /mnt; } && ` +
                `{ [ -L /workspace ] || grep -q ' /workspace ' /proc/mounts || { mkdir -p /workspace && mount --bind /mnt /workspace 2>/dev/null; } || { rmdir /workspace 2>/dev/null; ln -sfn /mnt /workspace; }; } && ` +
                `cd /workspace && cp /mnt/${inName} /mnt/${outName}\n`;
              instance.serial0_send(setupCmd);
            }

            if (probeSent) {
              const outBytes = await read9pFileBytes(instance.fs9p, outName);
              if (outBytes && dec.decode(outBytes).trim() === nonce) {
                delete9pFile(instance.fs9p, inName);
                delete9pFile(instance.fs9p, outName);
                settled = true;
                cleanup();
                vmState = 'online';
                vmError = null;
                emitStatus();
                resolve(instance);
              }
            }
          } catch (err) {
            fail(`Linux 6.18.7 boot probe failed: ${err?.message || err}`);
          }
        }, 200);
      });

      await syncWorkspaceToVm(activeAppId);
      return instance;
    })().catch((err) => {
      bootPromise = null;
      throw err;
    });

    return bootPromise;
  }

  async function execCommand(appId, command, { echoToSession = true, cwd = '/workspace', timeoutMs = 20000 } = {}) {
    const cmd = String(command ?? '').trim();
    if (!cmd) {
      return {
        exit_code: 0,
        stdout: '',
        stderr: '',
        cwd: '/workspace',
        runtime: 'linux-6.18.7-v86',
      };
    }

    const work = commandTail.then(async () => {
      const targetAppId = appId === undefined ? activeAppId : appId;
      activeAppId = targetAppId;
      const instance = await ensureBooted(targetAppId);
      if (mountedAppId !== targetAppId || targetAppId != null) {
        await syncWorkspaceToVm(targetAppId);
      }

      const tag = crypto.randomUUID().replaceAll('-', '').slice(0, 12);
      const scriptName = `.uor_cmd_${tag}.sh`;
      const outName = `.uor_out_${tag}.txt`;
      const errName = `.uor_err_${tag}.txt`;
      const rcName = `.uor_rc_${tag}.txt`;
      const pwdName = `.uor_pwd_${tag}.txt`;

      const safeCwd = typeof cwd === 'string' && cwd.startsWith('/') ? cwd : '/workspace';
      const scriptBody = `cd ${shellSingleQuote(safeCwd)} 2>/dev/null || cd /workspace\n${cmd}\n`;
      await write9pFile(instance.fs9p, scriptName, enc.encode(scriptBody));

      const invokeLine =
        `stty -echo 2>/dev/null; ` +
        `/bin/sh /workspace/${scriptName} > /workspace/${outName} 2> /workspace/${errName}; ` +
        `printf '%s' "$?" > /workspace/${rcName}; ` +
        `pwd > /workspace/${pwdName}; ` +
        `stty echo 2>/dev/null\n`;
      instance.serial0_send(invokeLine);

      const startedAt = Date.now();
      let rcBytes = null;
      while (Date.now() - startedAt < timeoutMs) {
        rcBytes = await read9pFileBytes(instance.fs9p, rcName);
        if (rcBytes && rcBytes.byteLength > 0) break;
        await new Promise((r) => setTimeout(r, 50));
      }

      const outBytes = (await read9pFileBytes(instance.fs9p, outName)) || new Uint8Array(0);
      const errBytes = (await read9pFileBytes(instance.fs9p, errName)) || new Uint8Array(0);
      const pwdBytes = (await read9pFileBytes(instance.fs9p, pwdName)) || new Uint8Array(0);

      for (const f of [scriptName, outName, errName, rcName, pwdName]) {
        delete9pFile(instance.fs9p, f);
      }

      const timedOut = !rcBytes;
      const exitCode = timedOut ? 124 : Number.parseInt(dec.decode(rcBytes).trim(), 10) || 0;
      const stdout = dec.decode(outBytes);
      const stderr = dec.decode(errBytes) + (timedOut ? `\nCommand timed out after ${timeoutMs}ms` : '');
      const nextCwd = dec.decode(pwdBytes).trim() || safeCwd;

      if (targetAppId != null) {
        await syncVmToWorkspace(targetAppId);
      }

      const activeSid = appSessions.get(appKeyOf(targetAppId));
      const session = activeSid ? sessions.get(activeSid) : null;
      if (session) {
        session.cwd = nextCwd;
        if (echoToSession) {
          emitChunkToActiveSessions(`\r\n[linux-6.18.7] $ ${cmd}\r\n`);
          if (stdout) emitChunkToActiveSessions(toCrLf(stdout));
          if (stderr) emitChunkToActiveSessions(`\u001b[31m${toCrLf(stderr)}\u001b[0m`);
          if ((stdout && !stdout.endsWith('\n')) || (stderr && !stderr.endsWith('\n'))) {
            emitChunkToActiveSessions('\r\n');
          }
        }
      }

      return {
        ok: exitCode === 0,
        exit_code: exitCode,
        stdout,
        stderr,
        cwd: nextCwd,
        runtime: 'linux-6.18.7-v86',
      };
    });

    commandTail = work.catch(() => {});
    return work;
  }

  async function openSession({ appId = null, cols = 80, rows = 24 } = {}) {
    const appKey = appKeyOf(appId);
    activeAppId = appId;
    const existingId = appSessions.get(appKey);
    if (existingId && sessions.has(existingId)) {
      const existing = sessions.get(existingId);
      if (!existing.exited) {
        if (vmState === 'online' && mountedAppId !== appId) {
          await syncWorkspaceToVm(appId);
        }
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

    const sessionId = `linux-${appKey}-${crypto.randomUUID().slice(0, 8)}`;
    const appName = (await getAppName?.(appId)) || 'UOR-R4 Workspace';
    const initialBanner =
      vmState === 'online'
        ? `Linux 6.18.7 (v86 x86 + host9p /workspace synced with Studio)\r\n/workspace # `
        : `Booting Linux 6.18.7 (v86 x86 + host9p /workspace)...\r\n`;

    const session = {
      sessionId,
      appId,
      appKey,
      shell: 'Linux 6.18.7 (v86 · /workspace)',
      cwd: '/workspace',
      appName,
      cols,
      rows,
      scrollback: initialBanner,
      exited: false,
    };
    sessions.set(sessionId, session);
    appSessions.set(appKey, sessionId);

    void ensureBooted(appId)
      .then(() => {
        if (mountedAppId !== appId) {
          return syncWorkspaceToVm(appId);
        }
        return null;
      })
      .then(() => {
        if (!session.exited) {
          emitChunkToActiveSessions(`\r\n[v86] Linux 6.18.7 ready (/workspace synced with Studio)\r\n/workspace # `);
        }
      })
      .catch((err) => {
        if (!session.exited) {
          emitChunkToActiveSessions(`\r\n\u001b[31m[v86 error] ${err?.message || err}\u001b[0m\r\n`);
        }
      });

    return {
      sessionId,
      shell: session.shell,
      cwd: session.cwd,
      appName: session.appName,
      scrollback: session.scrollback,
      created: true,
    };
  }

  async function writeInput({ sessionId, data }) {
    const session = sessions.get(sessionId);
    if (!session || session.exited) return { ok: true };
    const input = String(data ?? '');
    if (!input) return { ok: true };
    activeAppId = session.appId;
    const instance = await ensureBooted(session.appId);
    if (mountedAppId !== session.appId) {
      await syncWorkspaceToVm(session.appId);
    }
    instance.serial0_send(input);
    if (input.includes('\r') || input.includes('\n')) {
      scheduleVmToWorkspaceSync();
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

  function closeSession() {
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

  function injectEthFrame(frameBytes) {
    if (!vm || vmState !== 'online' || !frameBytes) return false;
    try {
      vm.bus.send('net0-receive', frameBytes instanceof Uint8Array ? frameBytes : new Uint8Array(frameBytes));
      return true;
    } catch {
      return false;
    }
  }

  async function destroy() {
    clearTimeout(writeSyncTimer);
    if (vm) {
      const target = vm;
      vm = null;
      bootPromise = null;
      vmState = 'idle';
      mountedAppId = undefined;
      try {
        await target.destroy?.();
      } catch {}
    }
    if (screenContainer?.parentNode) {
      screenContainer.parentNode.removeChild(screenContainer);
      screenContainer = null;
    }
    emitStatus();
    return { ok: true };
  }

  return {
    openSession,
    closeSession,
    killSession,
    writeInput,
    resizeSession,
    serializeSession,
    execCommand,
    syncWorkspaceToVm,
    syncVmToWorkspace,
    ensureBooted,
    injectEthFrame,
    getStatus,
    destroy,
  };
}
