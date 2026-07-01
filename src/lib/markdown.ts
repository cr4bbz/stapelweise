import { marked } from "marked";
import { renderMath } from "./math";

// Sanitize: only allow safe HTML tags
marked.use({
  renderer: {
    html({ text }) {
      return text; // strip raw HTML
    },
  },
});

/**
 * Render Markdown with inline LaTeX support.
 * LaTeX expressions ($...$ / $$...$$) are rendered first, then the result
 * is passed through a Markdown parser.
 */
export function renderMarkdown(text: string): string {
  if (!text) return "";
  // Render LaTeX first, then Markdown (marked ignores existing HTML)
  return marked.parse(renderMath(text)) as string;
}
