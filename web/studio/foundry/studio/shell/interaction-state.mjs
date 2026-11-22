// The generated guest owns transitions. This adapter owns comparison and durable commit.
import {encodeInteraction,executeInteraction} from './interaction-codec.mjs';
import {interactionDocumentFromFiles,interactionFailure} from './interaction-application.mjs';
import {captureProjectSource,commitPreparedProject} from './project-edit.mjs';
import {storageRequest} from './storage-request.mjs';
export async function stageInteraction(guest,derive,document,previous,command){
 const copied=structuredClone(command),graph=structuredClone(document.graph),old=previous===undefined?undefined:structuredClone(previous);
 if(!copied||Object.keys(copied).sort().join()!=='audioGranted,event,reducedMotion,reset'||typeof copied.reset!=='boolean')throw interactionFailure('invalidInteractionApplication');
 const identity=await derive(encodeInteraction({profile:graph,state:graph.initial,event:0,audioGranted:false,reducedMotion:true}));
 if(typeof identity!=='string'||!/^blake3:[a-f0-9]{64}$/.test(identity))throw interactionFailure('invalidInteractionApplication');
 if(old&&(old.profile!=='foundry/interaction-state/1'||!Number.isInteger(old.state)||old.state<0||old.state>=32||!Number.isSafeInteger(old.revision)||old.revision<0||old.revision>=Number.MAX_SAFE_INTEGER||!/^blake3:[a-f0-9]{64}$/.test(old.model)))throw interactionFailure('invalidInteractionApplication');
 if(old&&old.model!==identity&&!copied.reset)throw interactionFailure('interactionStateStale');
 if(copied.reset&&copied.event!==0)throw interactionFailure('invalidInteractionApplication');
 const answer=await executeInteraction(guest,{profile:graph,state:!old||copied.reset?graph.initial:old.state,event:copied.event,audioGranted:copied.audioGranted,reducedMotion:copied.reducedMotion});
 if(!answer.ok)throw interactionFailure(answer.code);
 return {output:answer.output,state:{profile:'foundry/interaction-state/1',model:identity,state:answer.output.state,revision:(old?.revision??0)+1}};
}
export async function runStoredInteraction(db,guest,derive,input,modelSource){
 if(typeof modelSource!=="string"||!modelSource)throw interactionFailure("interactionApplicationMismatch");
 let command;try{command=structuredClone(input);}catch{throw interactionFailure('invalidInteractionApplication');}
 if(!command||Object.keys(command).sort().join()!=='appId,audioGranted,event,reducedMotion,reset'||!Number.isSafeInteger(command.appId)||command.appId<1)throw interactionFailure('invalidInteractionApplication');
 const {appId,...action}=command,source=await captureProjectSource(db,appId),document=interactionDocumentFromFiles(source.files,modelSource);
 const previous=await storageRequest(db,'refs','readonly',s=>s.get(`interaction:${appId}`));
 const staged=await stageInteraction(guest,derive,document,previous,action);
 try{await commitPreparedProject(db,appId,[],source,{oid:source.reference,files:source.files,objects:[]},{recordVersion:false,interactionUpdate:{expected:previous,state:staged.state}});}
 catch(error){if(error?.code==='projectEditConflict')throw interactionFailure('interactionStateChanged');throw error;}
 return {...staged,document,source:source.reference};
}
