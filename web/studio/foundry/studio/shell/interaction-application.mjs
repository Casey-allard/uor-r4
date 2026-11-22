// FoundrySpecification.interactionJourneyBoundary: canonical source and bounded presentation.
import {encodeInteraction,executeInteraction} from './interaction-codec.mjs';
export const interactionFailure=code=>Object.assign(new Error(({invalidInteractionApplication:'Invalid interaction application',interactionApplicationMismatch:'The application does not match its canonical source',interactionStateChanged:'Application state changed; open it again',interactionStateStale:'The model changed; reset its saved state explicitly',interactionAudioUnavailable:'Audio is unavailable'})[code]||'Interaction refused'),{code});
const fail=()=>{throw interactionFailure('invalidInteractionApplication');};
const shape=(x,keys)=>{if(!x||typeof x!=='object'||Array.isArray(x)||Object.keys(x).sort().join()!==keys.split(',').sort().join())fail();};
const text=(s,max)=>{if(typeof s!=='string'||!s.trim()||s.length>max||/[\u0000-\u0008\u000b\u000c\u000e-\u001f]/.test(s))fail();};
export function readInteractionDocument(raw){
 try{
  if(typeof raw!=='string'||new TextEncoder().encode(raw).length>65536)fail();const d=JSON.parse(raw);
  shape(d,'profile,title,graph,screens,controls');if(d.profile!=='foundry/interaction-application/1')fail();text(d.title,120);
  encodeInteraction({profile:d.graph,state:d.graph.initial,event:0,audioGranted:false,reducedMotion:true});
  if(!Array.isArray(d.screens)||d.screens.length!==d.graph.screens||d.screens.length<1||d.screens.length>32)fail();
  for(const s of d.screens){shape(s,'title,description,illustration');text(s.title,120);text(s.description,2000);if(!['landscape','flower','geometry','night'].includes(s.illustration))fail();}
  if(!Array.isArray(d.controls)||d.controls.length>32)fail();const events=new Set();
  for(const c of d.controls){shape(c,'event,label');if(!Number.isInteger(c.event)||c.event<1||c.event>32||events.has(c.event))fail();events.add(c.event);text(c.label,80);}
  if(d.graph.edges.some(e=>!events.has(e.event)))fail();return d;
 }catch{fail();}
}
export function interactionModule(graph){
 encodeInteraction({profile:graph,state:graph.initial,event:0,audioGranted:false,reducedMotion:true});
 const list=(rows,type,fields)=>rows.reduceRight((tail,row)=>`(cons (record ${type} {${fields.map(k=>`${k} := ${row[k]}`).join(', ')}}) ${tail})`,`(nil ${type})`);
 const frames=list(graph.frames,'InteractionModel::InteractionFrame',['screen','palette','tone','motion']),edges=list(graph.edges,'InteractionModel::InteractionEdge',['origin','event','destination']);
 return `\\begin{lexlean}{Application}\n\\useglossary{lexlean.std.bool@1.2.0}\n\\useglossary{lexlean.std.nat@1.2.0}\n\\importmodule{InteractionModel}\n\\title{Boolean}\n\\begin{semanticmodule}\ndefinition{\n  name: applicationProfile,\n  parameters: [],\n  result: InteractionModel::InteractionProfile,\n  body: (record InteractionModel::InteractionProfile {version := ${graph.version}, initial := ${graph.initial}, screens := ${graph.screens}, frames := ${frames}, edges := ${edges}})\n}\n\\end{semanticmodule}\n\\end{lexlean}\n`;
}
export function interactionDocumentFromFiles(rows,modelSource){
 const files=new Map(rows),d=readInteractionDocument(files.get('prism/interaction.json'));
 if(modelSource!==undefined&&files.get('prism/InteractionModel.lex.tex')!==modelSource)throw interactionFailure('interactionApplicationMismatch');
 if(files.get('prism/Application.lex.tex')!==interactionModule(d.graph))throw interactionFailure('interactionApplicationMismatch');return d;
}
export async function validateInteractionDocument(raw,guest){
 const document=readInteractionDocument(raw),answer=await executeInteraction(guest,{profile:document.graph,state:document.graph.initial,event:0,audioGranted:false,reducedMotion:true});
 if(!answer.ok)throw interactionFailure(answer.code);return document;
}
export function interactionPreset(kind){
 if(!['gallery','garden'].includes(kind))fail();const garden=kind==='garden';
 return {profile:'foundry/interaction-application/1',title:garden?'Resonant garden':'Quiet gallery',graph:{version:1,initial:0,screens:2,frames:[{screen:0,palette:garden?2:0,tone:garden?220:0,motion:garden?10:0},{screen:1,palette:garden?3:1,tone:garden?440:0,motion:garden?20:0}],edges:[{origin:0,event:1,destination:1},{origin:1,event:2,destination:0}]},screens:garden?[{title:'Seed',description:'A quiet seed waits. Grow it to coordinate a new image, color, movement and optional tone.',illustration:'landscape'},{title:'Bloom',description:'The garden is in bloom. Return to the seed whenever you like.',illustration:'flower'}]:[{title:'Open landscape',description:'An open landscape in a quiet gallery. Visit the next room to see the night sky.',illustration:'landscape'},{title:'Night sky',description:'A still night sky. Return to the landscape using the previous room control.',illustration:'night'}],controls:[{event:1,label:garden?'Grow':'Next room'},{event:2,label:garden?'Return to seed':'Previous room'}]};
}
