// Byte representation for EventProtocol.boundary; the guest decides admission.
import { encodeComposition } from "./composition-codec.mjs";
const fail = code => Object.assign(new Error(`Event request refused: ${code}`), { code });
const shape = (value, fields) => {
  if (!value || typeof value !== "object" || Array.isArray(value) || Object.keys(value).sort().join(",") !== [...fields].sort().join(",")) throw fail("malformedEvent");
};
export function encodeEventRequest(input) {
  shape(input, ["policy", "features", "contract", "generation", "sequence", "envelope"]);
  let prefix;
  try { prefix = new TextDecoder().decode(encodeComposition(input.policy, input.features)).slice(8); }
  catch (error) { throw fail(error.code === "compositionTooLarge" ? "eventTooLarge" : "malformedEvent"); }
  const words = [];
  const scalar = value => { if (!Number.isInteger(value) || value < 0 || value > 0xffffffff) throw fail("malformedEvent"); words.push(value); };
  const ref = value => { shape(value, ["key", "version"]); scalar(value.key); scalar(value.version); };
  const list = (values, write, maximum) => {
    if (!Array.isArray(values) || values.length > maximum) throw fail("malformedEvent");
    scalar(values.length); for (const value of values) write(value);
  };
  const contract = input.contract;
  shape(contract, ["key", "version", "kind", "maximumBytes", "producers", "consumers"]);
  for (const field of ["key", "version", "kind", "maximumBytes"]) scalar(contract[field]);
  for (const field of ["producers", "consumers"]) list(contract[field], ref, 64);
  scalar(input.generation); scalar(input.sequence);
  const envelope = input.envelope;
  shape(envelope, ["key", "version", "generation", "sequence", "producer", "consumer", "payload"]);
  for (const field of ["key", "version", "generation", "sequence"]) scalar(envelope[field]);
  ref(envelope.producer); ref(envelope.consumer);
  const payload = envelope.payload; shape(payload, ["kind", "value"]);
  if (payload.kind === "natural") { scalar(0); scalar(payload.value); }
  else if (payload.kind === "flag") { scalar(1); if (typeof payload.value !== "boolean") throw fail("malformedEvent"); scalar(Number(payload.value)); }
  else if (payload.kind === "bytes") { scalar(2); list(payload.value, scalar, 1024); }
  else throw fail("malformedEvent");
  const bytes = new Uint8Array(words.length * 4), view = new DataView(bytes.buffer);
  words.forEach((word, i) => view.setUint32(i * 4, word, true));
  const payloadHex = prefix + Array.from(bytes, byte => byte.toString(16).padStart(2, "0")).join("");
  if (payloadHex.length > 32768) throw fail("eventTooLarge");
  return new TextEncoder().encode("event " + payloadHex);
}
const eventErrors = ["invalidEventContract", "invalidEventContext", "staleGeneration", "staleSequence", "wrongEvent", "undeclaredRoute", "invalidPayload", "malformedEvent", "eventTooLarge"];
const compositionErrors = ["emptyComposition", "invalidDeclaration", "duplicateFeature", "missingDependency", "incompatibleInput", "featureConflict", "permissionDenied", "unsupportedTarget", "missingEvidence", "budgetExceeded"];
const refusals = new Set([...eventErrors, ...compositionErrors.map(code => "compositionRejected:" + code)]);
export function decodeEventResponse(response) {
  try {
    if (!(response instanceof Uint8Array) || response.length > 64) throw Error();
    const text = new TextDecoder("utf-8", { fatal: true }).decode(response);
    if (/^ok [0-9a-f]{8}$/.test(text)) {
      const bytes = Uint8Array.from(text.slice(3).match(/../g), pair => Number.parseInt(pair, 16));
      return { ok: true, nextSequence: new DataView(bytes.buffer).getUint32(0, true) };
    }
    if (refusals.has(text)) return { ok: false, code: text };
  } catch {}
  return { ok: false, code: "invalidEventResponse" };
}
export async function admitEventWithGuest(guest, input) {
  let request;
  try { request = encodeEventRequest(input); }
  catch (error) { return { ok: false, code: ["malformedEvent", "eventTooLarge"].includes(error?.code) ? error.code : "malformedEvent" }; }
  let response;
  try { response = await guest.dispatch(request); }
  catch { return { ok: false, code: "eventRuntimeFailed" }; }
  return decodeEventResponse(response);
}
