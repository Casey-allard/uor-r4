// RoutingProtocol.boundary: representation only; ranking and admission execute in the guest.
import {encodeComposition} from './composition-codec.mjs';
const fail=code=>Object.assign(new Error(code==='routingRequestTooLarge'?'Routing request exceeds its bound':'Invalid routing request'),{code});
const exact=(value,names)=>{if(!value||typeof value!=='object'||Array.isArray(value)||Object.keys(value).sort().join()!==[...names].sort().join())throw fail('malformedRoutingRequest');};
const uint=value=>{if(!Number.isInteger(value)||value<0||value>0xffffffff)throw fail('malformedRoutingRequest');return value;};
export function encodeRoutingRequest(value){
 let input;try{input=structuredClone(value);}catch{throw fail('malformedRoutingRequest');}
 exact(input,['expectedIntent','registryRevision','intent','relations','policy','catalog']);exact(input.intent,['id','registryRevision','concepts','context']);
 const words=[1,uint(input.expectedIntent),uint(input.registryRevision),uint(input.intent.id),uint(input.intent.registryRevision)];
 const list=(items,max,emit)=>{if(!Array.isArray(items)||items.length>max)throw fail('malformedRoutingRequest');words.push(items.length);items.forEach(emit);};
 for(const field of ['concepts','context'])list(input.intent[field],32,item=>words.push(uint(item)));
 list(input.relations,128,item=>{exact(item,['fromConcept','toConcept','weight']);for(const field of ['fromConcept','toConcept','weight'])words.push(uint(item[field]));});
 let composition;try{if(!Array.isArray(input.catalog)||input.catalog.some(feature=>Object.values(feature).some(item=>Array.isArray(item)&&item.length>32)))throw Error();composition=encodeComposition(input.policy,input.catalog).slice(8);}catch{throw fail('malformedRoutingRequest');}
 const bytes=new Uint8Array(words.length*4),view=new DataView(bytes.buffer);words.forEach((word,index)=>view.setUint32(index*4,word,true));
 const prefix=new TextEncoder().encode('propose '+[...bytes].map(b=>b.toString(16).padStart(2,'0')).join(''));
 if(prefix.length-8+composition.length>32768)throw fail('routingRequestTooLarge');
 const encoded=new Uint8Array(prefix.length+composition.length);encoded.set(prefix);encoded.set(composition,prefix.length);return encoded;
}
const proposalCodes=['accepted','invalidRegistry','invalidProposal','staleIntent','staleRegistry','unresolvedAmbiguity','unresolvedConstraint','unregisteredFeature','compositionRejected'];
const compositionCodes=['','emptyComposition','invalidDeclaration','duplicateFeature','missingDependency','incompatibleInput','featureConflict','permissionDenied','unsupportedTarget','missingEvidence','budgetExceeded'];
const errors=new Set(['invalidSemanticIntent','invalidSemanticCatalog','invalidSemanticRelations','malformedRoutingRequest','routingRequestTooLarge']);
export function decodeRoutingResponse(bytes){
 try{
  if(!(bytes instanceof Uint8Array)||bytes.length>4096)throw Error();const text=new TextDecoder('utf-8',{fatal:true}).decode(bytes);
  if(errors.has(text))return {ok:false,code:text};if(!/^ok (?:[0-9a-f]{8})+$/.test(text))throw Error();
  const raw=Uint8Array.from(text.slice(3).match(/../g),pair=>parseInt(pair,16)),view=new DataView(raw.buffer);let offset=0;
  const word=()=>{if(offset>=raw.length)throw Error();const value=view.getUint32(offset,true);offset+=4;return value;};
  const list=read=>{const length=word();if(length>32)throw Error();return Array.from({length},read);};
  const reference=()=>{const key=word(),version=word();if(!key||!version)throw Error();return {key,version};};
  const version=word(),status=word(),detail=word(),intent=word(),registryRevision=word(),confidence=word();
  if(version!==1||status>8||!intent||!registryRevision||confidence!==0||(status===8?detail<1||detail>10:detail!==0))throw Error();
  const ranked=list(()=>{const value=reference(),score=word();if(!score)throw Error();return {...value,score};}),selected=list(reference),ambiguities=list(word),unresolved=list(word);
  for(const values of [ranked.map(r=>r.key),selected.map(r=>r.key),ambiguities,unresolved])if(values.some(v=>!v)||new Set(values).size!==values.length)throw Error();
  if(offset!==raw.length||(status===0&&(!ranked.length||!selected.length||ambiguities.length||unresolved.length)))throw Error();
  return {ok:status===0,...(status?{code:proposalCodes[status],...(detail?{cause:compositionCodes[detail]}:{})}:{}),ranked,proposal:{intent,registryRevision,confidence,selected,ambiguities,unresolved}};
 }catch{return {ok:false,code:'invalidRoutingResponse'};}
}
export async function routeWithGuest(guestPromise,input){
 let request;try{request=encodeRoutingRequest(input);}catch(error){return {ok:false,code:error.code};}
 try{return decodeRoutingResponse(await (await guestPromise).dispatch(request));}catch{return {ok:false,code:'routingRuntimeFailed'};}
}
