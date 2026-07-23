// Layout store for the modular dashboard.
//
// Owns everything about *how* modules are arranged and persisted: the order,
// per-module widths, single-card selections, spacer configs, and which decks or
// exams the user has hidden. All localStorage access lives here so components
// never touch storage keys directly. Mutations save immediately.

import type { Deck, Exam } from "$lib/types";
import {
  availableModuleIds,
  defaultModuleOrder,
  defaultWidthOf,
  deckIdOf,
  examIdOf,
  generateModuleSuffix,
  isDashboardModuleKey,
  isDeckModule,
  isExamModule,
  isSingleCardModule,
  isSpacerModule,
  isRemovable,
  moduleWidthOptions,
  type DashboardModuleKey,
  type BuiltinModuleId,
  type DeckModuleId,
  type ExamModuleId,
  type ModuleWidth,
  type SingleCardModuleId,
  type SpacerModuleConfig,
  type SpacerModuleId,
  type SpacerVariant,
} from "./registry";

const layoutStorageKey = "stapelweise.dashboard.modules.v5";
const widthStorageKey = "stapelweise.dashboard.widths.v4";
const legacyLayoutStorageKey = "stapelweise.dashboard.modules.v4";
const legacyGridStorageKey = "stapelweise.dashboard.grid.v1";
const singleCardStorageKey = "stapelweise.dashboard.singleCards.v1";
const spacerStorageKey = "stapelweise.dashboard.spacers.v1";
const hiddenDeckStorageKey = "stapelweise.dashboard.hiddenDecks.v1";
const hiddenExamStorageKey = "stapelweise.dashboard.hiddenExams.v1";

const VALID_WIDTHS: ModuleWidth[] = [2, 3, 4, 6, 8, 12];
const compactModuleIds: BuiltinModuleId[] = ["brand", "search", "settings"];

let order = $state<DashboardModuleKey[]>([...defaultModuleOrder]);
let widths = $state<Partial<Record<DashboardModuleKey, ModuleWidth>>>({});
let singleCards = $state<Record<string, string>>({});
let spacers = $state<Record<string, SpacerModuleConfig>>({});
let hiddenDeckIds = $state<string[]>([]);
let hiddenExamIds = $state<string[]>([]);
let loaded = $state(false);

function readJson<T>(key: string, fallback: T): T {
  try {
    const raw = localStorage.getItem(key);
    return raw === null ? fallback : (JSON.parse(raw) as T);
  } catch {
    return fallback;
  }
}

function persist(key: string, value: unknown) {
  localStorage.setItem(key, JSON.stringify(value));
}

/** Guarantee settings is always present so it can never be lost. */
function withSettings(next: DashboardModuleKey[]): DashboardModuleKey[] {
  return next.includes("settings") ? next : ["settings", ...next];
}

function commitOrder(next: DashboardModuleKey[]) {
  order = withSettings(next);
  persist(layoutStorageKey, order);
}

function commitWidths(next: Partial<Record<DashboardModuleKey, ModuleWidth>>) {
  widths = next;
  persist(widthStorageKey, next);
}

function commitHiddenDecks(next: string[]) {
  hiddenDeckIds = next;
  persist(hiddenDeckStorageKey, next);
}

function commitHiddenExams(next: string[]) {
  hiddenExamIds = next;
  persist(hiddenExamStorageKey, next);
}

function commitSingleCards(next: Record<string, string>) {
  singleCards = next;
  persist(singleCardStorageKey, next);
}

function commitSpacers(next: Record<string, SpacerModuleConfig>) {
  spacers = next;
  persist(spacerStorageKey, next);
}

