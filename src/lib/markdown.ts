import { marked } from "marked";
import katex from "katex";
import { mediaStore } from "$lib/stores/media";

const MATH_REGEX = /(\$\$[\s\S]*?\$\$|\$[^$\n]+?\$)/g;
const CONTROL_CHARS = /[\u0000-\u001f\u007f]/g;

function escapeAttribute(value: string): string {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

function normalizeUrlForSecurityCheck(value: string): string {
  let normalized = value.trim();
  for (let attempt = 0; attempt < 3; attempt += 1) {
    try {
      const decoded = decodeURIComponent(normalized);
      if (decoded === normalized) break;
      normalized = decoded;
    } catch {
      break;
    }
  }
  return normalized.replace(CONTROL_CHARS, "").trim().toLowerCase();
}

function hasDangerousScheme(value: string): boolean {
  return /^(?:javascript|vbscript|data):/.test(normalizeUrlForSecurityCheck(value));
}

function isSafeImageSource(value: string): boolean {
  const normalized = normalizeUrlForSecurityCheck(value);
  if (/^(?:javascript|vbscript):/.test(normalized)) return false;
  if (normalized.startsWith("data:image/svg+xml")) return false;
  if (normalized.startsWith("data:") && !normalized.startsWith("data:image/")) return false;
  return true;
}

function isLikelyCurrency(raw: string): boolean {
  if (raw.startsWith("$$")) return false;
  const body = raw.slice(1, -1).trim();
  return /^\d+(?:[.,]\d{1,2})?$/.test(body);
}

marked.use({
  renderer: {
    html({ text }: { text: string }) {
      return text
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;");
    },
    link({ href, title, text }: { href: string; title?: string | null; text: string }) {
      if (hasDangerousScheme(href)) return text;
      const titleAttr = title ? ` title="${escapeAttribute(title)}"` : "";
      return `<a href="${escapeAttribute(href)}"${titleAttr} rel="noopener noreferrer">${text}</a>`;
    },
    image({ href, title, text }: { href: string; title?: string | null; text: string }) {
      const resolvedSrc = mediaStore.resolveSrc(href);
      if (!isSafeImageSource(resolvedSrc)) return escapeAttribute(text || "");
      const titleAttr = title ? ` title="${escapeAttribute(title)}"` : "";
      const altAttr = text ? ` alt="${escapeAttribute(text)}"` : "";
      return `<img src="${escapeAttribute(resolvedSrc)}"${altAttr}${titleAttr} data-stapelweise-zoom="true" class="max-h-48 max-w-full rounded-xl shadow-md border border-white/10 hover:scale-[1.01] transition-transform cursor-pointer my-2 inline-block object-contain" />`;
    },
  },
});

/** Register one delegated image-zoom handler without inline JavaScript. */
export function registerImageZoom(root: Document | HTMLElement = document): () => void {
  const handleClick = (event: Event) => {
    const target = event.target;
    if (!(target instanceof Element)) return;
    const image = target.closest<HTMLImageElement>("img[data-stapelweise-zoom='true']");
    if (!image) return;
    if (root instanceof HTMLElement && !root.contains(image)) return;
    event.stopPropagation();
    const src = image.currentSrc || image.src;
    window.dispatchEvent(new CustomEvent("stapelweise:zoom-image", { detail: src }));
  };
  root.addEventListener("click", handleClick);
  return () => root.removeEventListener("click", handleClick);
}

/** Render Markdown and LaTeX while keeping untrusted HTML and URLs inert. */
export function renderMarkdown(text: string): string {
  if (!text) return "";

  const mathBlocks: string[] = [];
  const processed = text.replace(MATH_REGEX, (raw) => {
    if (isLikelyCurrency(raw)) return raw;
    const displayMode = raw.startsWith("$$") && raw.endsWith("$$");
    const formula = displayMode ? raw.slice(2, -2) : raw.slice(1, -1);
    const rendered = katex.renderToString(formula, {
      displayMode,
      throwOnError: false,
    });
    const placeholder = `\u0000MATH${mathBlocks.length}\u0000`;
    mathBlocks.push(rendered);
    return placeholder;
  });

  let result = marked.parse(processed) as string;
  for (let index = 0; index < mathBlocks.length; index += 1) {
    const placeholder = `\u0000MATH${index}\u0000`;
    result = result.replace(placeholder, () => mathBlocks[index]);
  }
  return result;
}
