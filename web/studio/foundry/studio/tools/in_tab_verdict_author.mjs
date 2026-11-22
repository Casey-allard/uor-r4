// Authors the in-tab verdict wire protocol and its corpus.
//
//   node studio/tools/in_tab_verdict_author.mjs          writes the files
//   node studio/tools/in_tab_verdict_author.mjs --check  refuses drift in any of them
//
// It writes app/src/InTabKernelProtocol.lex.tex, verification/in-tab-kernel-verdict-vectors.json, and the three
// places app/src/ForgeKernel.lex.tex names the protocol (its import, its entry, its acceptance vectors).
//
// The expected replies come from `expected` below, a reading of InTabKernel.verdictCode used only to write the
// corpus. It is not an authority: every vector is restated as a theorem Lean proves by evaluation, and replayed in
// the compiled guest, so a wrong reading here is refused there.
import fs from "node:fs";

const root = new URL("../../", import.meta.url);
const check = process.argv.includes("--check");
const ascii = (s) => "0x" + [...new TextEncoder().encode(s)].map((b) => b.toString(16).padStart(2, "0")).join("");
const hexWord = (n) => { const b = new Uint8Array(4); new DataView(b.buffer).setUint32(0, n, true); return [...b].map((x) => x.toString(16).padStart(2, "0")).join(""); };

// ---- the wire: version, hasBytes, four address pairs of eight words, two axiom sets, four kernel flags
export const WORDS = 72;
export const OFFSET = { version: 0, hasBytes: 1, libraryPinned: 2, libraryDerived: 10, bundleNamed: 18, bundleDerived: 26, kernelPinned: 34, kernelDerived: 42, specStatement: 50, checkedStatement: 58, axiomsDeclared: 66, axiomsObserved: 67, leanRan: 68, leanAccepted: 69, secondRan: 70, secondAccepted: 71 };
const address = (seed) => Array.from({ length: 8 }, (_, i) => seed * 16 + i + 1);
const base = { version: 1, hasBytes: 1, libraryPinned: address(1), libraryDerived: address(1), bundleNamed: address(2), bundleDerived: address(2), kernelPinned: address(3), kernelDerived: address(3), specStatement: address(4), checkedStatement: address(4), axiomsDeclared: 5, axiomsObserved: 5, leanRan: 1, leanAccepted: 1, secondRan: 0, secondAccepted: 0 };
export const wordsOf = (input) => Object.keys(OFFSET).flatMap((k) => input[k]);

function expected(input) {
  const w = wordsOf(input);
  const flags = [input.hasBytes, input.leanRan, input.leanAccepted, input.secondRan, input.secondAccepted];
  if (w.length !== WORDS || input.version !== 1 || flags.some((f) => f > 1)) return "malformedVerdictRequest";
  const same = (a, b) => input[a].every((x, i) => x === input[b][i]);
  let code;
  if (input.hasBytes !== 1) code = 2;
  else if (!(same("libraryPinned", "libraryDerived") && same("bundleNamed", "bundleDerived") && same("kernelPinned", "kernelDerived"))) code = 3;
  else if (!same("specStatement", "checkedStatement")) code = 3;
  else if (!(input.axiomsObserved === input.axiomsDeclared && input.axiomsObserved < 8)) code = 3;
  else if (input.leanRan === 1) code = input.leanAccepted !== 1 ? 3 : input.secondRan === 1 && input.secondAccepted !== 1 ? 3 : 0;
  else if (input.secondRan === 1) code = input.secondAccepted === 1 ? 1 : 3;
  else code = 2;
  return "ok " + hexWord(1) + hexWord(code) + hexWord(code);
}

