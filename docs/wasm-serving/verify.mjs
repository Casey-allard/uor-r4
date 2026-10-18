import { readFileSync } from 'fs';
const wasmBytes = readFileSync('./uor_r4_integer_bg.wasm');
globalThis.fetch = async (p) => new Response(wasmBytes, { status: 200, headers: { 'Content-Type': 'application/wasm' } });
const mod = await import('./uor_r4_integer.js');
await mod.default();
const artifact = readFileSync('./synthetic-stack.lut');
const handle = mod.loadStack(new Uint8Array(artifact));
let tok = 0; const run1 = [];
for (let i = 0; i < 32; i++) { const r = mod.step(handle, tok); tok = r[0] >>> 0; run1.push(tok); }
const state = mod.saveSession(handle);
mod.restoreSession(handle, state);
let tok2 = run1[run1.length - 1]; const cont = [];
for (let i = 0; i < 16; i++) { const r = mod.step(handle, tok2); tok2 = r[0] >>> 0; cont.push(tok2); }
const handle2 = mod.loadStack(new Uint8Array(artifact));
let tok3 = 0; const ref = [];
for (let i = 0; i < 48; i++) { const r = mod.step(handle2, tok3); tok3 = r[0] >>> 0; ref.push(tok3); }
const refTail = ref.slice(32);
const ok = refTail.join(',') === cont.join(',');
console.log(JSON.stringify({ handle, wpt: Number(mod.weightsPerToken(handle)), run1, stateBytes: state.length, cont, refTail, reloadIdentical: ok, run1MatchesRef: run1.join(',') === ref.slice(0,32).join(',') }));
