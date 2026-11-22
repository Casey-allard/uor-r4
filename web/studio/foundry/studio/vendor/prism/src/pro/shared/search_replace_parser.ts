// prism-prism replacement for the Functional Source License search-replace parser. Not Prism's code.
// Parses the blocks the chat renders:
//   <<<<<<< SEARCH
//   old lines
//   =======
//   new lines
//   >>>>>>> REPLACE
export type SearchReplaceBlock = {
  searchContent: string;
  replaceContent: string;
};

export function parseSearchReplaceBlocks(text: string): SearchReplaceBlock[] {
  const lines = text.split(/\r?\n/);
  const blocks: SearchReplaceBlock[] = [];
  let mode: "outside" | "search" | "replace" = "outside";
  let search: string[] = [];
  let replace: string[] = [];
  for (const line of lines) {
    if (mode === "outside") {
      if (/^<{7}\s*SEARCH>?\s*$/.test(line)) { mode = "search"; search = []; replace = []; }
    } else if (mode === "search") {
      if (/^={7}\s*$/.test(line)) mode = "replace";
      else search.push(line);
    } else {
      if (/^>{7}\s*REPLACE\s*$/.test(line)) { blocks.push({ searchContent: search.join("\n"), replaceContent: replace.join("\n") }); mode = "outside"; }
      else replace.push(line);
    }
  }
  return blocks;
}
