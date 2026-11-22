// FoundrySpecification.browserChatModeControlsBoundary: capture supported defaults.
import {chatModeCatalog} from './chat-mode-catalog.mjs';
import {UpdateChatParamsSchema} from './chat-contracts.mjs';
const refuse=()=>Object.assign(new Error('Invalid chat execution mode'),{code:'invalidChatExecutionMode'});
export function browserChatMode(requested,settings={}){
 const mode=requested!==undefined?requested:settings?.defaultChatMode!==undefined?settings.defaultChatMode:settings?.selectedChatMode!==undefined?settings.selectedChatMode:'build';
 try{
  UpdateChatParamsSchema.parse({chatId:1,chatMode:mode});
  if(!chatModeCatalog.modes.some(choice=>choice.id===mode))throw Error();
 }catch{throw refuse();}
 return mode;
}
export function assertBrowserChatModes(patch){
 for(const key of ['selectedChatMode','defaultChatMode'])if(patch?.[key]!==undefined)browserChatMode(patch[key]);
}
export function browserChatModeSettings(value){
 const settings={...value};
 if(settings.defaultChatMode===undefined)settings.defaultChatMode=settings.selectedChatMode??'build';
 return settings;
}