const cases = [
  ["proved_byLean", {}],
  ["proved_bothKernelsAgree", { secondRan: 1, secondAccepted: 1 }],
  ["kernelChecked_withoutLean", { leanRan: 0, leanAccepted: 0, secondRan: 1, secondAccepted: 1 }],
  ["checking_noBytes", { hasBytes: 0 }],
  ["checking_noKernelRan", { leanRan: 0, leanAccepted: 0 }],
  ["refused_alteredLibrary", { libraryDerived: address(9) }],
  ["refused_alteredBundle", { bundleDerived: address(9) }],
  ["refused_anotherKernel", { kernelDerived: address(9) }],
  ["refused_anotherStatement", { checkedStatement: address(9) }],
  ["refused_otherAxioms", { axiomsObserved: 7 }],
  ["refused_axiomOutsideTheAllowed", { axiomsDeclared: 8, axiomsObserved: 8 }],
  ["refused_leanRefused", { leanAccepted: 0 }],
  ["refused_kernelsDisagree", { secondRan: 1, secondAccepted: 0 }],
  ["refused_secondRefusedAlone", { leanRan: 0, leanAccepted: 0, secondRan: 1, secondAccepted: 0 }],
  ["malformed_flagNotBoolean", { leanRan: 2 }],
  ["malformed_version", { version: 2 }],
].map(([name, patch]) => { const input = { ...base, ...patch }; return { name, input, request: "verdict " + wordsOf(input).map(hexWord).join(""), response: expected(input) }; });
const raw = [
  { name: "raw_tooLarge", request: "verdict " + wordsOf(base).map(hexWord).join("") + "00", response: "verdictRequestTooLarge" },
  { name: "raw_tooShort", request: "verdict " + wordsOf(base).map(hexWord).join("").slice(8), response: "malformedVerdictRequest" },
  { name: "raw_notHex", request: "verdict " + "g" + wordsOf(base).map(hexWord).join("").slice(1), response: "malformedVerdictRequest" },
];
// The vectors the guest carries and replays when it is built. Each is a 584-byte request, several times the size of
// any other protocol's, so the guest carries three: proved, kernel-checked and refused. The whole corpus is replayed against the built guest.
const EMBEDDED = ["proved_byLean", "kernelChecked_withoutLean", "refused_alteredLibrary"];
const corpus = { schema: "prism/in-tab-kernel-verdict-vectors/1", words: WORDS, vectors: [...cases, ...raw].map((v) => ({ ...v, embedded: EMBEDDED.includes(v.name) })) };
if (corpus.vectors.filter((v) => v.embedded).length !== EMBEDDED.length) throw new Error("an embedded vector is not in the corpus");

