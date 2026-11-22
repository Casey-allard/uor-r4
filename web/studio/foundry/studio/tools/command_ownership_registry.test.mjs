import assert from 'node:assert/strict';
import fs from 'node:fs';
import {execFileSync} from 'node:child_process';
import {test} from 'node:test';
import {commandRegistryData} from '../shell/command-registry-data.mjs';
import {commandRegistry,createCommandRegistry,invokeBoundCommand} from '../shell/command-registry.mjs';
import {invokeHost} from '../shell/host-boundary.mjs';
const fresh=()=>structuredClone(commandRegistryData),inventory=()=>({handlers:commandRegistry.entries.filter(c=>c.availability==='adapter').map(c=>c.channel),inputs:commandRegistry.entries.filter(c=>c.input==='imported-renderer').map(c=>c.channel),outputs:commandRegistry.entries.filter(c=>c.output==='imported-renderer').map(c=>c.channel)});
test('canonical command projection names every actual owner declaration',()=>{
 const registryTex=new URL('../../app/src/CommandRegistry.lex.tex',import.meta.url);
 if(fs.existsSync(registryTex))execFileSync('python3',[new URL('./command_registry_projection.py',import.meta.url).pathname,'--check']);
 for(const f of commandRegistry.features){const [module,name]=f.owner.split('.'),texUrl=new URL(`../../app/src/${module}.lex.tex`,import.meta.url);if(fs.existsSync(texUrl)){const source=fs.readFileSync(texUrl,'utf8');assert.match(source,new RegExp('name: '+name+','));}}
 assert.equal(commandRegistry.entries.length,340);assert.equal(commandRegistry.entries.filter(c=>c.availability==='adapter').length,117);assert.equal(commandRegistry.entries.filter(c=>c.input==='imported-renderer').length,310);assert.equal(commandRegistry.entries.filter(c=>c.output==='imported-renderer').length,306);
 for(const id of ['mvm-terminal','uor-model'])assert.ok(commandRegistry.features.some(f=>f.id===id),id);
 for(const ch of ['terminal:open','mvm:exec','uor-r4:status'])assert.equal(commandRegistry.entries.find(c=>c.channel===ch)?.availability,'adapter',ch);
});
test('malformed duplicate unowned and misclassified registry data refuse',()=>{
 for(const change of [d=>d.commands.push(d.commands[0]),d=>d.features.push(d.features[0]),d=>d.commands[0].owner='Unknown.owner',d=>d.commands[0].feature='missing',d=>d.commands[0].version=2,d=>d.features[0].effects=['magic'],d=>d.commands[0].input='unchecked',d=>d.commands[0].channel='__proto__',d=>d.extra=true,d=>d.commands.reverse()]){const d=fresh();change(d);assert.throws(()=>createCommandRegistry(d),{code:'commandRegistryInvalid'});}
});
test('missing unavailable and changed bindings refuse before handler effects',async()=>{
 let effects=0;const effect=()=>effects++,invoke=(channel,args,handler)=>invokeHost(channel,args,handler,null);
 for(const [channel,handler,code] of [['unknown-command',effect,'commandBindingMissing'],['create-app',undefined,'commandBindingMismatch'],['agent-tool:get-tools',effect,'commandBindingMismatch'],['agent-tool:get-tools',undefined,'unsupportedChannel']]){const reply=await invokeBoundCommand(commandRegistry,channel,[],handler,invoke);assert.equal(reply.ok,false);assert.equal(reply.error.code,code);}
 assert.equal(effects,0);const result=await invokeBoundCommand(commandRegistry,'get-system-platform',[],()=>{effects++;return 'web';},invoke);assert.equal(result.value,'web');assert.equal(effects,1);
});
test('complete handler and imported schema inventories reject additions removals and duplicates',()=>{
 assert.equal(commandRegistry.assertInventory(inventory()),true);
 for(const field of ['handlers','inputs','outputs'])for(const edit of [list=>list.pop(),list=>list.push('undeclared-command'),list=>list.push(list[0])]){const data=inventory();edit(data[field]);assert.throws(()=>commandRegistry.assertInventory(data));}
});
test('registry snapshots cannot be mutated into authority after admission',()=>{
 const d=fresh(),registry=createCommandRegistry(d);d.commands[0].availability='unavailable';assert.equal(registry.entries[0].availability,'adapter');assert.throws(()=>registry.entries.pop(),TypeError);assert.throws(()=>registry.features[0].effects.push('network'),TypeError);assert.throws(()=>{registry.entries[0].owner='other';},TypeError);
});
test('actual dispatcher and full browser inventory oracle retain the registry boundary',()=>{
 const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8'),hostText=read('../shell/host.js');assert.ok(hostText.includes('return invokeBoundCommand(commandRegistry, channel, args, handlers.get(channel),'));
 const browser=read('./command_ownership_registry_browser.mjs');for(const token of ['window.__host.handlers','window.__inputContracts','window.__contracts','assertInventory','commandBindingMismatch','commandBindingMissing'])assert.ok(browser.includes(token),token);
 for(const c of commandRegistry.entries.filter(x=>x.availability==='adapter'))assert.ok(hostText.includes(`"${c.channel}"`)||hostText.includes(`'${c.channel}'`),`missing adapter channel in host.js: ${c.channel}`);
});
