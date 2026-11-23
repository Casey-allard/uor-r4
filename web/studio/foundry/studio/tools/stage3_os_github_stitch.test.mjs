import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import { createServer } from 'node:http';
import { test } from 'node:test';
import { createGitHubVault, validateGitHubApiBase } from '../shell/github-vault.mjs';
import { createGitHubClient, parseGitHubRepoSpec } from '../shell/github-client.mjs';
import { StitchSignalSession, createStitchNetwork } from '../shell/stitch-network.mjs';
import { reconcileVmWorkspaceForApp, syncVmSnapshotToWorkspace } from '../shell/linux-vm-host.mjs';
import { assertCanonicalGeneration } from '../shell/canonical-generation.mjs';
import { stageGenerationPatch } from '../shell/generation-patch.mjs';

function makeChaChaCore() {
  return {
    seal(key, plaintext) {
      const iv = crypto.randomBytes(12);
      const cipher = crypto.createCipheriv('chacha20-poly1305', key, iv, { authTagLength: 16 });
      const ct = Buffer.concat([cipher.update(plaintext), cipher.final()]);
      const tag = cipher.getAuthTag();
      return new Uint8Array(Buffer.concat([iv, ct, tag]));
    },
    open(key, sealed) {
      const buf = Buffer.from(sealed);
      const iv = buf.subarray(0, 12);
      const tag = buf.subarray(buf.length - 16);
      const ct = buf.subarray(12, buf.length - 16);
      const decipher = crypto.createDecipheriv('chacha20-poly1305', key, iv, { authTagLength: 16 });
      decipher.setAuthTag(tag);
      return new Uint8Array(Buffer.concat([decipher.update(ct), decipher.final()]));
    },
  };
}

