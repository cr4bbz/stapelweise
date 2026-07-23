import type { FreeTextContent, FreeTextEvaluationMode } from "$lib/types";

const defaultContent: FreeTextContent = {
  version: 1,
  evaluationMode: "manual",
};

export function parseFreeTextContent(content: string | null): FreeTextContent {
  if (!content) return { ...defaultContent };

  try {
    const parsed = JSON.parse(content) as Partial<FreeTextContent>;
    if (
      parsed.version === 1 &&
      (parsed.evaluationMode === "manual" || parsed.evaluationMode === "symbolic")
    ) {
      return {
        version: 1,
        evaluationMode: parsed.evaluationMode,
        expectedLatex: typeof parsed.expectedLatex === "string" ? parsed.expectedLatex : undefined,
      };
    }
  } catch {
    // Older or malformed free-text cards retain manual self-checking.
  }

  return { ...defaultContent };
}

export function serializeFreeTextContent(
  evaluationMode: FreeTextEvaluationMode,
  expectedLatex: string,
): string {
  const content: FreeTextContent = {
    version: 1,
    evaluationMode,
  };

  if (evaluationMode === "symbolic") {
    content.expectedLatex = expectedLatex.trim();
  }

  return JSON.stringify(content);
}
