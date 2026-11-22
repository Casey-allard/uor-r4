// ChatQueueProtocol.boundary: representation only. All transitions run in the guest.
const errors=new Set(['invalidQueueState','invalidQueueCommand','queueRevisionConflict','queueFull','queueItemMissing','queueItemDuplicate','queueActiveConflict','queuePaused','queueStopConflict','queueOwnerConflict','queueRevisionExhausted','malformedQueueRequest','queueRequestTooLarge']);
const failure=code=>Object.assign(new Error('Queue request was refused'),{code});
const exact=(value,keys)=>{if(!value||typeof value!=='object'||Array.isArray(value)||Object.keys(value).sort().join()!==keys.split(',').sort().join())throw failure('malformedQueueRequest');};
const uint=n=>{if(!Number.isInteger(n)||n<0||n>0xffffffff)throw failure('malformedQueueRequest');return n;};
const stateFields=['revision','paused','stopPolicy','active','owner'],commandFields=['kind','item','index','owner','observedStop'];
export function encodeQueueRequest(raw){
 let input;try{input=structuredClone(raw);}catch{throw failure('malformedQueueRequest');}
 exact(input,'state,expectedRevision,command');exact(input.state,'revision,paused,stopPolicy,active,owner,items');exact(input.command,'kind,item,index,owner,observedStop');
 if(!Array.isArray(input.state.items)||input.state.items.length>32)throw failure('malformedQueueRequest');
 const words=[1,...stateFields.map(k=>uint(input.state[k])),input.state.items.length,...input.state.items.map(uint),uint(input.expectedRevision),...commandFields.map(k=>uint(input.command[k]))];
 const bytes=new Uint8Array(words.length*4),view=new DataView(bytes.buffer);words.forEach((word,i)=>view.setUint32(i*4,word,true));return new TextEncoder().encode('queue '+[...bytes].map(b=>b.toString(16).padStart(2,'0')).join(''));
}
export function decodeQueueResponse(bytes){
 try{
  if(!(bytes instanceof Uint8Array)||bytes.length>512)throw Error();const text=new TextDecoder('utf-8',{fatal:true}).decode(bytes);if(errors.has(text))return {ok:false,code:text};if(!/^ok (?:[0-9a-f]{8})+$/.test(text))throw Error();
  const raw=Uint8Array.from(text.slice(3).match(/../g),x=>parseInt(x,16)),view=new DataView(raw.buffer);let offset=0;const word=()=>{if(offset>=raw.length)throw Error();const n=view.getUint32(offset,true);offset+=4;return n;};
  if(word()!==1)throw Error();const state=Object.fromEntries(stateFields.map(k=>[k,word()])),count=word();if(count>32||state.paused>1)throw Error();state.items=Array.from({length:count},word);
  if(offset!==raw.length||state.items.some(n=>!n)||new Set(state.items).size!==count||(state.active?state.items[0]!==state.active||!state.owner:state.owner!==0))throw Error();return {ok:true,state};
 }catch{return {ok:false,code:'invalidQueueResponse'};}
}
export async function queueWithGuest(ready,input){
 let request;try{request=encodeQueueRequest(input);}catch(error){return {ok:false,code:error.code};}
 try{return decodeQueueResponse(await (await ready).dispatch(request));}catch{return {ok:false,code:'queueRuntimeFailed'};}
}
