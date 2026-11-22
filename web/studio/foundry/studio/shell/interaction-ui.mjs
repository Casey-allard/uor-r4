import {mountShellAction} from "./shell-action-dock.mjs";
// Native keyboard controls and closed visual/audio adapters for accepted guest outputs.
import {interactionFailure} from './interaction-application.mjs';
const colors=['#f5ead1','#172c4c','#dcebd5','#f7d9e4','#e2def5','#d8edf0','#f4e2c7','#e5e7eb'];
export function createInteractionAudio(factory=()=>new AudioContext()){
 let context=null,oscillator=null,granted=false,epoch=0;
 const stop=()=>{if(oscillator){try{oscillator.stop();}catch{}oscillator.disconnect();oscillator=null;}};
 return {
  get granted(){return granted;},
  async enable(){const ticket=++epoch;try{context??=factory();const active=context;await active.resume();if(ticket!==epoch)return;if(active.state!=='running')throw Error();granted=true;}catch{if(ticket===epoch){granted=false;throw interactionFailure('interactionAudioUnavailable');}}},
  render(tone){stop();if(!granted||!tone)return;if(!Number.isInteger(tone)||tone<20||tone>20000)throw interactionFailure('invalidInteractionResponse');oscillator=context.createOscillator();const gain=context.createGain();gain.gain.value=0.025;oscillator.frequency.value=tone;oscillator.connect(gain);gain.connect(context.destination);oscillator.onended=()=>gain.disconnect();oscillator.start();},
  revoke(){epoch++;granted=false;stop();const old=context;context=null;void old?.close().catch(()=>{});}
 };
}
function element(tag,text,parent){const node=document.createElement(tag);if(text!==undefined)node.textContent=text;parent?.append(node);return node;}
function button(label,parent,action){const node=element('button',label,parent);node.type='button';node.style.cssText='padding:8px 12px;margin:4px;border:1px solid #687584;border-radius:6px;background:#fff;color:#17202a';node.addEventListener('click',action);return node;}
function dialog(title){const previous=document.activeElement,node=element('dialog');node.style.cssText='max-width:min(680px,90vw);max-height:85vh;overflow:auto;padding:24px;border:1px solid #607080;border-radius:12px;background:#fff;color:#17202a';const heading=element('h2',title,node);heading.id='interaction-heading-'+crypto.randomUUID();node.setAttribute('aria-labelledby',heading.id);document.body.append(node);node.addEventListener('close',()=>{node.remove();previous?.focus();},{once:true});node.showModal();return node;}
function illustration(kind,parent){
 const glyph={landscape:'⛰',flower:'✿',geometry:'◇',night:'☾'}[kind];const image=element('div',glyph,parent);image.setAttribute('aria-hidden','true');image.style.cssText='font-size:112px;text-align:center;line-height:1.5;transform-origin:center';return image;
}
export async function openInteractionApplication(appId,{run,probe}){
 const node=dialog('Interactive application'),status=element('p','Opening…',node);status.setAttribute('role','status');const display=element('section',undefined,node),controls=element('div',undefined,node),audio=createInteractionAudio(),motion=matchMedia('(prefers-reduced-motion: reduce)');let busy=false,closed=false,animation=null,current=null;
 const sound=button('Enable sound',node,async()=>{if(busy||closed)return;try{if(audio.granted)audio.revoke();else await audio.enable();sound.textContent=audio.granted?'Mute sound':'Enable sound';await transition(0);}catch(error){status.textContent=error.message;}});
 element('p','Sound is optional and stays off until you enable it. Closing or hiding this window stops sound. All application state stays in this browser.',node);
 const reset=button('Reset saved state',node,()=>transition(0,true));button('Close',node,()=>node.close());
 const hide=()=>{if(document.hidden){audio.revoke();sound.textContent='Enable sound';animation?.cancel();}};
 const preference=()=>{animation?.cancel();void transition(0);};document.addEventListener('visibilitychange',hide);motion.addEventListener('change',preference);
 node.addEventListener('close',()=>{closed=true;audio.revoke();animation?.cancel();document.removeEventListener('visibilitychange',hide);motion.removeEventListener('change',preference);},{once:true});
 async function transition(event,resetState=false){
  if(busy||closed)return;busy=true;sound.disabled=true;reset.disabled=true;for(const b of controls.querySelectorAll('button'))b.disabled=true;
  try{
   const value=await run({appId,event,reset:resetState,audioGranted:audio.granted,reducedMotion:motion.matches});if(closed)return;current=value;
   const screen=value.document.screens[value.output.screen];if(!screen)throw interactionFailure('invalidInteractionResponse');
   node.querySelector('h2').textContent=value.document.title;display.replaceChildren();display.style.cssText=`padding:20px;border-radius:10px;background:${colors[value.output.palette]};color:${value.output.palette===1?'#fff':'#17202a'}`;
   element('h3',screen.title,display);const image=illustration(screen.illustration,display);element('p',screen.description,display);animation?.cancel();
   if(value.output.motion&&!motion.matches&&!document.hidden)animation=image.animate([{transform:'translateY(0)'},{transform:`translateY(-${Math.min(value.output.motion,24)}px)`},{transform:'translateY(0)'}],{duration:3000,iterations:Infinity});
   audio.render(value.output.tone);controls.replaceChildren();
   for(const control of value.document.controls){const permitted=await probe({profile:value.document.graph,state:value.output.state,event:control.event,audioGranted:false,reducedMotion:true});if(closed)return;const b=button(control.label,controls,()=>transition(control.event));b.disabled=!permitted.ok;}
   status.textContent='Saved in this browser. Model '+value.state.model.slice(7,19)+'.';
  }catch(error){audio.revoke();animation?.cancel();sound.textContent='Enable sound';status.textContent=error?.message||'Application could not be opened';}
  finally{busy=false;if(!closed){sound.disabled=false;reset.disabled=false;}}
 }
 await transition(0);return {node,get current(){return current;}};
}
export function mountInteractionApplications({create,list,run,probe}){
 const install=()=>{
  const entry=button('Interactive apps',document.body,async()=>{
   const node=dialog('Interactive applications'),status=element('p','Gallery explores visual screens. Garden coordinates visual, motion and optional audio outputs.',node);status.setAttribute('role','status');
   const label=element('label','Application name ',node),name=element('input',undefined,label);name.value='My interactive application';name.maxLength=120;
   const catalog=element('div',undefined,node);
   const refresh=async()=>{catalog.replaceChildren();try{for(const app of await list())button(app.name,catalog,()=>{node.close();void openInteractionApplication(app.id,{run,probe});});}catch{status.textContent='Applications could not be listed';}};
   for(const kind of ['gallery','garden'])button('Create '+kind,node,async event=>{const target=event.currentTarget;target.disabled=true;try{const created=await create({name:name.value,kind});node.close();await openInteractionApplication(created.app.id,{run,probe});}catch(error){status.textContent=error.message;}finally{target.disabled=false;}});
   button('Close',node,()=>node.close());await refresh();
  });entry.id='foundry-interaction-applications';mountShellAction(entry);
 };if(document.readyState==='loading')document.addEventListener('DOMContentLoaded',install,{once:true});else install();
}
