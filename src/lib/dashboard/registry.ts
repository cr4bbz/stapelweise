// Central, declarative registry for dashboard modules.
//
// This file is the single source of truth for module metadata (title, tone,
// default width, allowed widths, layout hints). Everything else — the layout
// store, the Dashboard container, the individual module components — reads from
// here instead of re-deriving the same scattered constants. Each module stays a
// self-contained unit; the registry only describes how it slots into the grid.

export type ModuleWidth = 2 | 3 | 4 | 6 | 8 | 12;
export type ModuleTone = "primary" | "secondary" | "warm";
export type SpacerVariant = "blank" | "divider" | "note";
export type SpacerModuleConfig = { variant: SpacerVariant; note: string };

/** Built-in modules that live in the registry. */
export type BuiltinModuleId =
  | "brand"
  | "search"
  | "settings"
  | "focus"
  | "continue"
  | "timer"
  | "learning"
  | "problems"
  | "weekPlan"
  | "quickCapture"
  | "learningTime"
  | "milestones"
  | "tags"
  | "archive";

/** Dynamic modules are created at runtime and carry an id suffix. */
export type SingleCardModuleId = `singleCard:${string}`;
export type DeckModuleId = `deck:${string}`;
export type ExamModuleId = `exam:${string}`;
export type SpacerModuleId = `spacer:${string}`;

export type DashboardModuleKey =
  | BuiltinModuleId
  | SingleCardModuleId
  | DeckModuleId
  | ExamModuleId
  | SpacerModuleId;

/** Which grouping a module belongs to in the "add module" picker. */
export type ModuleCategory = "core" | "study" | "planning" | "organization";

export interface ModuleDefinition {
  id: BuiltinModuleId;
  /** Key handed to t(); the raw German phrase doubles as the fallback label. */
  titleKey: string;
  tone: ModuleTone;
  defaultWidth: ModuleWidth;
  widthOptions: ModuleWidth[];
  category: ModuleCategory;
  /** Square icon tiles (search, settings) instead of full panels. */
  icon?: boolean;
  /** settings is a permanent module and cannot be removed. */
  removable?: boolean;
}

const STANDARD_WIDTHS: ModuleWidth[] = [4, 6, 8, 12];
const ICON_WIDTHS: ModuleWidth[] = [2, 3, 4];

/**
 * Registry of every built-in module keyed by id. Insertion order here defines
 * the canonical default order of the dashboard (see {@link defaultModuleOrder}),
 * except for `archive`, which is available but hidden by default.
 */
export const moduleRegistry: Record<BuiltinModuleId, ModuleDefinition> = {
  brand: { id: "brand", titleKey: "Stapelweise", tone: "primary", defaultWidth: 6, widthOptions: STANDARD_WIDTHS, category: "core", removable: true },
  search: { id: "search", titleKey: "Suche", tone: "primary", defaultWidth: 2, widthOptions: ICON_WIDTHS, category: "core", icon: true, removable: true },
  settings: { id: "settings", titleKey: "Einstellungen", tone: "secondary", defaultWidth: 2, widthOptions: ICON_WIDTHS, category: "core", icon: true, removable: false },
  continue: { id: "continue", titleKey: "Weiterlernen", tone: "secondary", defaultWidth: 4, widthOptions: STANDARD_WIDTHS, category: "study", removable: true },
  focus: { id: "focus", titleKey: "Kleine Runde", tone: "primary", defaultWidth: 6, widthOptions: STANDARD_WIDTHS, category: "study", removable: true },
  learning: { id: "learning", titleKey: "Lernlage", tone: "primary", defaultWidth: 6, widthOptions: STANDARD_WIDTHS, category: "study", removable: true },
  problems: { id: "problems", titleKey: "Problemkarten", tone: "warm", defaultWidth: 4, widthOptions: STANDARD_WIDTHS, category: "study", removable: true },
  timer: { id: "timer", titleKey: "Lerntimer", tone: "warm", defaultWidth: 4, widthOptions: STANDARD_WIDTHS, category: "study", removable: true },
  learningTime: { id: "learningTime", titleKey: "Lernzeit", tone: "secondary", defaultWidth: 4, widthOptions: STANDARD_WIDTHS, category: "planning", removable: true },
  weekPlan: { id: "weekPlan", titleKey: "Wochenplan", tone: "secondary", defaultWidth: 6, widthOptions: STANDARD_WIDTHS, category: "planning", removable: true },
  quickCapture: { id: "quickCapture", titleKey: "Schnellerfassung", tone: "primary", defaultWidth: 6, widthOptions: STANDARD_WIDTHS, category: "planning", removable: true },
  milestones: { id: "milestones", titleKey: "Meilensteine", tone: "warm", defaultWidth: 8, widthOptions: STANDARD_WIDTHS, category: "planning", removable: true },
  tags: { id: "tags", titleKey: "Tags", tone: "secondary", defaultWidth: 4, widthOptions: STANDARD_WIDTHS, category: "organization", removable: true },
  archive: { id: "archive", titleKey: "Archiv", tone: "primary", defaultWidth: 4, widthOptions: STANDARD_WIDTHS, category: "organization", removable: true },
};

