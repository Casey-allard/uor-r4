import assert from 'node:assert/strict';
import fs from 'node:fs';
import {execFileSync} from 'node:child_process';
import {test} from 'node:test';
const read=p=>fs.readFileSync(new URL('../../'+p,import.meta.url),'utf8');
test('combined canonical dispatcher retains every application protocol and experience family',()=>{
 const model=read('app/src/ForgeKernel.lex.tex');for(const name of ['dispatchPreview','dispatchQueue','dispatchRouting','dispatchInteraction','dispatchEvent','dispatchWork','dispatchComposition','budgetWithin','layoutTabs','motionAllows','targetOk','recentYes','thinkAllows','IdeaReader::readIdea'])assert.ok(model.includes(name));
 assert.ok(model.includes('acceptanceVectors := (verdictAcceptance)'));assert.ok(model.includes('responseMaximum := (4096 : UInt32)'));assert.equal((model.match(/request :=/g)||[]).length,368);assert.ok(model.includes('(acceptanceExperience)'));
});
test('integration retains every feature registration and the main experience proof obligations',()=>{
 const ids=read('model/ids.toml');const retired=new Set([4,11,16,34,45,47]);for(let n=1;n<=59;n++)assert.equal(ids.includes('id = "FA-'+String(n).padStart(2,'0')+'"'),!retired.has(n));
 for(let n=12;n<=24;n++)assert.ok(ids.includes('id = "FK-'+n+'"'));
 assert.ok(read('app/src/FoundrySpecification.lex.tex').includes('name: applicationIntegrationBoundary,'));
});
test('worker activation retains verified complete closure and has no unverified asset cache bypass',()=>{
 const worker=read('studio/shell/sw.template.js');assert.ok(worker.includes('ShellClosure.prepare(self.registration.scope, CLOSURE)'));assert.ok(worker.includes('File absent from the active shell'));assert.ok(!worker.includes('caches.open("assets")'));assert.ok(!worker.includes('async function fillOne('));
});
test('renderer integration retains real privacy projection and local editor while applying main improvements',()=>{
 const config=read('studio/vendor/prism/vite.shell.config.mts'),screens=read('studio/vendor/prism/hologram.screens.mjs');assert.ok(config.includes('hologram.editor.ts'));assert.ok(config.includes('import ${JSON.stringify(editor)}'));assert.ok(screens.includes('privacySource(rel, out)'));assert.ok(screens.includes('...LEAN'));assert.ok(screens.includes('home-suggestions'));
 assert.ok(read('.github/workflows/studio.yml').includes('node tools/privacy_projection.mjs')); 
});
test('full Studio acceptance retains feature gates and canonical PR-only concurrency',()=>{
 const workflow=read('.github/workflows/studio.yml');for(const gate of ['verified_working_tree_browser','browser_window_coordination_browser','message_linked_restoration_browser','durable_version_preview_browser','durable_chat_queue_browser','semantic_application_discovery_browser','verified_shell_cache_browser','privacy_shell_browser','editor_browser'])assert.ok(workflow.includes(gate+'.mjs'),gate);
 assert.ok(workflow.includes('# BEGIN FoundrySpecification.studioConcurrencyYaml'));assert.ok(workflow.includes("cancel-in-progress: ${{ github.event_name == 'pull_request' }}"));
});
test('installed executable carries combined acceptance and actual experience gate passes',()=>{
 const meta=JSON.parse(read('studio/shell/kernel/kernel.json'));assert.ok(meta.vectors.length>=334,'the pinned kernel keeps every vector');
 execFileSync(process.execPath,['studio/tools/experience.mjs','--check'],{cwd:new URL('../../',import.meta.url),stdio:'pipe'});
});
test('performance lane stages locked compiler and renderer before complete shell projection',()=>{
 const workflow=read('.github/workflows/forge-perf.yml'),compiler=workflow.indexOf('npm ci --prefix runtime --ignore-scripts --no-audit --no-fund'),stage=workflow.indexOf('python3 ../../scripts/stamp_shell.py'),projection=workflow.indexOf('run: ./scripts/lane.sh');
 assert.ok(compiler>=0&&compiler<stage&&stage<projection);
 assert.ok(workflow.includes('node tools/perf_budget.mjs --browser'));
 assert.ok(workflow.includes('node vendor/prism/node_modules/playwright-core/cli.js install --with-deps chromium'));assert.ok(!workflow.includes('npm install --no-audit --no-fund --global'));
 const performance=read('studio/tools/perf_budget.mjs');assert.ok(performance.includes('../vendor/prism/node_modules/playwright-core/index.mjs'));assert.ok(performance.includes('/Open questions/.test(document.body.innerText)'));assert.ok(!performance.includes('Send this request'));assert.ok(!performance.includes('featherless'));assert.ok(!performance.includes('warmup.json'));assert.ok(!performance.includes('ui/index.html'));
 const gate=read('studio/tools/live_lexlean.mjs');assert.ok(gate.includes('value?.code === "invalidCanonicalModule"'));assert.ok(gate.includes('value.message === "The canonical decision module is invalid or unsupported"'));assert.ok(gate.includes('canonical reconstruction retains the example decisions'));assert.ok(gate.includes('alternate numeric bounds require canonical reconstruction'));assert.ok(gate.includes('unexpanded guarantees cannot bypass declaration comparison'));
});
