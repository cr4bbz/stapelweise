import { afterEach, describe, expect, it, vi } from "vitest";
import { registerImageZoom, renderMarkdown } from "./markdown";

describe("renderMarkdown security", () => {
  it("escapes raw HTML", () => {
    expect(renderMarkdown('<script>alert("x")</script>')).not.toContain("<script>");
  });

  it.each([
    "javascript:alert(1)",
    "jav%61script:alert(1)",
    "%20javascript:alert(1)",
    "java%0ascript:alert(1)",
    "data:text/html,hello",
  ])("blocks dangerous link %s", (href) => {
    const html = renderMarkdown(`[click](${href})`);
    expect(html).not.toContain("href=");
  });

  it("escapes link attributes", () => {
    const html = renderMarkdown('[safe](https://example.com/\"onmouseover=\"alert(1))');
    expect(html).not.toContain('onmouseover="alert');
    expect(html).toContain("&quot;");
  });

  it("keeps safe https links", () => {
    expect(renderMarkdown("[safe](https://example.com/path?q=1)")).toContain('href="https://example.com/path?q=1"');
  });

  it("removes inline event handlers from rendered images", () => {
    const html = renderMarkdown("![x](https://example.com/a.png)");
    expect(html).toContain('data-stapelweise-zoom="true"');
    expect(html).not.toContain("onclick=");
  });

  it("blocks SVG data images", () => {
    expect(renderMarkdown("![x](data:image/svg+xml;base64,PHN2Zz4=)")).not.toContain("<img");
  });

  it("keeps raster data images", () => {
    expect(renderMarkdown("![x](data:image/png;base64,AAAA)")).toContain("<img");
  });

  it("renders inline math", () => {
    const html = renderMarkdown("Euler: $e^{i\\pi}+1=0$");
    expect(html).toContain("katex");
  });

  it("renders display math", () => {
    const html = renderMarkdown("$$x^2 + y^2 = z^2$$");
    expect(html).toContain("katex-display");
  });

  it("does not interpret a numeric currency token as math", () => {
    const html = renderMarkdown("Preis: $5$");
    expect(html).toContain("$5$");
    expect(html).not.toContain("katex");
  });

  it("restores repeated identical formulas independently", () => {
    const html = renderMarkdown("$x$ und $x$");
    expect((html.match(/class=\"katex\"/g) ?? []).length).toBe(2);
  });
});

describe("registerImageZoom", () => {
  afterEach(() => {
    document.body.innerHTML = "";
  });

  it("delegates zoom clicks without inline JavaScript", () => {
    document.body.innerHTML = '<img data-stapelweise-zoom="true" src="https://example.com/a.png">';
    const listener = vi.fn();
    window.addEventListener("stapelweise:zoom-image", listener);
    const unregister = registerImageZoom(document);
    (document.querySelector("img") as HTMLImageElement).click();
    expect(listener).toHaveBeenCalledOnce();
    unregister();
    window.removeEventListener("stapelweise:zoom-image", listener);
  });
});
