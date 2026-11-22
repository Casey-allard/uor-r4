import { cleanFullResponse } from "@/ipc/utils/cleanFullResponse";
import { describe, it, expect } from "vitest";

describe("cleanFullResponse", () => {
  it("should replace < characters in prism-write attributes", () => {
    const input = `<prism-write path="src/file.tsx" description="Testing <a> tags.">content</prism-write>`;
    const expected = `<prism-write path="src/file.tsx" description="Testing ＜a＞ tags.">content</prism-write>`;

    const result = cleanFullResponse(input);
    expect(result).toBe(expected);
  });

  it("should replace < characters in multiple attributes", () => {
    const input = `<prism-write path="src/<component>.tsx" description="Testing <div> tags.">content</prism-write>`;
    const expected = `<prism-write path="src/＜component＞.tsx" description="Testing ＜div＞ tags.">content</prism-write>`;

    const result = cleanFullResponse(input);
    expect(result).toBe(expected);
  });

  it("should handle multiple nested HTML tags in a single attribute", () => {
    const input = `<prism-write path="src/file.tsx" description="Testing <div> and <span> and <a> tags.">content</prism-write>`;
    const expected = `<prism-write path="src/file.tsx" description="Testing ＜div＞ and ＜span＞ and ＜a＞ tags.">content</prism-write>`;

    const result = cleanFullResponse(input);
    expect(result).toBe(expected);
  });

  it("should handle complex example with mixed content", () => {
    const input = `
      BEFORE TAG
  <prism-write path="src/pages/locations/neighborhoods/louisville/Highlands.tsx" description="Updating Highlands neighborhood page to use <a> tags.">
import React from 'react';
</prism-write>
AFTER TAG
    `;

    const expected = `
      BEFORE TAG
  <prism-write path="src/pages/locations/neighborhoods/louisville/Highlands.tsx" description="Updating Highlands neighborhood page to use ＜a＞ tags.">
import React from 'react';
</prism-write>
AFTER TAG
    `;

    const result = cleanFullResponse(input);
    expect(result).toBe(expected);
  });

  it("should handle other prism tag types", () => {
    const input = `<prism-rename from="src/<old>.tsx" to="src/<new>.tsx"></prism-rename>`;
    const expected = `<prism-rename from="src/＜old＞.tsx" to="src/＜new＞.tsx"></prism-rename>`;

    const result = cleanFullResponse(input);
    expect(result).toBe(expected);
  });

  it("should handle prism-delete tags", () => {
    const input = `<prism-delete path="src/<component>.tsx"></prism-delete>`;
    const expected = `<prism-delete path="src/＜component＞.tsx"></prism-delete>`;

    const result = cleanFullResponse(input);
    expect(result).toBe(expected);
  });

  it("should not affect content outside prism tags", () => {
    const input = `Some text with <regular> HTML tags. <prism-write path="test.tsx" description="With <nested> tags.">content</prism-write> More <html> here.`;
    const expected = `Some text with <regular> HTML tags. <prism-write path="test.tsx" description="With ＜nested＞ tags.">content</prism-write> More <html> here.`;

    const result = cleanFullResponse(input);
    expect(result).toBe(expected);
  });

  it("should handle empty attributes", () => {
    const input = `<prism-write path="src/file.tsx">content</prism-write>`;
    const expected = `<prism-write path="src/file.tsx">content</prism-write>`;

    const result = cleanFullResponse(input);
    expect(result).toBe(expected);
  });

  it("should handle attributes without < characters", () => {
    const input = `<prism-write path="src/file.tsx" description="Normal description">content</prism-write>`;
    const expected = `<prism-write path="src/file.tsx" description="Normal description">content</prism-write>`;

    const result = cleanFullResponse(input);
    expect(result).toBe(expected);
  });
});
