// FoundrySpecification.sidebarToolAccessBoundary: the tools live on the Settings page, as its last card; the
// page owns layout, never effects.
const docks=new WeakMap();
export function createShellActionDock(document,window){
 const entries=new Map(),dock=document.createElement('div');
 dock.id='foundry-shell-actions';dock.setAttribute('role','group');dock.setAttribute('aria-label','Foundry tools');
 dock.style.cssText='display:flex;flex-direction:column;gap:6px;padding:8px;min-width:0;flex-shrink:0';
 let pending=false;
 const visible=node=>{const r=node.getBoundingClientRect();return r.width>0&&r.height>0&&r.right>0&&r.bottom>0&&r.left<window.innerWidth&&r.top<window.innerHeight;};
 const reconcile=()=>{
  pending=false;
  const target=[...document.querySelectorAll('div:has(> #general-settings)')].filter(visible).at(-1);
  if(!target){if(dock.parentNode)dock.remove();return;}
  if(dock.parentNode!==target)target.append(dock);
  for(const entry of entries.values())if(entry.parentNode!==dock)dock.append(entry);
 };
 const schedule=()=>{if(!pending){pending=true;window.requestAnimationFrame(reconcile);}};
 const observer=new window.MutationObserver(schedule);
 observer.observe(document.body,{childList:true,subtree:true,attributes:true,attributeFilter:['class','style','hidden','data-state']});
 window.addEventListener('resize',schedule);
 window.addEventListener('animationend',schedule,true);
 window.addEventListener('transitionend',schedule,true);
 return entry=>{
  const existing=entries.get(entry.id);
  if(existing){if(existing!==entry)entry.remove();return existing;}
  entry.remove();entry.title=entry.textContent;
  Object.assign(entry.style,{width:'100%',minWidth:'0',margin:'0',textAlign:'left',whiteSpace:'nowrap',overflow:'hidden',textOverflow:'ellipsis'});
  entries.set(entry.id,entry);schedule();return entry;
 };
}
export function mountShellAction(entry,document=globalThis.document){
 let register=docks.get(document);
 if(!register){register=createShellActionDock(document,document.defaultView);docks.set(document,register);}
 return register(entry);
}
