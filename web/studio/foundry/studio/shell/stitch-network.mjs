// Stitch opt-in peer networking layer aligned with afflom/stitch (stitch.signal/1),
// Ed25519 identity + replay-protected signaling + stitch:// fetch + NE2000 frame relay.
// Zero external network traffic runs on boot unless explicitly connected by user action.

const PROFILE = 'stitch.signal/1';
const ALLOWED_KINDS = new Set(['announce', 'hello', 'ready', 'offer', 'answer', 'ice']);
const MAX_FETCH_BYTES = 8192;

const enc = new TextEncoder();
const dec = new TextDecoder();

function b64urlEncode(bytes) {
  let s = '';
  for (let i = 0; i < bytes.length; i += 1) s += String.fromCharCode(bytes[i]);
  return btoa(s).replaceAll('+', '-').replaceAll('/', '_').replace(/=+$/, '');
}

function b64urlDecode(value, expectedLen = null) {
  if (typeof value !== 'string' || !/^[A-Za-z0-9_-]+$/.test(value)) {
    throw new Error('signal-encoding');
  }
  const raw = Uint8Array.from(
    atob(value.replaceAll('-', '+').replaceAll('_', '/')),
    (c) => c.charCodeAt(0),
  );
  if (expectedLen != null && (raw.length !== expectedLen || b64urlEncode(raw) !== value)) {
    throw new Error('signal-encoding');
  }
  return raw;
}

function canonicalSort(value) {
  if (Array.isArray(value)) return value.map(canonicalSort);
  if (value && typeof value === 'object') {
    const out = {};
    for (const k of Object.keys(value).sort()) {
      if (value[k] !== undefined) out[k] = canonicalSort(value[k]);
    }
    return out;
  }
  return value;
}

export function canonicalJsonBytes(value) {
  return enc.encode(JSON.stringify(canonicalSort(value)));
}

async function computeKeyId(rawPublicKey) {
  const hash = new Uint8Array(await crypto.subtle.digest('SHA-256', rawPublicKey));
  return 'key-' + b64urlEncode(hash);
}

export class StitchSignalSession {
  constructor(channelKey = 'uor-r4-studio') {
    this.channelKey = channelKey;
    this.session = b64urlEncode(crypto.getRandomValues(new Uint8Array(24)));
    this.active = true;
    this.sequence = 0;
    this.peers = new Map(); // peerId -> { session, lastSequence, publicKeyRaw }
    this.identityPromise = null;
  }

  identity() {
    return (this.identityPromise ??= (async () => {
      try {
        const pair = await crypto.subtle.generateKey({ name: 'Ed25519' }, false, ['sign', 'verify']);
        const raw = new Uint8Array(await crypto.subtle.exportKey('raw', pair.publicKey));
        const id = await computeKeyId(raw);
        return { algorithm: 'Ed25519', pair, raw, publicKeyText: b64urlEncode(raw), id };
      } catch {
        // Graceful fallback only if browser build disables WebCrypto Ed25519.
        const pair = await crypto.subtle.generateKey(
          { name: 'ECDSA', namedCurve: 'P-256' },
          false,
          ['sign', 'verify'],
        );
        const raw = new Uint8Array(await crypto.subtle.exportKey('raw', pair.publicKey));
        const id = await computeKeyId(raw);
        return { algorithm: 'ECDSA-P256', pair, raw, publicKeyText: b64urlEncode(raw), id };
      }
    })());
  }

  close() {
    this.active = false;
    this.peers.clear();
  }

