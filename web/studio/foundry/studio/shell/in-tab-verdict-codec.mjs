// Exact representation only; InTabKernel.verdictCode, run in the guest, owns every verdict.
// The caller states what it derived and what its kernels answered. It never states that two addresses agree: both
// are sent whole and the guest compares them. No field carries what a server said.
const addresses = ['libraryPinned', 'libraryDerived', 'bundleNamed', 'bundleDerived', 'kernelPinned', 'kernelDerived', 'specStatement', 'checkedStatement'];
const flags = ['hasBytes', 'leanRan', 'leanAccepted', 'secondRan', 'secondAccepted'];
const order = ['version', 'hasBytes', ...addresses, 'axiomsDeclared', 'axiomsObserved', 'leanRan', 'leanAccepted', 'secondRan', 'secondAccepted'];
const errors = new Set(['malformedVerdictRequest', 'verdictRequestTooLarge']);
const verdicts = ['proved', 'kernel-checked', 'checking', 'refused'];
const allowed = new Map([['propext', 1], ['Classical.choice', 2], ['Quot.sound', 4]]);
const fail = () => { throw Object.assign(new Error('Invalid verdict request'), { code: 'malformedVerdictRequest' }); };
const uint = (n) => { if (!Number.isInteger(n) || n < 0 || n > 0xffffffff) fail(); return n; };
const flag = (n) => { if (n !== 0 && n !== 1) fail(); return n; };
const address = (x) => { if (!Array.isArray(x) || x.length !== 8) fail(); return x.map(uint); };
const hex = (words) => { const bytes = new Uint8Array(words.length * 4), view = new DataView(bytes.buffer); words.forEach((n, i) => view.setUint32(i * 4, uint(n), true)); return [...bytes].map((b) => b.toString(16).padStart(2, '0')).join(''); };

// A SHA-256 address as the eight words the wire carries, all 256 bits.
export function verdictAddressWords(value) {
  if (typeof value !== 'string' || !/^sha256:[0-9a-f]{64}$/.test(value)) fail();
  const bytes = Uint8Array.from(value.slice(7).match(/../g), (s) => parseInt(s, 16)), view = new DataView(bytes.buffer);
  return Array.from({ length: 8 }, (_, i) => view.getUint32(i * 4, true));
}
// A set of axiom names as the wire's bit set. Any axiom outside the three the app admits sets 8, which the model refuses.
export function verdictAxiomSet(names) {
  if (!Array.isArray(names) || names.some((n) => typeof n !== 'string')) fail();
  return names.reduce((set, n) => set | (allowed.get(n) ?? 8), 0);
}
export function encodeVerdictRequest(raw) {
  try {
    const input = structuredClone(raw);
    if (!input || typeof input !== 'object' || Array.isArray(input) || Object.keys(input).sort().join() !== [...order].sort().join()) fail();
    if (input.version !== 1) fail();
    const words = order.flatMap((k) => addresses.includes(k) ? address(input[k]) : flags.includes(k) ? [flag(input[k])] : [uint(input[k])]);
    return new TextEncoder().encode('verdict ' + hex(words));
  } catch { fail(); }
}
export function decodeVerdictResponse(raw) {
  try {
    // Anything that is not bytes makes the decoder throw, and the two shapes below admit at most 27 bytes, so
    // neither is stated as a separate rule.
    const text = new TextDecoder('utf-8', { fatal: true }).decode(raw);
    if (errors.has(text)) return { ok: false, code: text };
    if (!/^ok [0-9a-f]{24}$/.test(text)) throw Error();
    const bytes = Uint8Array.from(text.slice(3).match(/../g), (s) => parseInt(s, 16)), view = new DataView(bytes.buffer);
    const [version, code, word] = [0, 1, 2].map((i) => view.getUint32(i * 4, true));
    // The model shows each code under its own word; a reply that pairs them otherwise is not the guest's.
    if (version !== 1 || code > 3 || word !== code) throw Error();
    return { ok: true, code, verdict: verdicts[code] };
  } catch { return { ok: false, code: 'invalidVerdictResponse' }; }
}
export async function verdictWithGuest(ready, input) {
  let bytes;
  try { bytes = encodeVerdictRequest(input); } catch { return { ok: false, code: 'malformedVerdictRequest' }; }
  try { const guest = await ready; return decodeVerdictResponse(await guest.dispatch(bytes)); } catch { return { ok: false, code: 'verdictRuntimeFailed' }; }
}