function startMockGitHub() {
  const refs = new Map();
  let nextId = 1;
  const sha = (s) => crypto.createHash('sha1').update(s).digest('hex');

  const blobs = new Map();
  const trees = new Map();
  const commits = new Map();
  const repos = new Map();

  function initRepo(owner, repo, initialFiles) {
    const entries = [];
    for (const [path, content] of Object.entries(initialFiles)) {
      const bSha = sha(`blob:${nextId++}:${path}:${content}`);
      blobs.set(bSha, { sha: bSha, content: Buffer.from(content, 'utf8').toString('base64'), encoding: 'base64' });
      entries.push({ path, mode: '100644', type: 'blob', sha: bSha, size: Buffer.byteLength(content, 'utf8') });
    }
    const tSha = sha(`tree:${nextId++}`);
    trees.set(tSha, entries);
    const cSha = sha(`commit:${nextId++}`);
    commits.set(cSha, { sha: cSha, tree: { sha: tSha }, message: 'Initial commit', parents: [] });
    refs.set(`${owner}/${repo}:heads/main`, cSha);
    repos.set(`${owner}/${repo}`, {
      name: repo,
      full_name: `${owner}/${repo}`,
      private: false,
      default_branch: 'main',
      html_url: `https://github.com/${owner}/${repo}`,
      owner: { login: owner },
    });
    return cSha;
  }

  initRepo('octocat', 'hello-uor', {
    'README.md': '# Hello UOR\n',
    'src/main.ts': 'export const greeting = "hello v1";\n',
  });

  const server = createServer(async (req, res) => {
    const url = new URL(req.url, 'http://127.0.0.1');
    const chunks = [];
    for await (const c of req) chunks.push(c);
    const rawBody = Buffer.concat(chunks).toString('utf8');
    const body = rawBody ? JSON.parse(rawBody) : null;

    const send = (code, data) => {
      res.writeHead(code, { 'content-type': 'application/json' });
      res.end(JSON.stringify(data));
    };

    if (req.headers.authorization !== 'Bearer ghp_unit_secret_token_12345') {
      return send(401, { message: 'Bad credentials' });
    }

    if (req.method === 'GET' && url.pathname === '/user') {
      return send(200, { login: 'octocat', name: 'The Octocat', avatar_url: 'https://example.invalid/a.png' });
    }
    if (req.method === 'GET' && url.pathname === '/user/repos') {
      return send(200, Array.from(repos.values()));
    }

    const repoMatch = url.pathname.match(/^\/repos\/([^/]+)\/([^/]+)(.*)$/);
    if (!repoMatch) return send(404, { message: 'Not Found' });
    const [, owner, repo, tail] = repoMatch;
    const key = `${owner}/${repo}`;

    if (req.method === 'GET' && tail === '') {
      const r = repos.get(key);
      return r ? send(200, r) : send(404, { message: 'Not Found' });
    }
    if (req.method === 'GET' && tail === '/branches') {
      const head = refs.get(`${key}:heads/main`);
      return send(200, [{ name: 'main', commit: { sha: head } }]);
    }
    if (req.method === 'GET' && tail === '/git/ref/heads/main') {
      const head = refs.get(`${key}:heads/main`);
      return head ? send(200, { ref: 'refs/heads/main', object: { sha: head, type: 'commit' } }) : send(404, { message: 'Not Found' });
    }
    if (req.method === 'PATCH' && tail === '/git/refs/heads/main') {
      if (body?.force !== false) return send(400, { message: 'CAS push requires force: false' });
      refs.set(`${key}:heads/main`, body.sha);
      return send(200, { ref: 'refs/heads/main', object: { sha: body.sha, type: 'commit' } });
    }
    let m;
    if (req.method === 'GET' && (m = tail.match(/^\/git\/commits\/([0-9a-f]+)$/))) {
      const c = commits.get(m[1]);
      return c ? send(200, c) : send(404, { message: 'Not Found' });
    }
    if (req.method === 'GET' && (m = tail.match(/^\/git\/trees\/([0-9a-f]+)$/))) {
      const t = trees.get(m[1]);
      return t ? send(200, { sha: m[1], tree: t }) : send(404, { message: 'Not Found' });
    }
    if (req.method === 'GET' && (m = tail.match(/^\/git\/blobs\/([0-9a-f]+)$/))) {
      const b = blobs.get(m[1]);
      return b ? send(200, b) : send(404, { message: 'Not Found' });
    }
    if (req.method === 'POST' && tail === '/git/blobs') {
      const bSha = sha(`blob:${nextId++}:${body.content}`);
      blobs.set(bSha, { sha: bSha, content: body.content, encoding: body.encoding || 'base64' });
      return send(201, { sha: bSha });
    }
    if (req.method === 'POST' && tail === '/git/trees') {
      const baseEntries = new Map((trees.get(body.base_tree) || []).map((e) => [e.path, e]));
      for (const item of body.tree || []) {
        if (item.sha === null) baseEntries.delete(item.path);
        else baseEntries.set(item.path, { path: item.path, mode: '100644', type: 'blob', sha: item.sha });
      }
      const tSha = sha(`tree:${nextId++}`);
      trees.set(tSha, Array.from(baseEntries.values()));
      return send(201, { sha: tSha });
    }
    if (req.method === 'POST' && tail === '/git/commits') {
      const cSha = sha(`commit:${nextId++}:${body.message}`);
      commits.set(cSha, { sha: cSha, tree: { sha: body.tree }, message: body.message, parents: (body.parents || []).map((p) => ({ sha: p })) });
      return send(201, { sha: cSha });
    }
    return send(404, { message: `Unhandled ${req.method} ${url.pathname}` });
  });

  return new Promise((resolve) => {
    server.listen(0, '127.0.0.1', () => {
      const addr = server.address();
      resolve({
        baseUrl: `http://127.0.0.1:${addr.port}`,
        advanceRemote(owner, repo, branch, path, content) {
          const curHead = refs.get(`${owner}/${repo}:heads/${branch}`);
          const parentCommit = commits.get(curHead);
          const baseEntries = new Map((trees.get(parentCommit.tree.sha) || []).map((e) => [e.path, e]));
          const bSha = sha(`blob:${nextId++}:${content}`);
          blobs.set(bSha, { sha: bSha, content: Buffer.from(content, 'utf8').toString('base64'), encoding: 'base64' });
          baseEntries.set(path, { path, mode: '100644', type: 'blob', sha: bSha });
          const tSha = sha(`tree:${nextId++}`);
          trees.set(tSha, Array.from(baseEntries.values()));
          const cSha = sha(`commit:${nextId++}:remote`);
          commits.set(cSha, { sha: cSha, tree: { sha: tSha }, message: 'Remote advance', parents: [{ sha: curHead }] });
          refs.set(`${owner}/${repo}:heads/${branch}`, cSha);
          return cSha;
        },
        deleteRemote(owner, repo, branch, path) {
          const curHead = refs.get(`${owner}/${repo}:heads/${branch}`);
          const parentCommit = commits.get(curHead);
          const baseEntries = new Map((trees.get(parentCommit.tree.sha) || []).map((e) => [e.path, e]));
          baseEntries.delete(path);
          const tSha = sha(`tree:${nextId++}`);
          trees.set(tSha, Array.from(baseEntries.values()));
          const cSha = sha(`commit:${nextId++}:delete:${path}`);
          commits.set(cSha, { sha: cSha, tree: { sha: tSha }, message: `Delete ${path}`, parents: [{ sha: curHead }] });
          refs.set(`${owner}/${repo}:heads/${branch}`, cSha);
          return cSha;
        },
        close: () => new Promise((done) => server.close(done)),
      });
    });
  });
}

