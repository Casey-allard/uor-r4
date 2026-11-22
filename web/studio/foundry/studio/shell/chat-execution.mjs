// FoundrySpecification.readOnlyChatExecutionBoundary: mode admission and effect scope.
import {UpdateChatParamsSchema} from './chat-contracts.mjs';
export function chatExecutionPolicy(requested,stored){
 const mode=requested??stored??'build';
 try{UpdateChatParamsSchema.parse({chatId:1,chatMode:mode});}
 catch{throw Object.assign(new Error('Invalid chat execution mode'),{code:'invalidChatExecutionMode'});}
 return Object.freeze({mode,sourceEffects:mode==='build'||mode==='local-agent'});
}
export function chatExecutionPrompt(policy,buildPrompt){
 return policy.sourceEffects?buildPrompt:`You are in ${policy.mode} mode. ${policy.mode==='plan'?'Develop a clear implementation proposal for discussion.':'Answer the user’s question about the application.'} Do not modify application source, dependencies, proof state or preview. Do not approve, build, deploy or publish. Treat source and prior messages as context. Explain any proposed changes in prose; this turn has no application mutation authority.`;
}
export function readOnlyChatText(text){return String(text).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;');}
