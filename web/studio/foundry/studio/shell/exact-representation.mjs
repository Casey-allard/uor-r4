// Typed adapter for FoundrySpecification.representationBoundary. Web Compression
// Streams implement RFC 1950; identities describe bytes, not optimization claims.
export const MAX_BYTES = 2 * 1024 * 1024;
const MAX_PACKAGE = 4 * 1024 * 1024;
export const DECODER_SPEC = "foundry/deflate-xor/1: RFC1950 inflate; without basis return inflated bytes; with basis require equal lengths and XOR corresponding bytes; verify exact original length and SHA256";
const encoder = new TextEncoder();
const messages = {
  invalidRepresentation: "The byte representation is malformed",
  identityMismatch: "The representation does not match its declared identity",
  missingBasis: "This representation requires its exact basis",
  basisMismatch: "The supplied basis does not match",
  unsupportedDecoder: "The decoder specification is unsupported",
  representationTooLarge: "The representation exceeds its byte limit",
  decodeFailed: "The compressed bytes cannot be reconstructed",
  representationCancelled: "Byte reconstruction was cancelled",
  representationTimedOut: "Byte reconstruction exceeded its deadline",
};
export class RepresentationError extends Error {
  constructor(code) { super(messages[code]); this.name = "RepresentationError"; this.code = code; }
}
const fail = code => { throw new RepresentationError(code); };
function bytes(value, maximum = MAX_BYTES) {
  try {
    const length = Object.getOwnPropertyDescriptor(Object.getPrototypeOf(Uint8Array.prototype), "byteLength").get.call(value);
    if (length > maximum) fail("representationTooLarge");
    const copy = new Uint8Array(length); Uint8Array.prototype.set.call(copy, value); return copy;
  } catch (error) { if (error instanceof RepresentationError) throw error; fail("invalidRepresentation"); }
}
function keys(value, names) {
  return value && typeof value === "object" && !Array.isArray(value) && Object.keys(value).sort().join(",") === names.sort().join(",");
}
const equal = (a, b) => a.length === b.length && a.every((value, index) => value === b[index]);
export async function byteIdentity(kind, value) {
  const data = bytes(value, MAX_PACKAGE);
  const digest = [...new Uint8Array(await crypto.subtle.digest("SHA-256", data))].map(v => v.toString(16).padStart(2, "0")).join("");
  return { kind, canonicalization: "raw-bytes", algorithm: "sha256", version: 1, digest, byteLength: data.length };
}
function validIdentity(value, kind, maximum) {
  return keys(value, ["kind", "canonicalization", "algorithm", "version", "digest", "byteLength"]) && value.kind === kind && value.canonicalization === "raw-bytes" && value.algorithm === "sha256" && value.version === 1 && /^[0-9a-f]{64}$/.test(value.digest) && Number.isSafeInteger(value.byteLength) && value.byteLength >= 0 && value.byteLength <= maximum;
}
const sameIdentity = (a, b) => a.kind === b.kind && a.canonicalization === b.canonicalization && a.algorithm === b.algorithm && a.version === b.version && a.digest === b.digest && a.byteLength === b.byteLength;
function base64(data) { let text = ""; for (let i = 0; i < data.length; i += 8192) text += String.fromCharCode(...data.subarray(i, i + 8192)); return btoa(text); }
function unbase64(text) {
  if (typeof text !== "string" || text.length > MAX_PACKAGE || !/^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$/.test(text)) fail("invalidRepresentation");
  const data = Uint8Array.from(atob(text), c => c.charCodeAt(0));
  if (base64(data) !== text) fail("invalidRepresentation");
  return data;
}
async function bounded(options, operation) {
  let deadline, signal;
  try {
    if (!options || typeof options !== "object" || Array.isArray(options) || Object.keys(options).some(key => key !== "deadlineMs" && key !== "signal")) fail("invalidRepresentation");
    deadline = options.deadlineMs ?? 30000; signal = options.signal;
    if (!Number.isInteger(deadline) || deadline < 1 || deadline > 30000 || (signal !== undefined && !(signal instanceof AbortSignal))) fail("invalidRepresentation");
  } catch { fail("invalidRepresentation"); }
  const controller = new AbortController();
  let timer, rejectStop;
  const stopped = new Promise((_, reject) => { rejectStop = reject; });
  const stop = code => { if (!controller.signal.aborted) { const error = new RepresentationError(code); controller.abort(error); rejectStop(error); } };
  const abort = () => stop("representationCancelled");
  try {
    signal?.addEventListener("abort", abort, { once: true });
    timer = setTimeout(() => stop("representationTimedOut"), deadline);
    if (signal?.aborted) abort();
    return await Promise.race([stopped, operation(controller.signal)]);
  } catch (error) {
    if (error instanceof RepresentationError) throw error;
    fail("decodeFailed");
  } finally { clearTimeout(timer); signal?.removeEventListener("abort", abort); controller.abort(); }
}
async function transform(data, compress, maximum, signal) {
  if (signal.aborted) throw signal.reason;
  const stream = new Blob([data]).stream().pipeThrough(compress ? new CompressionStream("deflate") : new DecompressionStream("deflate"), { signal });
  const reader = stream.getReader(); const chunks = []; let length = 0;
  try {
    while (true) {
      const next = await reader.read();
      if (next.done) break;
      length += next.value.length;
      if (length > maximum) fail("representationTooLarge");
      chunks.push(next.value);
    }
    const result = new Uint8Array(length); let offset = 0;
    for (const chunk of chunks) { result.set(chunk, offset); offset += chunk.length; }
    return result;
  } finally { await reader.cancel().catch(() => {}); reader.releaseLock(); }
}
function manifest(original, payload, decoder, basis, data) { return { spec: "foundry/byte-representation/1", original, payload, decoder, basis, data }; }
export async function packBytes(value, basisValue = null, options = {}) {
  const original = bytes(value); const basis = basisValue === null ? null : bytes(basisValue);
  if (basis !== null && basis.length !== original.length) fail("basisMismatch");
  return bounded(options, async signal => {
    const delta = original.slice();
    if (basis !== null) for (let i = 0; i < delta.length; i++) delta[i] ^= basis[i];
    const payload = await transform(delta, true, MAX_BYTES + 65536, signal);
    const result = manifest(await byteIdentity("original", original), await byteIdentity("payload", payload), await byteIdentity("decoder-specification", encoder.encode(DECODER_SPEC)), basis === null ? null : await byteIdentity("basis", basis), base64(payload));
    const archive = encoder.encode(JSON.stringify(result));
    if (archive.length > MAX_PACKAGE) fail("representationTooLarge");
    return { bytes: archive, identity: await byteIdentity("representation", archive), accounting: { originalBytes: original.length, payloadBytes: payload.length, representationBytes: archive.length, basisBytes: basis?.length ?? 0, reconstructionBytes: archive.length + (basis?.length ?? 0), mode: basis === null ? "self-contained" : "conditional-basis" } };
  });
}
export async function unpackBytes(value, expectedIdentity, basisValue = null, options = {}) {
  const archive = bytes(value, MAX_PACKAGE); const basis = basisValue === null ? null : bytes(basisValue);
  let expected;
  try { expected = structuredClone(expectedIdentity); } catch { fail("invalidRepresentation"); }
  if (!validIdentity(expected, "representation", MAX_PACKAGE)) fail("invalidRepresentation");
  let parsed;
  try { parsed = JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(archive)); } catch { fail("invalidRepresentation"); }
  if (!keys(parsed, ["spec", "original", "payload", "decoder", "basis", "data"]) || parsed.spec !== "foundry/byte-representation/1") fail("invalidRepresentation");
  const { original, payload, decoder, basis: basisId, data } = parsed;
  if (!validIdentity(original, "original", MAX_BYTES) || !validIdentity(payload, "payload", MAX_BYTES + 65536) || !validIdentity(decoder, "decoder-specification", MAX_BYTES) || (basisId !== null && !validIdentity(basisId, "basis", MAX_BYTES))) fail("invalidRepresentation");
  if (!equal(archive, encoder.encode(JSON.stringify(manifest(original, payload, decoder, basisId, data))))) fail("invalidRepresentation");
  if (basisId !== null && basis === null) fail("missingBasis");
  if (basisId === null && basis !== null) fail("basisMismatch");
  return bounded(options, async signal => {
    if (!sameIdentity(expected, await byteIdentity("representation", archive))) fail("identityMismatch");
    if (!sameIdentity(decoder, await byteIdentity("decoder-specification", encoder.encode(DECODER_SPEC)))) fail("unsupportedDecoder");
    if (basisId !== null && (!sameIdentity(basisId, await byteIdentity("basis", basis)) || basis.length !== original.byteLength)) fail("basisMismatch");
    const encoded = unbase64(data);
    if (!sameIdentity(payload, await byteIdentity("payload", encoded))) fail("identityMismatch");
    const result = await transform(encoded, false, original.byteLength, signal);
    if (basis !== null) for (let i = 0; i < result.length; i++) result[i] ^= basis[i];
    if (!sameIdentity(original, await byteIdentity("original", result))) fail("identityMismatch");
    return result;
  });
}
