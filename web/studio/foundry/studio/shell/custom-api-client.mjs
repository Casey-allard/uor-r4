// Encrypted-at-rest Custom OpenAI-Compatible AI API Endpoint Vault & Client for Foundry Web Studio.
// Seals optional API keys in IndexedDB refs["vault:custom-api:endpoints"] using core.wasm ChaCha20-Poly1305
// keyed from the origin workspace key (never stored in localStorage or exposed on window.__host).
// Zero unsolicited network requests on boot/reload: requests run only on explicit test or user chat turns.

const VAULT_REF_KEY = 'vault:custom-api:endpoints';
const enc = new TextEncoder();
const dec = new TextDecoder();

export function validateCustomApiBaseUrl(raw) {
  const rawStr = String(raw ?? '');
  if (/[\x00-\x1f\x7f]/.test(rawStr)) {
    throw new Error('API base URL must not contain control characters');
  }
  const input = rawStr.trim();
  if (!input) {
    throw new Error('API base URL is required');
  }
  let parsed;
  try {
    parsed = new URL(input);
  } catch {
    throw new Error('Invalid API base URL');
  }
  if (parsed.username || parsed.password) {
    throw new Error('API base URL must not contain embedded credentials');
  }
  const cleanOrigin = parsed.origin.replace(/\/+$/, '');
  const cleanPath = parsed.pathname.replace(/\/+$/, '');
  const normalized = `${cleanOrigin}${cleanPath}`;
  if (parsed.protocol === 'https:') {
    return normalized;
  }
  if (
    parsed.protocol === 'http:' &&
    (parsed.hostname === '127.0.0.1' ||
      parsed.hostname === 'localhost' ||
      parsed.hostname === '[::1]')
  ) {
    return normalized;
  }
  throw new Error('Custom API base URL must use https:// (or http://127.0.0.1 / localhost for local endpoints)');
}

export function resolveChatCompletionsUrl(baseUrl) {
  const clean = validateCustomApiBaseUrl(baseUrl);
  if (clean.endsWith('/chat/completions')) {
    return clean;
  }
  return `${clean}/chat/completions`;
}

export function validateCustomApiName(raw) {
  const rawStr = String(raw ?? '');
  if (/[\x00-\x1f\x7f]/.test(rawStr)) {
    throw new Error('Endpoint name must not contain control characters');
  }
  const name = rawStr.trim();
  if (!name || name.length > 80) {
    throw new Error('Endpoint name must be 1 to 80 characters');
  }
  return name;
}

export function validateCustomApiModel(raw) {
  const rawStr = String(raw ?? '');
  if (/[\x00-\x1f\x7f]/.test(rawStr)) {
    throw new Error('Model ID must be a valid non-empty string');
  }
  const model = rawStr.trim();
  if (!model || model.length > 256) {
    throw new Error('Model ID must be a valid non-empty string');
  }
  return model;
}

export function validateCustomApiKey(raw) {
  if (raw == null) return '';
  const rawStr = String(raw);
  if (/[\x00-\x1f\x7f]/.test(rawStr)) {
    throw new Error('Invalid API key format');
  }
  const key = rawStr.trim();
  if (!key) return '';
  if (key.length > 512) {
    throw new Error('Invalid API key format');
  }
  return key;
}

function normalizeEndpointId(raw) {
  const s = String(raw ?? '').trim();
  if (!s) {
    return `ep-${crypto.randomUUID().slice(0, 8)}`;
  }
  if (!/^[a-zA-Z0-9][a-zA-Z0-9._-]{0,63}$/.test(s)) {
    throw new Error('Invalid custom endpoint id');
  }
  return s;
}

function toPublicRecord(record) {
  return {
    id: String(record.id),
    name: String(record.name),
    baseUrl: String(record.baseUrl),
    model: String(record.model),
    contextWindow: Number.isSafeInteger(record.contextWindow) ? record.contextWindow : 32768,
    hasApiKey: Boolean(record.hasApiKey),
    createdAt: record.createdAt || null,
    updatedAt: record.updatedAt || null,
    encryption: 'chacha20-poly1305 (core.wasm)',
  };
}