// ---- the protocol module
const list = (words) => words.reduceRight((tail, w) => `(cons ${w} ${tail})`, "(nil Nat)");
const w = (i) => `(WorkProtocol::word words ${i})`;
const sameAt = (a, b) => Array.from({ length: 8 }, (_, i) => `(${w(a + i)} =? ${w(b + i)})`).reduceRight((tail, x) => `(${x} && ${tail})`, "true");
const bit = (e) => `(if ${e} then 1 else 0)`;
const pair = (pinned, derived) => bit(sameAt(OFFSET[pinned], OFFSET[derived]));
const flagsOk = ["hasBytes", "leanRan", "leanAccepted", "secondRan", "secondAccepted"].map((k) => `(${w(OFFSET[k])} <=? 1)`).reduceRight((tail, x) => `(${x} && ${tail})`, "true");
const theorems = cases.filter((c) => ["proved_byLean", "kernelChecked_withoutLean", "checking_noBytes", "refused_alteredLibrary", "refused_anotherKernel", "refused_anotherStatement", "refused_axiomOutsideTheAllowed", "refused_kernelsDisagree", "malformed_flagNotBoolean"].includes(c.name));
const protocol = `\\begin{lexlean}{InTabKernelProtocol}
\\useglossary{lexlean.std.bool@1.2.0}
\\useglossary{lexlean.std.nat@1.2.0}
\\importmodule{InTabKernel}
\\importmodule{CompositionProtocol}
\\importmodule{WorkProtocol}
\\title{Boolean}
\\begin{semanticmodule}
definition{
  name: verdictEvidence,
  parameters: [words : (List Nat)],
  result: InTabKernel::Evidence,
  body: (record InTabKernel::Evidence {hasBytes := ${w(OFFSET.hasBytes)}, libraryPinned := 1, libraryDerived := ${pair("libraryPinned", "libraryDerived")}, bundleNamed := 1, bundleDerived := ${pair("bundleNamed", "bundleDerived")}, kernelPinned := 1, kernelDerived := ${pair("kernelPinned", "kernelDerived")}, specStatement := 1, checkedStatement := ${pair("specStatement", "checkedStatement")}, axiomsDeclared := ${w(OFFSET.axiomsDeclared)}, axiomsObserved := ${w(OFFSET.axiomsObserved)}, leanRan := ${w(OFFSET.leanRan)}, leanAccepted := ${w(OFFSET.leanAccepted)}, secondRan := ${w(OFFSET.secondRan)}, secondAccepted := ${w(OFFSET.secondAccepted)}, serverSays := 0})
}

definition{
  name: verdictEncode,
  parameters: [code : Nat],
  result: Bytes,
  body: (prim append ${ascii("ok ")} (prim append (WorkProtocol::hexWord 1) (prim append (WorkProtocol::hexWord code) (WorkProtocol::hexWord (InTabKernel::wordOf code)) : Bytes) : Bytes) : Bytes)
}

definition{
  name: dispatchVerdictWords,
  parameters: [words : (List Nat)],
  result: Bytes,
  body: (if ((((prim length words : Nat) =? ${WORDS}) && (${w(OFFSET.version)} =? 1)) && ${flagsOk}) then (verdictEncode (InTabKernel::verdictCode (verdictEvidence words))) else ${ascii("malformedVerdictRequest")})
}

definition{
  name: dispatchVerdict,
  parameters: [bytes : Bytes],
  result: Bytes,
  body: (if (${WORDS * 8} <? (prim length bytes : Nat)) then ${ascii("verdictRequestTooLarge")} else (let decoded : CompositionProtocol::DecodedWords := (CompositionProtocol::decodeWords bytes 0 ${WORDS} (nil Nat)); (if (project decoded valid) then (dispatchVerdictWords (project decoded value)) else ${ascii("malformedVerdictRequest")}))),
  axioms: [Quot.sound, propext]
}

inductive{
  name: VerdictWireError,
  type_parameters: [],
  parameters: [],
  constructors: [
    {name: malformedVerdictRequest, fields: []},
    {name: verdictRequestTooLarge, fields: []},
    {name: verdictRuntimeFailed, fields: []},
    {name: invalidVerdictResponse, fields: []}
  ]
}

definition{
  name: verdictWireBoundary,
  parameters: [],
  result: String,
  body: "Requests use verdict-space then exactly ${WORDS} lowercase little-endian uint32 hex words: version 1; a has-bytes flag; the pinned and the derived address of the library, of the bundle and of the kernel, then the spec statement's and the checked statement's address, eight words each; the declared and the observed axiom set; and four flags: Lean's kernel ran, it accepted, the second kernel ran, it accepted. The payload is exactly ${WORDS * 8} ASCII bytes. An address is all 256 bits; the guest compares the two of a pair, so the caller never states that they agree. An axiom set is a bit set over propext, Classical.choice and Quot.sound, with 8 set when any other axiom was observed. Replies use ok-space then three words: version 1, the verdict code and the word shown. No field carries what a server said. Typed errors retain their exact spelling. The codec captures inputs before awaiting guest readiness, bounds output to 27 bytes, and refuses a reply of any other shape."
}
${theorems.map((c) => `
theorem{
  name: verdictWire_${c.name},
  parameters: [],
  statement: ((dispatchVerdictWords ${list(wordsOf(c.input))}) = ${ascii(c.response)}),
  proof: reflexivity{},
  axioms: []
}
`).join("")}\\end{semanticmodule}
\\end{lexlean}
`;

