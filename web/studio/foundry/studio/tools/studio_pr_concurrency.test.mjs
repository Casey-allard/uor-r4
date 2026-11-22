import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {test} from 'node:test';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8'),workflow=read('../../.github/workflows/studio.yml'),model=read('../../app/src/FoundrySpecification.lex.tex');
const expected="concurrency:\n  group: ${{ github.event_name == 'pull_request' && format('studio-pr-{0}', github.event.pull_request.number) || format('studio-run-{0}', github.run_id) }}\n  cancel-in-progress: ${{ github.event_name == 'pull_request' }}\n";
function check({source=model,yaml=workflow,script=read('./studio_concurrency_projection.py')}={}){const root=fs.mkdtempSync(path.join(os.tmpdir(),'foundry-concurrency-'));try{for(const [name,body] of [['app/src/FoundrySpecification.lex.tex',source],['.github/workflows/studio.yml',yaml],['studio/tools/studio_concurrency_projection.py',script]]){const p=path.join(root,name);fs.mkdirSync(path.dirname(p),{recursive:true});fs.writeFileSync(p,body);}return spawnSync('python3',[path.join(root,'studio/tools/studio_concurrency_projection.py'),'--check'],{encoding:'utf8'});}finally{fs.rmSync(root,{recursive:true,force:true});}}
test('imported concurrency semantics bind same PR cancellation and unique non-PR runs',()=>{
 const match=model.match(/name: studioConcurrencyYaml,\s*parameters: \[\],\s*result: String,\s*body: ("(?:[^"\\]|\\.)*")/);assert.ok(match);assert.equal(JSON.parse(match[1]),expected);assert.equal(workflow.split('# BEGIN FoundrySpecification.studioConcurrencyYaml\n')[1].split('# END')[0],expected);assert.equal(check().status,0);
});
test('changed cancellation grouping and missing boundaries are rejected by the actual projector',()=>{
 for(const yaml of [workflow.replace("cancel-in-progress: ${{ github.event_name == 'pull_request' }}",'cancel-in-progress: true'),workflow.replace("format('studio-pr-{0}', github.event.pull_request.number)","format('studio-pr-{0}', github.head_ref)"),workflow.replace('# END FoundrySpecification.studioConcurrencyYaml\n',''),workflow+expected]){
  assert.notEqual(check({yaml}).status,0);
 }
});
test('missing duplicated or malformed canonical literals cannot be silently accepted',()=>{
 const definition=model.match(/definition\{\s*name: studioConcurrencyYaml,[\s\S]*?\n\}/)[0];
 for(const source of [model.replace('name: studioConcurrencyYaml,','name: absentConcurrency,'),model+definition,model.replace('body: "concurrency:', 'body: "invalid:')])assert.notEqual(check({source}).status,0);
});
test('retained Studio runs execute all browser SDK-adjacent and mutation obligations',()=>{
 assert.ok(workflow.includes('python3 tools/studio_concurrency_projection.py --check'));
 for(const command of ['./scripts/lane.sh','node tools/check_shell_closure.mjs','node tools/kernel_browser.mjs','node tools/durable_chat_queue_browser.mjs','node tools/durable_version_preview_browser.mjs','node tools/message_linked_restoration_browser.mjs'])assert.ok(workflow.includes(command),command);
 assert.equal((workflow.match(/^concurrency:/gm)||[]).length,1);assert.ok(!workflow.includes('if: false'));assert.ok(model.includes('name: studioConcurrencyBoundary,'));
});