test('github-vault seals PAT with ChaCha20-Poly1305 and never leaks token in public status', async () => {
  const refs = new Map();
  const key = crypto.randomBytes(32);
  const core = makeChaChaCore();
  const vault = createGitHubVault({
    coreReady: Promise.resolve(core),
    deriveVaultKey: async () => key,
    getRef: async (k) => refs.get(k),
    putRef: async (k, v) => { refs.set(k, v); },
    deleteRef: async (k) => { refs.delete(k); },
  });

  const secret = 'ghp_unit_secret_token_12345';
  const pub = await vault.saveCredentials({ token: secret, login: 'octocat', apiBase: 'http://127.0.0.1:9999' });
  assert.equal(pub.connected, true);
  assert.equal(pub.login, 'octocat');
  assert.equal('token' in pub, false);
  const rawStored = refs.get('vault:github:credentials');
  assert.ok(rawStored.sealedToken instanceof Uint8Array);
  assert.equal(JSON.stringify(rawStored).includes(secret), false);

  const opened = await vault.readCredentials();
  assert.equal(opened.token, secret);

  assert.throws(() => validateGitHubApiBase('http://evil.example.com'), /https/i);
  await vault.clear();
  assert.equal((await vault.getPublicStatus()).connected, false);
});

test('github-client performs clone, diff, CAS push (force:false), conflict refusal, and pull', async () => {
  const mock = await startMockGitHub();
  try {
    assert.deepEqual(parseGitHubRepoSpec({ url: 'https://github.com/octocat/hello-uor.git' }), { owner: 'octocat', repo: 'hello-uor' });
    const refs = new Map();
    const apps = new Map();
    const files = new Map();
    const emitted = [];
    let appSeq = 0;

    const vault = createGitHubVault({
      coreReady: Promise.resolve(makeChaChaCore()),
      deriveVaultKey: async () => crypto.randomBytes(32),
      getRef: async (k) => refs.get(k),
      putRef: async (k, v) => { refs.set(k, v); },
      deleteRef: async (k) => { refs.delete(k); },
    });
    const client = createGitHubClient({
      vault,
      getApp: async (id) => apps.get(id),
      listApps: async () => Array.from(apps.values()),
      putApp: async (id, app) => { apps.set(id, app); },
      listAppFiles: async (id) =>
        Array.from(files.keys())
          .filter((k) => k.startsWith(`${id}:`))
          .map((k) => k.slice(`${id}:`.length))
          .sort(),
      readAppFile: async (id, p) => files.get(`${id}:${p}`),
      getRef: async (k) => refs.get(k),
      putRef: async (k, v) => { refs.set(k, v); },
      deleteRef: async (k) => { refs.delete(k); },
      emit: (channel, payload) => emitted.push({ channel, payload }),
      createClonedProject: async ({ name, files: initial, appDefaults }) => {
        const id = ++appSeq;
        const app = { id, name, path: name, ...appDefaults };
        apps.set(id, app);
        for (const [p, content] of Object.entries(initial)) {
          files.set(`${id}:${p}`, content);
        }
        return { app, chatId: id };
      },
      applyWorkspacePatch: async (id, patch) => {
        const source = {
          name: apps.get(id)?.name || `app-${id}`,
          reference: `blake3:${'0'.repeat(64)}`,
          files: Array.from(files.keys())
            .filter((k) => k.startsWith(`${id}:`))
            .sort()
            .map((k) => [k.slice(`${id}:`.length), files.get(k)]),
          dependencies: [],
        };
        assertCanonicalGeneration(source, patch, { canonicalSource: () => ({ name: 'Model' }) });
        stageGenerationPatch(source, patch);
        for (const w of patch.writes || []) files.set(`${id}:${w.path}`, w.content);
        for (const d of patch.deletes || []) files.delete(`${id}:${d.path}`);
      },
    });

    assert.deepEqual(await client.getFlowState(), { status: 'disconnected', revision: 0 });
    const connected = await client.connect({ token: 'ghp_unit_secret_token_12345', apiBase: mock.baseUrl });
    assert.equal(connected.login, 'octocat');
    const connectedFlow = await client.getFlowState();
    assert.equal(connectedFlow.status, 'connected');
    assert.equal(connectedFlow.provider, 'github');
    assert.equal(connectedFlow.invocationRef?.kind, 'connection-flow');
    assert.equal(connectedFlow.invocationRef?.entityKey, 'github');
    assert.ok(connectedFlow.invocationRef?.operationId);
    assert.equal(connectedFlow.revision, 1);
    const flowEvt = emitted.find((e) => e.channel === 'connection-flow:state-changed');
    assert.deepEqual(flowEvt?.payload?.state, connectedFlow);

    const cloned = await client.cloneRepoFromUrl({ url: 'octocat/hello-uor' });
    assert.equal(cloned.ok, true);
    assert.equal(cloned.fileCount, 2);
    assert.equal(files.get(`${cloned.app.id}:src/main.ts`), 'export const greeting = "hello v1";\n');

    files.set(`${cloned.app.id}:src/main.ts`, 'export const greeting = "hello v2";\n');
    const d = await client.diff({ appId: cloned.app.id });
    assert.deepEqual(d.modified, ['src/main.ts']);

    const pushed = await client.push({ appId: cloned.app.id, message: 'Update greeting' });
    assert.equal(pushed.ok, true);
    assert.equal(pushed.filesPushed, 1);

    mock.advanceRemote('octocat', 'hello-uor', 'main', 'REMOTE.md', 'from remote\n');
    const stale = await client.push({ appId: cloned.app.id, message: 'Stale' });
    assert.equal(stale.ok, false);
    assert.equal(stale.conflict, true);

    const pulled = await client.pull({ appId: cloned.app.id });
    assert.equal(pulled.ok, true);
    assert.equal(files.get(`${cloned.app.id}:REMOTE.md`), 'from remote\n');

    // Remote deletion on pull must emit canonical { path } delete objects and pass generation-patch validation.
    mock.deleteRemote('octocat', 'hello-uor', 'main', 'REMOTE.md');
    const pulledDelete = await client.pull({ appId: cloned.app.id });
    assert.equal(pulledDelete.ok, true);
    assert.equal(pulledDelete.deletedCount, 1);
    assert.equal(files.has(`${cloned.app.id}:REMOTE.md`), false);

    // Linking an existing remote repo must allow an immediate pull() without returning alreadyUpToDate.
    apps.set(99, { id: 99, name: 'local-app', path: 'local-app' });
    files.set('99:LOCAL.md', 'local\n');
    const linked = await client.linkRepo({ appId: 99, owner: 'octocat', repo: 'hello-uor', branch: 'main' });
    assert.equal(linked.ok, true);
    const linkedPull = await client.pull({ appId: 99 });
    assert.equal(linkedPull.ok, true);
    assert.equal(linkedPull.alreadyUpToDate, false);
    assert.equal(files.get('99:src/main.ts'), 'export const greeting = "hello v2";\n');

    // Verify FOUNDRY_SYSTEM_PROMPT signature: github_clone(repo="owner/repo") without explicit url/owner.
    assert.deepEqual(parseGitHubRepoSpec({ repo: 'octocat/hello-uor' }), { owner: 'octocat', repo: 'hello-uor' });
    const clonedByPromptSig = await client.cloneRepoFromUrl({ repo: 'octocat/hello-uor', appName: 'prompt-cloned' });
    assert.equal(clonedByPromptSig.ok, true);
    assert.equal(clonedByPromptSig.app.name, 'prompt-cloned');

    await client.disconnect();
    assert.deepEqual(await client.getFlowState(), { status: 'disconnected', revision: 2 });
  } finally {
    await mock.close();
  }
});

