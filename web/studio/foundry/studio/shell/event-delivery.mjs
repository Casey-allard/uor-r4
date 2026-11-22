// FoundrySpecification.eventDeliveryBoundary: storage acknowledges delivery, not effect execution.
import { encodeEventRequest, admitEventWithGuest } from "./event-codec.mjs";
import { captureProjectSource, commitPreparedProject } from "./project-edit.mjs";
import { storageRequest } from "./storage-request.mjs";
const messages = {
  invalidEventDelivery: "Invalid event delivery request",
  eventDeliveryUnconfigured: "This project has no event delivery configuration",
  eventDeliveryConflict: "Event delivery changed; read its current state again",
  eventDeliverySourceChanged: "The event configuration belongs to a different project source",
  eventDeliveryPending: "Acknowledge pending events before replacing their configuration",
  eventDeliveryLimitExceeded: "The pending event journal is full",
  eventDeliveryAcknowledgementRefused: "The acknowledgement does not match the pending consumer delivery",
};
const fail = code => Object.assign(new Error(messages[code]), {code});
const uint = n => Number.isInteger(n) && n >= 0 && n <= 0xffffffff;
const address = s => typeof s === "string" && /^blake3:[0-9a-f]{64}$/.test(s);
const same = (a,b) => JSON.stringify(a) === JSON.stringify(b);
function journal(value) {
  if (!value) throw fail("eventDeliveryUnconfigured");
  if (value.profile !== "foundry/event-journal/1" || !address(value.source) || !Array.isArray(value.pending) || !uint(value.nextSequence) || !uint(value.initialSequence) || !uint(value.context?.generation)) throw fail("invalidEventDelivery");
  if (value.pending.length > 256) throw fail("eventDeliveryLimitExceeded");
  try { if (new TextEncoder().encode(JSON.stringify(value)).length > 1024*1024) throw fail("eventDeliveryLimitExceeded"); return structuredClone(value); }
  catch(error) { if(error?.code) throw error;throw fail("invalidEventDelivery"); }
}
export async function prepareEventJournal(input, source, previous, guest) {
  let probe;
  try { encodeEventRequest(input); probe=structuredClone(input); } catch { throw fail("invalidEventDelivery"); }
  if (!address(source)) throw fail("invalidEventDelivery");
  if (previous) { const old=journal(previous);if(old.pending.length)throw fail("eventDeliveryPending");if(probe.generation<=old.context.generation)throw fail("eventDeliveryConflict"); }
  const answer=await admitEventWithGuest(guest,probe);
  if(!answer.ok)return answer;
  if(answer.nextSequence!==probe.sequence+1)return {ok:false,code:"invalidEventResponse"};
  const {envelope,sequence,...context}=probe;
  return {ok:true,state:journal({profile:"foundry/event-journal/1",source,context,initialSequence:sequence,nextSequence:sequence,pending:[],lastAcknowledged:null})};
}
export async function stageEventDelivery(current, input, source, guest) {
  const state=journal(current);
  if(state.source!==source)throw fail("eventDeliverySourceChanged");
  if(state.pending.length>=256)throw fail("eventDeliveryLimitExceeded");
  let envelope;
  try { encodeEventRequest({...state.context,sequence:state.nextSequence,envelope:input});envelope=structuredClone(input); } catch {throw fail("invalidEventDelivery");}
  const answer=await admitEventWithGuest(guest,{...state.context,sequence:state.nextSequence,envelope});
  if(!answer.ok)return answer;
  if(answer.nextSequence!==state.nextSequence+1)return {ok:false,code:"invalidEventResponse"};
  state.pending.push(envelope);state.nextSequence=answer.nextSequence;
  return {ok:true,state:journal(state)};
}
export function acknowledgeEventDelivery(current, input) {
  const state=journal(current);
  if(!input || Object.keys(input).sort().join()!=="consumer,generation,sequence" || !uint(input.generation) || !uint(input.sequence) || !input.consumer || Object.keys(input.consumer).sort().join()!=="key,version" || !uint(input.consumer.key) || !uint(input.consumer.version)) throw fail("invalidEventDelivery");
  const receipt={generation:input.generation,sequence:input.sequence,consumer:{key:input.consumer.key,version:input.consumer.version}};
  if(same(receipt,state.lastAcknowledged))return state;
  const head=state.pending[0];
  if(!head || receipt.generation!==state.context.generation || receipt.sequence!==head.sequence || (receipt.consumer.key!==head.consumer.key || receipt.consumer.version!==head.consumer.version))throw fail("eventDeliveryAcknowledgementRefused");
  state.pending.shift();state.lastAcknowledged=receipt;return state;
}
export function copyEventCommand(operation, input) {
  const fields={configure:"appId,context",deliver:"appId,envelope",pending:"appId",acknowledge:"appId,receipt"};
  if (!Object.hasOwn(fields,operation) || !input || Object.keys(input).sort().join()!==fields[operation]) throw fail("invalidEventDelivery");
  appIdOf(input.appId);
  try {
    if(operation==="configure")encodeEventRequest(input.context);
    if(new TextEncoder().encode(JSON.stringify(input)).length>1024*1024)throw fail("invalidEventDelivery");
    return structuredClone(input);
  } catch {throw fail("invalidEventDelivery");}
}
const appIdOf = id => {if(!Number.isSafeInteger(id)||id<1)throw fail("invalidEventDelivery");return id;};
const read = (db,id) => storageRequest(db,"refs","readonly",store=>store.get(`events:${id}`));
export async function readEventJournal(db, appId) {
  appIdOf(appId);const state=journal(await read(db,appId));
  const reference=await storageRequest(db,"refs","readonly",store=>store.get(`${appId}:main`));
  return {state,current:state.source===reference};
}
export async function updateEventJournal(db, guest, appId, operation, input) {
  appIdOf(appId);let copied;
  try {copied=structuredClone(input);}catch{throw fail("invalidEventDelivery");}
  const source=await captureProjectSource(db,appId),previous=await read(db,appId);
  let result;
  if(operation==="configure")result=await prepareEventJournal(copied,source.reference,previous,guest);
  else if(operation==="deliver")result=await stageEventDelivery(previous,copied,source.reference,guest);
  else if(operation==="acknowledge")result={ok:true,state:acknowledgeEventDelivery(previous,copied)};
  else throw fail("invalidEventDelivery");
  if(!result.ok)return result;
  try {
    await commitPreparedProject(db,appId,[],source,{oid:source.reference,files:source.files,objects:[]},{recordVersion:false,eventUpdate:{expected:previous,state:result.state}});
  } catch(error) {if(error?.code==="projectEditConflict")throw fail("eventDeliveryConflict");throw error;}
  return result;
}
