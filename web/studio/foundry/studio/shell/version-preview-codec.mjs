// Exact representation only; VersionPreviewProtocol owns every preview transition.
const stateFields=['revision','active','owner','origin','current'],commandFields=['kind','owner','target'];
const errors=new Set(['invalidPreviewState','invalidPreviewCommand','previewRevisionConflict','previewRevisionExhausted','previewAlreadyOwned','previewInactive','previewWrongOwner','malformedPreviewRequest','previewRequestTooLarge']);
const fail=()=>{throw Object.assign(new Error('Invalid preview request'),{code:'malformedPreviewRequest'});};
const uint=n=>{if(!Number.isInteger(n)||n<0||n>0xffffffff)fail();return n;};
const exact=(x,keys)=>{if(!x||typeof x!=='object'||Array.isArray(x)||Object.keys(x).sort().join()!==[...keys].sort().join())fail();};
const address=x=>{if(!Array.isArray(x)||x.length!==8)fail();return x.map(uint);};
const stateWords=x=>{exact(x,stateFields);return [uint(x.revision),uint(x.active),uint(x.owner),...address(x.origin),...address(x.current)];};
const hex=words=>{const bytes=new Uint8Array(words.length*4),view=new DataView(bytes.buffer);words.forEach((n,i)=>view.setUint32(i*4,uint(n),true));return [...bytes].map(b=>b.toString(16).padStart(2,'0')).join('');};
export function previewAddressWords(value){if(typeof value!=='string'||!/^blake3:[0-9a-f]{64}$/.test(value))fail();const bytes=Uint8Array.from(value.slice(7).match(/../g),s=>parseInt(s,16)),view=new DataView(bytes.buffer);return Array.from({length:8},(_,i)=>view.getUint32(i*4,true));}
export function previewAddressIdentity(value){return 'blake3:'+hex(address(value));}
export function encodePreviewRequest(raw){try{const input=structuredClone(raw);exact(input,['state','expectedRevision','command']);exact(input.command,commandFields);return new TextEncoder().encode('preview '+hex([1,...stateWords(input.state),uint(input.expectedRevision),uint(input.command.kind),uint(input.command.owner),...address(input.command.target)]));}catch{fail();}}
export function capturePreviewState(raw){const state=structuredClone(raw);stateWords(state);if(state.active>1||(state.active?state.owner===0:state.owner!==0||state.origin.some(Boolean)||state.current.some(Boolean)))fail();return state;}
export function decodePreviewResponse(raw){
 try{if(!(raw instanceof Uint8Array)||raw.length>235)throw Error();const text=new TextDecoder('utf-8',{fatal:true}).decode(raw);if(errors.has(text))return {ok:false,code:text};if(!/^ok [0-9a-f]{232}$/.test(text))throw Error();
  const bytes=Uint8Array.from(text.slice(3).match(/../g),s=>parseInt(s,16)),view=new DataView(bytes.buffer),words=Array.from({length:29},(_,i)=>view.getUint32(i*4,true));if(words[0]!==1||words[20]>1)throw Error();
  const state={revision:words[1],active:words[2],owner:words[3],origin:words.slice(4,12),current:words.slice(12,20)};
  capturePreviewState(state);return {ok:true,state,writeRequested:!!words[20],target:words.slice(21)};
 }catch{return {ok:false,code:'invalidPreviewResponse'};}
}
export async function previewWithGuest(ready,input){let bytes;try{bytes=encodePreviewRequest(input);}catch{return {ok:false,code:'malformedPreviewRequest'};}try{const guest=await ready;return decodePreviewResponse(await guest.dispatch(bytes));}catch{return {ok:false,code:'previewRuntimeFailed'};}}
