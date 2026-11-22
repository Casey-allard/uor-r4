import assert from "node:assert/strict";
import {test} from "node:test";
import fs from "node:fs";
import vm from "node:vm";
import {invokeHost} from "../shell/host-boundary.mjs";
import {describePreview,previewPermissionRecord,authorizePreview,mountPreviewPermissions} from "../shell/preview-permissions.mjs";
const request={id:"preview-id",source:"blake3:"+"a".repeat(64),endpoints:["https://esm.sh","https://cdn.tailwindcss.com"]};
const code=c=>e=>e.code===c;
function dom(){const nodes=[];class Element extends EventTarget{constructor(tag){super();this.tag=tag;this.style={};this.children=[];nodes.push(this);}append(...c){this.children.push(...c);}setAttribute(){}remove(){this.removed=true;}showModal(){}show(){}focus(){doc.activeElement=this;}click(){this.dispatchEvent(new Event("click"));}}const doc={createElement:t=>new Element(t),body:new Element("body"),activeElement:{focus(){}}};return{document:doc,nodes,button:label=>nodes.find(n=>!n.removed&&n.tag==="button"&&n.textContent===label)};}
test("preview disclosure bounds identities and exact HTTPS origins",()=>{
 const input=structuredClone(request),d=describePreview(input);input.endpoints.length=0;assert.equal(d.endpoints.length,2);assert.ok(Object.isFrozen(d.endpoints));
 for(const patch of [{id:"../escape"},{source:"unbound"},{endpoints:["http://unsafe.example"]},{endpoints:["https://esm.sh/path"]},{endpoints:["https://esm.sh","https://esm.sh"]},{endpoints:Array(33).fill("https://esm.sh")}])assert.throws(()=>describePreview({...request,...patch}),code("invalidPreviewPermission"));
});
test("permission records bind immutable disclosure and reject expired approval",()=>{
 const d=describePreview(request),record=previewPermissionRecord({disclosure:d,expires:1000},500);assert.equal(record.previewConsent.source,request.source);assert.deepEqual(record.network.endpoints,request.endpoints);assert.equal(record.serve,true);
 assert.throws(()=>previewPermissionRecord({disclosure:d,expires:1000},1000),code("previewPermissionExpired"));
});
test("denial Escape offline and deadline grant nothing",async()=>{
 for(const action of ["deny","escape","timeout"]){const ui=dom();const p=authorizePreview(request,{document:ui.document,online:()=>true,timeoutMs:10});if(action==="deny")ui.button("Don't allow").click();if(action==="escape")ui.nodes.find(n=>n.tag==="dialog").dispatchEvent(new Event("cancel"));await assert.rejects(p,code(action==="timeout"?"previewPermissionExpired":"previewPermissionDenied"));}
 await assert.rejects(authorizePreview(request,{document:{},online:()=>false}),code("previewPermissionOffline"));
 await assert.rejects(authorizePreview(request,{document:{},online:()=>true}),code("previewPermissionUnavailable"));
});
test("approval snapshots endpoints focuses refusal and rechecks online state",async()=>{
 const ui=dom(),input=structuredClone(request);const p=authorizePreview(input,{document:ui.document,online:()=>true});assert.equal(ui.document.activeElement,ui.button("Don't allow"));input.endpoints.push("https://other.example");ui.button("Allow this preview").click();const approved=await p;assert.deepEqual(approved.disclosure.endpoints,request.endpoints);assert.ok(Object.isFrozen(approved));
 const next=dom();let online=true;const pending=authorizePreview(request,{document:next.document,online:()=>online});online=false;next.button("Allow this preview").click();await assert.rejects(pending,code("previewPermissionOffline"));
});
test("persistent permission control lists grants and awaits revocation",async()=>{
 const ui=dom();let revoked=null;mountPreviewPermissions({document:ui.document,mount:button=>{ui.document.body.append(button);return button;},list:async()=>[{id:request.id,source:request.source,endpoints:request.endpoints}],revoke:async id=>{revoked=id;}});ui.button("Preview permissions").click();await new Promise(r=>setTimeout(r,0));ui.button("Revoke preview").click();await new Promise(r=>setTimeout(r,0));assert.equal(revoked,request.id);
});
test("actual host binds approval before compilation and grant after source comparison",async()=>{
 const host=fs.readFileSync(new URL("../shell/host.js",import.meta.url),"utf8"),start=host.indexOf("async function buildPreview("),body=host.slice(start,host.indexOf("// ---- the live preview",start));
 assert.ok(body.indexOf("await authorizePreview(")>0);assert.ok(body.indexOf("await authorizePreview(")<body.indexOf("await esbuild()"));assert.ok(body.indexOf("await grantPreview(")>body.indexOf("await assertBuildSourceCurrent("));assert.match(body,/permission\.disclosure\.endpoints/);
 for(const code of ["invalidPreviewPermission","previewPermissionDenied","previewPermissionExpired","previewPermissionOffline","previewPermissionUnavailable","previewPermissionStorageFailed","previewPermissionChanged"]){const reply=await invokeHost("preview",[],()=>{throw Object.assign(Error("private content"),{code});},null);assert.equal(reply.error.code,code);assert.ok(!JSON.stringify(reply).includes("private content"));}
 assert.match(host,/mountPreviewPermissions\(/);assert.match(host,/previewPermissionStorageFailed/);
 const worker=fs.readFileSync(new URL("../shell/sw.template.js",import.meta.url),"utf8"),begin=worker.indexOf("function previewGrantValid("),end=worker.indexOf("function policyOf(",begin);
 assert.ok(begin>=0&&end>begin,"missing worker consent boundary");
 const valid=vm.runInNewContext(worker.slice(begin,end)+";previewGrantValid");
 const record=previewPermissionRecord({disclosure:describePreview(request),expires:Date.now()+1000});
 assert.equal(valid(record,request.id),true);assert.equal(valid({...record,previewConsent:undefined},request.id),false);assert.equal(valid(record,"wrong-id"),false);assert.equal(valid({serve:true,network:{endpoints:[]}},request.id),true);
 const policyStart=worker.indexOf("async function policyFor("),policyEnd=worker.indexOf("function untrusted(",policyStart);
 const policy=(record)=>vm.runInNewContext(worker.slice(policyStart,policyEnd)+";policyFor",{ENDPOINT_FORM:vm.runInNewContext(worker.match(/const ENDPOINT_FORM = (.+);/)[1]),grants:async()=>record,gate:async(_origin,_effect,exists,granted,denied)=>exists&&granted&&!denied,audit:async()=>{},policyOf:endpoints=>JSON.stringify(endpoints),previewGrantValid:valid});
 assert.equal(await policy({...record,previewConsent:undefined})("p",request.id),null);
 assert.equal(await policy(record)("p",request.id),JSON.stringify(request.endpoints));
 assert.match(worker,/previewGrantValid\(record, who.id\)/);assert.match(worker,/previewGrantValid\(record, id\)/);
 const model=fs.readFileSync(new URL("../../app/src/FoundrySpecification.lex.tex",import.meta.url),"utf8");assert.match(model,/name: previewPermissionBoundary/);
});
