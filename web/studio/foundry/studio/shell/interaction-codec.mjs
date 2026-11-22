// InteractionProtocol.interactionWireBoundary: representation, never semantic evaluation.
const fault=code=>Object.assign(new Error(`Interaction request refused: ${code}`),{code});
const shape=(value,keys)=>{if(!value||typeof value!=="object"||Array.isArray(value)||Object.keys(value).sort().join()!==[...keys].sort().join())throw fault("malformedInteraction");};
export function encodeInteraction(input){
 shape(input,["profile","state","event","audioGranted","reducedMotion"]);const p=input.profile;shape(p,["version","initial","screens","frames","edges"]);
 const words=[];const word=value=>{if(!Number.isInteger(value)||value<0||value>0xffffffff)throw fault("malformedInteraction");words.push(value);};
 for(const key of ["version","initial","screens"])word(p[key]);
 for(const [key,fields,max] of [["frames",["screen","palette","tone","motion"],32],["edges",["origin","event","destination"],128]]){
  if(!Array.isArray(p[key]))throw fault("malformedInteraction");if(p[key].length>max)throw fault("interactionTooLarge");word(p[key].length);
  for(const item of p[key]){shape(item,fields);for(const field of fields)word(item[field]);}
 }
 word(input.state);word(input.event);for(const key of ["audioGranted","reducedMotion"]){if(typeof input[key]!=="boolean")throw fault("malformedInteraction");word(Number(input[key]));}
 const bytes=new Uint8Array(words.length*4),view=new DataView(bytes.buffer);words.forEach((value,i)=>view.setUint32(i*4,value,true));
 const hex=Array.from(bytes,byte=>byte.toString(16).padStart(2,"0")).join("");if(hex.length>8192)throw fault("interactionTooLarge");return new TextEncoder().encode("interact "+hex);
}
const refusals=new Set(["invalidInteractionModel","invalidInteractionState","unknownInteractionEvent","malformedInteraction","interactionTooLarge"]);
export function decodeInteraction(bytes){
 try{
  if(!(bytes instanceof Uint8Array)||bytes.length>64)throw Error();const text=new TextDecoder("utf-8",{fatal:true}).decode(bytes);
  if(refusals.has(text))return {ok:false,code:text};
  if(!/^ok [0-9a-f]{40}$/.test(text))throw Error();const data=Uint8Array.from(text.slice(3).match(/../g),pair=>Number.parseInt(pair,16)),view=new DataView(data.buffer);
  const output=Object.fromEntries(["state","screen","palette","tone","motion"].map((key,i)=>[key,view.getUint32(i*4,true)]));
  if(output.state>=32||output.screen>=32||output.palette>=8||output.tone>20000||(output.tone!==0&&output.tone<20)||output.motion>1000)throw Error();return {ok:true,output};
 }catch{return {ok:false,code:"invalidInteractionResponse"};}
}
export async function executeInteraction(guest,input){
 let bytes;try{bytes=encodeInteraction(input);}catch(error){return {ok:false,code:error?.code==="interactionTooLarge"?error.code:"malformedInteraction"};}
 try{return decodeInteraction(await (await guest).dispatch(bytes));}catch{return {ok:false,code:"interactionRuntimeFailed"};}
}
