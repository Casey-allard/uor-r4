import assert from "node:assert/strict";
import fs from "node:fs";
import {test} from "node:test";
import {encodeInteraction,decodeInteraction,executeInteraction} from "../shell/interaction-codec.mjs";
const corpus=JSON.parse(fs.readFileSync(new URL("../../verification/interaction-vectors.json",import.meta.url)));
const fresh=()=>structuredClone(corpus.vectors[0].input),encode=s=>new TextEncoder().encode(s);
test("interaction encoding reproduces the independently packed corpus",()=>{
 let count=0;for(const vector of corpus.vectors)if(vector.input){assert.equal(new TextDecoder().decode(encodeInteraction(vector.input)),vector.request,vector.name);count++;}assert.equal(count,16);assert.equal(corpus.vectors.length,21);
});
test("malformed shapes scalars flags and oversized lists refuse before dispatch",()=>{
 for(const input of [null,{}, {...fresh(),extra:true}])assert.throws(()=>encodeInteraction(input));
 for(const value of [-1,1.2,NaN,Infinity,0x100000000,"0"]){const input=fresh();input.state=value;assert.throws(()=>encodeInteraction(input));}
 for(const value of [0,1,"true",null]){const input=fresh();input.audioGranted=value;assert.throws(()=>encodeInteraction(input));}
 for(const [field,count] of [["frames",33],["edges",129]]){const input=fresh();input.profile[field]=Array(count).fill(input.profile[field][0]);assert.throws(()=>encodeInteraction(input));}
});
test("typed replies preserve every coordinated output and exact refusal",()=>{
 for(const vector of corpus.vectors){const reply=decodeInteraction(encode(vector.response));if(vector.response.startsWith("ok ")){assert.equal(reply.ok,true);assert.deepEqual(Object.keys(reply.output),["state","screen","palette","tone","motion"]);}else assert.deepEqual(reply,{ok:false,code:vector.response});}
 const audible=decodeInteraction(encode(corpus.vectors.find(v=>v.name==="gardenAudio").response));assert.deepEqual(audible.output,{state:1,screen:1,palette:3,tone:440,motion:20});
});
test("malformed replies cannot create accepted application output",()=>{
 for(const raw of [null,"ok",encode("ok "),encode("ok "+"0".repeat(41)),encode("unknown"),new Uint8Array(65),new Uint8Array([255])])assert.equal(decodeInteraction(raw).code,"invalidInteractionResponse");
 const NativeDecoder=globalThis.TextDecoder;let decoded=0;try{globalThis.TextDecoder=class extends NativeDecoder{decode(value){decoded++;return super.decode(value);}};assert.equal(decodeInteraction(new Uint8Array(65)).code,"invalidInteractionResponse");assert.equal(decoded,0,"oversized reply decoded before its bound");}finally{globalThis.TextDecoder=NativeDecoder;}
});
test("interaction execution delegates all model semantics to the guest",async()=>{
 const request=fresh();request.profile.version=2;let calls=0;
 const result=await executeInteraction({dispatch:async bytes=>{calls++;assert.deepEqual(bytes,encodeInteraction(request));return encode("invalidInteractionModel");}},request);
 assert.deepEqual(result,{ok:false,code:"invalidInteractionModel"});assert.equal(calls,1);
 assert.deepEqual(await executeInteraction({dispatch:async()=>{throw Error("private detail");}},fresh()),{ok:false,code:"interactionRuntimeFailed"});
 assert.equal((await executeInteraction({dispatch:()=>assert.fail("invalid input dispatched")},{})).code,"malformedInteraction");
 let release;const waiting=new Promise(resolve=>release=resolve),mutable=fresh(),expected=encodeInteraction(mutable),resultWaiting=executeInteraction(waiting,mutable);mutable.event=99;release({dispatch:async bytes=>{assert.deepEqual(bytes,expected);return encode(corpus.vectors[0].response);}});assert.equal((await resultWaiting).ok,true);
});
test("the canonical profile and wire preserve proof controls and guest-only admission",()=>{
 const read=p=>fs.readFileSync(new URL(p,import.meta.url),"utf8"),model=read("../../app/src/InteractionModel.lex.tex"),wire=read("../../app/src/InteractionProtocol.lex.tex");
 for(const name of ["interactionSilentWithoutConsent","interactionReducedMotionStatic","interactionOutputState","interactionForward","interactionBackward","interactionCurrent","interactionBadState","interactionMissingRoute","interactionWrongVersion","interactionEmptyFrames","interactionDuplicateRoute","interactionBadDestination"])assert.equal((model.match(new RegExp("name: "+name+",","g"))||[]).length,1,name);
 for(const token of ["InteractionModel::interactionStep","8192","1024","malformedInteraction","interactionTooLarge","interactionRuntimeFailed","invalidInteractionResponse"])assert.ok(wire.includes(token),token);
 assert.ok(read("../../app/src/ForgeKernel.lex.tex").includes("InteractionProtocol::dispatchInteraction"));
 assert.ok(read("../shell/host.js").includes('handlers.set("forge:interact", input => executeInteraction(kernelReady, input))'));
 for(const vector of corpus.vectors)assert.ok(read("../../app/src/ForgeKernel.lex.tex").includes(Buffer.from(vector.request).toString("hex")),vector.name);
});
