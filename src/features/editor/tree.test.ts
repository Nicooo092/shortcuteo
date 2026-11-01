import { describe, expect, it } from "vitest";

import type { Step } from "../../bindings/Step";
import {
  cloneStep,
  containerOf,
  countSteps,
  duplicateStep,
  findStep,
  flatten,
  moveStep,
  removeStep,
  stepsBefore,
} from "./tree";

const s = (uuid: string, extra: Partial<Step> = {}): Step => ({
  uuid,
  action: "scripting.nothing",
  ...extra,
});

const doc = (): Step[] => [
  s("a"),
  s("if", { action: "control.if", body: [s("b")], otherwise: [s("c")] }),
  s("menu", { action: "control.menu", cases: [{ label: "x", body: [s("d")] }] }),
];

describe("tree", () => {
  it("finds steps and their container at any depth", () => {
    expect(findStep(doc(), "d")?.uuid).toBe("d");
    expect(containerOf(doc(), "c")).toBe("if:otherwise");
    expect(containerOf(doc(), "d")).toBe("menu:case:0");
    expect(containerOf(doc(), "a")).toBe("root");
  });

  it("moves a step into a block body", () => {
    const moved = moveStep(doc(), "a", "if:body", 1);
    expect(moved.map((x) => x.uuid)).toEqual(["if", "menu"]);
    expect(findStep(moved, "if")?.body?.map((x) => x.uuid)).toEqual(["b", "a"]);
  });

  it("moves down within the same list without skipping a slot", () => {
    const moved = moveStep(doc(), "a", "root", 2);
    expect(moved.map((x) => x.uuid)).toEqual(["if", "a", "menu"]);
  });

  it("refuses to drop a block inside itself", () => {
    const before = doc();
    expect(moveStep(before, "if", "if:body", 0)).toBe(before);
  });

  it("removes nested steps without touching others", () => {
    const out = removeStep(doc(), "b");
    expect(findStep(out, "b")).toBeUndefined();
    expect(countSteps(out)).toBe(countSteps(doc()) - 1);
  });

  it("duplicates with fresh uuids and remapped references", () => {
    const steps: Step[] = [
      s("blk", {
        action: "control.repeat",
        body: [s("in"), s("use", { params: { text: "{{output:in}} {{output:outside}}" } })],
      }),
    ];
    const copy = cloneStep(steps[0]!);
    const inner = copy.body!;
    expect(copy.uuid).not.toBe("blk");
    expect(inner[0]!.uuid).not.toBe("in");
    expect(inner[1]!.params!.text).toBe(`{{output:${inner[0]!.uuid}}} {{output:outside}}`);
    expect(duplicateStep(steps, "blk")).toHaveLength(2);
  });

  it("lists the steps that run before a given one", () => {
    expect(stepsBefore(doc(), "c").map((x) => x.uuid)).toEqual(["a", "b"]);
    expect(stepsBefore(doc(), "d").map((x) => x.uuid)).toEqual(["a", "b", "c", "if"]);
  });

  it("flattens with depth", () => {
    expect(flatten(doc()).map((r) => `${r.step.uuid}:${r.depth}`)).toEqual([
      "a:0",
      "if:0",
      "b:1",
      "c:1",
      "menu:0",
      "d:1",
    ]);
  });
});
