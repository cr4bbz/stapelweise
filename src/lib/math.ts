import katex from "katex";

const MATH_REGEX = /(\$\$[\s\S]*?\$\$|\$[\s\S]*?\$)/g;

// Reset stateful regex before use
function mathRegex(): RegExp {
  MATH_REGEX.lastIndex = 0;
  return MATH_REGEX;
}

export function hasMath(text: string): boolean {
  return mathRegex().test(text);
}

/** Render a standalone LaTeX expression, with optional $ delimiters. */
export function renderLatexExpression(text: string): string {
  const trimmed = text.trim();
  if (!trimmed) return "";

  const startsDisplay = trimmed.startsWith("$$");
  const displayMode = startsDisplay && trimmed.endsWith("$$") && trimmed.length > 3;
  const startsInline = !startsDisplay && trimmed.startsWith("$");
  const inlineMode = startsInline && trimmed.endsWith("$") && trimmed.length > 1;
  const formula = displayMode
    ? trimmed.slice(2, -2)
    : inlineMode
      ? trimmed.slice(1, -1)
      : startsDisplay
        ? trimmed.slice(2)
        : startsInline
          ? trimmed.slice(1)
          : trimmed;

  return katex.renderToString(formula, {
    displayMode,
    throwOnError: false,
  });
}

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

/**
 * Render LaTeX math inside $...$ (inline) and $$...$$ (display block) to HTML.
 * Non-math text is HTML-escaped to prevent XSS.
 */
export function renderMath(text: string): string {
  if (!text) return "";

  const regex = mathRegex();
  const parts: string[] = [];
  let lastIndex = 0;
  let match: RegExpExecArray | null;

  while ((match = regex.exec(text)) !== null) {
    const before = text.slice(lastIndex, match.index);
    parts.push(escapeHtml(before));

    const raw = match[0];
    if (raw.startsWith("$$") && raw.endsWith("$$")) {
      const formula = raw.slice(2, -2);
      parts.push(
        katex.renderToString(formula, {
          displayMode: true,
          throwOnError: false,
        })
      );
    } else {
      const formula = raw.slice(1, -1);
      parts.push(
        katex.renderToString(formula, {
          displayMode: false,
          throwOnError: false,
        })
      );
    }

    lastIndex = match.index + raw.length;
  }

  parts.push(escapeHtml(text.slice(lastIndex)));
  return parts.join("");
}
