import { marked } from "marked";
import katex from "katex";
import { mediaStore } from "$lib/stores/media";

const MATH_REGEX = /(\$\$[\s\S]*?\$\$|\$[\s\S]*?\$)/g;

// ── XSS protection ─────────────────────────────────────

marked.use({
  renderer: {
    // Strip raw HTML — escape it so it appears as text, not executable HTML
    html({ text }: { text: string }) {
      return text
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;");
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
    // Block dangerous javascript: URLs in images, block SVG data-urls for security, allow raster images and mediaStore refs
    image({ href, title, text }: { href: string; title?: string | null; text: string }) {
      const resolvedSrc = mediaStore.resolveSrc(href);
      const lower = resolvedSrc.trim().toLowerCase();
      if (lower.startsWith("javascript:") || lower.startsWith("data:image/svg+xml")) {
        return text;
      }
      if (lower.startsWith("data:") && !lower.startsWith("data:image/")) {
        return text;
      }
      const titleAttr = title ? ` title="${title.replace(/"/g, "&quot;")}"` : "";
      const altAttr = text ? ` alt="${text.replace(/"/g, "&quot;")}"` : "";
      return `<img src="${resolvedSrc}"${altAttr}${titleAttr} class="max-h-48 max-w-full rounded-xl shadow-md border border-white/10 hover:scale-[1.01] transition-transform cursor-pointer my-2 inline-block object-contain" onclick="event.stopPropagation(); window.dispatchEvent(new CustomEvent('stapelweise:zoom-image', { detail: this.src }))" />`;
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
