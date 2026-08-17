import type { Card, JsonCardInput } from "$lib/types";

function parseDelimited(text: string, separator: string): string[][] {
  const rows: string[][] = [];
  let row: string[] = [];
  let field = "";
  let quoted = false;

  for (let index = 0; index < text.length; index += 1) {
    const character = text[index];
    if (quoted) {
      if (character === '"' && text[index + 1] === '"') {
        field += '"';
        index += 1;
      } else if (character === '"') {
        quoted = false;
      } else {
        field += character;
      }
      continue;
    }
    if (character === '"' && field.length === 0) {
      quoted = true;
    } else if (character === separator) {
      row.push(field);
      field = "";
    } else if (character === "\n") {
      row.push(field.replace(/\r$/, ""));
      rows.push(row);
      row = [];
      field = "";
    } else {
      field += character;
    }
  }
  if (field.length > 0 || row.length > 0) {
    row.push(field.replace(/\r$/, ""));
    rows.push(row);
  }
  return rows;
}

function sourceSeparator(text: string) {
  const header = text.match(/^#separator:(.+)$/m)?.[1]?.trim().toLowerCase();
  if (header === "tab" || header === "\\t") return "\t";
  if (header === "comma") return ",";
  if (header === "semicolon") return ";";
  const firstDataLine = text.split(/\r?\n/).find((line) => !line.startsWith("#")) ?? "";
  if (firstDataLine.includes("\t")) return "\t";
  if (firstDataLine.includes(";")) return ";";
  return ",";
}

function decodeHtmlEntitiesOnce(value: string) {
  return value
    .replace(/&nbsp;/gi, " ")
    .replace(/&lt;/gi, "<")
    .replace(/&gt;/gi, ">")
    .replace(/&quot;/gi, '"')
    .replace(/&#39;|&apos;/gi, "'")
    .replace(/&amp;/gi, "&");
}

function cleanAnkiField(value: string, html: boolean) {
  if (!html) return value.trim();
  return decodeHtmlEntitiesOnce(
    value
      .replace(/<br\s*\/?>/gi, "\n")
      .replace(/<\/p\s*>/gi, "\n")
      .replace(/<[^>]+>/g, "")
  ).trim();
}

function parseTags(value: string): string[] {
  const trimmed = value.trim();
  if (!trimmed) return [];
  if (trimmed.startsWith("[")) {
    try {
      const parsed = JSON.parse(trimmed);
      if (Array.isArray(parsed) && parsed.every((item) => typeof item === "string")) {
        return parsed.map((tag) => tag.trim()).filter(Boolean);
      }
    } catch {
      // Fall back to Anki's conventional whitespace-separated tag column.
    }
  }
  return trimmed.split(/\s+/).map((tag) => tag.trim()).filter(Boolean);
}

function serializeTags(tags: string[]): string {
  return tags.some((tag) => /\s/.test(tag)) ? JSON.stringify(tags) : tags.join(" ");
}

export function parseAnkiText(text: string): JsonCardInput[] {
  const normalized = text.replace(/^\uFEFF/, "");
  const separator = sourceSeparator(normalized);
  const html = /^#html:true\s*$/im.test(normalized);
  return parseDelimited(normalized, separator)
    .filter((row) => row.length >= 2 && !row[0].trimStart().startsWith("#"))
    .map((row) => ({
      front: cleanAnkiField(row[0], html),
      back: cleanAnkiField(row[1], html),
      tags: parseTags(row[2] ?? ""),
    }))
    .filter((card) => card.front.length > 0 && card.back.length > 0);
}

function quoteTsv(value: string) {
  return `"${value.replace(/"/g, '""')}"`;
}

export function toAnkiTsv(cards: Card[]) {
  const header = ["#separator:Tab", "#html:false", "#columns:Front\tBack\tTags", "#tags column:3"];
  const rows = cards.map((card) =>
    [card.front, card.back, serializeTags(card.tags)].map(quoteTsv).join("\t")
  );
  return `${[...header, ...rows].join("\n")}\n`;
}
