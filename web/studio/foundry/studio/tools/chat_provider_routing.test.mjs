import assert from 'node:assert/strict';
import fs from 'node:fs';
import {test} from 'node:test';
import {chatModelRoute} from '../shell/chat-model-route.mjs';
test('explicit cloud selections retain provider and exact model identity',()=>{
 for(const provider of ['openrouter','featherless'])for(const name of ['zai-org/GLM-5.3-Flash','openrouter/free']){
  const route=chatModelRoute({provider,name},{provider:'local',name:'foundry:reader'});
  assert.deepEqual(route,{provider,model:provider+'/'+name,paid:true});assert.ok(Object.isFrozen(route));
 }
});
test('only absent chat selection uses defaults and device identity remains exact',()=>{
 for(const selection of [null,undefined])assert.deepEqual(chatModelRoute(selection,{provider:'local',name:'foundry:reader'}),{provider:'local',model:'foundry:reader',paid:false});
 assert.equal(chatModelRoute(null,{provider:'openrouter',name:'openrouter/free'}).model,'openrouter/openrouter/free');
 // The former local identifiers resolve to the declared local model, as a chat selection and as the default.
 for(const former of ['webgpu:BitNet','webgpu:Bonsai','webgpu:Qwen']){
  assert.deepEqual(chatModelRoute({provider:'local',name:former},{provider:'featherless',name:'zai-org/GLM-5.3-Flash'}),{provider:'local',model:'foundry:reader',paid:false});
  assert.deepEqual(chatModelRoute(null,{provider:'local',name:former}),{provider:'local',model:'foundry:reader',paid:false});
 }
});
test('malformed explicit selection and unknown providers refuse without fallback',()=>{
 const fallback={provider:'local',name:'foundry:reader'};
 for(const selection of [{},{provider:'',name:'x'},{provider:'local',name:'other'},{provider:'featherless',name:''},{provider:'featherless',name:'x'.repeat(257)},{provider:'featherless',name:'x\n'},[],7])assert.throws(()=>chatModelRoute(selection,fallback),{code:'invalidChatModelSelection'});
 assert.throws(()=>chatModelRoute({provider:'unknown',name:'x'},fallback),{code:'unsupportedChatProvider'});
 assert.throws(()=>chatModelRoute(null,null),{code:'invalidChatModelSelection'});
});
test('actual execution token accounting typed failures and mandatory gates share the boundary',()=>{
 const read=p=>fs.readFileSync(new URL('../../'+p,import.meta.url),'utf8'),host=read('studio/shell/host.js');
 assert.equal((host.match(/chatModelRoute\(chat\??\.modelSelection,settings.selectedModel\)/g)||[]).length,2);
 assert.ok(host.includes('const {model,provider} = route;'));
 assert.ok(!host.includes('a selection left on OpenRouter answers on the device'));
 for(const code of ['invalidChatModelSelection','unsupportedChatProvider'])assert.ok(read('studio/shell/host-boundary.mjs').includes(code));
 assert.ok(read('.github/workflows/studio.yml').includes('node tools/chat_provider_routing_browser.mjs'));
 assert.ok(read('studio/tools/shell_parse.mjs').includes('chat-model-route.mjs'));
 assert.ok(read('app/src/FoundrySpecification.lex.tex').includes('name: chatProviderRoutingBoundary,'));
});
