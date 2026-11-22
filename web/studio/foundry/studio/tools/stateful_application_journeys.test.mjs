import assert from 'node:assert/strict';
import fs from 'node:fs';
import {test} from 'node:test';
import vm from 'node:vm';
import {observeReducedMotion} from './reduced_motion_oracle.mjs';
import {interactionPreset,readInteractionDocument,interactionModule,interactionDocumentFromFiles,validateInteractionDocument} from '../shell/interaction-application.mjs';
import {createInteractionAudio} from '../shell/interaction-ui.mjs';
import {assertCanonicalGeneration} from '../shell/canonical-generation.mjs';
import {stageInteraction} from '../shell/interaction-state.mjs';
const enc=new TextEncoder(),id='blake3:'+'a'.repeat(64),derive=async()=>id;
const reply=output=>enc.encode('ok '+output.map(n=>{const b=new Uint8Array(4);new DataView(b.buffer).setUint32(0,n,true);return Buffer.from(b).toString('hex');}).join(''));
const command={event:0,audioGranted:false,reducedMotion:true,reset:false};
const fixture=()=>interactionPreset('garden');
test('gallery and garden documents reconstruct literal canonical model authority',()=>{
 for(const kind of ['gallery','garden']){const d=interactionPreset(kind),raw=JSON.stringify(d),source=interactionModule(d.graph);assert.deepEqual(readInteractionDocument(raw),d);assert.deepEqual(interactionDocumentFromFiles([['prism/interaction.json',raw],['prism/Application.lex.tex',source]]),d);assert.match(source,/record InteractionModel::InteractionProfile/);assert.throws(()=>interactionDocumentFromFiles([['prism/interaction.json',raw],['prism/Application.lex.tex',source],['prism/InteractionModel.lex.tex','changed']], 'accepted'),{code:'interactionApplicationMismatch'});assert.throws(()=>interactionDocumentFromFiles([['prism/interaction.json',raw],['prism/Application.lex.tex',source+' ']]),{code:'interactionApplicationMismatch'});}
});
test('unbounded malformed or executable presentation data refuse',()=>{
 for(const path of ['prism/interaction.json','prism/Application.lex.tex','prism/InteractionModel.lex.tex'])assert.throws(()=>assertCanonicalGeneration({files:[['prism/interaction.json','{}']]},{writes:[{path,content:'changed'}],deletes:[],renames:[],deps:[]},{}),{code:'canonicalGenerationRefused'});
 const mutations=[d=>d.title='',d=>d.extra=true,d=>d.screens[0].illustration='<script>',d=>d.controls.push(d.controls[0]),d=>d.controls=[],d=>d.screens[0].description='a'.repeat(2001),d=>d.graph.frames=Array(33).fill(d.graph.frames[0])];
 for(const change of mutations){const d=fixture();change(d);assert.throws(()=>readInteractionDocument(JSON.stringify(d)),{code:'invalidInteractionApplication'});}for(const raw of [null,'{}','x'.repeat(65537)])assert.throws(()=>readInteractionDocument(raw),{code:'invalidInteractionApplication'});
});
test('application admission and transition always execute the generated guest',async()=>{
 let calls=0;const guest={dispatch:async bytes=>{calls++;assert.match(new TextDecoder().decode(bytes),/^interact /);return reply([1,1,3,0,0]);}};
 await validateInteractionDocument(JSON.stringify(fixture()),guest);const result=await stageInteraction(guest,derive,fixture(),undefined,{...command,event:1});assert.equal(calls,2);assert.deepEqual(result.output,{state:1,screen:1,palette:3,tone:0,motion:0});assert.equal(result.state.model,id);assert.equal(result.state.state,1);
 await assert.rejects(stageInteraction({dispatch:async()=>enc.encode('unknownInteractionEvent')},derive,fixture(),result.state,{...command,event:4}),{code:'unknownInteractionEvent'});
});
test('stored semantic identity requires explicit reset after changed models',async()=>{
 let calls=0;const guest={dispatch:async()=>{calls++;return reply([0,0,2,0,0]);}},old={profile:'foundry/interaction-state/1',model:'blake3:'+'b'.repeat(64),state:1,revision:1};
 await assert.rejects(stageInteraction(guest,derive,fixture(),old,command),{code:'interactionStateStale'});assert.equal(calls,0);
 const restored=await stageInteraction(guest,derive,fixture(),old,{...command,reset:true});assert.equal(restored.state.state,0);assert.equal(restored.state.model,id);
 await assert.rejects(stageInteraction(guest,derive,fixture(),old,{...command,reset:true,event:1}),{code:'invalidInteractionApplication'});
});
test('mutable commands and graph cannot change an already captured interaction',async()=>{
 let release;const waiting=new Promise(r=>release=r),d=fixture(),action={...command},seen=[];
 const result=stageInteraction({dispatch:async b=>{seen.push(Buffer.from(b).toString());return reply([0,0,2,0,0]);}},async()=>{await waiting;return id;},d,undefined,action);d.graph.initial=1;action.event=1;release();await result;assert.equal(seen.length,1);assert.match(seen[0],/00000000000000000000000001000000$/);
 let resume;const resumed=new Promise(r=>resume=r),audio=createInteractionAudio(()=>({resume:()=>resumed,state:'running',close:async()=>{}}));const enabling=audio.enable();audio.revoke();resume();await enabling;assert.equal(audio.granted,false,'late audio resume renewed revoked consent');
 const unavailable=createInteractionAudio(()=>{throw Error('private device detail');});await assert.rejects(unavailable.enable(),error=>error.code==='interactionAudioUnavailable'&&!error.message.includes('private'));assert.equal(unavailable.granted,false);

});
test('storage and host route enforce comparison deletion ownership and private errors',()=>{
 const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
 for(const token of ['interactionRead','JSON.stringify(currentInteraction) !== JSON.stringify(interaction.expected)','interaction.state, `interaction:${appId}`'])assert.ok(read('../shell/project-edit.mjs').includes(token),token);
 assert.ok(read('../shell/project-lifecycle.mjs').includes('refs.delete(`interaction:${appId}`)'));
 assert.ok(read('../shell/canonical-generation.mjs').includes('prism/interaction.json'));
 for(const name of ['forge:interaction-create','forge:interaction-run','mountInteractionApplications'])assert.ok(read('../shell/host.js').includes(name),name);
 assert.ok(read('../shell/host-boundary.mjs').includes('interactionStateStale'));
});

