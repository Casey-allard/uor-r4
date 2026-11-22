import assert from 'node:assert/strict';
import fs from 'node:fs';
import {test} from 'node:test';
import {chatExecutionPolicy,chatExecutionPrompt,readOnlyChatText} from '../shell/chat-execution.mjs';
test('queued mode wins and read-only modes never gain source effects',()=>{
 for(const mode of ['build','local-agent','ask','plan']){
  const p=chatExecutionPolicy(mode,'build');assert.equal(p.mode,mode);assert.equal(p.sourceEffects,['build','local-agent'].includes(mode));assert.ok(Object.isFrozen(p));
 }
 assert.equal(chatExecutionPolicy(null,'ask').sourceEffects,false);assert.equal(chatExecutionPolicy(undefined,null).mode,'build');assert.equal(chatExecutionPolicy('build','ask').sourceEffects,true);
});
test('unknown modes fail with a fixed typed error',()=>{
 for(const mode of ['agent','publish','',{},[],4])assert.throws(()=>chatExecutionPolicy(mode,'build'),e=>e.code==='invalidChatExecutionMode'&&!e.message.includes('publish'));
 assert.throws(()=>chatExecutionPolicy(null,'invalid'),{code:'invalidChatExecutionMode'});
});
test('read-only instructions preserve mode and context without build instructions',()=>{
 for(const mode of ['ask','plan']){const p=chatExecutionPolicy(mode,null);const prompt=chatExecutionPrompt(p,'BUILD-INSTRUCTIONS');assert.ok(prompt.includes(mode));assert.ok(prompt.includes('Do not modify'));assert.ok(!prompt.includes('BUILD-INSTRUCTIONS'));}
 assert.equal(chatExecutionPrompt(chatExecutionPolicy('build',null),'BUILD-INSTRUCTIONS'),'BUILD-INSTRUCTIONS');
 assert.equal(readOnlyChatText('<prism-write>&x</prism-write>'),'&lt;prism-write&gt;&amp;x&lt;/prism-write&gt;');
});
test('host effects and acceptance gates bind to the modeled policy',()=>{
 const read=p=>fs.readFileSync(new URL('../../'+p,import.meta.url),'utf8'),host=read('studio/shell/host.js');
 for(const text of ['chatExecutionPolicy(intent.requestedChatMode,chat.chatMode)','execution.sourceEffects && prism.isApproval','execution.sourceEffects && prism.isPublish','execution.sourceEffects && !(await hasRules','execution.sourceEffects && full.includes','if (execution.sourceEffects) {','chatExecutionPrompt(chatExecutionPolicy(null,chat?.chatMode),prism.SPEC_RULES)'])assert.ok(host.includes(text),text);
 assert.ok(read('app/src/FoundrySpecification.lex.tex').includes('name: readOnlyChatExecutionBoundary,'));assert.ok(read('.github/workflows/studio.yml').includes('node tools/read_only_chat_execution_browser.mjs'));assert.ok(read('studio/tools/shell_parse.mjs').includes('chat-execution.mjs'));
});
