// Representation adapter for CompositionProtocol.boundary. Admission belongs
// solely to the generated guest; this module serializes declarations, not rules.
const fault = (code, message) => Object.assign(new Error(message), { code });
const ids = ["vocabulary", "outputs", "state", "events", "errors", "effects", "conflicts", "targets", "accessibility", "quality"];
export function encodeComposition(policy, features) {
  const words = [];
  const scalar = value => {
    if (!Number.isInteger(value) || value < 0 || value > 0xffffffff) throw fault("malformedComposition", "Composition fields must be unsigned 32-bit integers");
    words.push(value);
  };
  const list = (values, emit, maximum = 64) => {
    if (!Array.isArray(values) || values.length > maximum) throw fault("malformedComposition", "Composition list exceeds its wire bound");
    scalar(values.length);
    for (const value of values) emit(value);
  };
  const shape = (value, fields) => {
    if (!value || typeof value !== "object" || Array.isArray(value) || Object.keys(value).sort().join(",") !== [...fields].sort().join(",")) throw fault("malformedComposition", "Composition fields do not match the wire schema");
  };
  shape(policy, ["cpu", "memory", "grants", "targets", "evidence"]);
  scalar(policy.cpu); scalar(policy.memory);
  for (const field of ["grants", "targets", "evidence"]) list(policy[field], scalar);
  list(features, feature => {
    shape(feature, ["key", "version", "cpu", "memory", ...ids, "needs", "inputs"]);
    for (const field of ["key", "version", "cpu", "memory"]) scalar(feature[field]);
    for (const field of ids) list(feature[field], scalar);
    list(feature.needs, reference => { shape(reference, ["key", "version"]); scalar(reference.key); scalar(reference.version); });
    list(feature.inputs, input => { shape(input, ["producer", "schema"]); shape(input.producer, ["key", "version"]); scalar(input.producer.key); scalar(input.producer.version); scalar(input.schema); });
  }, 32);
  if (words.length * 4 > 16384) throw fault("compositionTooLarge", "Composition payload exceeds 32768 hexadecimal characters");
  const bytes = new Uint8Array(words.length * 4);
  const view = new DataView(bytes.buffer);
  words.forEach((word, i) => view.setUint32(i * 4, word, true));
  const hex = Array.from(bytes, byte => byte.toString(16).padStart(2, "0")).join("");
  return new TextEncoder().encode("compose " + hex);
}

const refusals = new Set(["emptyComposition", "invalidDeclaration", "duplicateFeature", "missingDependency", "incompatibleInput", "featureConflict", "permissionDenied", "unsupportedTarget", "missingEvidence", "budgetExceeded", "malformedComposition", "compositionTooLarge"]);
export async function admitWithGuest(guest, policy, features) {
  let request;
  try { request = encodeComposition(policy, features); }
  catch (error) { return { ok: false, code: error.code, error: error.message }; }
  let response;
  try { response = await guest.dispatch(request); }
  catch { return { ok: false, code: "compositionRuntimeFailed", error: "The composition guest could not execute" }; }
  try {
    if (!(response instanceof Uint8Array) || response.length > 64) throw Error();
    const code = new TextDecoder("utf-8", { fatal: true }).decode(response);
    if (code === "accepted") return { ok: true };
    if (refusals.has(code)) return { ok: false, code, error: `Composition refused: ${code}` };
  } catch {}
  return { ok: false, code: "invalidCompositionResponse", error: "The composition guest returned an unknown response" };
}