test('reduced-motion oracle distinguishes settled effects and refuses active motion within a bound',async()=>{
 const effect=(change={})=>({constructor:{name:'CSSTransition'},transitionProperty:'color',playState:'running',pending:false,currentTime:0,playbackRate:1,effect:{target:{tagName:'SECTION'},getComputedTiming:()=>({duration:0.01,delay:0,endDelay:0,iterations:1,endTime:0.01,fill:'backwards'})},...change});
 const timed=timing=>effect({effect:{target:{tagName:'SECTION'},getComputedTiming:()=>({duration:0.01,delay:0,endDelay:0,iterations:1,endTime:0.01,fill:'none',...timing})}});
 function run({effects=()=>[],media=true,present=true,connected=true,stalled=false}={}){
  let frame=0,next,deadline,timerCleared=false;
  const root={isConnected:connected,getAnimations(options){assert.equal(options.subtree,true);return effects(frame);}};
  const globals={document:{querySelector(selector){assert.equal(selector,'dialog section');return present?root:null;}},matchMedia(query){assert.equal(query,'(prefers-reduced-motion: reduce)');return {matches:media};},requestAnimationFrame(callback){next=callback;return 1;},cancelAnimationFrame(){next=undefined;},setTimeout(callback,ms){assert.equal(ms,1000);deadline=callback;return 2;},clearTimeout(){timerCleared=true;}};
  const result=vm.runInNewContext(`(${observeReducedMotion.toString()})()`,globals);
  if(stalled)deadline?.();
  else {while(next&&frame<100){const callback=next;next=undefined;frame++;callback();}if(next)deadline();}
  return {result,get frames(){return frame;},get timerCleared(){return timerCleared;}};
 }
 const empty=run();await empty.result;assert.equal(empty.frames,2);assert.equal(empty.timerCleared,true);
 const settling=run({effects:frame=>frame<5?Array.from({length:9},()=>effect({pending:frame<3})):[]});
 const settled=await settling.result;assert.ok(settling.frames>=7);assert.equal(settled.maximumSettlingEffects,9);
 const retained=run({effects:()=>[effect({playState:'finished',effect:{target:{tagName:'DIV'},getComputedTiming:()=>({duration:3000,delay:0,endDelay:0,iterations:1,endTime:3000,fill:'forwards'})}})]});
 assert.equal((await retained.result).finishedEffects,1);
 for(const animation of [timed({duration:160,endTime:160}),timed({delay:100,endTime:100.01}),timed({endDelay:100,endTime:100.01}),timed({iterations:Infinity,endTime:Infinity}),timed({duration:'auto'}),timed({endTime:NaN}),effect({playbackRate:0}),effect({playState:'paused'}),effect({constructor:{name:'Animation'}}),effect({effect:null}),effect({effect:{}}),effect({playState:'finished',pending:true,effect:{getComputedTiming:()=>({duration:3000,endTime:3000})}})]){
  const active=run({effects:()=>[animation]});await assert.rejects(active.result,/Reduced motion: active effect/);assert.equal(active.frames,0);assert.equal(active.timerCleared,true);
 }
 await assert.rejects(run({effects:()=>[effect()]}).result,/Reduced motion: settlement timeout.*CSSTransition.*color/);
 await assert.rejects(run({stalled:true}).result,/Reduced motion: settlement timeout/);
 await assert.rejects(run({media:false}).result,/Reduced motion: preference is not active/);
 await assert.rejects(run({present:false}).result,/Reduced motion: application section is missing/);
 await assert.rejects(run({connected:false}).result,/Reduced motion: application section was detached/);
 // Keep the real journey and its planted controls attached to this tested oracle.
 const journey=fs.readFileSync(new URL('./stateful_application_journeys_browser.mjs',import.meta.url),'utf8');
 assert.match(journey,/import \{observeReducedMotion\} from ['"]\.\/reduced_motion_oracle\.mjs['"]/);
 assert.equal((journey.match(/page\.evaluate\(observeReducedMotion\)/g)||[]).length,3);
 assert.match(journey,/^  for\(const kind of \['waapi','css'\]\)\{$/m);
 assert.match(journey,/^    try \{await assert\.rejects\(page\.evaluate\(observeReducedMotion\),\/Reduced motion: active effect\/\);\}$/m);
 assert.ok(journey.includes('iterations:Infinity')&&journey.includes('reducedMotionNegativeControls:2'));
 assert.ok(journey.includes('animation: foundry-motion-mutant 3s linear infinite !important'));
 assert.ok(!journey.includes('requestAnimationFrame(()=>requestAnimationFrame(resolve))'));
 const wrapper=fs.readFileSync(new URL('../../crates/conformance/tests/stateful_application_journeys.rs',import.meta.url),'utf8');
 assert.match(wrapper,/#\[test\]\s+fn stateful_application_journeys_fa_42\(\)/);
 assert.ok(wrapper.includes('.args(["--test", "--test-reporter=tap"])'));
 for(const expected of ['# tests 7','# pass 7','# fail 0','# cancelled 0','# skipped 0','# todo 0'])assert.ok(wrapper.includes(JSON.stringify(expected)),expected);

});
