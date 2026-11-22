import assert from 'node:assert/strict';
import fs from 'node:fs';
import {test} from 'node:test';
import {createShellActionDock,mountShellAction} from '../shell/shell-action-dock.mjs';
function fixture(){
 let observed=0,callback;const frames=[],listeners=new Map(),sidebars=[];
 class Node{constructor(label=''){this.id=label;this.textContent=label;this.style={};this.children=[];this.parentNode=null;this.rect={width:240,height:600,left:0,top:0,right:240,bottom:600};}setAttribute(k,v){this[k]=v;}append(node){node.remove();this.children.push(node);node.parentNode=this;}remove(){if(this.parentNode)this.parentNode.children=this.parentNode.children.filter(x=>x!==this);this.parentNode=null;}getBoundingClientRect(){return this.rect;}}
 const window={innerWidth:1280,innerHeight:720,MutationObserver:class{constructor(fn){callback=fn;observed++;}observe(){}},requestAnimationFrame(fn){frames.push(fn);},addEventListener(name,fn){listeners.set(name,fn);}};
 const document={body:new Node(),defaultView:window,createElement:()=>new Node(),querySelectorAll:()=>sidebars};
 return {document,window,sidebars,make:label=>new Node(label),get observed(){return observed;},get pending(){return frames.length;},mutate:()=>callback(),resize:()=>listeners.get('resize')(),animate:()=>listeners.get('animationend')(),transition:()=>listeners.get('transitionend')(),flush:()=>{const work=frames.splice(0);for(const fn of work)fn();}};
}
test('one page dock preserves node identity and handlers while sidebar is replaced',()=>{
 const f=fixture(),register=createShellActionDock(f.document,f.window),a=f.make('first'),b=f.make('second'),sidebar=f.make('sidebar');a.listener=()=>42;f.sidebars.push(sidebar);register(a);register(b);f.flush();const dock=sidebar.children[0];assert.equal(dock.id,'foundry-shell-actions');assert.deepEqual(dock.children,[a,b]);assert.equal(a.listener(),42);
 const replacement=f.make('replacement');f.sidebars.splice(0,1,replacement);f.mutate();f.flush();assert.equal(dock.parentNode,replacement);assert.deepEqual(dock.children,[a,b]);assert.equal(sidebar.children.length,0);
});
test('offscreen and closed sidebars retain detached controls until visible',()=>{
 const f=fixture(),register=createShellActionDock(f.document,f.window),entry=f.make('tool');register(entry);f.flush();assert.equal(entry.parentNode,null);
 const hidden=f.make('hidden');hidden.rect={width:240,height:600,left:-240,right:0,top:0,bottom:600};f.sidebars.push(hidden);f.mutate();f.flush();assert.equal(hidden.children.length,0);
 const visible=f.make('drawer');f.sidebars.push(visible);f.resize();f.flush();const dock=entry.parentNode;assert.equal(dock.parentNode,visible);
 f.sidebars.pop();f.mutate();f.flush();assert.equal(dock.parentNode,null);f.sidebars.push(visible);f.animate();f.flush();assert.equal(entry.parentNode,dock);assert.equal(dock.parentNode,visible);
});
test('bursts coalesce and repeated registration cannot duplicate observers or entries',()=>{
 const f=fixture(),sidebar=f.make('sidebar');f.sidebars.push(sidebar);const entry=f.make('tool');assert.equal(mountShellAction(entry,f.document),entry);assert.equal(mountShellAction(entry,f.document),entry);const duplicate=f.make('tool');assert.equal(mountShellAction(duplicate,f.document),entry);
 for(let i=0;i<100;i++){f.mutate();f.resize();f.animate();f.transition();}assert.equal(f.pending,1);assert.equal(f.observed,1);f.flush();assert.deepEqual(sidebar.children[0].children,[entry]);assert.equal(duplicate.parentNode,null);f.mutate();f.flush();assert.equal(sidebar.children.length,1);
});
test('every actual tool entry uses the canonical dock and retains browser qualification',()=>{
 const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
 for(const file of ['interaction-ui.mjs','discovery-ui.mjs','preview-permissions.mjs']){const source=read('../shell/'+file);assert.ok(source.includes('mountShellAction'));assert.ok(!source.includes('position:fixed'));}
 assert.ok(read('../shell/appearance.css').includes('dialog:modal { pointer-events: auto; }'));
 assert.ok(read('../../app/src/FoundrySpecification.lex.tex').includes('name: sidebarToolAccessBoundary,'));assert.ok(read('../../.github/workflows/studio.yml').includes('node tools/sidebar_tool_access_browser.mjs'));
 const browser=read('./sidebar_tool_access_browser.mjs');for(const token of ['elementFromPoint','auxiliary-actions-menu','setViewportSize','Foundry tools','Stored preview permissions','Discover an application'])assert.ok(browser.includes(token));
});