/** Canonical order of modules shown on a fresh dashboard. */
export const defaultModuleOrder: BuiltinModuleId[] = [
  "brand",
  "search",
  "settings",
  "continue",
  "focus",
  "learning",
  "problems",
  "timer",
  "learningTime",
  "weekPlan",
  "quickCapture",
  "milestones",
  "tags",
];

/** Every built-in module id, including ones hidden by default (archive). */
export const availableModuleIds: BuiltinModuleId[] = [
  ...defaultModuleOrder,
  ...(Object.keys(moduleRegistry) as BuiltinModuleId[]).filter(
    (id) => !defaultModuleOrder.includes(id),
  ),
];

// ── Type predicates ──────────────────────────────────────────

export const isSingleCardModule = (id: string): id is SingleCardModuleId => id.startsWith("singleCard:");
export const isDeckModule = (id: string): id is DeckModuleId => id.startsWith("deck:");
export const isExamModule = (id: string): id is ExamModuleId => id.startsWith("exam:");
export const isSpacerModule = (id: string): id is SpacerModuleId => id.startsWith("spacer:");
export const isBuiltinModule = (id: string): id is BuiltinModuleId => id in moduleRegistry;

export const isDashboardModuleKey = (id: unknown): id is DashboardModuleKey =>
  typeof id === "string" &&
  (isBuiltinModule(id) || isSingleCardModule(id) || isDeckModule(id) || isExamModule(id) || isSpacerModule(id));

// ── Id helpers ───────────────────────────────────────────────

export const deckIdOf = (id: DeckModuleId): string => id.slice("deck:".length);
export const examIdOf = (id: ExamModuleId): string => id.slice("exam:".length);

export const spacerTitle = (variant: SpacerVariant): string =>
  variant === "divider" ? "Trennlinie" : variant === "note" ? "Notizfläche" : "Leerraum";

// ── Pure metadata resolvers (work for both built-in and dynamic ids) ──

export function moduleTone(id: DashboardModuleKey): ModuleTone {
  if (isSingleCardModule(id) || isDeckModule(id)) return "primary";
  if (isExamModule(id)) return "warm";
  if (isSpacerModule(id)) return "secondary";
  return moduleRegistry[id]?.tone ?? "primary";
}

export function defaultWidthOf(id: DashboardModuleKey): ModuleWidth {
  if (isSingleCardModule(id) || isDeckModule(id) || isExamModule(id) || isSpacerModule(id)) return 4;
  return moduleRegistry[id]?.defaultWidth ?? 4;
}

export function moduleWidthOptions(id: DashboardModuleKey): ModuleWidth[] {
  if (isDeckModule(id) || isExamModule(id)) return [4, 6, 8];
  if (isBuiltinModule(id)) return moduleRegistry[id].widthOptions;
  return STANDARD_WIDTHS;
}

/** Square icon tiles (search / settings) get special grid treatment. */
export function isIconModule(id: DashboardModuleKey): boolean {
  return isBuiltinModule(id) && Boolean(moduleRegistry[id].icon);
}

export function isRemovable(id: DashboardModuleKey): boolean {
  if (isBuiltinModule(id)) return moduleRegistry[id].removable !== false;
  return true;
}

export function generateModuleSuffix(): string {
  return typeof crypto !== "undefined" && "randomUUID" in crypto
    ? crypto.randomUUID()
    : `${Date.now()}-${Math.random().toString(16).slice(2)}`;
}
