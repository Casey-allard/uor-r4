// Wire-only adapter for WorkProtocol; all state transitions execute in the guest.
const fields = ["owner", "revision", "capacity", "spent", "reserved", "checkpoint", "remaining", "phase"];
const errors = new Set(["invalidState", "wrongOwner", "staleRevision", "invalidPhase", "invalidAmount", "budgetExceeded", "reservationMissing", "reservationExceeded", "obligationExceeded", "checkpointNotAdvanced", "unfinishedWork", "invalidOwner", "invalidWorkRequest", "workCounterExhausted"]);
const actions = { reserve: [0, "amount"], start: [1], progress: [2, "cost", "units", "checkpoint"], pause: [3], resume: [4], finish: [5], transfer: [6, "owner"] };
const invalid = () => { throw Object.assign(new Error("Invalid work request"), { code: "invalidWorkRequest" }); };
function exact(value, names) { if (!value || typeof value !== "object" || Array.isArray(value) || Object.keys(value).sort().join(",") !== [...names].sort().join(",")) invalid(); }
function scalar(value) { if (!Number.isInteger(value) || value < 0 || value > 0xffffffff) invalid(); return value; }
function encode(words) { const bytes = new Uint8Array(words.length * 4); const view = new DataView(bytes.buffer); words.forEach((word, index) => view.setUint32(index * 4, scalar(word), true)); return new TextEncoder().encode("work " + [...bytes].map(byte => byte.toString(16).padStart(2, "0")).join("")); }
export function encodeWorkRequest(value) {
  try {
    const input = structuredClone(value);
    if (input?.kind === "initialize") { exact(input, ["kind", "owner", "capacity", "remaining"]); return encode([0, input.owner, input.capacity, input.remaining]); }
    exact(input, ["kind", "state", "owner", "revision", "action"]);
    if (input.kind !== "transition") invalid();
    exact(input.state, fields);
    const action = Object.hasOwn(actions, input.action?.kind) ? actions[input.action.kind] : null;
    if (!action) invalid();
    const [code, ...parameters] = action;
    exact(input.action, ["kind", ...parameters]);
    const args = parameters.map(name => input.action[name]); while (args.length < 3) args.push(0);
    return encode([1, ...fields.map(name => input.state[name]), input.owner, input.revision, code, ...args]);
  } catch { invalid(); }
}
export function decodeWorkResponse(bytes) {
  try {
    if (!(bytes instanceof Uint8Array) || bytes.length > 128) throw Error();
    const text = new TextDecoder("utf-8", { fatal: true }).decode(bytes);
    if (errors.has(text)) return { ok: false, code: text };
    if (!/^ok [0-9a-f]{64}$/.test(text)) throw Error();
    const raw = Uint8Array.from(text.slice(3).match(/../g), pair => parseInt(pair, 16));
    const view = new DataView(raw.buffer); const state = Object.fromEntries(fields.map((name, index) => [name, view.getUint32(index * 4, true)]));
    if (state.phase > 3) throw Error();
    return { ok: true, state };
  } catch { return { ok: false, code: "workRuntimeFailed" }; }
}
export async function executeWorkWithGuest(kernel, input) {
  let request;
  try { request = encodeWorkRequest(input); } catch { return { ok: false, code: "invalidWorkRequest" }; }
  try { return decodeWorkResponse(await kernel.dispatch(request)); } catch { return { ok: false, code: "workRuntimeFailed" }; }
}