// Minimal faithful mock of libv86 fs9p directory/file inodes (including SearchPath().name === undefined quirk).
function createMockFs9p() {
  const root = new Map();
  const files = new Map(); // relPath -> Uint8Array
  const dirs = new Set(['']);
  let nextId = 1;
  const pathIds = new Map([['', 0]]);
  const idPaths = new Map([[0, '']]);
  const clean = (p) => String(p || '').replace(/^\/+|\/+$/g, '');
  return {
    _files: files,
    Search(parentId, name) {
      const parent = idPaths.get(parentId);
      if (parent == null) return -1;
      const full = parent ? `${parent}/${name}` : name;
      return pathIds.has(full) ? pathIds.get(full) : -1;
    },
    SearchPath(p) {
      const c = clean(p);
      if (!c) return { id: 0, parentid: -1, name: undefined };
      const id = pathIds.has(c) ? pathIds.get(c) : -1;
      const parentStr = c.includes('/') ? c.slice(0, c.lastIndexOf('/')) : '';
      const pid = pathIds.has(parentStr) ? pathIds.get(parentStr) : -1;
      return { id, parentid: pid, name: id === -1 ? c.split('/').pop() : undefined };
    },
    IsDirectory(id) {
      return dirs.has(idPaths.get(id));
    },
    GetInode(id) {
      const full = idPaths.get(id);
      if (full == null) return null;
      const buf = files.get(full);
      return { size: buf ? buf.byteLength : 0 };
    },
      CreateDirectory(seg, parentId) {
        const parent = idPaths.get(parentId);
        const full = parent ? `${parent}/${seg}` : seg;
        const id = nextId++;
        pathIds.set(full, id);
        idPaths.set(id, full);
        dirs.add(full);
        if (!parent) root.set(seg, id);
      },
      async CreateBinaryFile(fileName, parentId, bytes) {
        const parent = idPaths.get(parentId);
        const full = parent ? `${parent}/${fileName}` : fileName;
        const id = nextId++;
        pathIds.set(full, id);
        idPaths.set(id, full);
        files.set(full, new Uint8Array(bytes));
        if (!parent) root.set(fileName, id);
      },
      Unlink(parentId, name) {
        const parent = idPaths.get(parentId);
        const full = parent ? `${parent}/${name}` : name;
        files.delete(full);
        pathIds.delete(full);
        if (!parent) root.delete(name);
      },
      RecursiveDelete(p) {
        // Faithfully mirror libv86.js: RecursiveDelete only unlinks descendants, leaving directory `p` itself!
        const prefix = clean(p);
        for (const k of Array.from(pathIds.keys())) {
          if (k.startsWith(prefix + '/')) {
            const id = pathIds.get(k);
            pathIds.delete(k);
            idPaths.delete(id);
            files.delete(k);
            dirs.delete(k);
          }
        }
      },
      read_dir(dirQuery) {
        const c = clean(dirQuery);
        if (!c) return Array.from(root.keys());
        const out = [];
        for (const k of pathIds.keys()) {
          if (k.startsWith(c + '/') && !k.slice(c.length + 1).includes('/')) {
            out.push(k.slice(c.length + 1));
          }
        }
        return out;
      },
      async read_file(p) {
        return files.get(clean(p)) || null;
      },
  };
}

