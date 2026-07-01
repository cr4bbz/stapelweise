import { marked } from "marked";
import katex from "katex";

const MATH_REGEX = /(\$\$[\s\S]*?\$\$|\$[\s\S]*?\$)/g;

// ── XSS protection ─────────────────────────────────────

marked.use({
  renderer: {
    // Strip raw HTML — any user-supplied HTML is returned as plain text
    html({ text }: { text: string }) {
      return text;
    },
    // Block javascript: and data: URLs in links
    link({ href, title, text }: { href: string; title?: string | null; text: string }) {
      const lower = href.trim().toLowerCase();
      if (lower.startsWith("javascript:") || lower.startsWith("data:")) {
        return text;
      }
      const titleAttr = title ? ` title="${title.replace(/"/g, "&quot;")}"` : "";
      return `<a href="${href}"${titleAttr} rel="noopener noreferrer">${text}</a>`;
    },
    // Block javascript: URLs in images
    image({ href, title, text }: { href: string; title?: string | null; text: string }) {
      const lower = href.trim().toLowerCase();
      if (lower.startsWith("javascript:") || lower.startsWith("data:")) {
        return text;
      }
      const titleAttr = title ? ` title="${title.replace(/"/g, "&quot;")}"` : "";
      return `<img src="${href}" alt="${text}"${titleAttr} />`;
    },
  },
});

/**
 * Render a card text that may contain Markdown and LaTeX math to safe HTML.
 *
 * Strategy:
 * 1. Extract and render LaTeX ($...$ / $$...$$) with KaTeX → placeholders
 * 2. Run marked (Markdown → HTML) with raw HTML stripped and dangerous
 *    URLs blocked
 * 3. Restore KaTeX HTML from placeholders
 *
 * This allows Markdown formatting while keeping LaTeX math intact and
 * preventing XSS through raw HTML or javascript: links.
 */
export function renderMarkdown(text: string): string {
  if (!text) return "";

  // Step 1: extract and render LaTeX, replace with placeholders
  const mathBlocks: string[] = [];
  const regex = new RegExp(MATH_REGEX.source, "g");
  let processed = text;

  let match: RegExpExecArray | null;
  while ((match = regex.exec(text)) !== null) {
    const raw = match[0];
    let rendered: string;
    if (raw.startsWith("$$") && raw.endsWith("$$")) {
      const formula = raw.slice(2, -2);
      rendered = katex.renderToString(formula, {
        displayMode: true,
        throwOnError: false,
      });
    } else {
      const formula = raw.slice(1, -1);
      rendered = katex.renderToString(formula, {
        displayMode: false,
        throwOnError: false,
      });
    }
    const placeholder = `\x00MATH${mathBlocks.length}\x00`;
    mathBlocks.push(rendered);
    processed = processed.replace(raw, placeholder);
  }

  // Step 2: run marked (raw HTML stripped, dangerous URLs blocked)
  const html = marked.parse(processed) as string;

  // Step 3: restore KaTeX HTML
  let result = html;
  for (let i = 0; i < mathBlocks.length; i++) {
    result = result.replace(`\x00MATH${i}\x00`, mathBlocks[i]);
  }

  return result;
}
