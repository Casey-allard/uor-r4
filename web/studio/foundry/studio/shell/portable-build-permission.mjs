// FoundrySpecification.portableBuildPermissionBoundary: one captured source, one revocable build.
const messages={portableBuildStyleIncomplete:'The styling output does not cover the rendered classes',invalidPortableBuildPermission:'Invalid portable build permission',portableBuildPermissionDenied:'Portable build declined',portableBuildPermissionExpired:'Portable build permission expired',portableBuildPermissionRevoked:'Portable build stopped or permission revoked',portableBuildPermissionOffline:'Portable build networking is unavailable offline',portableBuildPermissionUnavailable:'Portable build permission is unavailable'};
export const portablePermissionFailure=code=>Object.assign(new Error(messages[code]||messages.portableBuildPermissionUnavailable),{code:Object.hasOwn(messages,code)?code:'portableBuildPermissionUnavailable'});
export function describePortableBuild(input){
 if(!input||Object.keys(input).sort().join()!=='id,source'||typeof input.id!=='string'||!/^portable-[a-z0-9-]{1,80}$/.test(input.id)||typeof input.source!=='string'||!/^blake3:[a-f0-9]{64}$/.test(input.source))throw portablePermissionFailure('invalidPortableBuildPermission');
 return Object.freeze({id:input.id,source:input.source,endpoints:Object.freeze(['https://esm.sh','https://cdn.tailwindcss.com'])});
}
export function authorizePortableBuild(input,{document=globalThis.document,online=()=>globalThis.navigator.onLine,timeoutMs=120000}={}){
 let disclosure;try{disclosure=describePortableBuild(input);if(!Number.isInteger(timeoutMs)||timeoutMs<1||timeoutMs>120000)throw portablePermissionFailure('invalidPortableBuildPermission');if(!online())throw portablePermissionFailure('portableBuildPermissionOffline');}catch(error){return Promise.reject(error);}
 return new Promise((resolve,reject)=>{
  const controller=new AbortController(),previous=document?.activeElement,expires=Date.now()+timeoutMs;let dialog,settled=false,timer;
  const close=()=>{clearTimeout(timer);dialog?.remove();previous?.focus?.();};
  const stop=code=>{controller.abort();close();if(!settled){settled=true;reject(portablePermissionFailure(code));}};
  try{
   const el=(tag,text)=>{const node=document.createElement(tag);if(text!==undefined)node.textContent=text;return node;};
   dialog=el('dialog');dialog.setAttribute('aria-label','Review portable build');dialog.style.cssText='max-width:min(90vw,44rem);max-height:85vh;overflow:auto;padding:24px;border-radius:12px;color-scheme:light dark';
   dialog.append(el('h2','Build a portable application?'),el('p','This runs generated project code locally and retrieves packages and styling from the destinations below. They can receive package names, requests and data sent by that code, your IP address and browser information. No configured provider API key is sent by this permission.'),el('p','Captured source: '+disclosure.source));
   const list=el('ul');for(const origin of disclosure.endpoints)list.append(el('li',origin));dialog.append(list,el('p','Approval covers this build only, for at most two minutes. The resulting archive stays in this browser. Stop aborts browser requests and removes the styling frame; it cannot recall transmitted data or guarantee that remote work stops.'));
   const deny=el('button',"Don't build"),allow=el('button','Build this source');deny.type=allow.type='button';deny.autofocus=true;deny.addEventListener('click',()=>stop(settled?'portableBuildPermissionRevoked':'portableBuildPermissionDenied'));
   dialog.addEventListener('cancel',event=>{event.preventDefault();stop(settled?'portableBuildPermissionRevoked':'portableBuildPermissionDenied');});
   allow.addEventListener('click',()=>{if(settled)return;if(Date.now()>=expires){stop('portableBuildPermissionExpired');return;}if(!online()){stop('portableBuildPermissionOffline');return;}settled=true;allow.remove();deny.textContent='Stop build';resolve({purpose:"portable-build",disclosure,expires,signal:controller.signal,close});});
   dialog.append(deny,allow);document.body.append(dialog);timer=setTimeout(()=>stop('portableBuildPermissionExpired'),timeoutMs);dialog.showModal();deny.focus();
  }catch{stop('portableBuildPermissionUnavailable');}
 });
}
export async function withPortableBuildPermission(input,operation,{authorize=authorizePortableBuild,assertSource,grant,readGrant,revoke,online=()=>globalThis.navigator.onLine,now=()=>Date.now(),pollMs=100}={}){
 const disclosure=describePortableBuild(input);if(typeof operation!=='function'||![assertSource,grant,readGrant,revoke].every(f=>typeof f==='function')||!Number.isInteger(pollMs)||pollMs<10||pollMs>1000)throw portablePermissionFailure('invalidPortableBuildPermission');
 const approved=await authorize({id:disclosure.id,source:disclosure.source});
 const controller=new AbortController();let timer,watcher,reason,closed=false,checking=false,installed=false,rejectStop;
 const stopped=new Promise((_,reject)=>{rejectStop=reject;});stopped.catch(()=>{});
 const stop=code=>{if(!reason){reason=portablePermissionFailure(code);controller.abort();rejectStop(reason);}};
 const aborted=()=>stop(now()>=approved.expires?'portableBuildPermissionExpired':'portableBuildPermissionRevoked');
 const active=()=>{if(reason)throw reason;if(closed)throw portablePermissionFailure('portableBuildPermissionRevoked');if(now()>=approved.expires){stop('portableBuildPermissionExpired');throw reason;}if(!online()){stop('portableBuildPermissionOffline');throw reason;}};
 const check=async()=>{active();let record;try{record=await readGrant(disclosure.id);}catch{stop('portableBuildPermissionUnavailable');throw reason;}active();if(record?.serve!==true||record.previewConsent?.profile!=='foundry/preview-consent/1'||record.previewConsent.id!==disclosure.id||record.previewConsent.source!==disclosure.source||JSON.stringify(record.network?.endpoints)!==JSON.stringify(disclosure.endpoints)){stop('portableBuildPermissionRevoked');throw reason;}};
 try{
  if(!approved||approved.purpose!=="portable-build"||approved.disclosure?.id!==disclosure.id||approved.disclosure?.source!==disclosure.source||JSON.stringify(approved.disclosure?.endpoints)!==JSON.stringify(disclosure.endpoints)||!Number.isFinite(approved.expires)||approved.expires>now()+120000||!approved.signal||typeof approved.signal.addEventListener!=='function'||typeof approved.close!=='function')throw portablePermissionFailure('invalidPortableBuildPermission');
  approved.signal.addEventListener('abort',aborted,{once:true});if(approved.signal.aborted)aborted();active();await assertSource();active();await grant(disclosure.id,approved);installed=true;await check();
  timer=setTimeout(()=>stop('portableBuildPermissionExpired'),Math.max(1,approved.expires-now()));
  watcher=setInterval(async()=>{if(checking||closed)return;checking=true;try{await check();}catch{}finally{checking=false;}},pollMs);
  const lease=Object.freeze({id:disclosure.id,source:disclosure.source,signal:controller.signal,check});
  const result=await Promise.race([Promise.resolve().then(()=>operation(lease)),stopped]);await check();await assertSource();active();return result;
 }catch(error){if(reason)throw reason;throw error;}
 finally{closed=true;clearTimeout(timer);clearInterval(watcher);controller.abort();approved?.signal?.removeEventListener?.('abort',aborted);approved?.close?.();if(installed)try{await revoke(disclosure.id);}catch{throw portablePermissionFailure('portableBuildPermissionUnavailable');}}
}
