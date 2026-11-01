import { describe, expect, it } from "vitest";

import { fuzzyFilter, fuzzyScore } from "../picker/fuzzy";
import { isSingleToken, parseTokens } from "./tokens";

describe("tokens", () => {
  it("splits text and known tokens", () => {
    expect(parseTokens("hi {{var:name}}, {{output:abc}}!")).toEqual([
      { kind: "text", text: "hi " },
      { kind: "token", token: "var:name", raw: "{{var:name}}" },
      { kind: "text", text: ", " },
      { kind: "token", token: "output:abc", raw: "{{output:abc}}" },
      { kind: "text", text: "!" },
    ]);
  });

  it("keeps unknown and broken tokens as text", () => {
    expect(parseTokens("{{nope}} {{var:x")).toEqual([{ kind: "text", text: "{{nope}} {{var:x" }]);
  });

  it("detects a single token", () => {
    expect(isSingleToken("{{input}}")).toBe(true);
    expect(isSingleToken("{{input}} ")).toBe(false);
  });
});

describe("fuzzy", () => {
  it("ignores accents and case", () => {
    expect(fuzzyScore("ecran", "Écran")).toBeGreaterThan(0);
  });

  it("ranks word starts above scattered matches", () => {
    const items = ["Get variable", "Set volume", "Show alert"];
    expect(fuzzyFilter(items, "vol", (x) => [x])[0]).toBe("Set volume");
    expect(fuzzyFilter(items, "sa", (x) => [x])).toContain("Show alert");
    expect(fuzzyFilter(items, "zz", (x) => [x])).toEqual([]);
  });
});
