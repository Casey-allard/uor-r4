import assert from 'node:assert/strict';
import fs from 'node:fs';
import vm from 'node:vm';
import {webcrypto} from 'node:crypto';
const self={};vm.runInNewContext(fs.readFileSync(new URL('../shell/kernel.js',import.meta.url),'utf8'),{self,crypto:webcrypto,WebAssembly,TextEncoder,TextDecoder,Uint8Array,atob});
const guest=await self.ForgeKernel.load(async path=>new Response(fs.readFileSync(new URL('../shell/'+path,import.meta.url))));assert.equal(guest.vectors,JSON.parse(fs.readFileSync(new URL('../shell/kernel/kernel.json',import.meta.url))).vectors.length,'Install the signed queue-capable guest');
const vectors=JSON.parse(fs.readFileSync(new URL('../../verification/chat-queue-vectors.json',import.meta.url))).vectors;assert.equal(vectors.length,38);
for(const row of vectors)assert.equal(new TextDecoder().decode(await guest.dispatch(new TextEncoder().encode(row.request))),row.response,row.name);
console.log(JSON.stringify({embedded:guest.vectors,queueWireVectors:vectors.length}));
