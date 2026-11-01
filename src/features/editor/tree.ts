import type { Step } from "../../bindings/Step";

// a container is where steps live: the root list, a block body, an else branch, a menu case
export type ContainerId =
  "root" | `${string}:body` | `${string}:otherwise` | `${string}:case:${number}`;

export function newUuid() {
  return crypto.randomUUID();
}

export function childrenOf(steps: Step[], container: ContainerId): Step[] | undefined {
  if (container === "root") return steps;
  const [uuid, kind, index] = container.split(":");
  const owner = uuid ? findStep(steps, uuid) : undefined;
  if (!owner) return undefined;
  if (kind === "body") return owner.body ?? [];
  if (kind === "otherwise") return owner.otherwise ?? [];
  return owner.cases?.[Number(index)]?.body;
}

export function findStep(steps: Step[], uuid: string): Step | undefined {
  for (const s of steps) {
    if (s.uuid === uuid) return s;
    const inner =
      findStep(s.body ?? [], uuid) ??
      findStep(s.otherwise ?? [], uuid) ??
      s.cases?.reduce<Step | undefined>((hit, c) => hit ?? findStep(c.body, uuid), undefined);
    if (inner) return inner;
  }
  return undefined;
}

export function containerOf(
  steps: Step[],
  uuid: string,
  here: ContainerId = "root",
): ContainerId | undefined {
  for (const s of steps) {
    if (s.uuid === uuid) return here;
    const found =
      containerOf(s.body ?? [], uuid, `${s.uuid}:body`) ??
      containerOf(s.otherwise ?? [], uuid, `${s.uuid}:otherwise`) ??
      s.cases?.reduce<ContainerId | undefined>(
        (hit, c, i) => hit ?? containerOf(c.body, uuid, `${s.uuid}:case:${i}`),
        undefined,
      );
    if (found) return found;
  }
  return undefined;
}

// every helper below is immutable: zustand and react compare by reference
function mapSteps(steps: Step[], fn: (s: Step) => Step): Step[] {
  return steps.map((s) => {
    const next = fn(s);
    return {
      ...next,
      ...(next.body && { body: mapSteps(next.body, fn) }),
      ...(next.otherwise && { otherwise: mapSteps(next.otherwise, fn) }),
      ...(next.cases && { cases: next.cases.map((c) => ({ ...c, body: mapSteps(c.body, fn) })) }),
    };
  });
}

function setChildren(steps: Step[], container: ContainerId, children: Step[]): Step[] {
  if (container === "root") return children;
  const [uuid, kind, index] = container.split(":");
  return mapSteps(steps, (s) => {
    if (s.uuid !== uuid) return s;
    if (kind === "body") return { ...s, body: children };
    if (kind === "otherwise") return { ...s, otherwise: children };
    const cases = (s.cases ?? []).map((c, i) =>
      i === Number(index) ? { ...c, body: children } : c,
    );
    return { ...s, cases };
  });
}

export function withParam(step: Step, key: string, value: unknown): Step {
  const rest = Object.fromEntries(Object.entries(step.params ?? {}).filter(([k]) => k !== key));
  return { ...step, params: value === undefined ? rest : { ...rest, [key]: value } };
}

export function updateStep(steps: Step[], uuid: string, patch: (s: Step) => Step): Step[] {
  return mapSteps(steps, (s) => (s.uuid === uuid ? patch(s) : s));
}

export function removeStep(steps: Step[], uuid: string): Step[] {
  const filter = (list: Step[]): Step[] =>
    list
      .filter((s) => s.uuid !== uuid)
      .map((s) => ({
        ...s,
        ...(s.body && { body: filter(s.body) }),
        ...(s.otherwise && { otherwise: filter(s.otherwise) }),
        ...(s.cases && { cases: s.cases.map((c) => ({ ...c, body: filter(c.body) })) }),
      }));
  return filter(steps);
}

export function insertStep(
  steps: Step[],
  container: ContainerId,
  index: number,
  step: Step,
): Step[] {
  const list = childrenOf(steps, container);
  if (!list) return steps;
  const next = [...list];
  next.splice(Math.max(0, Math.min(index, next.length)), 0, step);
  return setChildren(steps, container, next);
}

function contains(step: Step, uuid: string): boolean {
  return findStep([step], uuid) !== undefined;
}

export function moveStep(steps: Step[], uuid: string, to: ContainerId, index: number): Step[] {
  const step = findStep(steps, uuid);
  const from = containerOf(steps, uuid);
  if (!step || !from) return steps;
  // a block cannot be dropped inside itself
  if (to !== "root" && contains(step, to.split(":")[0] ?? "")) return steps;
  let target = index;
  if (from === to) {
    const current = (childrenOf(steps, from) ?? []).findIndex((s) => s.uuid === uuid);
    if (current < target) target -= 1;
  }
  return insertStep(removeStep(steps, uuid), to, target, step);
}

// copies get fresh uuids, and references inside the copy follow them
export function cloneStep(step: Step): Step {
  const renames = new Map<string, string>();
  const collect = (s: Step) => {
    renames.set(s.uuid, newUuid());
    [...(s.body ?? []), ...(s.otherwise ?? []), ...(s.cases ?? []).flatMap((c) => c.body)].forEach(
      collect,
    );
  };
  collect(step);
  const json = JSON.stringify(step).replace(
    /"uuid":"([^"]+)"|\{\{output:([^}]+)\}\}/g,
    (m, uuid: string | undefined, ref: string | undefined) => {
      if (uuid) return `"uuid":"${renames.get(uuid) ?? uuid}"`;
      if (ref) return `{{output:${renames.get(ref) ?? ref}}}`;
      return m;
    },
  );
  return JSON.parse(json) as Step;
}

export function duplicateStep(steps: Step[], uuid: string): Step[] {
  const step = findStep(steps, uuid);
  const container = containerOf(steps, uuid);
  if (!step || !container) return steps;
  const index = (childrenOf(steps, container) ?? []).findIndex((s) => s.uuid === uuid);
  return insertStep(steps, container, index + 1, cloneStep(step));
}

export function countSteps(steps: Step[]): number {
  return steps.reduce(
    (n, s) =>
      n +
      1 +
      countSteps(s.body ?? []) +
      countSteps(s.otherwise ?? []) +
      (s.cases ?? []).reduce((m, c) => m + countSteps(c.body), 0),
    0,
  );
}

// steps that run before `uuid`, in order: these are the outputs it can reference
export function stepsBefore(steps: Step[], uuid: string): Step[] {
  const out: Step[] = [];
  const walk = (list: Step[]): boolean => {
    for (const s of list) {
      if (s.uuid === uuid) return true;
      if (
        walk(s.body ?? []) ||
        walk(s.otherwise ?? []) ||
        (s.cases ?? []).some((c) => walk(c.body))
      ) {
        return true;
      }
      out.push(s);
    }
    return false;
  };
  walk(steps);
  return out;
}

export interface FlatRow {
  step: Step;
  depth: number;
  container: ContainerId;
  index: number;
}

// flattened view used by the editor list and keyboard navigation
export function flatten(steps: Step[], depth = 0, container: ContainerId = "root"): FlatRow[] {
  return steps.flatMap((step, index) => [
    { step, depth, container, index },
    ...flatten(step.body ?? [], depth + 1, `${step.uuid}:body`),
    ...flatten(step.otherwise ?? [], depth + 1, `${step.uuid}:otherwise`),
    ...(step.cases ?? []).flatMap((c, i) => flatten(c.body, depth + 1, `${step.uuid}:case:${i}`)),
  ]);
}
