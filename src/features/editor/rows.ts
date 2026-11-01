import type { Step } from "../../bindings/Step";
import type { ContainerId } from "./tree";

// the editor shows one flat sortable list; markers are the "Otherwise" / "End" lines of a
// block and double as drop targets for the container they close or open
export type Row =
  | { kind: "step"; id: string; step: Step; depth: number; container: ContainerId; index: number }
  | {
      kind: "marker";
      id: string;
      label: "otherwise" | "endIf" | "endRepeat" | "endMenu" | "case";
      owner: Step;
      caseIndex?: number;
      depth: number;
      drop: { container: ContainerId; index: number | "end" };
    };

export function buildRows(steps: Step[], blocks: Set<string>, hide?: string): Row[] {
  const rows: Row[] = [];
  const walk = (list: Step[], depth: number, container: ContainerId) => {
    list.forEach((step, index) => {
      rows.push({ kind: "step", id: step.uuid, step, depth, container, index });
      if (step.uuid === hide || !blocks.has(step.action)) return;
      const u = step.uuid;
      if (step.action === "control.menu") {
        const cases = step.cases ?? [];
        cases.forEach((c, i) => {
          rows.push({
            kind: "marker",
            id: `${u}:case:${i}:start`,
            label: "case",
            owner: step,
            caseIndex: i,
            depth: depth + 1,
            drop: { container: `${u}:case:${i}`, index: 0 },
          });
          walk(c.body, depth + 2, `${u}:case:${i}`);
        });
        rows.push({
          kind: "marker",
          id: `${u}:end`,
          label: "endMenu",
          owner: step,
          depth,
          drop: cases.length
            ? { container: `${u}:case:${cases.length - 1}`, index: "end" }
            : { container: "root", index: index + 1 },
        });
        return;
      }
      walk(step.body ?? [], depth + 1, `${u}:body`);
      if (step.action === "control.if") {
        rows.push({
          kind: "marker",
          id: `${u}:otherwise`,
          label: "otherwise",
          owner: step,
          depth,
          drop: { container: `${u}:body`, index: "end" },
        });
        walk(step.otherwise ?? [], depth + 1, `${u}:otherwise`);
      }
      rows.push({
        kind: "marker",
        id: `${u}:end`,
        label: step.action === "control.if" ? "endIf" : "endRepeat",
        owner: step,
        depth,
        drop: {
          container: step.action === "control.if" ? `${u}:otherwise` : `${u}:body`,
          index: "end",
        },
      });
    });
  };
  walk(steps, 0, "root");
  return rows;
}
