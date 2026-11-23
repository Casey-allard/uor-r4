// Encrypted-at-rest GitHub credential vault for Foundry Web Studio.
// Seals tokens in IndexedDB refs["vault:github:credentials"] using core.wasm ChaCha20-Poly1305
// keyed from the origin workspace key (never stored in localStorage or exposed on window.__host).

const VAULT_REF_KEY = 'vault:github:credentials';
const DEFAULT_API_BASE = 'https://api.github.com';
const enc = new TextEncoder();
const dec = new TextDecoder();

export function validateGitHubApiBase(raw) {
  const input = String(raw || '').trim() || DEFAULT_API_BASE;
  let parsed;
  try {
    parsed = new URL(input);
  } catch {
    throw new Error('Invalid GitHub API base URL');
  }
  const cleanOrigin = parsed.origin.replace(/\/+$/, '');
  const cleanPath = parsed.pathname.replace(/\/+$/, '');
  const normalized = `${cleanOrigin}${cleanPath}`;
  if (parsed.protocol === 'https:') {
    return normalized;
  }
  if (
    parsed.protocol === 'http:' &&
    (parsed.hostname === '127.0.0.1' || parsed.hostname === 'localhost')
  ) {
    return normalized;
  }
  throw new Error('GitHub API base URL must use https:// (or http://127.0.0.1 / localhost for local tests)');
}

export function validateGitHubTokenFormat(rawToken) {
  const token = String(rawToken || '').trim();
  if (!token) {
    throw new Error('GitHub token is required');
  }
  if (token.length < 8 || token.length > 512 || /[\r\n\0]/.test(token)) {
    throw new Error('Invalid GitHub token format');
  }
  return token;
}

export function createGitHubVault({ coreReady, prismFrame, getRef, putRef, deleteRef, deriveVaultKey } = {}) {
  let cachedSubkey = null;

  async function resolveVaultKey() {
    if (cachedSubkey instanceof Uint8Array && cachedSubkey.byteLength === 32) {
      return cachedSubkey;
    }
    if (typeof deriveVaultKey === 'function') {
      const k = await deriveVaultKey();
      if (!(k instanceof Uint8Array) || k.byteLength !== 32) {
        throw new Error('Vault key derivation failed');
      }
      cachedSubkey = k;
      return cachedSubkey;
    }
    const frame = await prismFrame();
    const workspaceSecret = await frame.workspaceKey();
    const secretBytes =
      workspaceSecret instanceof Uint8Array
        ? workspaceSecret
        : new Uint8Array(workspaceSecret);
    const domain = enc.encode('uor.github.vault/1:');
    const input = new Uint8Array(domain.length + secretBytes.length);
    input.set(domain, 0);
    input.set(secretBytes, domain.length);
    const digest = await crypto.subtle.digest('SHA-256', input);
    cachedSubkey = new Uint8Array(digest);
    return cachedSubkey;
  }

  async function saveCredentials({
    token,
    login,
    name = null,
    avatarUrl = null,
    apiBase = DEFAULT_API_BASE,
    oauthRelayUrl = null,
  }) {
    const cleanToken = validateGitHubTokenFormat(token);
    const cleanBase = validateGitHubApiBase(apiBase);
    const cleanLogin = String(login || '').trim();
    if (!cleanLogin) {
      throw new Error('GitHub user login is required');
    }
    const [core, key] = await Promise.all([coreReady, resolveVaultKey()]);
    const plaintext = enc.encode(JSON.stringify({ token: cleanToken, login: cleanLogin }));
    const sealedToken = core.seal(key, plaintext);
    if (!(sealedToken instanceof Uint8Array) || sealedToken.byteLength < 28) {
      throw new Error('core.wasm ChaCha20-Poly1305 seal failed');
    }
    const record = {
      scheme: 'uor.github.vault/1',
      alg: 'chacha20-poly1305',
      sealedToken,
      login: cleanLogin,
      name: name ? String(name) : cleanLogin,
      avatarUrl: avatarUrl ? String(avatarUrl) : null,
      apiBase: cleanBase,
      oauthRelayUrl: oauthRelayUrl ? String(oauthRelayUrl).trim() : null,
      connectedAt: new Date().toISOString(),
    };
    await putRef(VAULT_REF_KEY, record);
    return getPublicStatus();
  }

  async function readCredentials() {
    const record = await getRef(VAULT_REF_KEY);
    if (!record || !(record.sealedToken instanceof Uint8Array)) {
      return null;
    }
    const [core, key] = await Promise.all([coreReady, resolveVaultKey()]);
    const opened = core.open(key, record.sealedToken);
    if (!(opened instanceof Uint8Array)) {
      throw new Error('Failed to decrypt GitHub vault credentials');
    }
    const parsed = JSON.parse(dec.decode(opened));
    return {
      token: validateGitHubTokenFormat(parsed.token),
      login: String(record.login || parsed.login || ''),
      name: String(record.name || record.login || ''),
      avatarUrl: record.avatarUrl || null,
      apiBase: validateGitHubApiBase(record.apiBase || DEFAULT_API_BASE),
      oauthRelayUrl: record.oauthRelayUrl || null,
      connectedAt: record.connectedAt || null,
    };
  }

  async function getPublicStatus() {
    const record = await getRef(VAULT_REF_KEY);
    let configuredApiBase = DEFAULT_API_BASE;
    try {
      const override = globalThis.localStorage?.getItem?.('uor.github.apiBase');
      if (override) configuredApiBase = validateGitHubApiBase(override);
    } catch {}

    if (!record || !(record.sealedToken instanceof Uint8Array)) {
      return {
        connected: false,
        login: null,
        name: null,
        avatarUrl: null,
        apiBase: configuredApiBase,
        oauthRelayUrl: null,
        connectedAt: null,
        encryption: 'chacha20-poly1305 (core.wasm)',
      };
    }
    return {
      connected: true,
      login: String(record.login || ''),
      name: String(record.name || record.login || ''),
      avatarUrl: record.avatarUrl || null,
      apiBase: record.apiBase || configuredApiBase,
      oauthRelayUrl: record.oauthRelayUrl || null,
      connectedAt: record.connectedAt || null,
      encryption: 'chacha20-poly1305 (core.wasm)',
    };
  }

  async function clear() {
    await deleteRef(VAULT_REF_KEY);
    return getPublicStatus();
  }

  return {
    saveCredentials,
    readCredentials,
    getPublicStatus,
    clear,
  };
}