  async sign(payload) {
    if (!this.active) throw new Error('signal-closed');
    const copy = structuredClone(payload);
    if (!ALLOWED_KINDS.has(copy?.kind)) throw new Error('signal-kind');
    const id = await this.identity();
    const to = copy.to ?? null;
    const targetSession = copy.targetSession ?? (to ? this.peers.get(to)?.session ?? null : null);
    if (copy.kind !== 'announce' && (!to || !targetSession)) {
      throw new Error('signal-unbound-peer');
    }
    delete copy.to;
    delete copy.targetSession;
    const body = {
      profile: PROFILE,
      channelKey: this.channelKey,
      from: id.id,
      algorithm: id.algorithm,
      publicKey: id.publicKeyText,
      session: this.session,
      sequence: ++this.sequence,
      to,
      targetSession,
      payload: copy,
    };
    const sigParams = id.algorithm === 'Ed25519' ? 'Ed25519' : { name: 'ECDSA', hash: 'SHA-256' };
    const sigBuf = await crypto.subtle.sign(sigParams, id.pair.privateKey, canonicalJsonBytes(body));
    return { ...body, signature: b64urlEncode(new Uint8Array(sigBuf)) };
  }

  async verify(msg) {
    try {
      if (!this.active || !msg || typeof msg !== 'object') return null;
      if (msg.profile !== PROFILE || msg.channelKey !== this.channelKey) return null;
      if (!ALLOWED_KINDS.has(msg.payload?.kind)) return null;
      if (!Number.isSafeInteger(msg.sequence) || msg.sequence < 1) return null;
      b64urlDecode(msg.session, 24);
      const id = await this.identity();
      if (msg.from === id.id) return null;
      if (
        msg.payload.kind === 'announce'
          ? msg.to !== null || msg.targetSession !== null
          : msg.to !== id.id || msg.targetSession !== this.session
      ) {
        return null;
      }
      const raw = b64urlDecode(msg.publicKey, msg.algorithm === 'ECDSA-P256' ? 65 : 32);
      if ((await computeKeyId(raw)) !== msg.from) return null;
      const sigBytes = b64urlDecode(msg.signature, 64);
      const { signature: _, ...body } = msg;
      const importAlg =
        msg.algorithm === 'ECDSA-P256' ? { name: 'ECDSA', namedCurve: 'P-256' } : 'Ed25519';
      const verifyAlg =
        msg.algorithm === 'ECDSA-P256' ? { name: 'ECDSA', hash: 'SHA-256' } : 'Ed25519';
      const key = await crypto.subtle.importKey('raw', raw, importAlg, false, ['verify']);
      const valid = await crypto.subtle.verify(verifyAlg, key, sigBytes, canonicalJsonBytes(body));
      if (!valid) return null;

      const prev = this.peers.get(msg.from);
      if (prev) {
        if (prev.session !== msg.session || msg.sequence <= prev.lastSequence) return null;
        prev.lastSequence = msg.sequence;
      } else {
        this.peers.set(msg.from, { session: msg.session, lastSequence: msg.sequence, raw });
      }
      return msg;
    } catch {
      return null;
    }
  }
}

