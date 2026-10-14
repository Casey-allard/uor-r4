'use strict';
// BYTE CODEC ONLY. Exact supplied current golden score/pack/unpack and FBS
// construction functions. No DOM, geometry route search, oscillator, waveform,
// attention, model, training or CUDA execution.
const fs = require('node:fs');
const vm = require('node:vm');
const crypto = require('node:crypto');
const assert = require('node:assert/strict');
const input = process.argv[2] || '/Users/casey.allard/Downloads/view-source_https___spiralcore.netlify.app.html';
const bytes = fs.readFileSync(input);
const sha = b => crypto.createHash('sha256').update(b).digest('hex');
assert.equal(sha(bytes), '770142b4f343842cd0693d362d49f95272a5ded029d9682211a5215d64810fd8');
const source = bytes.toString('utf8');
const script = source.split('<script id="v00PCMMusical">')[1].split('</script>')[0];
const prefix = script.slice(script.indexOf("'use strict';"), script.indexOf('async function render('));
const fbs = source.slice(source.indexOf('function fbsInt('), source.indexOf('function fbsFormal('));
assert(prefix.includes("NATIVE_63_ROUTE_GOLDEN_PCM_V1"));
assert(prefix.includes('function score(bytes)'));
assert(fbs.includes('function fbsClose(children)'));
const sandbox = vm.createContext({});
vm.runInContext(prefix + '\n' + fbs + '\nglobalThis.codec={score,pack,unpack,profile,fbsAtom,fbsClose,fbsV3,fbsWrappers,fbsDigitSum};', sandbox);
const c = sandbox.codec;
let roundTrips = 0, structuralChecks = 0, refusals = 0;
function exercise(data) {
  const scored = c.score(data);
  const p = c.pack(scored.digits);
  const digits = c.unpack(p, scored.digits.length);
  assert.deepEqual(Array.from(digits), Array.from(scored.digits));
  const roots = []; let promotions = 0;
  for (let i=0; i<digits.length; i++) {
    roots.push(c.fbsAtom(BigInt(digits[i]+2), 1n, ''));
    let carries=0;
    while(roots.length>=3 && roots.slice(-3).every(x=>x.level===roots.at(-1).level)) {
      roots.push(c.fbsClose(roots.splice(-3,3))); promotions++; carries++;
    }
    assert.equal(BigInt(carries), c.fbsV3(BigInt(i+1)));
    assert.equal(BigInt(roots.length), c.fbsDigitSum(BigInt(i+1)));
    structuralChecks += 2;
  }
  assert.equal(BigInt(promotions), c.fbsWrappers(BigInt(digits.length)));
  let n=0n, count=0;
  function walk(node) {
    if(node.kind==='atom') {
      assert.equal(node.payload, ''); assert.equal(node.mode.g, 1n);
      n=n*63n+(node.mode.tau-2n); count++;
    } else node.children.forEach(walk);
  }
  roots.forEach(walk); assert.equal(count,digits.length);
  const recovered=Buffer.alloc(data.length);
  for(let i=data.length-1;i>=0;i--) {recovered[i]=Number(n%256n); n/=256n;}
  assert.equal(n,0n); assert.deepEqual(recovered,Buffer.from(data));
  roundTrips++;
  return {sourceBytes:data.length,routeChoices:digits.length,packedBytes:p.length,promotions,frontierRoots:roots.length};
}
for(let value=0;value<256;value++) exercise(Uint8Array.of(value));
for(let length=1;length<=32;length++) {
  exercise(new Uint8Array(length));
  exercise(new Uint8Array(length).fill(255));
  exercise(crypto.createHash('sha256').update('golden-byte-codec-'+length).digest().subarray(0,length));
}
// Independent exact radix-bound check, no logarithmic reference calculation.
let cardinalityChecks=0;
for(let length=1;length<=256;length++) {
  const count=c.score(new Uint8Array(length)).digits.length;
  const states=256n**BigInt(length);
  assert(63n**BigInt(count)>=states);
  assert(63n**BigInt(count-1)<states);
  cardinalityChecks++;
}
const alphabet=Uint8Array.from({length:63},(_,i)=>i);
assert.deepEqual(Array.from(c.unpack(c.pack(alphabet),alphabet.length)),Array.from(alphabet));
function refuses(fn) { assert.throws(fn); refusals++; }
refuses(()=>c.score(new Uint8Array(0)));
refuses(()=>c.score(new Uint8Array(257)));
refuses(()=>c.pack(Uint8Array.of(63)));
refuses(()=>c.unpack(Uint8Array.of(252),1));
refuses(()=>c.unpack(Uint8Array.of(1),1));
refuses(()=>c.unpack(new Uint8Array(0),1));
refuses(()=>c.unpack(new Uint8Array(2),1));
// Actual upload packs a whole page's concatenated cell choices together.
const pageDigits=[];
for(let i=0;i<128;i++) pageDigits.push(...c.score(new Uint8Array(32).fill(i)).digits);
const packedPage=c.pack(Uint8Array.from(pageDigits));
assert.equal(pageDigits.length,5504); assert.equal(packedPage.length,4128);
assert.deepEqual(Array.from(c.unpack(packedPage,pageDigits.length)),pageDigits);
const full32ByteCell=exercise(new Uint8Array(32).fill(255));
console.log(JSON.stringify({scope:'CURRENT_GOLDEN_SCORE_PACK_UNPACK_AND_ORDERED_FBS_BYTE_PRIMITIVES_ONLY',node:process.version,sourceSHA256:sha(bytes),extractedGoldenPrefixSHA256:sha(prefix),extractedFbsFunctionsSHA256:sha(fbs),roundTrips,structuralChecks,cardinalityChecks,routeAlphabetCases:63,malformedRefusals:refusals,full32ByteCell,full4096BytePage:{sourceBytes:4096,routeChoices:pageDigits.length,packedBytes:packedPage.length},waveformExecuted:false,oscillatorExecuted:false,attentionExecuted:false,result:'PASS'},null,2));