/** Load persisted layout from localStorage. Call once from a component onMount. */
function load() {
  try {
    singleCards = readJson<Record<string, string>>(singleCardStorageKey, {});

    const savedHiddenDecks = readJson<unknown>(hiddenDeckStorageKey, []);
    hiddenDeckIds = Array.isArray(savedHiddenDecks)
      ? savedHiddenDecks.filter((id): id is string => typeof id === "string")
      : [];

    const savedHiddenExams = readJson<unknown>(hiddenExamStorageKey, []);
    hiddenExamIds = Array.isArray(savedHiddenExams)
      ? savedHiddenExams.filter((id): id is string => typeof id === "string")
      : [];

    const savedSpacers = readJson<Record<string, unknown>>(spacerStorageKey, {});
    spacers = Object.fromEntries(
      Object.entries(savedSpacers).filter(
        ([id, config]) =>
          isSpacerModule(id) &&
          typeof config === "object" &&
          config !== null &&
          ["blank", "divider", "note"].includes((config as SpacerModuleConfig).variant),
      ),
    ) as Record<string, SpacerModuleConfig>;

    const savedWidths = readJson<Record<string, unknown>>(widthStorageKey, {});
    widths = Object.fromEntries(
      Object.entries(savedWidths).filter(
        ([id, width]) => isDashboardModuleKey(id) && VALID_WIDTHS.includes(width as ModuleWidth),
      ),
    ) as Partial<Record<DashboardModuleKey, ModuleWidth>>;

    localStorage.removeItem(legacyGridStorageKey);

    const savedLayout = localStorage.getItem(layoutStorageKey);
    if (savedLayout !== null) {
      const savedOrder = JSON.parse(savedLayout);
      const filtered = Array.isArray(savedOrder)
        ? [...new Set(savedOrder.filter(isDashboardModuleKey))]
        : [...defaultModuleOrder];
      order = withSettings(filtered);
    } else {
      order = migrateLegacyLayout();
      persist(layoutStorageKey, order);
    }
  } catch {
    order = [...defaultModuleOrder];
  } finally {
    loaded = true;
  }
}

function migrateLegacyLayout(): DashboardModuleKey[] {
  const legacyRaw = localStorage.getItem(legacyLayoutStorageKey);
  if (legacyRaw === null) return [...defaultModuleOrder];

  let legacyOrder: unknown;
  try {
    legacyOrder = JSON.parse(legacyRaw);
  } catch {
    return [...defaultModuleOrder];
  }

  const knownLegacy = Array.isArray(legacyOrder)
    ? legacyOrder.filter((id: unknown): id is BuiltinModuleId =>
        typeof id === "string" && defaultModuleOrder.includes(id as BuiltinModuleId),
      )
    : [];
  const visibleCompact = compactModuleIds.filter(
    (id) => id === "settings" || knownLegacy.includes(id),
  );
  const remaining = knownLegacy.filter((id) => !compactModuleIds.includes(id));
  return [...visibleCompact, ...remaining];
}

// ── Width helpers ────────────────────────────────────────────

function widthOf(id: DashboardModuleKey): ModuleWidth {
  return widths[id] ?? defaultWidthOf(id);
}

function resize(id: DashboardModuleKey, direction: -1 | 1) {
  const options = moduleWidthOptions(id);
  const index = options.indexOf(widthOf(id));
  const nextIndex = Math.max(0, Math.min(options.length - 1, index + direction));
  if (nextIndex === index) return;
  commitWidths({ ...widths, [id]: options[nextIndex] });
}

// ── Ordering ─────────────────────────────────────────────────

function move(source: DashboardModuleKey, target: DashboardModuleKey, placement: "before" | "after") {
  if (source === target) return;
  const next = order.filter((id) => id !== source);
  const targetIndex = next.indexOf(target);
  if (targetIndex < 0) return;
  next.splice(placement === "after" ? targetIndex + 1 : targetIndex, 0, source);
  commitOrder(next);
}

function moveBy(id: DashboardModuleKey, offset: number) {
  const currentIndex = order.indexOf(id);
  const nextIndex = Math.max(0, Math.min(order.length - 1, currentIndex + offset));
  if (currentIndex === nextIndex) return;
  const next = [...order];
  next.splice(currentIndex, 1);
  next.splice(nextIndex, 0, id);
  commitOrder(next);
}

// ── Add / remove modules ─────────────────────────────────────

function addBuiltin(id: BuiltinModuleId) {
  if (order.includes(id)) return;
  commitOrder([...order, id]);
}

function addDeck(deckId: string) {
  const moduleId: DeckModuleId = `deck:${deckId}`;
  if (hiddenDeckIds.includes(deckId)) commitHiddenDecks(hiddenDeckIds.filter((id) => id !== deckId));
  if (!order.includes(moduleId)) commitOrder([...order, moduleId]);
}

function addExam(examId: string) {
  const moduleId: ExamModuleId = `exam:${examId}`;
  if (hiddenExamIds.includes(examId)) commitHiddenExams(hiddenExamIds.filter((id) => id !== examId));
  if (!order.includes(moduleId)) commitOrder([...order, moduleId]);
}

function addSingleCard() {
  commitOrder([...order, `singleCard:${generateModuleSuffix()}`]);
}

