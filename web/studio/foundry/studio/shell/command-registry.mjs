import {refuseCommandBinding} from "./host-boundary.mjs";
// FoundrySpecification.commandRegistryBoundary: inventory admission is not effect permission.
import {commandRegistryData} from './command-registry-data.mjs';
const fail=code=>Object.assign(new Error(code==='commandBindingMissing'?'This command has no canonical binding':code==='commandBindingMismatch'?'This command differs from its canonical binding':code==='unsupportedChannel'?'This capability is unavailable in the browser':'The canonical command registry is invalid'),{code});
const keys=(value,expected)=>value&&typeof value==='object'&&!Array.isArray(value)&&Object.keys(value).sort().join()===expected.split(',').sort().join();
const freeze=value=>{if(value&&typeof value==='object'){for(const child of Object.values(value))freeze(child);Object.freeze(value);}return value;};
export function createCommandRegistry(data){
 try{
  const copied=structuredClone(data);
  if(!keys(copied,'profile,features,commands')||copied.profile!=='foundry/command-registry/1'||!Array.isArray(copied.features)||!Array.isArray(copied.commands)||copied.features.length<1||copied.features.length>128||copied.commands.length<1||copied.commands.length>1024)throw Error();
  const features=new Map(),commands=new Map();
  for(const feature of copied.features){
   if(!keys(feature,'id,version,owner,effects,targets,assurance,quality')||typeof feature.id!=='string'||!/^[-a-z]+$/.test(feature.id)||feature.version!==1||typeof feature.owner!=='string'||!/^\w+\.\w+$/.test(feature.owner)||features.has(feature.id)||!Array.isArray(feature.effects)||new Set(feature.effects).size!==feature.effects.length||feature.effects.some(e=>!['storage','network','execution'].includes(e))||JSON.stringify(feature.targets)!=='["browser"]'||feature.assurance!=='boundary-inventory'||typeof feature.quality!=='string'||feature.quality.length<1||feature.quality.length>1024)throw Error();features.set(feature.id,feature);
  }
  let last='';
  for(const command of copied.commands){
   if(!keys(command,'channel,feature,version,owner,adapter,availability,input,output')||typeof command.channel!=='string'||!/^[-A-Za-z0-9:]+$/.test(command.channel)||command.channel.length>128||command.channel<=last||commands.has(command.channel)||!features.has(command.feature)||command.version!==features.get(command.feature).version||command.owner!==features.get(command.feature).owner||!['adapter','unavailable'].includes(command.availability)||command.adapter!==(command.availability==='adapter'?'studio/shell/host.js':'studio/shell/host-boundary.mjs')||![command.input,command.output].every(v=>['imported-renderer','product-boundary'].includes(v))||(command.availability==='unavailable'&&(command.feature!=='unavailable'||features.get(command.feature).effects.length)))throw Error();
   commands.set(command.channel,command);last=command.channel;
  }
  freeze(copied);
  return Object.freeze({
   entries:copied.commands,features:copied.features,
   resolve(channel,handler){const binding=commands.get(channel);if(!binding)throw fail('commandBindingMissing');if((typeof handler==='function')!==(binding.availability==='adapter'))throw fail('commandBindingMismatch');if(binding.availability==='unavailable')throw fail('unsupportedChannel');return binding;},
   assertInventory({handlers,inputs,outputs}){
    if(![handlers,inputs,outputs].every(v=>Array.isArray(v)&&v.every(x=>typeof x==='string')&&new Set(v).size===v.length))throw fail('commandRegistryInvalid');
    const same=(left,right)=>JSON.stringify([...left].sort())===JSON.stringify([...right].sort());
    if(!same(handlers,copied.commands.filter(c=>c.availability==='adapter').map(c=>c.channel))||!same(inputs,copied.commands.filter(c=>c.input==='imported-renderer').map(c=>c.channel))||!same(outputs,copied.commands.filter(c=>c.output==='imported-renderer').map(c=>c.channel)))throw fail('commandBindingMismatch');return true;
   }
  });
 }catch{throw fail('commandRegistryInvalid');}
}
export const commandRegistry=createCommandRegistry(commandRegistryData);
export function invokeBoundCommand(registry,channel,args,handler,invoke){
 try{registry.resolve(channel,handler);}catch(error){return Promise.resolve(refuseCommandBinding(channel,error.code));}
 return invoke(channel,args,handler);
}
