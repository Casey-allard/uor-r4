// FoundrySpecification.modulePresentationBoundary; this comparison grants no effects.
import {validateSpec} from './prism/spec.mjs';
import {checkView} from './live/project.mjs';
// FoundrySpecification.largeModulePresentationBoundary: the imported limit is
// a proof profile. Validate every structural rule without shortening the module.
function validatePresentationSpec(spec){
 if(!Array.isArray(spec?.rules)||spec.rules.length<=24){validateSpec(spec);return;}
 for(let offset=0;offset<spec.rules.length;offset+=24)validateSpec({...spec,rules:spec.rules.slice(offset,offset+24)});
}
const interfaceOf=spec=>JSON.stringify({name:spec.name,fields:spec.fields.map(field=>field.kind==='choice'?{name:field.name,kind:field.kind,values:field.values}:{name:field.name,kind:field.kind,min:field.min??0,max:field.max}),outcomes:spec.outcomes});
export function modulePresentation(previous,incoming){
 let next;
 try{next=structuredClone(incoming);validatePresentationSpec(next);if(checkView(next).length)throw Error();}
 catch{return {code:'invalidCanonicalModule',error:'The canonical decision module is invalid or unsupported'};}
 if(previous===null||previous===undefined)return {spec:next,reset:false};
 try{
  const prior=structuredClone(previous);validatePresentationSpec(prior);
  if(checkView(prior).length||interfaceOf(prior)!==interfaceOf(next))return {spec:next,reset:true};
  const candidate=structuredClone(next);
  for(const key of ['purpose','view','meanings'])if(prior[key]!==undefined)candidate[key]=prior[key];
  validatePresentationSpec(candidate);
  return {spec:candidate,reset:false};
 }catch{return {spec:next,reset:true};}
}