export function createStitchNetwork({ emit, resolveLocalResource, onEthFrame } = {}) {
  let channelKey = 'uor-r4-studio';
  let signalSession = new StitchSignalSession(channelKey);
  let connected = false;
  let ethernetSharing = false;
  let relayUrl = null;
  let bc = null;
  let ws = null;
  const rtcPeers = new Map(); // peerId -> { pc, dc, state }
  const discoveredPeers = new Map(); // peerId -> { peerId, session, seenAt, transport }
  const pendingFetch = new Map(); // reqId -> { resolve, timer }

  let identityCache = null;
  const ensureIdentity = async () => {
    if (!identityCache) {
      identityCache = await signalSession.identity();
    }
    return identityCache;
  };
  void ensureIdentity().catch(() => {});

  function emitStatus() {
    emit?.('stitch:status-changed', getStatusSync());
  }

  function getStatusSync() {
    return {
      profile: PROFILE,
      connected,
      channelKey,
      relayUrl,
      ethernetSharing,
      peerId: identityCache?.id || null,
      algorithm: identityCache?.algorithm || 'Ed25519',
      peers: Array.from(discoveredPeers.values()),
      webrtcPeers: Array.from(rtcPeers.entries()).map(([id, p]) => ({
        peerId: id,
        state: p.dc?.readyState || p.pc?.connectionState || 'connecting',
      })),
    };
  }

  async function getStatus() {
    await ensureIdentity();
    return getStatusSync();
  }

  function handlePeerMessage(fromPeerId, rawData) {
    let msg;
    try {
      msg = typeof rawData === 'string' ? JSON.parse(rawData) : rawData;
    } catch {
      return;
    }
    if (!msg || typeof msg !== 'object') return;
    if (msg.type === 'eth_frame' && ethernetSharing && Array.isArray(msg.frame)) {
      onEthFrame?.(new Uint8Array(msg.frame));
      return;
    }
    if (msg.type === 'stitch_fetch_req' && typeof msg.id === 'string' && typeof msg.path === 'string') {
      void (async () => {
        const res = (await resolveLocalResource?.(msg.path, msg.method || 'GET')) || {
          ok: false,
          status: 404,
          error: 'Resource not found',
        };
        const payload = {
          type: 'stitch_fetch_res',
          id: msg.id,
          path: msg.path,
          to: fromPeerId,
          ok: Boolean(res.ok),
          status: res.status ?? (res.ok ? 200 : 404),
          body_preview: String(res.body_preview || '').slice(0, MAX_FETCH_BYTES),
          error: res.error || undefined,
        };
        sendPeerPacket(fromPeerId, payload);
      })();
      return;
    }
    if (msg.type === 'stitch_fetch_res' && typeof msg.id === 'string') {
      const waiter = pendingFetch.get(msg.id);
      if (waiter) {
        clearTimeout(waiter.timer);
        pendingFetch.delete(msg.id);
        waiter.resolve({
          ok: Boolean(msg.ok),
          status: msg.status ?? 200,
          url: `stitch://${fromPeerId}/${msg.path || ''}`,
          body_preview: String(msg.body_preview || '').slice(0, MAX_FETCH_BYTES),
          ...(msg.error ? { error: msg.error } : {}),
        });
      }
    }
  }

  function sendPeerPacket(targetPeerId, packet) {
    const entry = rtcPeers.get(targetPeerId);
    if (entry?.dc?.readyState === 'open') {
      entry.dc.send(JSON.stringify(packet));
      return true;
    }
    if (bc) {
      bc.postMessage({ kind: 'packet', from: identityCache?.id, to: targetPeerId, packet });
      return true;
    }
    return false;
  }

  function sendSignalEnvelope(envelope) {
    if (bc) {
      bc.postMessage({ kind: 'signal', envelope });
    }
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify(envelope));
    }
  }

  function ensureRtcPeer(remotePeerId, initiator) {
    if (rtcPeers.has(remotePeerId)) return rtcPeers.get(remotePeerId);
    if (typeof RTCPeerConnection === 'undefined') return null;
    const pc = new RTCPeerConnection({ iceServers: [] });
    const entry = { pc, dc: null };
    rtcPeers.set(remotePeerId, entry);

    const wireDc = (dc) => {
      entry.dc = dc;
      dc.onopen = () => emitStatus();
      dc.onclose = () => emitStatus();
      dc.onmessage = (ev) => handlePeerMessage(remotePeerId, ev.data);
    };

    if (initiator) {
      wireDc(pc.createDataChannel('stitch'));
      void (async () => {
        const offer = await pc.createOffer();
        await pc.setLocalDescription(offer);
        const signed = await signalSession.sign({
          kind: 'offer',
          to: remotePeerId,
          sdp: pc.localDescription?.sdp || offer.sdp,
        });
        sendSignalEnvelope(signed);
      })().catch(() => {});
    } else {
      pc.ondatachannel = (ev) => wireDc(ev.channel);
    }

    pc.onicecandidate = (ev) => {
      if (!ev.candidate) return;
      void signalSession
        .sign({ kind: 'ice', to: remotePeerId, candidate: ev.candidate.toJSON() })
        .then(sendSignalEnvelope)
        .catch(() => {});
    };

    pc.onconnectionstatechange = () => {
      if (['failed', 'closed'].includes(pc.connectionState)) {
        rtcPeers.delete(remotePeerId);
      }
      emitStatus();
    };
    return entry;
  }

  async function processIncomingSignal(envelope) {
    const verified = await signalSession.verify(envelope);
    if (!verified) return false;
    const { from, payload } = verified;
    discoveredPeers.set(from, {
      peerId: from,
      session: verified.session,
      seenAt: Date.now(),
      transport: ws ? 'websocket+webrtc' : 'broadcast-channel+webrtc',
    });

    const localId = await ensureIdentity();
    if (payload.kind === 'announce') {
      const hello = await signalSession.sign({ kind: 'hello', to: from, targetSession: verified.session });
      sendSignalEnvelope(hello);
      if (localId.id < from) {
        ensureRtcPeer(from, true);
      }
    } else if (payload.kind === 'hello' && localId.id < from) {
      ensureRtcPeer(from, true);
    } else if (payload.kind === 'offer' && typeof payload.sdp === 'string') {
      const entry = ensureRtcPeer(from, false);
      if (entry) {
        await entry.pc.setRemoteDescription({ type: 'offer', sdp: payload.sdp });
        const answer = await entry.pc.createAnswer();
        await entry.pc.setLocalDescription(answer);
        const signed = await signalSession.sign({
          kind: 'answer',
          to: from,
          targetSession: verified.session,
          sdp: entry.pc.localDescription?.sdp || answer.sdp,
        });
        sendSignalEnvelope(signed);
      }
    } else if (payload.kind === 'answer' && typeof payload.sdp === 'string') {
      const entry = rtcPeers.get(from);
      if (entry) {
        await entry.pc.setRemoteDescription({ type: 'answer', sdp: payload.sdp });
      }
    } else if (payload.kind === 'ice' && payload.candidate) {
      const entry = rtcPeers.get(from);
      if (entry) {
        await entry.pc.addIceCandidate(payload.candidate).catch(() => {});
      }
    }
    emitStatus();
    return true;
  }

  async function joinChannel({ channelKey: nextChannel = 'uor-r4-studio', relayUrl: nextRelay = null, ethernet = false } = {}) {
    await disconnect();
    channelKey = String(nextChannel || 'uor-r4-studio').trim() || 'uor-r4-studio';
    ethernetSharing = Boolean(ethernet);
    signalSession = new StitchSignalSession(channelKey);
    identityCache = await signalSession.identity();
    connected = true;

    if (typeof BroadcastChannel !== 'undefined') {
      bc = new BroadcastChannel(`${PROFILE}:${channelKey}`);
      bc.onmessage = (ev) => {
        const msg = ev.data;
        if (!msg) return;
        if (msg.kind === 'signal' && msg.envelope) {
          void processIncomingSignal(msg.envelope).catch(() => {});
        } else if (msg.kind === 'packet' && msg.to === identityCache?.id && msg.packet) {
          handlePeerMessage(msg.from, msg.packet);
        }
      };
    }

    if (nextRelay) {
      const parsed = new URL(String(nextRelay));
      if (!['wss:', 'ws:'].includes(parsed.protocol)) {
        throw new Error('Stitch relay URL must use wss:// or ws://');
      }
      if (parsed.protocol === 'ws:' && !['127.0.0.1', 'localhost'].includes(parsed.hostname)) {
        throw new Error('Insecure ws:// relay is allowed for loopback testing only');
      }
      relayUrl = parsed.href;
      ws = new WebSocket(relayUrl);
      ws.onmessage = (ev) => {
        try {
          const parsedMsg = JSON.parse(String(ev.data || ''));
          void processIncomingSignal(parsedMsg).catch(() => {});
        } catch {}
      };
      ws.onopen = () => {
        void signalSession.sign({ kind: 'announce' }).then(sendSignalEnvelope).catch(() => {});
      };
    } else {
      relayUrl = null;
    }

    const announce = await signalSession.sign({ kind: 'announce' });
    sendSignalEnvelope(announce);
    emitStatus();
    return getStatus();
  }

  async function disconnect() {
    connected = false;
    ethernetSharing = false;
    relayUrl = null;
    if (bc) {
      bc.close();
      bc = null;
    }
    if (ws) {
      ws.close();
      ws = null;
    }
    for (const { pc } of rtcPeers.values()) {
      try {
        pc.close();
      } catch {}
    }
    rtcPeers.clear();
    discoveredPeers.clear();
    for (const waiter of pendingFetch.values()) {
      clearTimeout(waiter.timer);
      waiter.resolve({ ok: false, status: 499, error: 'Stitch network disconnected' });
    }
    pendingFetch.clear();
    signalSession.close();
    emitStatus();
    return getStatus();
  }

  function broadcastEthFrame(frameBytes) {
    if (!connected || !ethernetSharing || !frameBytes) return false;
    const frame = Array.from(frameBytes instanceof Uint8Array ? frameBytes : new Uint8Array(frameBytes));
    let sent = 0;
    for (const peerId of rtcPeers.keys()) {
      if (sendPeerPacket(peerId, { type: 'eth_frame', from: identityCache?.id, frame })) {
        sent += 1;
      }
    }
    if (bc) {
      for (const peerId of discoveredPeers.keys()) {
        if (!rtcPeers.has(peerId)) {
          bc.postMessage({
            kind: 'packet',
            from: identityCache?.id,
            to: peerId,
            packet: { type: 'eth_frame', from: identityCache?.id, frame },
          });
          sent += 1;
        }
      }
    }
    return sent > 0;
  }

  async function fetchPeer(stitchUrl, { method = 'GET', timeoutMs = 4000 } = {}) {
    const raw = String(stitchUrl || '').trim();
    if (!raw.startsWith('stitch://')) {
      return { ok: false, error: 'Expected stitch://<peer>/<path> URL' };
    }
    const rest = raw.slice('stitch://'.length);
    const slash = rest.indexOf('/');
    const peerId = slash === -1 ? rest : rest.slice(0, slash);
    const path = slash === -1 ? '' : rest.slice(slash + 1);
    const id = await ensureIdentity();

    if (peerId === 'local' || peerId === 'self' || peerId === id.id) {
      const localRes = await resolveLocalResource?.(path, method);
      if (!localRes) {
        return { ok: false, status: 404, url: raw, error: `Local Stitch resource not found: ${path}` };
      }
      return {
        ok: Boolean(localRes.ok),
        status: localRes.status ?? (localRes.ok ? 200 : 404),
        url: raw,
        body_preview: String(localRes.body_preview || '').slice(0, MAX_FETCH_BYTES),
        ...(localRes.error ? { error: localRes.error } : {}),
      };
    }

    if (!connected || (!discoveredPeers.has(peerId) && !rtcPeers.has(peerId))) {
      return {
        ok: false,
        status: 404,
        url: raw,
        error: `Stitch peer ${peerId} is not connected in active channel "${channelKey}".`,
      };
    }

    const reqId = crypto.randomUUID();
    return new Promise((resolve) => {
      const timer = setTimeout(() => {
        pendingFetch.delete(reqId);
        resolve({ ok: false, status: 504, url: raw, error: `Timed out waiting for peer ${peerId}` });
      }, timeoutMs);
      pendingFetch.set(reqId, { resolve, timer });
      const ok = sendPeerPacket(peerId, { type: 'stitch_fetch_req', id: reqId, path, method });
      if (!ok) {
        clearTimeout(timer);
        pendingFetch.delete(reqId);
        resolve({ ok: false, status: 503, url: raw, error: `No active transport to peer ${peerId}` });
      }
    });
  }

  return {
    getStatus,
    joinChannel,
    disconnect,
    fetchPeer,
    broadcastEthFrame,
    onOutboundFrame: broadcastEthFrame,
  };
}
