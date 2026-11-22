import { normalizePath } from "../../../shared/normalizePath";
import { unescapeXmlAttr, unescapeXmlContent } from "../../../shared/xmlEscape";
import log from "electron-log";
import { SqlQuery } from "../../lib/schemas";

const logger = log.scope("prism_tag_parser");

interface PrismFileTag {
  path: string;
  content: string;
  description?: string;
}

function escapeRegexLiteral(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

/**
 * Parse `<tagName path="..." description="...">content</tagName>` occurrences
 * into file tags. Used for `<prism-write>`: a `path`/`description` plus a body
 * with optional surrounding markdown fences.
 */
function parsePrismFileTags(
  fullResponse: string,
  tagName: string,
): PrismFileTag[] {
  const escapedTagName = escapeRegexLiteral(tagName);
  const tagRegex = new RegExp(
    `<${escapedTagName}([^>]*)>([\\s\\S]*?)</${escapedTagName}>`,
    "gi",
  );
  const pathRegex = /path="([^"]+)"/;
  const descriptionRegex = /description="([^"]+)"/;

  let match;
  const tags: PrismFileTag[] = [];

  while ((match = tagRegex.exec(fullResponse)) !== null) {
    const attributesString = match[1];
    let content = unescapeXmlContent(match[2].trim());

    const pathMatch = pathRegex.exec(attributesString);
    const descriptionMatch = descriptionRegex.exec(attributesString);

    if (pathMatch && pathMatch[1]) {
      const path = unescapeXmlAttr(pathMatch[1]);
      const description = descriptionMatch?.[1]
        ? unescapeXmlAttr(descriptionMatch[1])
        : undefined;

      const contentLines = content.split("\n");
      if (contentLines[0]?.startsWith("```")) {
        contentLines.shift();
      }
      if (contentLines[contentLines.length - 1]?.startsWith("```")) {
        contentLines.pop();
      }
      content = contentLines.join("\n");

      tags.push({ path: normalizePath(path), content, description });
    } else {
      logger.warn(
        `Found <${tagName}> tag without a valid 'path' attribute:`,
        match[0],
      );
    }
  }
  return tags;
}

export function getPrismWriteTags(fullResponse: string): PrismFileTag[] {
  return parsePrismFileTags(fullResponse, "prism-write");
}

export function getPrismRenameTags(fullResponse: string): {
  from: string;
  to: string;
}[] {
  const prismRenameRegex =
    /<prism-rename from="([^"]+)" to="([^"]+)"[^>]*>([\s\S]*?)<\/prism-rename>/g;
  let match;
  const tags: { from: string; to: string }[] = [];
  while ((match = prismRenameRegex.exec(fullResponse)) !== null) {
    tags.push({
      from: normalizePath(unescapeXmlAttr(match[1])),
      to: normalizePath(unescapeXmlAttr(match[2])),
    });
  }
  return tags;
}

export function getPrismCopyTags(fullResponse: string): {
  from: string;
  to: string;
  description?: string;
}[] {
  const prismCopyRegex = /<prism-copy([^>]*?)(?:>([\s\S]*?)<\/prism-copy>|\/>)/gi;
  const fromRegex = /from="([^"]+)"/;
  const toRegex = /to="([^"]+)"/;
  const descriptionRegex = /description="([^"]+)"/;

  let match;
  const tags: { from: string; to: string; description?: string }[] = [];

  while ((match = prismCopyRegex.exec(fullResponse)) !== null) {
    const attrs = match[1];
    const fromMatch = fromRegex.exec(attrs);
    const toMatch = toRegex.exec(attrs);
    const descriptionMatch = descriptionRegex.exec(attrs);

    if (fromMatch?.[1] && toMatch?.[1]) {
      tags.push({
        from: normalizePath(unescapeXmlAttr(fromMatch[1])),
        to: normalizePath(unescapeXmlAttr(toMatch[1])),
        description: descriptionMatch?.[1]
          ? unescapeXmlAttr(descriptionMatch[1])
          : undefined,
      });
    } else {
      logger.warn(
        "Found <prism-copy> tag without valid 'from' or 'to' attributes:",
        match[0],
      );
    }
  }
  return tags;
}

