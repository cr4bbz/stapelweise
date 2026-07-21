export type SymbolicEvaluationStatus =
  | "checking"
  | "empty"
  | "equivalent"
  | "not-equivalent"
  | "invalid-input"
  | "undecidable"
  | "unavailable";

export interface SymbolicEvaluationResult {
  status: SymbolicEvaluationStatus;
}

let computeEnginePromise: Promise<typeof import("@cortex-js/compute-engine")> | null = null;

function loadComputeEngine() {
  computeEnginePromise ??= import("@cortex-js/compute-engine");
  return computeEnginePromise;
}

/**
 * Compare two LaTeX expressions without changing the learning schedule.
 * This is intentionally informational: the learner still rates recall via SM-2.
 */
export async function evaluateSymbolicAnswer(
  answer: string,
  expectedLatex: string,
): Promise<SymbolicEvaluationResult> {
  if (!answer.trim()) {
    return { status: "empty" };
  }

  if (!expectedLatex.trim()) {
    return { status: "invalid-input" };
  }

  try {
    const { parse } = await loadComputeEngine();
    const userExpression = parse(answer);
    const expectedExpression = parse(expectedLatex);

    if (!userExpression.isValid || !expectedExpression.isValid) {
      return { status: "invalid-input" };
    }

    const equivalent = userExpression.isEqual(expectedExpression);
    if (equivalent === true) return { status: "equivalent" };
    if (equivalent === false) return { status: "not-equivalent" };
    return { status: "undecidable" };
  } catch {
    return { status: "unavailable" };
  }
}
