import assert from 'node:assert/strict';
import {test} from 'node:test';
import fs from 'node:fs';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {interpretDiscovery,createDiscovery} from '../shell/application-discovery.mjs';
const enc=new TextEncoder();
// Literal response fixtures check the adapter only; actual guest/browser checks are mandatory.
const word=n=>{const b=Buffer.alloc(4);b.writeUInt32LE(n);return b.toString('hex');};
const accepted=enc.encode('ok '+[1,0,0,1,1,0,1,11,1,32001000,2,10,1,11,1,0,0].map(word).join(''));
const refused=enc.encode('invalidSemanticIntent');
const derive=async bytes=>'diagnostic-sha256:'+createHash('sha256').update(bytes).digest('hex');
function fixture(response=accepted){const writes=[];let calls=0;const adapter=createDiscovery({guest:{dispatch:async()=>{calls++;return response;}},derive,create:async input=>{writes.push(input);return {app:{id:1}};}});return {...adapter,writes,get calls(){return calls;}};}
test('finite vocabulary preserves all unknown requirements and bounded input',()=>{
 const value=interpretDiscovery('Please create a photo gallery.');assert.deepEqual(value.request.intent.concepts,[101]);assert.deepEqual(value.unknown,[]);
 assert.deepEqual(interpretDiscovery('garden with sound and motion').request.intent.concepts,[201,202,203]);
 for(const text of ['gallery without sound','gallery database']){const got=interpretDiscovery(text);assert.ok(got.unknown.length);assert.ok(got.request.intent.concepts.includes(0xffffffff));}
 for(const text of ['',null,'gallery'.repeat(74),Array(33).fill('garden').join(' '),'gallery <script>','gallery / garden','an app'])assert.throws(()=>interpretDiscovery(text),{code:'invalidDiscoveryIntent'});
 const got=interpretDiscovery('gallery');got.request.catalog[1].vocabulary=[999];assert.deepEqual(interpretDiscovery('gallery').request.catalog[1].vocabulary,[101]);
});
test('review delegates to guest without writes and captures input before await',async()=>{
 let release;const waits=new Promise(resolve=>release=resolve),writes=[];const adapter=createDiscovery({guest:waits,derive,create:input=>writes.push(input)});const input={text:'gallery'},pending=adapter.discover(input);input.text='garden';release({dispatch:async()=>accepted});const result=await pending;assert.equal(result.text,'gallery');assert.equal(result.application.kind,'gallery');assert.equal(result.confidence,0);assert.deepEqual(writes,[]);
});
test('creation reruns admission and rejects changed or forged review without writes',async()=>{
 const adapter=fixture(),review=await adapter.discover({text:'gallery'});assert.equal(adapter.calls,1);
 for(const input of [{text:'photo gallery',name:'X',review:review.review},{text:'gallery',name:'X',review:'forged'}])await assert.rejects(adapter.accept(input),{code:'discoveryReviewChanged'});
 assert.deepEqual(adapter.writes,[]);const input={text:'gallery',name:'Saved',review:review.review},pending=adapter.accept(input);input.name='Changed';assert.deepEqual(await pending,{app:{id:1}});assert.deepEqual(adapter.writes,[{name:'Saved',kind:'gallery'}]);assert.equal(adapter.calls,4);
});
test('guest failure unknown requirements and failed identities cannot create',async()=>{
 const adapter=fixture(refused);assert.equal((await adapter.discover({text:'gallery'})).ok,false);await assert.rejects(adapter.accept({text:'gallery',name:'X',review:'x'}),{code:'discoveryRefused'});assert.deepEqual(adapter.writes,[]);
 const staleSelection=fixture(enc.encode('ok '+[1,4,0,1,1,0,1,11,1,32001000,2,10,1,11,1,0,0].map(word).join('')));assert.equal((await staleSelection.discover({text:'gallery'})).ok,false);
 const falseSuccess=fixture();assert.equal((await falseSuccess.discover({text:'gallery database'})).ok,false);
 const broken=createDiscovery({guest:{dispatch:async()=>accepted},derive:async()=>{throw Error('private');},create:()=>assert.fail()});await assert.rejects(broken.discover({text:'gallery'}),{code:'discoveryUnavailable'});
 for(const input of [{text:'gallery',extra:1},null])await assert.rejects(adapter.discover(input),{code:'invalidDiscoveryIntent'});
 for(const input of [{text:'gallery',name:'',review:'x'},{text:'gallery',name:'X',review:7}])await assert.rejects(adapter.accept(input),{code:'invalidDiscoveryIntent'});
});
test('canonical data projection and command ownership match actual creation adapters',async()=>{
 execFileSync('python3',[new URL('./discovery_catalog_projection.py',import.meta.url).pathname,'--check']);const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8'),host=read('../shell/host.js');assert.ok(host.includes('createDiscovery({guest:kernelReady,derive:kappa,create:input=>handlers.get("forge:interaction-create")(input)})'));assert.ok(host.includes('handlers.set("forge:discover", input=>discovery.discover(input))'));assert.ok(host.includes('handlers.set("forge:discovery-create", input=>discovery.accept(input))'));
 const registry=await import('../shell/command-registry.mjs');for(const channel of ['forge:discover','forge:discovery-create'])assert.equal(registry.commandRegistry.resolve(channel,()=>{}).owner,'FoundrySpecification.discoveryBoundary');
 const boundary=await import('../shell/host-boundary.mjs');for(const code of ['invalidDiscoveryIntent','discoveryRefused','discoveryReviewChanged','discoveryUnavailable']){const answer=await boundary.invokeHost('forge:discover',[],()=>{throw Object.assign(Error('private'),{code});},null);assert.equal(answer.error.code,code);assert.ok(!answer.error.message.includes('private'));}
});
test('native review exposes limits invalidates edits and requires real browser qualification',()=>{
 const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8'),ui=read('../shell/discovery-ui.mjs');assert.ok(ui.includes("prompt.addEventListener('input'"));assert.ok(ui.includes('ticket!==revision'));assert.ok(ui.includes('if(!creating)dialog.close()'));assert.ok(ui.includes('if(creating)event.preventDefault()'));assert.ok(ui.includes("create.disabled=true"));assert.ok(ui.includes("node.textContent=text"));assert.ok(!ui.includes('innerHTML'));assert.ok(ui.includes('Sound remains off'));assert.ok(read('../../.github/workflows/studio.yml').includes('node tools/semantic_application_discovery_browser.mjs'));assert.ok(read('./shell_parse.mjs').includes('discovery-ui.mjs'));
});
