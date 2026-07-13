const STORAGE_KEY = "stapelweise_media_v1";

// In-memory cache for fast sync lookups
const mediaCache = new Map<string, string>();

function initCache() {
  if (typeof window === "undefined") return;
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) {
      const parsed = JSON.parse(raw);
      for (const [key, val] of Object.entries(parsed)) {
        if (typeof val === "string") {
          mediaCache.set(key, val);
        }
      }
    }
  } catch (e) {
    console.error("Failed to load media store:", e);
  }
}

function persistCache() {
  if (typeof window === "undefined") return;
  try {
    const obj: Record<string, string> = {};
    for (const [key, val] of mediaCache.entries()) {
      obj[key] = val;
    }
    localStorage.setItem(STORAGE_KEY, JSON.stringify(obj));
  } catch (e) {
    console.error("Failed to persist media store:", e);
  }
}

// Initialise on load
initCache();

export const mediaStore = {
  /**
   * Save a Data URL image and return a concise reference tag ID (e.g. "media:img_1720891234_x7a")
   */
  saveMedia(dataUrl: string): string {
    if (!dataUrl.startsWith("data:")) return dataUrl;
    
    // Check if image data already exists in cache
    for (const [key, val] of mediaCache.entries()) {
      if (val === dataUrl) {
        return `media:${key}`;
      }
    }

    const id = `img_${Date.now()}_${Math.random().toString(36).substring(2, 7)}`;
    mediaCache.set(id, dataUrl);
    persistCache();
    return `media:${id}`;
  },

  /**
   * Get Data URL for a media reference ID (e.g. "media:img_123" or "img_123")
   */
  getMedia(idOrRef: string): string | null {
    if (!idOrRef) return null;
    if (idOrRef.startsWith("data:")) return idOrRef;
    const cleanId = idOrRef.startsWith("media:") ? idOrRef.slice(6) : idOrRef;
    return mediaCache.get(cleanId) || null;
  },

  /**
   * Replace all "media:img_..." references in an image src string or markdown text with full Data URLs
   */
  resolveSrc(src: string): string {
    if (!src) return "";
    if (src.startsWith("data:") || src.startsWith("http://") || src.startsWith("https://") || src.startsWith("file://")) {
      return src;
    }
    if (src.startsWith("media:") || src.startsWith("img_")) {
      const found = this.getMedia(src);
      if (found) return found;
    }
    return src;
  },

  /**
   * Automatically convert raw data:image/ base64 markdown strings in a text into clean media:img_ references
   */
  compactMarkdown(text: string): string {
    if (!text) return "";
    return text.replace(/!\[(.*?)\]\((data:image\/[a-zA-Z0-9+\/=;]+)\)/g, (_, alt, dataUrl) => {
      const ref = this.saveMedia(dataUrl);
      return `![${alt}](${ref})`;
    });
  }
};