test('reconcileVmWorkspaceForApp isolates /workspace across app switches and purges stale files', async () => {
  const fs9p = createMockFs9p();
  const appFiles = new Map([
    [1, { 'appA.txt': 'from-A', 'app1_dir/sub/index.ts': 'A' }],
    [2, { 'appB.txt': 'from-B' }],
  ]);
  const lastSyncedHashes = new Map();
  const listAppFiles = async (id) => Object.keys(appFiles.get(id) || {});
  const readAppFile = async (id, p) => appFiles.get(id)?.[p];

  const r1 = await reconcileVmWorkspaceForApp({ fs9p, mountedAppId: undefined, targetAppId: 1, lastSyncedHashes, listAppFiles, readAppFile });
  assert.equal(r1.written, 2);
  assert.equal(fs9p.read_dir('/').includes('app1_dir'), true);

  const r2 = await reconcileVmWorkspaceForApp({ fs9p, mountedAppId: 1, targetAppId: 2, lastSyncedHashes, listAppFiles, readAppFile });
  assert.equal(r2.purged, true);
  assert.equal(fs9p._files.has('appA.txt'), false);
  assert.equal(fs9p._files.has('app1_dir/sub/index.ts'), false);
  assert.equal(fs9p.read_dir('/').includes('app1_dir'), false);
  assert.equal(fs9p.SearchPath('/app1_dir').id, -1);
  assert.equal(fs9p._files.has('appB.txt'), true);

  // Switching to null (Home Chat) purges App 2, and switching back to 1 purges any null scratch files.
  const rNull = await reconcileVmWorkspaceForApp({ fs9p, mountedAppId: 2, targetAppId: null, lastSyncedHashes, listAppFiles, readAppFile });
  assert.equal(rNull.purged, true);
  assert.equal(fs9p.read_dir('/').length, 0);
  await fs9p.CreateBinaryFile('home_scratch.txt', 0, Buffer.from('scratch'));
  const rBackTo1 = await reconcileVmWorkspaceForApp({ fs9p, mountedAppId: null, targetAppId: 1, lastSyncedHashes, listAppFiles, readAppFile });
  assert.equal(rBackTo1.purged, true);
  assert.equal(fs9p.read_dir('/').includes('home_scratch.txt'), false);
});

