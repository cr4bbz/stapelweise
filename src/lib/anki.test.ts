import { describe, expect, it } from "vitest";
import { parseAnkiText, toAnkiTsv } from "./anki";
import type { Card } from "./types";

function card(overrides: Partial<Card> = {}): Card {
  return {
    id: "c1",
    deck_id: "d1",
    card_type: "basic",
    content: null,
    reasoning: null,
    front: "Front",
    back: "Back",
    front_language: null,
    back_language: null,
    tags: [],
    created_at: "2026-01-01",
    updated_at: "2026-01-01",
    ...overrides,
  } as Card;
}

describe("parseAnkiText", () => {
  it.each([
    ["#separator:Tab\nA\tB\n", "A", "B"],
    ["#separator:comma\nA,B\n", "A", "B"],
    ["#separator:semicolon\nA;B\n", "A", "B"],
  ])("parses declared separators", (source, front, back) => {
    expect(parseAnkiText(source)[0]).toMatchObject({ front, back });
  });

  it("keeps literal HTML-like text when #html:false", () => {
    const [parsed] = parseAnkiText("#separator:Tab\n#html:false\n<x>\t<b>literal</b>\n");
    expect(parsed.front).toBe("<x>");
    expect(parsed.back).toBe("<b>literal</b>");
  });

  it("strips HTML only when #html:true", () => {
    const [parsed] = parseAnkiText("#separator:Tab\n#html:true\n<b>Front</b>\tA<br>B\n");
    expect(parsed.front).toBe("Front");
    expect(parsed.back).toBe("A\nB");
  });

  it("decodes entities only once", () => {
    const [parsed] = parseAnkiText("#separator:Tab\n#html:true\n&amp;lt;\t&amp;amp;\n");
    expect(parsed.front).toBe("&lt;");
    expect(parsed.back).toBe("&amp;");
  });

  it("supports quoted multiline fields", () => {
    const [parsed] = parseAnkiText('#separator:Tab\n"A\nB"\t"C\nD"\n');
    expect(parsed.front).toBe("A\nB");
    expect(parsed.back).toBe("C\nD");
  });

  it("keeps LaTeX unchanged in plain-text exports", () => {
    const [parsed] = parseAnkiText("#separator:Tab\n#html:false\n$e^{i\\pi}+1=0$\t$$x^2$$\n");
    expect(parsed.front).toBe("$e^{i\\pi}+1=0$");
    expect(parsed.back).toBe("$$x^2$$");
  });

  it("parses conventional whitespace-separated tags", () => {
    const [parsed] = parseAnkiText("#separator:Tab\nA\tB\tlogic math\n");
    expect(parsed.tags).toEqual(["logic", "math"]);
  });

  it("parses JSON tag lists used to preserve spaces", () => {
    const [parsed] = parseAnkiText('#separator:Tab\nA\tB\t["formal logic","math"]\n');
    expect(parsed.tags).toEqual(["formal logic", "math"]);
  });

  it("ignores metadata header rows", () => {
    expect(parseAnkiText("#separator:Tab\n#html:false\n#columns:Front\tBack\tTags\nA\tB\n")).toHaveLength(1);
  });

  it("ignores incomplete rows", () => {
    expect(parseAnkiText("only one column\n")).toEqual([]);
  });
});

describe("toAnkiTsv", () => {
  it("writes Anki metadata headers", () => {
    const output = toAnkiTsv([card()]);
    expect(output).toContain("#separator:Tab");
    expect(output).toContain("#html:false");
  });

  it("quotes embedded tabs, quotes and newlines", () => {
    const output = toAnkiTsv([card({ front: 'A\t"B"\nC' })]);
    expect(output).toContain('"A\t""B""\nC"');
    expect(parseAnkiText(output)[0].front).toBe('A\t"B"\nC');
  });

  it("round-trips ordinary tags", () => {
    const source = card({ tags: ["logic", "math"] });
    expect(parseAnkiText(toAnkiTsv([source]))[0].tags).toEqual(source.tags);
  });

  it("round-trips tags containing spaces", () => {
    const source = card({ tags: ["formal logic", "set theory"] });
    expect(parseAnkiText(toAnkiTsv([source]))[0].tags).toEqual(source.tags);
  });

  it.each([
    ["Ampersand", "A & B", "C & D"],
    ["Angles", "A < B", "C > D"],
    ["Entities", "&lt;literal&gt;", "&amp;literal"],
    ["Unicode", "Gödel ∀x", "Mengenlehre ∈"],
    ["Markdown", "**fett**", "`code`"],
    ["LaTeX", "$x+y$", "$$x^2$$"],
  ])("round-trips %s content", (_name, front, back) => {
    const parsed = parseAnkiText(toAnkiTsv([card({ front, back })]))[0];
    expect(parsed.front).toBe(front);
    expect(parsed.back).toBe(back);
  });
});
