import assert from "node:assert/strict";
import { test } from "node:test";
import fs from "node:fs";
import { encodeEventRequest, decodeEventResponse, admitEventWithGuest } from "../shell/event-codec.mjs";
const corpus = JSON.parse(fs.readFileSync(new URL("../../verification/event-wire-vectors.json", import.meta.url), "utf8"));
const enc = s => new TextEncoder().encode(s), dec = b => new TextDecoder().decode(b);
const fresh = () => structuredClone(corpus.vectors[0].input);
test("event requests reproduce the independent word corpus without deciding admission", () => {
  let count = 0;
  for (const vector of corpus.vectors) if (vector.input && vector.name !== 'excessBytes') { assert.equal(dec(encodeEventRequest(vector.input)), vector.request, vector.name); count++; }
  assert.equal(count, 23); assert.equal(corpus.vectors.length, 30);
});
test("malformed shapes scalars flag values and list bounds refuse", () => {
  for (const value of [null, {}, { ...fresh(), extra: true }]) assert.throws(() => encodeEventRequest(value), e => e.code === 'malformedEvent');
  for (const value of [-1, 1.5, NaN, Infinity, 0x100000000, '3']) { const input=fresh(); input.generation=value;assert.throws(()=>encodeEventRequest(input)); }
  for(const value of [0,1,'true',null]) { const input=fresh();input.envelope.payload={kind:'flag',value};assert.throws(()=>encodeEventRequest(input)); }
  const input=fresh();input.contract.producers=Array(65).fill({key:1,version:1});assert.throws(()=>encodeEventRequest(input));
  assert.throws(()=>encodeEventRequest(corpus.vectors.find(v=>v.name==='excessBytes').input));
});
test("all valid responses preserve exact next-sequence or typed nested refusal", () => {
  for(const vector of corpus.vectors){const result=decodeEventResponse(enc(vector.response));if(vector.response.startsWith('ok ')){assert.equal(result.ok,true);assert.equal(result.nextSequence,vector.name==='lastSequence'?0xffffffff:3);}else assert.deepEqual(result,{ok:false,code:vector.response});}
});
test("malformed or oversized responses fail closed", () => {
  for(const value of ['ok','ok 0000000','ok 000000000','ok FFFFFFFF','accepted','compositionRejected:unknown','staleSequence\n','x'.repeat(65)])assert.deepEqual(decodeEventResponse(enc(value)),{ok:false,code:'invalidEventResponse'});
  for(const value of [null,'ok 03000000',new Uint8Array([255])])assert.equal(decodeEventResponse(value).code,'invalidEventResponse');
  const NativeDecoder = globalThis.TextDecoder; let decoded = 0;
  try {
    globalThis.TextDecoder = class extends NativeDecoder { decode(value) { decoded++; return super.decode(value); } };
    assert.equal(decodeEventResponse(new Uint8Array(65)).code, 'invalidEventResponse');
    assert.equal(decoded, 0, 'oversized replies must refuse before UTF-8 decoding');
  } finally { globalThis.TextDecoder = NativeDecoder; }
});
test("only the guest determines semantic acceptance and runtime errors stay private", async()=>{
  const input=fresh();input.envelope.generation=0;
  const answer=await admitEventWithGuest({dispatch:async request=>{assert.equal(dec(request),dec(encodeEventRequest(input)));return enc('staleGeneration');}},input);
  assert.deepEqual(answer,{ok:false,code:'staleGeneration'});
  assert.deepEqual(await admitEventWithGuest({dispatch:async()=>{throw Error('secret');}},fresh()),{ok:false,code:'eventRuntimeFailed'});
  let calls=0;assert.deepEqual(await admitEventWithGuest({dispatch:()=>{calls++;}},{}),{ok:false,code:'malformedEvent'});assert.equal(calls,0);
});
test("canonical model contains bounded parser generated dispatch and executable wire proofs",()=>{
  const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
  const model=read('../../app/src/EventProtocol.lex.tex');
  const others = fs.readdirSync(new URL('../../app/src/', import.meta.url)).filter(name => name.endsWith('.lex.tex') && name !== 'EventProtocol.lex.tex').map(name => read('../../app/src/' + name)).join('\n');
  for (const name of ['readEventPayload', 'readEventContract', 'readEventEnvelope', 'encodeEventResult', 'dispatchEventWords', 'dispatchEvent']) {
    assert.equal((model.match(new RegExp('name: ' + name + ',', 'g')) || []).length, 1, name);
    assert.ok(!new RegExp('name: ' + name + ',').test(others), 'generated Rust symbol collision: ' + name);
  }
  for(const token of ['EventAdmission::admitEvent','CompositionProtocol::decodeWords','32768','1024','wire_natural','wire_revoked','wire_staleGeneration','wire_trailing','wire_flag'])assert.ok(model.includes(token),token);
  assert.ok(read('../../app/src/ForgeKernel.lex.tex').includes('EventProtocol::dispatchEvent'));
  assert.ok(read('../../app/src/ForgeKernel.lex.tex').includes('acceptanceVectors := (verdictAcceptance)'));
  assert.ok(read('../shell/host.js').includes('handlers.set("forge:event", async input => admitEventWithGuest(await kernelReady, input))'));
  assert.ok(read('./kernel_browser.mjs').includes('assert.equal(documentResult.events, 30)'));
  assert.ok(read('./kernel_browser.mjs').includes('assert.equal(documentResult.embedded, JSON.parse('));
});