// ---- the kernel: import, entry, acceptance vectors
const kernelPath = new URL("app/src/ForgeKernel.lex.tex", root);
let kernel = fs.readFileSync(kernelPath, "utf8");
const once = (text, find, label) => { if (text.split(find).length !== 2) throw new Error("ForgeKernel: expected exactly one " + label); };
const IMPORT = "\\importmodule{InTabKernelProtocol}\n";
if (!kernel.includes(IMPORT)) { once(kernel, "\\importmodule{VersionPreviewProtocol}\n", "preview import"); kernel = kernel.replace("\\importmodule{VersionPreviewProtocol}\n", "\\importmodule{VersionPreviewProtocol}\n" + IMPORT); }
// The existing dispatcher keeps its body, whole and unchanged, under the name dispatchApplication. The guest's entry,
// dispatch, is a small definition in front of it. Wrapping the body itself in one more arm made Lean elaborate the
// whole of it again, several seconds on a stage the SDK bounds at 600.
const ENTRY = `definition{
  name: dispatch,
  parameters: [value : Bytes],
  result: Bytes,
  body: (if (prim equal (sliceOf value 0 8) ${ascii("verdict ")} : Bool) then (InTabKernelProtocol::dispatchVerdict (sliceOf value 8 (prim subtract (Foundation.Bytes::byteLength value) 8 : Nat))) else (if (prim equal (sliceOf value 0 5) ${ascii("rule ")} : Bool) then (RuleJudge::dispatchRule (sliceOf value 5 (prim subtract (Foundation.Bytes::byteLength value) 5 : Nat))) else (dispatchApplication value))),
  axioms: [Quot.sound, propext]
}

`;
const OLD_HEAD = "definition{\n  name: dispatch,\n  parameters: [value : Bytes],\n  result: Bytes,\n  body: (if (prim equal (sliceOf value 0 8) " + ascii("preview ") + " : Bool) then ";
if (!kernel.includes("  name: dispatchApplication,")) {
  once(kernel, OLD_HEAD, "application dispatcher");
  const start = kernel.indexOf(OLD_HEAD), BODY_END = ",\n  axioms: [Quot.sound, propext]\n}\n\n";
  const end = kernel.indexOf(BODY_END, start);
  if (end < 0 || kernel.slice(start, end).includes("\ndefinition{")) throw new Error("ForgeKernel: the end of the dispatcher was not found");
  kernel = kernel.slice(0, start) + OLD_HEAD.replace("  name: dispatch,", "  name: dispatchApplication,") + kernel.slice(start + OLD_HEAD.length, end + BODY_END.length) + ENTRY + kernel.slice(end + BODY_END.length);
}
once(kernel, ENTRY, "verdict entry");
const vector = (v) => `(record Foundation.View.Text.V1.Model::AcceptanceVector {request := ${ascii(v.request)}, response := ${ascii(v.response)}})`;
const acceptance = `definition{
  name: verdictAcceptance,
  parameters: [],
  result: (List Foundation.View.Text.V1.Model::AcceptanceVector),
  body: ${corpus.vectors.filter((v) => v.embedded).reduceRight((tail, v) => `(cons ${vector(v)} ${tail})`, "(ruleAcceptance)")}
}

`;
const marker = /definition\{\n  name: verdictAcceptance,[\s\S]*?\n\}\n\n/;
if (marker.test(kernel)) kernel = kernel.replace(marker, () => acceptance);
else { once(kernel, "definition{\n  name: application,", "application definition"); kernel = kernel.replace("definition{\n  name: application,", () => acceptance + "definition{\n  name: application,"); }
if (!kernel.includes("acceptanceVectors := (verdictAcceptance)")) { once(kernel, "acceptanceVectors := (previewAcceptance)", "acceptance root"); kernel = kernel.replace("acceptanceVectors := (previewAcceptance)", "acceptanceVectors := (verdictAcceptance)"); }

const outputs = [
  [new URL("app/src/InTabKernelProtocol.lex.tex", root), protocol],
  [new URL("verification/in-tab-kernel-verdict-vectors.json", root), JSON.stringify(corpus, null, 1) + "\n"],
  [kernelPath, kernel],
];
let drift = 0;
for (const [url, text] of outputs) {
  const now = fs.existsSync(url) ? fs.readFileSync(url, "utf8") : null;
  if (now === text) continue;
  if (check) { console.error("drift: " + url.pathname.split("/").slice(-3).join("/")); drift++; } else fs.writeFileSync(url, text);
}
if (drift) process.exit(1);
console.log(JSON.stringify({ vectors: corpus.vectors.length, embedded: EMBEDDED.length, theorems: theorems.length, check }));
