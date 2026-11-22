// FoundrySpecification.discoveryBoundary; interpretation is finite catalog data.
import {discoveryCatalog as source} from './discovery-catalog.mjs';
import {interactionPreset} from './interaction-application.mjs';
import {routeWithGuest} from './routing-codec.mjs';
const catalog=structuredClone(source);
const messages={invalidDiscoveryIntent:'Use up to 32 supported words (512 characters).',discoveryRefused:'This catalog cannot fulfill the complete request.',discoveryReviewChanged:'The request or catalog changed. Review it again.',discoveryUnavailable:'Application discovery is unavailable.'};
export const discoveryFailure=code=>Object.assign(new Error(messages[code]||messages.discoveryUnavailable),{code});
function exact(value,keys){if(!value||typeof value!=='object'||Array.isArray(value)||Object.keys(value).sort().join()!==keys.split(',').sort().join())throw discoveryFailure('invalidDiscoveryIntent');}
export function interpretDiscovery(text){
 if(typeof text!=='string'||!text.trim()||text.length>catalog.maximumCharacters||!/^[a-zA-Z\s,.!?]+$/.test(text))throw discoveryFailure('invalidDiscoveryIntent');
 const words=text.toLowerCase().split(/[\s,.!?]+/).filter(Boolean);if(!words.length||words.length>catalog.maximumWords)throw discoveryFailure('invalidDiscoveryIntent');
 const unknown=[],concepts=[];
 for(const word of words){if(catalog.ignore.includes(word))continue;const concept=Object.hasOwn(catalog.words,word)?catalog.words[word]:undefined;if(concept===undefined){unknown.push(word);concepts.push(0xffffffff);}else concepts.push(concept);}
 if(!concepts.length)throw discoveryFailure('invalidDiscoveryIntent');
 return {unknown:[...new Set(unknown)],request:{expectedIntent:1,registryRevision:catalog.revision,intent:{id:1,registryRevision:catalog.revision,concepts:[...new Set(concepts)],context:[]},relations:structuredClone(catalog.relations),policy:structuredClone(catalog.policy),catalog:structuredClone(catalog.catalog)}};
}
export function createDiscovery({guest,derive,create}){
 async function discover(raw){
  let input;try{input=structuredClone(raw);exact(input,'text');}catch{throw discoveryFailure('invalidDiscoveryIntent');}
  const {request,unknown}=interpretDiscovery(input.text),answer=await routeWithGuest(guest,request);
  const applications=catalog.applications.filter(app=>answer.proposal?.selected.some(ref=>ref.key===app.key&&ref.version===app.version));
  if(!answer.ok||unknown.length||applications.length!==1)return {ok:false,code:'discoveryRefused',reason:answer.code||'unresolvedConstraint',unknown,ambiguities:answer.proposal?.ambiguities||[]};
  let review;try{review=await derive(new TextEncoder().encode(JSON.stringify({text:input.text,catalog,answer,profile:interactionPreset(applications[0].kind)})));}catch{throw discoveryFailure('discoveryUnavailable');}
  if(typeof review!=='string'||!review)throw discoveryFailure('discoveryUnavailable');
  return {ok:true,text:input.text,review,application:structuredClone(applications[0]),selected:answer.proposal.selected,confidence:0};
 }
 async function accept(raw){
  let input;try{input=structuredClone(raw);exact(input,'text,name,review');if(typeof input.review!=='string'||input.review.length>256||typeof input.name!=='string'||!input.name.trim()||input.name.length>120)throw Error();}catch{throw discoveryFailure('invalidDiscoveryIntent');}
  const proposal=await discover({text:input.text});
  if(!proposal.ok)throw discoveryFailure('discoveryRefused');
  if(input.review!==proposal.review)throw discoveryFailure('discoveryReviewChanged');
  return create({name:input.name,kind:proposal.application.kind});
 }
 return {discover,accept};
}