export function getPrismDeleteTags(fullResponse: string): string[] {
  const prismDeleteRegex =
    /<prism-delete path="([^"]+)"[^>]*>([\s\S]*?)<\/prism-delete>/g;
  let match;
  const paths: string[] = [];
  while ((match = prismDeleteRegex.exec(fullResponse)) !== null) {
    paths.push(normalizePath(unescapeXmlAttr(match[1])));
  }
  return paths;
}

export function getPrismAddDependencyTags(fullResponse: string): string[] {
  const prismAddDependencyRegex =
    /<prism-add-dependency packages="([^"]+)">[^<]*<\/prism-add-dependency>/g;
  let match;
  const packages: string[] = [];
  while ((match = prismAddDependencyRegex.exec(fullResponse)) !== null) {
    packages.push(...unescapeXmlAttr(match[1]).trim().split(/\s+/));
  }
  return packages;
}

export function getPrismChatSummaryTag(fullResponse: string): string | null {
  const prismChatSummaryRegex =
    /<prism-chat-summary>([\s\S]*?)<\/prism-chat-summary>/g;
  const match = prismChatSummaryRegex.exec(fullResponse);
  if (match && match[1]) {
    return unescapeXmlContent(match[1].trim());
  }
  return null;
}

export function getPrismExecuteSqlTags(fullResponse: string): SqlQuery[] {
  const prismExecuteSqlRegex =
    /<prism-execute-sql([^>]*)>([\s\S]*?)<\/prism-execute-sql>/g;
  const descriptionRegex = /description="([^"]+)"/;
  let match;
  const queries: { content: string; description?: string }[] = [];

  while ((match = prismExecuteSqlRegex.exec(fullResponse)) !== null) {
    const attributesString = match[1] || "";
    let content = unescapeXmlContent(match[2].trim());
    const descriptionMatch = descriptionRegex.exec(attributesString);
    const description = descriptionMatch?.[1]
      ? unescapeXmlAttr(descriptionMatch[1])
      : undefined;

    // Handle markdown code blocks if present
    const contentLines = content.split("\n");
    if (contentLines[0]?.startsWith("```")) {
      contentLines.shift();
    }
    if (contentLines[contentLines.length - 1]?.startsWith("```")) {
      contentLines.pop();
    }
    content = contentLines.join("\n");

    queries.push({ content, description });
  }

  return queries;
}

export function getPrismCommandTags(fullResponse: string): string[] {
  const prismCommandRegex =
    /<prism-command type="([^"]+)"[^>]*><\/prism-command>/g;
  let match;
  const commands: string[] = [];

  while ((match = prismCommandRegex.exec(fullResponse)) !== null) {
    commands.push(unescapeXmlAttr(match[1]));
  }

  return commands;
}

export function getPrismSearchReplaceTags(fullResponse: string): {
  path: string;
  content: string;
  description?: string;
}[] {
  const prismSearchReplaceRegex =
    /<prism-search-replace([^>]*)>([\s\S]*?)<\/prism-search-replace>/gi;
  const pathRegex = /path="([^"]+)"/;
  const descriptionRegex = /description="([^"]+)"/;

  let match;
  const tags: { path: string; content: string; description?: string }[] = [];

  while ((match = prismSearchReplaceRegex.exec(fullResponse)) !== null) {
    const attributesString = match[1] || "";
    let content = unescapeXmlContent(match[2].trim());

    const pathMatch = pathRegex.exec(attributesString);
    const descriptionMatch = descriptionRegex.exec(attributesString);

    if (pathMatch && pathMatch[1]) {
      const path = unescapeXmlAttr(pathMatch[1]);
      const description = descriptionMatch?.[1]
        ? unescapeXmlAttr(descriptionMatch[1])
        : undefined;

      // Handle markdown code fences if present
      const contentLines = content.split("\n");
      if (contentLines[0]?.startsWith("```")) {
        contentLines.shift();
      }
      if (contentLines[contentLines.length - 1]?.startsWith("```")) {
        contentLines.pop();
      }
      content = contentLines.join("\n");

      tags.push({ path: normalizePath(path), content, description });
    } else {
      logger.warn(
        "Found <prism-search-replace> tag without a valid 'path' attribute:",
        match[0],
      );
    }
  }
  return tags;
}
