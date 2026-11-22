// The rules an app cover and its card words must meet (shell/covers.mjs), each with the defect it refuses.
import assert from "node:assert/strict";
import { test } from "node:test";
import { DESCRIPTION, TAGS, checkCard, readCard, isBlank, withResponder, cardMessages } from "../shell/covers.mjs";

const good = "Make images smaller without losing quality by comparing formats side by side as you go.";

test("a sentence in the band, ending with a period, with a listed tag, is a card", () => {
  assert.ok(good.length >= DESCRIPTION.min && good.length <= DESCRIPTION.max);
  assert.deepEqual(checkCard({ description: good, tag: "Creative" }), []);
});
test("a sentence outside the band is refused, so every card fills two rows", () => {
  assert.match(checkCard({ description: "Track tasks.", tag: "Productivity" })[0], /characters/);
  assert.match(checkCard({ description: good + " And it also does a great many other things too.", tag: "Tools" })[0], /characters/);
});
test("technical words and praise are refused", () => {
  const jargon = "Build pages faster with a React editor that previews every change as you type it out.";
  assert.ok(checkCard({ description: jargon, tag: "Tools" }).some((i) => /"React"/.test(i)));
  const hype = "Plan every trip with the ultimate planner that keeps flights, stays and maps together.";
  assert.ok(checkCard({ description: hype, tag: "Travel" }).some((i) => /"ultimate"/.test(i)));
});
test("a tag outside the list is refused", () => {
  assert.ok(checkCard({ description: good, tag: "Image Tools" }).some((i) => /tag/.test(i)));
  assert.ok(TAGS.includes("Tools"));
});
test("the model's answer is read through prose, fences and thinking, and its tag's case is forgiven", () => {
  const answer = `<think>let me see</think>Here it is:\n\`\`\`json\n{"description": "${good}", "tag": "creative"}\n\`\`\``;
  assert.deepEqual(readCard(answer), { card: { description: good, tag: "Creative" }, issues: [] });
  assert.equal(readCard("no json here").card, null);
  assert.equal(readCard('{"description": "Too short.", "tag": "Games"}').card, null);
});
test("a refused answer is asked again with its reasons", () => {
  const m = cardMessages("Squoosh", "Drop OR Paste", ["description is 40 characters, not 84 to 92"]);
  assert.match(m[1].content, /refused: description is 40 characters/);
});
test("a uniform frame is blank; a frame with content is not", () => {
  const flat = new Uint8ClampedArray(96 * 48 * 4).fill(250);
  assert.equal(isBlank(flat), true);
  const busy = new Uint8ClampedArray(96 * 48 * 4);
  for (let i = 0; i < busy.length; i += 4) busy[i] = busy[i + 1] = busy[i + 2] = (i / 4) % 7 === 0 ? 20 : 240;
  assert.equal(isBlank(busy), false);
});
test("the responder goes last in the body, and a library cannot close its own script early", () => {
  const html = withResponder("<html><body><div id=root></div></body></html>", 'var x = "</script><b>";');
  assert.ok(html.indexOf('<div id=root>') < html.indexOf("<script>"));
  assert.ok(html.endsWith("</body></html>"));
  assert.ok(!html.includes('"</script><b>"'));
  assert.match(html, /hologram\/cover-request/);
});
