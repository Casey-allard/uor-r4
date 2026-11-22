import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { admitPublication, sameArchiveIdentity } from "../shell/publication-identity.mjs";
const digest = "0123456789abcdef".repeat(4);
const accepted = { total: 2, passed: 2, failed: [] };

test("canonical exact identity produces a typed body-fingerprint receipt", () => {
  assert.deepEqual(admitPublication(digest, digest, accepted), {
    ok: true, identity: { kind: "archiveFingerprint", algorithm: "blake3", profile: "holo-v4-body", version: 1, digest }, vectors: { passed: 2, total: 2 },
  });
});
test("suffixes, algorithms, coercions and malformed digests cannot establish identity", () => {
  for (const malformed of [undefined, null, "", digest.slice(1), digest + "a", "blake3:" + digest, "sha256:" + digest, "prefix" + digest, digest.toUpperCase(), digest + "\n", { toString: () => digest }]) {
    assert.equal(sameArchiveIdentity(malformed, digest), false);
    assert.equal(sameArchiveIdentity(digest, malformed), false);
    assert.equal(admitPublication(malformed, digest, accepted).code, "invalidArchiveIdentity");
  }
});
test("different complete digests report mismatch", () => {
  assert.equal(admitPublication("a".repeat(64), digest, accepted).code, "archiveIdentityMismatch");
});
test("empty or inconsistent runtime reports cannot establish acceptance", () => {
  for (const report of [undefined, null, {}, { total: 0, passed: 0, failed: [] }, { total: 2, passed: 1, failed: [] }, { total: 1, passed: 2, failed: [] }, { total: -1, passed: -1, failed: [] }, { total: 1.5, passed: 1.5, failed: [] }, { total: "2", passed: 2, failed: [] }, { total: 1, passed: 1, failed: null }, { total: Infinity, passed: Infinity, failed: [] }]) {
    assert.equal(admitPublication(digest, digest, report).code, "invalidAcceptance");
  }
});
test("a complete report with a failed vector refuses", () => {
  assert.equal(admitPublication(digest, digest, { total: 2, passed: 1, failed: [{ index: 1 }] }).code, "acceptanceFailed");
});

test("on-device publication admits the kernel its page build named, after its vectors replay here", () => {
  const host = fs.readFileSync(new URL("../shell/host.js", import.meta.url), "utf8");
  assert.match(host, /admitPublication\(\(await \(await fetch\(new URL\("app\.json", at\)[^\n]+attestations\?\.build\?\.id, kernel\.identities\?\.archiveFingerprint, await kernel\.acceptance\(\)\);\n  if \(!admitted\.ok\) throw new Error\(admitted\.error\);/);
});
