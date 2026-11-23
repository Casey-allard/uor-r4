// Browser-native GitHub REST v3 & Git Data API client for Foundry Web Studio.
// Supports Fine-Grained PAT (or OAuth Worker exchange) sealed via ChaCha20-Poly1305 in github-vault.mjs,
// repo listing, branches, repo creation, clone into IndexedDB, diff, pull, and CAS push (force: false).

import { validateGitHubApiBase, validateGitHubTokenFormat } from './github-vault.mjs';

const enc = new TextEncoder();
const dec = new TextDecoder('utf-8', { fatal: false });

const MAX_CLONE_FILES = 256;
const MAX_CLONE_FILE_BYTES = 512 * 1024;
const SKIP_EXT_RE = /\.(png|jpe?g|gif|webp|ico|woff2?|ttf|otf|wasm|bin|lut|d11|holo|zip|gz|xz|tar|pdf)$/i;

function encodeUtf8Base64(text) {
  const bytes = enc.encode(String(text ?? ''));
  let bin = '';
  for (let i = 0; i < bytes.length; i += 32768) {
    bin += String.fromCharCode(...bytes.subarray(i, i + 32768));
  }
  return btoa(bin);
}

function decodeBase64Utf8(b64Text) {
  const clean = String(b64Text || '').replace(/\s+/g, '');
  const bin = atob(clean);
  const bytes = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i += 1) {
    bytes[i] = bin.charCodeAt(i);
  }
  return dec.decode(bytes);
}

async function sha256Hex(text) {
  const digest = await crypto.subtle.digest('SHA-256', enc.encode(String(text ?? '')));
  return Array.from(new Uint8Array(digest), (b) => b.toString(16).padStart(2, '0')).join('');
}

function portableRepoPath(p) {
  const s = String(p || '').replaceAll('\\', '/').replace(/^\/+/, '');
  if (!s || /[\u0000-\u001f:]/.test(s)) return null;
  const parts = s.split('/');
  if (parts.some((seg) => !seg || seg === '.' || seg === '..' || seg === '.git')) return null;
  return s;
}

export function parseGitHubRepoSpec({ url, owner, repo } = {}) {
  const cleanSeg = (s) => {
    const v = String(s || '').trim();
    if (!v || !/^[A-Za-z0-9._-]+$/.test(v) || v === '.' || v === '..') {
      return '';
    }
    return v;
  };
  if (owner && repo) {
    const o = cleanSeg(owner);
    const r = cleanSeg(String(repo).replace(/\.git$/i, ''));
    if (!o || !r) throw new Error('Invalid GitHub owner or repo name');
    return { owner: o, repo: r };
  }
  let raw = String(url || '').trim();
  if (!raw && typeof repo === 'string' && repo.trim()) {
    // Support FOUNDRY_SYSTEM_PROMPT signature: github_clone(repo="owner/repo" | "https://...", branch?)
    raw = repo.trim();
  }
  if (!raw) throw new Error('Repository URL or owner/repo is required');

  const sshMatch = raw.match(/^git@[^:]+:([^/]+)\/([^/]+?)(?:\.git)?\/?$/);
  if (sshMatch) {
    const o = cleanSeg(sshMatch[1]);
    const r = cleanSeg(sshMatch[2]);
    if (!o || !r) throw new Error('Invalid GitHub repository URL');
    return { owner: o, repo: r };
  }

  if (/^https?:\/\//i.test(raw)) {
    const u = new URL(raw);
    const segs = u.pathname.replace(/^\/+|\/+$/g, '').split('/');
    if (segs.length < 2) throw new Error('GitHub URL must include /owner/repo');
    const o = cleanSeg(segs[0]);
    const r = cleanSeg(segs[1].replace(/\.git$/i, ''));
    if (!o || !r) throw new Error('Invalid GitHub repository URL');
    return { owner: o, repo: r };
  }

  const slashMatch = raw.replace(/^\/+|\/+$/g, '').split('/');
  if (slashMatch.length === 2) {
    const o = cleanSeg(slashMatch[0]);
    const r = cleanSeg(slashMatch[1].replace(/\.git$/i, ''));
    if (!o || !r) throw new Error('Invalid owner/repo specification');
    return { owner: o, repo: r };
  }

  throw new Error('Unsupported GitHub repository format; expected owner/repo or https://github.com/owner/repo');
}

