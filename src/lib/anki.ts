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
  const firstLine = text.split(/\r?\n/, 1)[0] ?? "";
  if (firstLine.includes("\t")) return "\t";
  if (firstLine.includes(";")) return ";";
  return ",";
}

function cleanAnkiField(value: string) {
  return value
    .replace(/<br\s*\/?>/gi, "\n")
    .replace(/<\/p\s*>/gi, "\n")
    .replace(/<[^>]+>/g, "")
    .replace(/&nbsp;/gi, " ")
    .replace(/&amp;/gi, "&")
    .replace(/&lt;/gi, "<")
    .replace(/&gt;/gi, ">")
    .replace(/&quot;/gi, '"')
    .trim();
}

export function parseAnkiText(text: string): JsonCardInput[] {
  const separator = sourceSeparator(text);
  return parseDelimited(text.replace(/^\uFEFF/, ""), separator)
    .filter((row) => row.length >= 2 && !row[0].trimStart().startsWith("#"))
    .map((row) => ({
      front: cleanAnkiField(row[0]),
      back: cleanAnkiField(row[1]),
      tags: (row[2] ?? "").split(/\s+/).map((tag) => tag.trim()).filter(Boolean),
    }))
    .filter((card) => card.front.length > 0 && card.back.length > 0);
}

function quoteTsv(value: string) {
  return `"${value.replace(/"/g, '""')}"`;
}

export function toAnkiTsv(cards: Card[]) {
  const header = ["#separator:Tab", "#html:false", "#columns:Front\tBack\tTags", "#tags column:3"];
  const rows = cards.map((card) => [card.front, card.back, card.tags.join(" ")].map(quoteTsv).join("\t"));
  return `${[...header, ...rows].join("\n")}\n`;
}