test('syncVmSnapshotToWorkspace emits canonical {path} deletes and preserves hashes on commit failure', async () => {
  const fs9p = createMockFs9p();
  const appFiles = new Map([[1, { 'keep.txt': 'keep', 'gone.txt': 'bye' }]]);
  const lastSyncedHashes = new Map();
  const listAppFiles = async (id) => Object.keys(appFiles.get(id) || {});
  const readAppFile = async (id, p) => appFiles.get(id)?.[p];

  await reconcileVmWorkspaceForApp({ fs9p, mountedAppId: undefined, targetAppId: 1, lastSyncedHashes, listAppFiles, readAppFile });
  assert.equal(lastSyncedHashes.get('1').has('gone.txt'), true);

  fs9p.Unlink(0, 'gone.txt');
  const syncOk = await syncVmSnapshotToWorkspace({
    fs9p,
    appId: 1,
    appKey: '1',
    lastSyncedHashes,
    listAppFiles,
    readAppFile,
    applyWorkspaceChanges: async (id, patch) => {
      const store = appFiles.get(id);
      const source = {
        name: `app-${id}`,
        reference: `blake3:${'0'.repeat(64)}`,
        files: Object.entries(store).sort(),
        dependencies: [],
      };
      assertCanonicalGeneration(source, patch, { canonicalSource: () => ({ name: 'Model' }) });
      stageGenerationPatch(source, patch);
      for (const w of patch.writes || []) store[w.path] = w.content;
      for (const d of patch.deletes || []) delete store[d.path];
    },
  });
  assert.equal(syncOk.ok, true);
  assert.equal(syncOk.deletes, 1);
  assert.equal('gone.txt' in appFiles.get(1), false);
  assert.equal(lastSyncedHashes.get('1').has('gone.txt'), false);

  // If applyWorkspaceChanges throws/refuses, lastSyncedHashes must NOT drop the deleted file before commit.
  fs9p.Unlink(0, 'keep.txt');
  await assert.rejects(
    () =>
      syncVmSnapshotToWorkspace({
        fs9p,
        appId: 1,
        appKey: '1',
        lastSyncedHashes,
        listAppFiles,
        readAppFile,
        applyWorkspaceChanges: async () => {
          throw Object.assign(new Error('Refused'), { code: 'canonicalGenerationRefused' });
        },
      }),
    /Refused/,
  );
  assert.equal(lastSyncedHashes.get('1').has('keep.txt'), true);
});

test('stitch-network validates Ed25519 stitch.signal/1 envelopes, rejects replay, and fetches local resources', async () => {
  const alice = new StitchSignalSession('uor-test-room');
  const bob = new StitchSignalSession('uor-test-room');

  const announce = await alice.sign({ kind: 'announce' });
  const verified1 = await bob.verify(announce);
  assert.ok(verified1);
  assert.equal(verified1.profile, 'stitch.signal/1');

  // Replay of same sequence must be rejected.
  assert.equal(await bob.verify(announce), null);

  // Tampered payload must fail signature verification.
  const tampered = structuredClone(await alice.sign({ kind: 'hello', to: (await bob.identity()).id, targetSession: bob.session }));
  tampered.sequence += 1;
  assert.equal(await bob.verify(tampered), null);

  const net = createStitchNetwork({
    resolveLocalResource: async (path) => ({ ok: path === 'app.json', status: path === 'app.json' ? 200 : 404, body_preview: '{"name":"UOR-R4"}' }),
  });
  const localRes = await net.fetchPeer('stitch://local/app.json');
  assert.equal(localRes.ok, true);
  assert.ok(localRes.body_preview.includes('UOR-R4'));
  const offlineRes = await net.fetchPeer('stitch://key-nonexistent/app.json');
  assert.equal(offlineRes.ok, false);
  await net.disconnect();
});