function addSpacer(variant: SpacerVariant) {
  const moduleId: SpacerModuleId = `spacer:${generateModuleSuffix()}`;
  commitSpacers({ ...spacers, [moduleId]: { variant, note: "" } });
  commitOrder([...order, moduleId]);
}

function remove(id: DashboardModuleKey) {
  if (!isRemovable(id)) return;
  commitOrder(order.filter((moduleId) => moduleId !== id));

  const { [id]: _removedWidth, ...remainingWidths } = widths;
  commitWidths(remainingWidths);

  if (isSingleCardModule(id)) {
    const { [id]: _selection, ...rest } = singleCards;
    commitSingleCards(rest);
  }
  if (isDeckModule(id)) {
    commitHiddenDecks([...new Set([...hiddenDeckIds, deckIdOf(id)])]);
  }
  if (isExamModule(id)) {
    commitHiddenExams([...new Set([...hiddenExamIds, examIdOf(id)])]);
  }
  if (isSpacerModule(id)) {
    const { [id]: _spacer, ...rest } = spacers;
    commitSpacers(rest);
  }
}

/** Move a deck/exam module to just before the archive slot (used on archive). */
function relocateBeforeArchive(id: DashboardModuleKey) {
  const filtered = order.filter((moduleId) => moduleId !== id);
  if (!filtered.includes("archive")) commitOrder([...filtered, "archive"]);
  else commitOrder(filtered);
}

// ── Single card & spacer content ─────────────────────────────

function selectSingleCard(id: SingleCardModuleId, cardId: string) {
  commitSingleCards({ ...singleCards, [id]: cardId });
}

function updateSpacerNote(id: SpacerModuleId, note: string) {
  const config = spacers[id] ?? { variant: "note" as const, note: "" };
  commitSpacers({ ...spacers, [id]: { ...config, note } });
}

// ── Reconciliation with live decks / exams ───────────────────

/**
 * Keep the module order in sync with the decks and exams that actually exist:
 * drop modules for deleted decks/exams and append modules for new ones that the
 * user has not hidden. Called from a component $effect whenever data changes.
 */
function reconcile(decks: Deck[], exams: Exam[], examsLoaded: boolean) {
  if (!loaded) return;
  const activeDeckIds = new Set(decks.map((deck) => deck.id));
  const activeExamIds = new Set(exams.map((exam) => exam.id));

  const withoutStale = order.filter(
    (id) =>
      (!isDeckModule(id) || activeDeckIds.has(deckIdOf(id))) &&
      (!examsLoaded || !isExamModule(id) || activeExamIds.has(examIdOf(id))),
  );

  const missingDecks = decks
    .filter((deck) => !hiddenDeckIds.includes(deck.id) && !withoutStale.includes(`deck:${deck.id}`))
    .map((deck) => `deck:${deck.id}` as DeckModuleId);

  const missingExams = (examsLoaded ? exams : [])
    .filter((exam) => !hiddenExamIds.includes(exam.id) && !withoutStale.includes(`exam:${exam.id}`))
    .map((exam) => `exam:${exam.id}` as ExamModuleId);

  const next = [...withoutStale, ...missingDecks, ...missingExams];
  const changed = next.length !== order.length || next.some((id, index) => id !== order[index]);
  if (changed) commitOrder(next);
}

// ── Reset ────────────────────────────────────────────────────

function reset() {
  commitOrder([...defaultModuleOrder]);
  commitWidths({});
  commitSingleCards({});
  commitSpacers({});
  commitHiddenDecks([]);
  commitHiddenExams([]);
  localStorage.removeItem(legacyGridStorageKey);
}

export function getDashboardLayout() {
  return {
    get order() {
      return order;
    },
    get loaded() {
      return loaded;
    },
    get singleCards() {
      return singleCards;
    },
    get spacers() {
      return spacers;
    },
    get hiddenDeckIds() {
      return hiddenDeckIds;
    },
    get hiddenExamIds() {
      return hiddenExamIds;
    },
    /** Built-in modules not currently on the dashboard (for the picker). */
    get hiddenBuiltins(): BuiltinModuleId[] {
      return availableModuleIds.filter((id) => !order.includes(id));
    },
    load,
    widthOf,
    resize,
    move,
    moveBy,
    addBuiltin,
    addDeck,
    addExam,
    addSingleCard,
    addSpacer,
    remove,
    relocateBeforeArchive,
    selectSingleCard,
    updateSpacerNote,
    reconcile,
    reset,
  };
}

const dashboardLayout = getDashboardLayout();
export { dashboardLayout };
