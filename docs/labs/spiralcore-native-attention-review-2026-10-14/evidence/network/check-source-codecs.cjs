const fs=require('node:fs'),vm=require('node:vm'),assert=require('node:assert/strict'),crypto=require('node:crypto');
const path=process.argv[2]||'/Users/casey.allard/Downloads/view-source_https___spiralcore.netlify.app.html';
const html=fs.readFileSync(path,'utf8');
const sha=x=>crypto.createHash('sha256').update(x).digest('hex');
assert.equal(sha(html),'770142b4f343842cd0693d362d49f95272a5ded029d9682211a5215d64810fd8','Original HTML source pin');
const context=vm.createContext({Uint8Array});
const addressStart=html.indexOf('function ipv4ShapeToIPv6Documentation('),addressEnd=html.indexOf('function nodeAddrV6(');
const scoreStart=html.indexOf("const V00_SCORE_ALPHABET='"),scoreEnd=html.indexOf('function v00ScoreB64(');
assert(addressStart>=0&&addressEnd>addressStart&&scoreStart>=0&&scoreEnd>scoreStart);
const address=html.slice(addressStart,addressEnd),score=html.slice(scoreStart,scoreEnd);
vm.runInContext(address+'\n'+score,context);
let addressCases=0,prefixCases=0,scoreCases=0,malformedRefusals=0;
for(let a=0;a<256;a++)for(let b=0;b<256;b++){
 const s=`${a}.${b}.0.0/16`;
 assert.equal(context.ipv6DocumentationToIPv4Shape(context.ipv4ShapeToIPv6Documentation(s)),s);prefixCases++;
}
for(let i=0;i<10000;i++){
 const b=crypto.createHash('sha256').update('address-case:'+i).digest();
 const base=Array.from(b.subarray(0,4)).join('.');
 const s=(i%7<6?`[demarc=${i%7}] `:'')+base;
 assert.equal(context.ipv6DocumentationToIPv4Shape(context.ipv4ShapeToIPv6Documentation(s)),s);addressCases++;
}
const examples=[[[0x2001,0xdb8,0,0,0,0,0,1],'2001:db8::1'],[[0x2001,0xdb8,0,1,0,1,0,1],'2001:db8:0:1:0:1:0:1'],[[0x2001,0,0,1,0,0,1,1],'2001::1:0:0:1:1'],[[0,0,0,0,0,0,0,0],'::']];
for(const [a,s]of examples)assert.equal(context.formatIPv6RFC5952(a),s);
for(const s of ['240.14.1.2/16','240.14.0.0/24','[demarc=6] 1.2.3.4','01.2.3.4','256.2.3.4','1.2.3']){assert.equal(context.ipv4ShapeToIPv6Documentation(s),null);malformedRefusals++;}
for(let n=0;n<=96;n++)for(let mode=0;mode<3;mode++){
 const b=new Uint8Array(n);if(mode===1)b.fill(255);if(mode===2)for(let i=0;i<n;i++)b[i]=crypto.createHash('sha256').update(`score:${n}:${i}`).digest()[0];
 const enc=context.v00ScoreEncode(b),dec=context.v00ScoreDecode(enc,n);assert.deepEqual([...dec],[...b]);scoreCases++;
}
for(const [s,n]of [['',1],['!',1],['__',1],['000',1]]){assert.throws(()=>context.v00ScoreDecode(s,n));malformedRefusals++;}
console.log(JSON.stringify({scope:'SOURCE_EXTRACTED_ADDRESS_AND_BASE63_CODEC_ONLY_NOT_MODEL',node:process.version,htmlSha256:sha(html),addressCases,prefixCases,rfc5952Examples:examples.length,scoreCases,malformedRefusals,result:'PASS',extraction:{address:{startInclusive:'function ipv4ShapeToIPv6Documentation(',endExclusive:'function nodeAddrV6(',sha256:sha(address)},score:{startInclusive:"const V00_SCORE_ALPHABET='",endExclusive:'function v00ScoreB64(',sha256:sha(score)}}},null,2));
