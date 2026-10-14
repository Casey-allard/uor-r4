'use strict';
// Directly exercises the supplied JavaScript byte codec. No model execution.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const {createHash} = require('node:crypto');
const {deflateSync} = require('node:zlib');
const C = require('./showcase/core.js');
const T = require('./showcase/transport.js');
const result = {scope: 'finite byte codec and tree accounting only; no model compute', node: process.version, tests: [], runs: []};
const pass = name => {result.tests.push(name); console.log('PASS', name);};
const manifest = (name, data, fold) => ({id:name, name, type:'application/octet-stream', size:data.length, created:1, modified:1, n:fold.n, carries:fold.carries, roots:fold.roots, sha256:fold.sha256});
const summarize = r => ({name:r.name,inputBytes:r.inputBytes,rootBytes:r.rootPacketBytes,metadataBytes:r.metadataBytes,requestBytes:r.requestBytes,seedBytes:r.seedBytes,receiptBytes:r.receiptBytes,totalBytes:r.totalBytes,newNodes:r.newNodes,retainedNodes:r.receiver.nodes,retainedRecordBytes:r.receiver.recordBytes,rounds:r.rounds,verified:r.verified});
async function main() {
  assert(63n**42n < 2n**256n && 2n**256n < 63n**43n);
  result.radix = {pow63_42:String(63n**42n), pow2_256:String(2n**256n), pow63_43:String(63n**43n), digits:43, packedBits:258, frameBytes:33};
  let roots = 0, carries = 0, maxRoots = 0;
  for (let n=1;n<=1000000;n++) {
    let v=0,m=n,s=0;
    while(m%3===0) {v++;m/=3;}
    for(m=n;m;m=Math.floor(m/3)) s+=m%3;
    roots+=1-2*v;carries+=v;maxRoots=Math.max(maxRoots,roots);
    assert.equal(roots,s);assert.equal(carries,(n-s)/2);
  }
  result.frontier={throughN:1000000,roots,carries,maxRoots};
  pass('Exact carry and base-3 frontier identities for N=1..1000000; radix capacity inequality');
  for(let length=1;length<=32;length++) for(let value=0;value<256;value++) {
    const bytes=new Uint8Array(length).fill(value);
    assert.deepEqual(T.decodeFrame(T.encodeFrame(bytes)), bytes);
  }
  assert.throws(()=>T.decodeFrame(new Uint8Array(32)),/Truncated/);
  assert.throws(()=>T.decodeFrame(new Uint8Array(33)),/length/);
  const invalid=T.encodeFrame(new Uint8Array(32));invalid[0]|=63;
  assert.throws(()=>T.decodeFrame(invalid),/digit/);
  pass('8192 direct Tier-3 round trips; malformed frame length, zero length tag and reserved digit rejected');
  const repeated=new Uint8Array(6*32), folded=await C.fold(repeated), f=manifest('six-identical-atoms',repeated,folded);
  assert.equal(folded.roots.length,2);assert.equal(new Set(folded.roots).size,1);assert.equal(folded.nodes.length,2);
  assert.deepEqual(await C.rebuild(f,new Map(folded.nodes.map(n=>[n.id,n]))),repeated);
  result.occurrenceVsDedup={atoms:6,frontierReferences:2,uniqueFrontierIDs:1,uniqueStoredNodes:2,uniqueGraphConnectedComponents:1};
  pass('Frontier occurrence count differs from connected components of deduplicated DAG: six identical atoms');
  const novel=new Uint8Array(131072);
  for(let i=0;i<novel.length/32;i++) novel.set(createHash('sha256').update('UOR_BYTE_CODEC_NOVEL_BLOCK_'+i).digest(),i*32);
  const r=await C.fold(novel), original=manifest('novel-128KiB.bin',novel,r), source=new Map(r.nodes.map(n=>[n.id,n])), b=new T.Receiver();
  const cold=await T.run(original,source,b);result.runs.push(summarize(cold));
  result.rawDeflateComparison={inputBytes:novel.length,deflateBytes:deflateSync(novel).length,tier3FramedDeflateBytes:5+Math.ceil(deflateSync(novel).length/32)*33};
  result.runs.push(summarize(await T.run(original,source,b.clone())));
  const edit=novel.slice();edit[12345]^=1;const editFold=await C.fold(edit);
  result.runs.push(summarize(await T.run(manifest('one-byte-overwrite.bin',edit,editFold),new Map(editFold.nodes.map(n=>[n.id,n])),b.clone())));
  const insert=new Uint8Array(novel.length+1);insert[0]=91;insert.set(novel,1);const insertFold=await C.fold(insert);
  result.runs.push(summarize(await T.run(manifest('one-byte-prefix.bin',insert,insertFold),new Map(insertFold.nodes.map(n=>[n.id,n])),b.clone())));
  assert(result.runs.every(x=>x.verified));assert.equal(result.runs[1].seedBytes,0);
  assert(result.runs[0].totalBytes>novel.length);
  assert(result.runs[3].newNodes>result.runs[2].newNodes);
  pass('Novel-byte cold/warm, overwrite, and prefix-insertion endpoint reconstructions and full byte accounting');
  fs.writeFileSync(__dirname+'/independent-results.json',JSON.stringify(result,null,2)+'\n');
  console.log(JSON.stringify(result,null,2));
}
main().catch(error=>{console.error(error);process.exitCode=1;});
