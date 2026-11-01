import type { ActionMeta } from "../../bindings/ActionMeta";
import type { Localized } from "../../bindings/Localized";
import type { Step } from "../../bindings/Step";
import type { Tile } from "../../bindings/Tile";
import { containerOf, findStep, stepsBefore } from "./tree";

export interface MagicVar {
  token: string;
  label: Localized | string;
  tile: Tile;
  icon: string;
}

const LOOPS = new Set(["control.repeat", "control.repeat_each"]);

function insideLoop(steps: Step[], uuid: string): boolean {
  let container = containerOf(steps, uuid);
  while (container && container !== "root") {
    const owner = findStep(steps, container.split(":")[0] ?? "");
    if (!owner) return false;
    if (LOOPS.has(owner.action)) return true;
    container = containerOf(steps, owner.uuid);
  }
  return false;
}

// what a step can reference: earlier outputs, variables set earlier, and the built-ins
export function magicFor(
  steps: Step[],
  uuid: string,
  byId: Record<string, ActionMeta>,
): MagicVar[] {
  const before = stepsBefore(steps, uuid);
  const out: MagicVar[] = [];
  const vars = new Set<string>();
  for (const s of before) {
    const meta = byId[s.action];
    if (!meta) continue;
    if (meta.output !== "nothing") {
      out.push({ token: `output:${s.uuid}`, label: meta.name, tile: meta.tile, icon: meta.icon });
    }
    const name = s.params?.name;
    if (
      (s.action === "variables.set" || s.action === "variables.add") &&
      typeof name === "string" &&
      name
    ) {
      vars.add(name);
    }
  }
  for (const v of vars) out.push({ token: `var:${v}`, label: v, tile: "orchid", icon: "variable" });
  if (insideLoop(steps, uuid)) {
    out.push({
      token: "item",
      label: { en: "Repeat item", fr: "Élément" },
      tile: "slate",
      icon: "list-restart",
    });
    out.push({
      token: "index",
      label: { en: "Repeat index", fr: "Tour" },
      tile: "slate",
      icon: "hash",
    });
  }
  out.push({
    token: "input",
    label: { en: "Shortcut input", fr: "Entrée du raccourci" },
    tile: "slate",
    icon: "log-in",
  });
  out.push({
    token: "now",
    label: { en: "Current date", fr: "Date actuelle" },
    tile: "amber",
    icon: "calendar",
  });
  return out.reverse();
}

export function describeToken(
  token: string,
  steps: Step[],
  byId: Record<string, ActionMeta>,
): MagicVar {
  if (token.startsWith("output:")) {
    const step = findStep(steps, token.slice(7));
    const meta = step ? byId[step.action] : undefined;
    if (meta) return { token, label: meta.name, tile: meta.tile, icon: meta.icon };
    return { token, label: "?", tile: "slate", icon: "circle-help" };
  }
  if (token.startsWith("var:"))
    return { token, label: token.slice(4), tile: "orchid", icon: "variable" };
  const builtin: Record<string, MagicVar> = {
    input: {
      token,
      label: { en: "Shortcut input", fr: "Entrée du raccourci" },
      tile: "slate",
      icon: "log-in",
    },
    item: {
      token,
      label: { en: "Repeat item", fr: "Élément" },
      tile: "slate",
      icon: "list-restart",
    },
    index: { token, label: { en: "Repeat index", fr: "Tour" }, tile: "slate", icon: "hash" },
    now: {
      token,
      label: { en: "Current date", fr: "Date actuelle" },
      tile: "amber",
      icon: "calendar",
    },
  };
  return builtin[token] ?? { token, label: token, tile: "slate", icon: "braces" };
}

export function variableNames(steps: Step[]): string[] {
  const names = new Set<string>();
  const walk = (list: Step[]) =>
    list.forEach((s) => {
      const n = s.params?.name;
      if (s.action.startsWith("variables.") && typeof n === "string" && n) names.add(n);
      walk(s.body ?? []);
      walk(s.otherwise ?? []);
      s.cases?.forEach((c) => walk(c.body));
    });
  walk(steps);
  return [...names].sort();
}