export function buildOpenAiMessages({ systemPrompt = '', messages = [], prompt = '' } = {}) {
  const out = [];
  const sys = String(systemPrompt || '').trim();
  if (sys) {
    out.push({ role: 'system', content: sys });
  }
  if (Array.isArray(messages)) {
    for (const msg of messages) {
      if (!msg || (msg.role !== 'user' && msg.role !== 'assistant' && msg.role !== 'system')) continue;
      const content = String(msg.content ?? '');
      if (!content && msg.role === 'assistant') continue;
      out.push({ role: msg.role, content });
    }
  }
  const userText = String(prompt ?? '');
  if (userText.trim()) {
    const last = out[out.length - 1];
    if (!(last && last.role === 'user' && last.content === userText)) {
      out.push({ role: 'user', content: userText });
    }
  }
  return out;
}

export function createCustomApiClient({
  coreReady,
  prismFrame,
  getRef,
  putRef,
  deleteRef,
  deriveVaultKey,
  fetchFn = globalThis.fetch,
} = {}) {
  let cachedSubkey = null;

  async function resolveVaultKey() {
    if (cachedSubkey instanceof Uint8Array && cachedSubkey.byteLength === 32) {
      return cachedSubkey;
    }
    if (typeof deriveVaultKey === 'function') {
      const k = await deriveVaultKey();
      if (!(k instanceof Uint8Array) || k.byteLength !== 32) {
        throw new Error('Custom API vault key derivation failed');
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
    const domain = enc.encode('uor.custom-api.vault/1:');
    const input = new Uint8Array(domain.length + secretBytes.length);
    input.set(domain, 0);
    input.set(secretBytes, domain.length);
    const digest = await crypto.subtle.digest('SHA-256', input);
    cachedSubkey = new Uint8Array(digest);
    return cachedSubkey;
  }

  async function readAllRecords() {
    const stored = await getRef(VAULT_REF_KEY);
    return Array.isArray(stored) ? stored.filter((r) => r && typeof r.id === 'string') : [];
  }

  async function writeAllRecords(records) {
    if (!records.length && typeof deleteRef === 'function') {
      await deleteRef(VAULT_REF_KEY);
      return;
    }
    await putRef(VAULT_REF_KEY, records);
  }

  async function sealSecret(apiKey) {
    if (!apiKey) return null;
    const [core, key] = await Promise.all([coreReady, resolveVaultKey()]);
    const plaintext = enc.encode(JSON.stringify({ apiKey }));
    const sealed = core.seal(key, plaintext);
    if (!(sealed instanceof Uint8Array) || sealed.byteLength < 28) {
      throw new Error('core.wasm ChaCha20-Poly1305 seal failed for API key');
    }
    return sealed;
  }

  async function openSecret(sealedApiKey) {
    if (!(sealedApiKey instanceof Uint8Array)) return '';
    const [core, key] = await Promise.all([coreReady, resolveVaultKey()]);
    const opened = core.open(key, sealedApiKey);
    if (!(opened instanceof Uint8Array)) {
      throw new Error('Failed to decrypt custom API key');
    }
    const parsed = JSON.parse(dec.decode(opened));
    return validateCustomApiKey(parsed?.apiKey || '');
  }

  async function listEndpoints() {
    const records = await readAllRecords();
    return records.map(toPublicRecord);
  }

  async function saveEndpoint({
    id,
    name,
    baseUrl,
    model,
    apiKey,
    clearApiKey = false,
    contextWindow = 32768,
  } = {}) {
    const cleanId = normalizeEndpointId(id);
    const cleanName = validateCustomApiName(name);
    const cleanBase = validateCustomApiBaseUrl(baseUrl);
    const cleanModel = validateCustomApiModel(model);
    const cleanKey = validateCustomApiKey(apiKey);
    const cw = Number(contextWindow);
    const cleanContextWindow =
      Number.isInteger(cw) && cw >= 1024 && cw <= 2000000 ? cw : 32768;

    const records = await readAllRecords();
    const existingIndex = records.findIndex((r) => r.id === cleanId);
    const existing = existingIndex >= 0 ? records[existingIndex] : null;
    const timestamp = new Date().toISOString();

    let sealedApiKey = existing?.sealedApiKey ?? null;
    let hasApiKey = Boolean(existing?.hasApiKey && sealedApiKey instanceof Uint8Array);
    if (clearApiKey) {
      sealedApiKey = null;
      hasApiKey = false;
    } else if (cleanKey) {
      sealedApiKey = await sealSecret(cleanKey);
      hasApiKey = true;
    }

    const nextRecord = {
      scheme: 'uor.custom-api.vault/1',
      alg: 'chacha20-poly1305',
      id: cleanId,
      name: cleanName,
      baseUrl: cleanBase,
      model: cleanModel,
      contextWindow: cleanContextWindow,
      hasApiKey,
      sealedApiKey,
      createdAt: existing?.createdAt || timestamp,
      updatedAt: timestamp,
    };

    if (existingIndex >= 0) {
      records[existingIndex] = nextRecord;
    } else {
      records.push(nextRecord);
    }
    await writeAllRecords(records);
    return {
      endpoint: toPublicRecord(nextRecord),
      endpoints: records.map(toPublicRecord),
    };
  }

  async function deleteEndpoint({ id } = {}) {
    const cleanId = String(id ?? '').trim();
    const records = await readAllRecords();
    const next = records.filter((r) => r.id !== cleanId);
    const deleted = next.length !== records.length;
    if (deleted) {
      await writeAllRecords(next);
    }
    return {
      deleted,
      endpoints: next.map(toPublicRecord),
    };
  }

  async function resolveEndpoint(id) {
    const cleanId = String(id ?? '').trim();
    if (!cleanId) return null;
    const records = await readAllRecords();
    const match = records.find((r) => r.id === cleanId);
    if (!match) return null;
    const apiKey = match.hasApiKey ? await openSecret(match.sealedApiKey) : '';
    return {
      ...toPublicRecord(match),
      apiKey,
    };
  }

  async function readErrorMessage(resp) {
    let text = '';
    try {
      text = await resp.text();
      const json = JSON.parse(text);
      const msg = json?.error?.message || json?.message || json?.error;
      if (typeof msg === 'string' && msg.trim()) {
        return msg.trim().slice(0, 400);
      }
    } catch {}
    return text ? text.trim().slice(0, 400) : `HTTP ${resp.status}`;
  }

  async function testEndpoint({ id, baseUrl, model, apiKey } = {}) {
    let resolvedBaseUrl = baseUrl ? validateCustomApiBaseUrl(baseUrl) : '';
    let resolvedModel = model ? validateCustomApiModel(model) : '';
    let resolvedKey = validateCustomApiKey(apiKey);
    if (id) {
      const saved = await resolveEndpoint(id);
      if (!saved) {
        return { ok: false, error: `Unknown custom endpoint: ${id}` };
      }
      if (!resolvedBaseUrl) resolvedBaseUrl = saved.baseUrl;
      if (!resolvedModel) resolvedModel = saved.model;
      if (!resolvedKey) resolvedKey = saved.apiKey;
    }
    if (!resolvedBaseUrl) {
      return { ok: false, error: 'API base URL is required' };
    }
    if (!resolvedModel) {
      return { ok: false, error: 'Model ID is required' };
    }

    const endpointUrl = resolveChatCompletionsUrl(resolvedBaseUrl);
    const headers = { 'Content-Type': 'application/json' };
    if (resolvedKey) {
      headers.Authorization = `Bearer ${resolvedKey}`;
    }

    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(new Error('Connection timed out after 15s')), 15000);
    const t0 = performance.now();
    try {
      const resp = await fetchFn(endpointUrl, {
        method: 'POST',
        headers,
        body: JSON.stringify({
          model: resolvedModel,
          messages: [{ role: 'user', content: 'Reply with OK.' }],
          max_tokens: 16,
          stream: false,
        }),
        credentials: 'omit',
        redirect: 'error',
        signal: controller.signal,
      });
      const latencyMs = Math.max(1, Math.round(performance.now() - t0));
      if (!resp.ok) {
        const msg = await readErrorMessage(resp);
        return { ok: false, status: resp.status, latencyMs, error: `HTTP ${resp.status}: ${msg}` };
      }
      const data = await resp.json().catch(() => null);
      const replyPreview = String(
        data?.choices?.[0]?.message?.content ??
          data?.choices?.[0]?.text ??
          'Connected',
      )
        .trim()
        .slice(0, 120);
      return {
        ok: true,
        status: resp.status,
        latencyMs,
        model: String(data?.model || resolvedModel),
        replyPreview: replyPreview || 'Connected',
      };
    } catch (err) {
      return {
        ok: false,
        error: String(err?.message || err),
      };
    } finally {
      clearTimeout(timer);
    }
  }

  async function generateChatCompletion({
    endpoint,
    systemPrompt = '',
    messages = [],
    prompt = '',
    maxNewTokens = 2048,
    stopSequences = ['</tool_call>'],
    signal,
    onToken,
  } = {}) {
    if (!endpoint?.baseUrl || !endpoint?.model) {
      throw new Error('Custom AI API endpoint is incomplete or removed.');
    }
    const endpointUrl = resolveChatCompletionsUrl(endpoint.baseUrl);
    const chatMessages = buildOpenAiMessages({ systemPrompt, messages, prompt });
    const headers = { 'Content-Type': 'application/json' };
    if (endpoint.apiKey) {
      headers.Authorization = `Bearer ${endpoint.apiKey}`;
    }

    const payload = {
      model: endpoint.model,
      messages: chatMessages,
      max_tokens: maxNewTokens,
      stream: true,
      ...(Array.isArray(stopSequences) && stopSequences.length ? { stop: stopSequences } : {}),
    };

    const resp = await fetchFn(endpointUrl, {
      method: 'POST',
      headers,
      body: JSON.stringify(payload),
      credentials: 'omit',
      redirect: 'error',
      signal,
    });
    if (!resp.ok) {
      const msg = await readErrorMessage(resp);
      throw new Error(`Custom AI API error (${resp.status}): ${msg}`);
    }

    const contentType = String(resp.headers?.get?.('content-type') || '').toLowerCase();
    let fullText = '';

    const stops = Array.isArray(stopSequences) ? stopSequences.filter((s) => typeof s === 'string' && s) : [];
    const applyStop = () => {
      for (const seq of stops) {
        const idx = fullText.indexOf(seq);
        if (idx >= 0) {
          fullText = fullText.slice(0, idx + seq.length);
          return true;
        }
      }
      return false;
    };

    if (contentType.includes('text/event-stream') && resp.body?.getReader) {
      const reader = resp.body.getReader();
      const utf8 = new TextDecoder('utf-8');
      let buffer = '';
      try {
        while (true) {
          const { done, value } = await reader.read();
          if (done) break;
          buffer += utf8.decode(value, { stream: true });
          const lines = buffer.split(/\r?\n/);
          buffer = lines.pop() ?? '';
          for (const rawLine of lines) {
            const line = rawLine.trim();
            if (!line || !line.startsWith('data:')) continue;
            const dataStr = line.slice(5).trim();
            if (dataStr === '[DONE]') {
              void reader.cancel().catch(() => {});
              return { fullText, weightsMatched: true };
            }
            let evt;
            try {
              evt = JSON.parse(dataStr);
            } catch {
              continue;
            }
            const delta = evt?.choices?.[0]?.delta?.content ?? evt?.choices?.[0]?.text ?? '';
            if (typeof delta === 'string' && delta) {
              fullText += delta;
              const hit = applyStop();
              onToken?.({ chunk: delta, fullText });
              if (hit) {
                void reader.cancel().catch(() => {});
                return { fullText, weightsMatched: true };
              }
            }
          }
        }
        if (buffer.trim().startsWith('data:')) {
          const tail = buffer.trim().slice(5).trim();
          if (tail && tail !== '[DONE]') {
            try {
              const evt = JSON.parse(tail);
              const delta = evt?.choices?.[0]?.delta?.content ?? evt?.choices?.[0]?.text ?? '';
              if (typeof delta === 'string' && delta) {
                fullText += delta;
                applyStop();
                onToken?.({ chunk: delta, fullText });
              }
            } catch {}
          }
        }
      } finally {
        reader.releaseLock();
      }
      return { fullText, weightsMatched: true };
    }

    const data = await resp.json();
    fullText = String(
      data?.choices?.[0]?.message?.content ?? data?.choices?.[0]?.text ?? '',
    );
    applyStop();
    if (fullText) {
      onToken?.({ chunk: fullText, fullText });
    }
    return { fullText, weightsMatched: true };
  }

  return {
    listEndpoints,
    saveEndpoint,
    deleteEndpoint,
    resolveEndpoint,
    testEndpoint,
    generateChatCompletion,
  };
}
