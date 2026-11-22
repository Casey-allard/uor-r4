import assert from "node:assert/strict";
import fs from "node:fs";
import {invokeHost} from "../shell/host-boundary.mjs";
import {test} from "node:test";
import {copyEventCommand,prepareEventJournal,stageEventDelivery,acknowledgeEventDelivery} from "../shell/event-delivery.mjs";
const input=JSON.parse(fs.readFileSync(new URL("../../verification/event-wire-vectors.json",import.meta.url))).vectors[0].input;
const source="blake3:"+"a".repeat(64);
const guest={dispatch:async()=>new TextEncoder().encode("ok 03000000")};
const configured=async()=> (await prepareEventJournal(input,source,null,guest)).state;
const code=c=>e=>e.code===c;
test("configuration probes the guest without queuing its envelope",async()=>{
 const result=await prepareEventJournal(input,source,null,guest);assert.equal(result.ok,true);assert.equal(result.state.pending.length,0);assert.equal(result.state.nextSequence,2);assert.equal(result.state.context.envelope,undefined);assert.equal(result.state.source,source);
 const denied=await prepareEventJournal(input,source,null,{dispatch:async()=>new TextEncoder().encode("compositionRejected:permissionDenied")});assert.equal(denied.ok,false);assert.equal(denied.state,undefined);
});
test("delivery uses stored context and commits only guest-admitted next sequence",async()=>{
 const state=await configured();let request;
 const result=await stageEventDelivery(state,input.envelope,source,{dispatch:async bytes=>{request=new TextDecoder().decode(bytes);return new TextEncoder().encode("ok 03000000");}});
 assert.equal(result.state.nextSequence,3);assert.deepEqual(result.state.pending,[input.envelope]);assert.ok(typeof request==="string"&&request.startsWith("event "));assert.equal(state.pending.length,0);
 const denied=await stageEventDelivery(state,input.envelope,source,{dispatch:async()=>new TextEncoder().encode("staleSequence")});assert.deepEqual(denied,{ok:false,code:"staleSequence"});
});
test("source changes reconfiguration pending work and queue bounds refuse",async()=>{
 const state=await configured(),queued=(await stageEventDelivery(state,input.envelope,source,guest)).state;
 await assert.rejects(stageEventDelivery(state,input.envelope,"blake3:"+"b".repeat(64),guest),code("eventDeliverySourceChanged"));
 await assert.rejects(prepareEventJournal({...input,generation:4},source,queued,guest),code("eventDeliveryPending"));
 await assert.rejects(prepareEventJournal(input,source,state,guest),code("eventDeliveryConflict"));
 await assert.rejects(stageEventDelivery({...state,pending:Array(256).fill(input.envelope)},input.envelope,source,guest),code("eventDeliveryLimitExceeded"));
});
test("ordered consumer acknowledgement is durable and latest duplicate is idempotent",async()=>{
 const state=(await stageEventDelivery(await configured(),input.envelope,source,guest)).state;
 const receipt={generation:3,sequence:2,consumer:{key:2,version:1}};
 const reversed=structuredClone(state);reversed.pending[0].consumer={version:1,key:2};assert.equal(acknowledgeEventDelivery(reversed,receipt).pending.length,0);
 for(const patch of [{generation:4},{sequence:3},{consumer:{key:1,version:1}}])assert.throws(()=>acknowledgeEventDelivery(state,{...receipt,...patch}),code("eventDeliveryAcknowledgementRefused"));
 const result=acknowledgeEventDelivery(state,receipt);assert.equal(result.pending.length,0);assert.deepEqual(result.lastAcknowledged,receipt);assert.deepEqual(acknowledgeEventDelivery(result,receipt),result);assert.equal(state.pending.length,1);
});
test("caller mutations cannot change an in-flight admitted envelope",async()=>{
 for(const raw of [null,{}, {appId:1,envelope:input.envelope,extra:true}])assert.throws(()=>copyEventCommand("deliver",raw),code("invalidEventDelivery"));
 const command={appId:1,envelope:structuredClone(input.envelope)},copy=copyEventCommand("deliver",command);command.envelope.payload.value=100;assert.equal(copy.envelope.payload.value,42);
 const state=await configured(),envelope=structuredClone(input.envelope);let resolve;const wait=new Promise(r=>resolve=r);
 const pending=stageEventDelivery(state,envelope,source,{dispatch:async()=>{await wait;return new TextEncoder().encode("ok 03000000");}});envelope.payload.value=999;state.pending.length=12;resolve();assert.equal((await pending).state.pending[0].payload.value,42);
});
test("actual host and project transaction bind the durable command routes",async()=>{
 const read=p=>fs.readFileSync(new URL(p,import.meta.url),"utf8"),host=read("../shell/host.js"),edit=read("../shell/project-edit.mjs");
 for(const channel of ["forge:event-configure","forge:event-deliver","forge:event-pending","forge:event-acknowledge"])assert.ok(host.includes('handlers.set("'+channel+'"'));
 assert.match(edit,/eventUpdate/);assert.match(edit,/JSON.stringify\(currentEvents\) !== JSON.stringify\(events.expected\)/);assert.match(edit,/`events:\$\{appId\}`/);
 for(const code of ["invalidEventDelivery","eventDeliveryUnconfigured","eventDeliveryConflict","eventDeliverySourceChanged","eventDeliveryPending","eventDeliveryLimitExceeded","eventDeliveryAcknowledgementRefused"]){const reply=await invokeHost("events",[],()=>{throw Object.assign(Error("private payload"),{code});},null);assert.equal(reply.error.code,code);assert.ok(!JSON.stringify(reply).includes("private payload"));}
 assert.match(read("../../app/src/FoundrySpecification.lex.tex"),/name: eventDeliveryBoundary/);
});