export function createGitHubClient({
  vault,
  getApp,
  listApps,
  putApp,
  listAppFiles,
  readAppFile,
  getRef,
  putRef,
  deleteRef,
  createClonedProject,
  applyWorkspacePatch,
  onWorkspaceSynced,
  emit,
  fetchImpl = globalThis.fetch.bind(globalThis),
}) {
  let flowRevision = 0;
  let flowOperationId = 'github-vault';

  function buildFlowState(connected) {
    if (!connected) {
      return {
        status: 'disconnected',
        revision: flowRevision,
      };
    }
    return {
      status: 'connected',
      provider: 'github',
      revision: flowRevision,
      invocationRef: {
        kind: 'connection-flow',
        entityKey: 'github',
        operationId: flowOperationId || 'github-vault',
      },
    };
  }

  function resolveConfiguredBase(overrideBase) {
    if (overrideBase) return validateGitHubApiBase(overrideBase);
    try {
      const fromStorage = globalThis.localStorage?.getItem?.('uor.github.apiBase');
      if (fromStorage) return validateGitHubApiBase(fromStorage);
    } catch {}
    return 'https://api.github.com';
  }

  function emitConnectionState(connected) {
    flowRevision += 1;
    emit?.('connection-flow:state-changed', {
      provider: 'github',
      state: buildFlowState(connected),
    });
    void vault
      .getPublicStatus()
      .then((status) => emit?.('github:status-changed', status))
      .catch(() => {});
  }

  async function apiRequest(path, { method = 'GET', body, token: explicitToken, apiBase: explicitBase, allowNotFound = false } = {}) {
    let token = explicitToken || null;
    let base = explicitBase ? validateGitHubApiBase(explicitBase) : null;
    if (!token) {
      const creds = await vault.readCredentials();
      if (!creds) {
        throw new Error('GitHub is not connected. Connect a Personal Access Token in Settings first.');
      }
      token = creds.token;
      if (!base) base = creds.apiBase;
    }
    if (!base) {
      base = resolveConfiguredBase();
    }
    const url = path.startsWith('http://') || path.startsWith('https://')
      ? path
      : `${base}${path.startsWith('/') ? '' : '/'}${path}`;
    const headers = {
      Accept: 'application/vnd.github+json',
      Authorization: `Bearer ${token}`,
      'X-GitHub-Api-Version': '2022-11-28',
    };
    const init = {
      method,
      headers,
      credentials: 'omit',
      redirect: 'error',
    };
    if (body !== undefined) {
      headers['Content-Type'] = 'application/json';
      init.body = JSON.stringify(body);
    }
    const resp = await fetchImpl(url, init);
    if (allowNotFound && resp.status === 404) {
      return { notFound: true, status: 404, data: null };
    }
    let data = null;
    const text = await resp.text();
    if (text) {
      try {
        data = JSON.parse(text);
      } catch {
        data = { message: text };
      }
    }
    if (!resp.ok) {
      const msg = data?.message || `GitHub API error HTTP ${resp.status}`;
      const err = new Error(msg);
      err.status = resp.status;
      err.data = data;
      throw err;
    }
    return { notFound: false, status: resp.status, data };
  }

  async function connect({ token, apiBase, oauthRelayUrl } = {}) {
    const cleanToken = validateGitHubTokenFormat(token);
    const cleanBase = resolveConfiguredBase(apiBase);
    const { data: user } = await apiRequest('/user', {
      token: cleanToken,
      apiBase: cleanBase,
    });
    if (!user?.login) {
      throw new Error('GitHub /user response did not include a login');
    }
    const status = await vault.saveCredentials({
      token: cleanToken,
      login: user.login,
      name: user.name || user.login,
      avatarUrl: user.avatar_url || null,
      apiBase: cleanBase,
      oauthRelayUrl: oauthRelayUrl || null,
    });
    flowOperationId = globalThis.crypto?.randomUUID?.() || `github-vault-${Date.now()}`;
    emitConnectionState(true);
    return { ok: true, ...status };
  }

  async function disconnect() {
    const status = await vault.clear();
    emitConnectionState(false);
    return { ok: true, ...status };
  }

  async function getStatus() {
    return vault.getPublicStatus();
  }

  async function getFlowState() {
    const status = await vault.getPublicStatus();
    return buildFlowState(Boolean(status.connected));
  }

  async function listRepos({ perPage = 50 } = {}) {
    const limit = Math.max(1, Math.min(100, Number(perPage) || 50));
    const { data } = await apiRequest(
      `/user/repos?sort=updated&per_page=${limit}&affiliation=owner,collaborator,organization_member`,
    );
    const rows = Array.isArray(data) ? data : [];
    return rows.map((r) => ({
      name: String(r.name || ''),
      full_name: String(r.full_name || ''),
      private: Boolean(r.private),
      default_branch: String(r.default_branch || 'main'),
      html_url: String(r.html_url || ''),
      owner: r.owner?.login ? String(r.owner.login) : String(r.full_name || '').split('/')[0] || '',
    }));
  }

  async function getRepoBranches({ owner, repo }) {
    const target = parseGitHubRepoSpec({ owner, repo });
    const { data } = await apiRequest(`/repos/${encodeURIComponent(target.owner)}/${encodeURIComponent(target.repo)}/branches?per_page=100`);
    const rows = Array.isArray(data) ? data : [];
    return rows.map((b) => ({
      name: String(b.name || ''),
      commit: { sha: String(b.commit?.sha || '') },
    }));
  }

  async function isRepoAvailable({ org, repo }) {
    const target = parseGitHubRepoSpec({ owner: org, repo });
    const res = await apiRequest(
      `/repos/${encodeURIComponent(target.owner)}/${encodeURIComponent(target.repo)}`,
      { allowNotFound: true },
    );
    return { available: res.notFound };
  }

  async function createRepo({ name, description = '', isPrivate = true, org = null } = {}) {
    const cleanName = String(name || '').trim();
    if (!cleanName || !/^[A-Za-z0-9._-]+$/.test(cleanName)) {
      throw new Error('Invalid repository name');
    }
    const endpoint = org ? `/orgs/${encodeURIComponent(org)}/repos` : '/user/repos';
    const { data } = await apiRequest(endpoint, {
      method: 'POST',
      body: {
        name: cleanName,
        description: String(description || ''),
        private: Boolean(isPrivate),
        auto_init: true,
      },
    });
    return {
      ok: true,
      name: String(data.name),
      full_name: String(data.full_name),
      owner: String(data.owner?.login || org || ''),
      private: Boolean(data.private),
      default_branch: String(data.default_branch || 'main'),
      html_url: String(data.html_url || ''),
    };
  }

  async function fetchBranchTree(owner, repo, branch) {
    const encOwner = encodeURIComponent(owner);
    const encRepo = encodeURIComponent(repo);
    const { data: refData, notFound } = await apiRequest(
      `/repos/${encOwner}/${encRepo}/git/ref/heads/${encodeURIComponent(branch)}`,
      { allowNotFound: true },
    );
    if (notFound || !refData?.object?.sha) {
      throw new Error(`Remote branch "${branch}" was not found on ${owner}/${repo}`);
    }
    const headSha = String(refData.object.sha);
    const { data: commitData } = await apiRequest(`/repos/${encOwner}/${encRepo}/git/commits/${headSha}`);
    const treeSha = String(commitData.tree?.sha || '');
    const { data: treeData } = await apiRequest(`/repos/${encOwner}/${encRepo}/git/trees/${treeSha}?recursive=1`);
    const entries = Array.isArray(treeData?.tree) ? treeData.tree : [];
    return { headSha, treeSha, entries };
  }

  async function readSyncState(appId) {
    const state = await getRef(`github:app:${appId}`);
    return state && typeof state === 'object' ? state : null;
  }

  async function writeSyncState(appId, state) {
    await putRef(`github:app:${appId}`, state);
  }

  async function computeHashes(filesObj) {
    const out = {};
    for (const [p, text] of Object.entries(filesObj || {})) {
      out[p] = await sha256Hex(text);
    }
    return out;
  }

  async function uniqueAppName(desired) {
    const existing = new Set(((await listApps?.()) || []).map((a) => String(a.name || '').toLowerCase()));
    const base = String(desired || 'github-app').trim() || 'github-app';
    if (!existing.has(base.toLowerCase())) return base;
    for (let i = 2; i < 1000; i += 1) {
      const candidate = `${base}-${i}`;
      if (!existing.has(candidate.toLowerCase())) return candidate;
    }
    return `${base}-${Date.now()}`;
  }

  async function cloneRepoFromUrl({ url, owner, repo, branch, appName } = {}) {
    const target = parseGitHubRepoSpec({ url, owner, repo });
    const encOwner = encodeURIComponent(target.owner);
    const encRepo = encodeURIComponent(target.repo);
    const { data: repoMeta } = await apiRequest(`/repos/${encOwner}/${encRepo}`);
    const targetBranch = String(branch || repoMeta.default_branch || 'main');
    const { headSha, treeSha, entries } = await fetchBranchTree(target.owner, target.repo, targetBranch);

    const files = {};
    const baseBlobs = {};
    let loaded = 0;
    for (const item of entries) {
      if (item.type !== 'blob') continue;
      if (loaded >= MAX_CLONE_FILES) break;
      const safePath = portableRepoPath(item.path);
      if (!safePath || SKIP_EXT_RE.test(safePath)) continue;
      if (typeof item.size === 'number' && item.size > MAX_CLONE_FILE_BYTES) continue;

      const { data: blob } = await apiRequest(`/repos/${encOwner}/${encRepo}/git/blobs/${item.sha}`);
      const content =
        blob.encoding === 'base64'
          ? decodeBase64Utf8(blob.content || '')
          : String(blob.content || '');
      files[safePath] = content;
      baseBlobs[safePath] = { blobSha: String(item.sha), sha256: await sha256Hex(content) };
      loaded += 1;
    }

    if (Object.keys(files).length === 0) {
      files['README.md'] = `# ${target.repo}\n\nCloned from ${target.owner}/${target.repo} (${targetBranch}).\n`;
    }

    const finalName = await uniqueAppName(appName || target.repo);
    const created = await createClonedProject({
      name: finalName,
      files,
      appDefaults: {
        githubOrg: target.owner,
        githubRepo: target.repo,
        githubBranch: targetBranch,
      },
    });

    await writeSyncState(created.app.id, {
      owner: target.owner,
      repo: target.repo,
      branch: targetBranch,
      headSha,
      treeSha,
      baseBlobs,
      syncedFromRemote: true,
      syncedAt: new Date().toISOString(),
    });
    await onWorkspaceSynced?.(created.app.id);

    return {
      ok: true,
      app: created.app,
      chatId: created.chatId,
      owner: target.owner,
      repo: target.repo,
      branch: targetBranch,
      commitSha: headSha,
      fileCount: loaded,
      hasAiRules: 'AI_RULES.md' in files,
    };
  }

  async function linkRepo({ appId, owner, repo, branch = 'main', createIfMissing = false } = {}) {
    if (!Number.isSafeInteger(appId) || appId < 1) {
      throw new Error('Valid appId is required');
    }
    const app = await getApp(appId);
    if (!app) throw new Error(`App ${appId} not found`);
    const target = parseGitHubRepoSpec({ owner, repo });

    let resolvedBranch = branch;
    let headSha = null;
    let treeSha = null;

    const existing = await apiRequest(
      `/repos/${encodeURIComponent(target.owner)}/${encodeURIComponent(target.repo)}`,
      { allowNotFound: true },
    );
    if (existing.notFound) {
      if (!createIfMissing) {
        throw new Error(`Repository ${target.owner}/${target.repo} not found on GitHub`);
      }
      const created = await createRepo({ name: target.repo });
      resolvedBranch = created.default_branch || 'main';
    } else {
      resolvedBranch = branch || existing.data.default_branch || 'main';
    }

    try {
      const remote = await fetchBranchTree(target.owner, target.repo, resolvedBranch);
      headSha = remote.headSha;
      treeSha = remote.treeSha;
    } catch {
      headSha = null;
      treeSha = null;
    }

    const updatedApp = {
      ...app,
      githubOrg: target.owner,
      githubRepo: target.repo,
      githubBranch: resolvedBranch,
      updatedAt: new Date().toISOString(),
    };
    await putApp(appId, updatedApp);
    await writeSyncState(appId, {
      owner: target.owner,
      repo: target.repo,
      branch: resolvedBranch,
      headSha,
      treeSha,
      baseBlobs: {},
      syncedFromRemote: false,
      syncedAt: new Date().toISOString(),
    });
    return { ok: true, owner: target.owner, repo: target.repo, branch: resolvedBranch, headSha };
  }

  async function unlinkRepo({ appId } = {}) {
    if (!Number.isSafeInteger(appId) || appId < 1) throw new Error('Valid appId is required');
    const app = await getApp(appId);
    if (!app) throw new Error(`App ${appId} not found`);
    await putApp(appId, {
      ...app,
      githubOrg: null,
      githubRepo: null,
      githubBranch: null,
      updatedAt: new Date().toISOString(),
    });
    await deleteRef(`github:app:${appId}`);
    return { ok: true };
  }

  async function readCurrentWorkspaceFiles(appId) {
    const paths = (await listAppFiles(appId)) || [];
    const files = {};
    for (const p of paths) {
      const content = await readAppFile(appId, p);
      if (typeof content === 'string') {
        files[p] = content;
      }
    }
    return files;
  }

  async function resolveLinkedTarget(appId, branchOverride) {
    if (!Number.isSafeInteger(appId) || appId < 1) {
      throw new Error('Open a Studio app first before running GitHub sync operations.');
    }
    const app = await getApp(appId);
    if (!app) throw new Error(`App ${appId} not found`);
    const state = (await readSyncState(appId)) || {};
    const owner = app.githubOrg || state.owner;
    const repo = app.githubRepo || state.repo;
    const branch = branchOverride || app.githubBranch || state.branch || 'main';
    if (!owner || !repo) {
      throw new Error('This app is not linked to a GitHub repository yet.');
    }
    return { app, owner, repo, branch, state };
  }

  async function diff({ appId } = {}) {
    const { owner, repo, branch, state } = await resolveLinkedTarget(appId);
    const current = await readCurrentWorkspaceFiles(appId);
    const currentHashes = await computeHashes(current);
    const baseBlobs = state.baseBlobs || {};
    const added = [];
    const modified = [];
    const deleted = [];

    for (const [path, hash] of Object.entries(currentHashes)) {
      if (!(path in baseBlobs)) added.push(path);
      else if (baseBlobs[path].sha256 !== hash) modified.push(path);
    }
    for (const path of Object.keys(baseBlobs)) {
      if (!(path in currentHashes)) deleted.push(path);
    }
    return {
      ok: true,
      owner,
      repo,
      branch,
      headSha: state.headSha || null,
      added,
      modified,
      deleted,
      totalChanged: added.length + modified.length + deleted.length,
      clean: added.length + modified.length + deleted.length === 0,
    };
  }

  async function pull({ appId, branch: branchOverride } = {}) {
    const { owner, repo, branch, state } = await resolveLinkedTarget(appId, branchOverride);
    const { headSha, treeSha, entries } = await fetchBranchTree(owner, repo, branch);
    const prevBase = state.baseBlobs || {};
    if (state.headSha === headSha && state.syncedFromRemote !== false) {
      return {
        ok: true,
        alreadyUpToDate: true,
        owner,
        repo,
        branch,
        commitSha: headSha,
        updatedCount: 0,
        deletedCount: 0,
      };
    }

    const encOwner = encodeURIComponent(owner);
    const encRepo = encodeURIComponent(repo);
    const current = await readCurrentWorkspaceFiles(appId);
    const nextBase = {};
    const writes = [];
    const deletes = [];
    const seen = new Set();

    let count = 0;
    for (const item of entries) {
      if (item.type !== 'blob') continue;
      if (count >= MAX_CLONE_FILES) break;
      const safePath = portableRepoPath(item.path);
      if (!safePath || SKIP_EXT_RE.test(safePath)) continue;
      if (typeof item.size === 'number' && item.size > MAX_CLONE_FILE_BYTES) continue;
      seen.add(safePath);
      count += 1;

      const prevEntry = prevBase[safePath];
      if (prevEntry && prevEntry.blobSha === item.sha && safePath in current) {
        nextBase[safePath] = prevEntry;
        continue;
      }

      const { data: blob } = await apiRequest(`/repos/${encOwner}/${encRepo}/git/blobs/${item.sha}`);
      const content =
        blob.encoding === 'base64'
          ? decodeBase64Utf8(blob.content || '')
          : String(blob.content || '');
      nextBase[safePath] = { blobSha: String(item.sha), sha256: await sha256Hex(content) };
      if (current[safePath] !== content) {
        writes.push({ path: safePath, description: `github:pull ${owner}/${repo}@${branch}`, content });
      }
    }

    for (const oldPath of Object.keys(prevBase)) {
      if (!seen.has(oldPath) && oldPath in current) {
        deletes.push({ path: oldPath });
      }
    }

    if (writes.length || deletes.length) {
      await applyWorkspacePatch(appId, { writes, deletes, renames: [], deps: [] });
    }
    await writeSyncState(appId, {
      owner,
      repo,
      branch,
      headSha,
      treeSha,
      baseBlobs: nextBase,
      syncedFromRemote: true,
      syncedAt: new Date().toISOString(),
    });
    await onWorkspaceSynced?.(appId);

    return {
      ok: true,
      alreadyUpToDate: false,
      owner,
      repo,
      branch,
      commitSha: headSha,
      updatedCount: writes.length,
      deletedCount: deletes.length,
    };
  }

  async function push({ appId, message, branch: branchOverride } = {}) {
    const { owner, repo, branch, state } = await resolveLinkedTarget(appId, branchOverride);
    const encOwner = encodeURIComponent(owner);
    const encRepo = encodeURIComponent(repo);

    const { headSha: remoteHeadSha, treeSha: remoteTreeSha } = await fetchBranchTree(owner, repo, branch);
    if (state.headSha && remoteHeadSha !== state.headSha) {
      return {
        ok: false,
        conflict: true,
        owner,
        repo,
        branch,
        localSha: state.headSha,
        remoteSha: remoteHeadSha,
        error: `Remote branch ${branch} has new commits (${remoteHeadSha.slice(0, 7)}). Pull latest changes before pushing.`,
      };
    }

    const current = await readCurrentWorkspaceFiles(appId);
    const prevBase = state.baseBlobs || {};
    const treeEntries = [];
    const nextBase = { ...prevBase };

    for (const [path, content] of Object.entries(current)) {
      const hash = await sha256Hex(content);
      const prev = prevBase[path];
      if (prev && prev.sha256 === hash && prev.blobSha) {
        continue;
      }
      const { data: blobRes } = await apiRequest(`/repos/${encOwner}/${encRepo}/git/blobs`, {
        method: 'POST',
        body: { content: encodeUtf8Base64(content), encoding: 'base64' },
      });
      treeEntries.push({
        path,
        mode: '100644',
        type: 'blob',
        sha: blobRes.sha,
      });
      nextBase[path] = { blobSha: String(blobRes.sha), sha256: hash };
    }

    for (const oldPath of Object.keys(prevBase)) {
      if (!(oldPath in current)) {
        treeEntries.push({
          path: oldPath,
          mode: '100644',
          type: 'blob',
          sha: null,
        });
        delete nextBase[oldPath];
      }
    }

    if (treeEntries.length === 0) {
      return {
        ok: true,
        nothingToPush: true,
        owner,
        repo,
        branch,
        commitSha: remoteHeadSha,
        filesPushed: 0,
      };
    }

    const { data: treeRes } = await apiRequest(`/repos/${encOwner}/${encRepo}/git/trees`, {
      method: 'POST',
      body: {
        base_tree: remoteTreeSha,
        tree: treeEntries,
      },
    });
    const commitMsg = String(message || '').trim() || `Update ${repo} from Foundry Studio`;
    const { data: commitRes } = await apiRequest(`/repos/${encOwner}/${encRepo}/git/commits`, {
      method: 'POST',
      body: {
        message: commitMsg,
        tree: treeRes.sha,
        parents: [remoteHeadSha],
      },
    });
    const newCommitSha = String(commitRes.sha);

    try {
      // Compare-and-swap update: force is ALWAYS false so concurrent remote commits cannot be overwritten.
      await apiRequest(`/repos/${encOwner}/${encRepo}/git/refs/heads/${encodeURIComponent(branch)}`, {
        method: 'PATCH',
        body: {
          sha: newCommitSha,
          force: false,
        },
      });
    } catch (err) {
      if (err?.status === 422 || err?.status === 409) {
        return {
          ok: false,
          conflict: true,
          owner,
          repo,
          branch,
          error: 'Fast-forward CAS ref update refused (remote branch changed concurrently). Pull latest changes first.',
        };
      }
      throw err;
    }

    await writeSyncState(appId, {
      owner,
      repo,
      branch,
      headSha: newCommitSha,
      treeSha: treeRes.sha,
      baseBlobs: nextBase,
      syncedFromRemote: true,
      syncedAt: new Date().toISOString(),
    });

    return {
      ok: true,
      nothingToPush: false,
      owner,
      repo,
      branch,
      commitSha: newCommitSha,
      filesPushed: treeEntries.length,
    };
  }

  return {
    connect,
    disconnect,
    getStatus,
    getFlowState,
    listRepos,
    getRepoBranches,
    isRepoAvailable,
    createRepo,
    cloneRepoFromUrl,
    linkRepo,
    unlinkRepo,
    diff,
    pull,
    push,
  };
}
