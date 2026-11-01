import catalog from "../bindings/catalog.json";
import type { ActionMeta } from "../bindings/ActionMeta";
import type { HostRequest } from "../bindings/HostRequest";
import type { HostRequestEvent } from "../bindings/HostRequestEvent";
import type { RunEventPayload } from "../bindings/RunEventPayload";
import type { Step } from "../bindings/Step";
import type { Value } from "../bindings/Value";
import type { Workflow } from "../bindings/Workflow";
import { gallery } from "../features/gallery/examples";
import type { Api } from "./index";

// browser-only stand-in for the rust side: real catalog, fake engine. enough to work on the ui
// with `npm run dev` without building tauri. it does not try to be a second interpreter.

const STORE = "shortcuteo.mock.library";
const runListeners = new Set<(e: RunEventPayload) => void>();
const hostListeners = new Set<(e: HostRequestEvent) => void>();
const pending = new Map<number, (answer: unknown) => void>();
const cancelled = new Set<string>();
let nextId = 1;

function readLibrary(): Workflow[] {
  try {
    const raw = localStorage.getItem(STORE);
    if (raw) return JSON.parse(raw) as Workflow[];
  } catch {
    // corrupt storage, start over
  }
  const seeded = gallery.slice(0, 3);
  writeLibrary(seeded);
  return seeded;
}

function writeLibrary(list: Workflow[]) {
  try {
    localStorage.setItem(STORE, JSON.stringify(list));
  } catch {
    // private mode, keep it in memory only
  }
}

const text = (s: string): Value => ({ type: "text", value: s });
const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

function ask(runId: string, request: HostRequest): Promise<unknown> {
  const id = nextId++;
  return new Promise((resolve) => {
    pending.set(id, resolve);
    hostListeners.forEach((cb) => cb({ id, runId, request }));
  });
}

class Stop {
  constructor(readonly value: Value) {}
}

async function runSteps(runId: string, steps: Step[], input: Value): Promise<Value> {
  let last = input;
  for (const step of steps) {
    if (step.disabled) continue;
    if (cancelled.has(runId)) {
      throw { step: step.uuid, action: step.action, message: "cancelled", cancelled: true };
    }
    runListeners.forEach((cb) => cb({ runId, event: { type: "stepStarted", step: step.uuid } }));
    await sleep(380);
    const p = (step.params ?? {}) as Record<string, unknown>;
    const str = (k: string) => (typeof p[k] === "string" ? (p[k] as string) : "");
    switch (step.action) {
      case "text.text":
        last = text(str("text"));
        break;
      case "scripting.show_alert":
        if (!(await ask(runId, { kind: "alert", title: str("title"), message: str("message") })))
          throw new Stop({ type: "nothing" });
        break;
      case "scripting.ask_input": {
        const a = await ask(runId, { kind: "ask", prompt: str("prompt"), default: str("default") });
        if (typeof a !== "string") throw new Stop({ type: "nothing" });
        last = text(a);
        break;
      }
      case "scripting.show_result":
        await ask(runId, { kind: "show", value: last });
        break;
      case "control.menu": {
        const options = (step.cases ?? []).map((c) => c.label);
        const a = await ask(runId, {
          kind: "choose",
          prompt: str("prompt"),
          options,
          multiple: false,
        });
        const i = Array.isArray(a) ? (a[0] as number) : -1;
        const chosen = step.cases?.[i];
        if (!chosen) throw new Stop({ type: "nothing" });
        last = await runSteps(runId, chosen.body, last);
        break;
      }
      case "control.if":
        last = await runSteps(runId, step.body ?? [], last);
        break;
      case "control.repeat":
      case "control.repeat_each":
        last = await runSteps(runId, step.body ?? [], last);
        break;
      case "scripting.output":
        throw new Stop(p.value === undefined ? last : text(str("value")));
      case "scripting.stop":
        throw new Stop({ type: "nothing" });
      default:
        break;
    }
    runListeners.forEach((cb) =>
      cb({ runId, event: { type: "stepFinished", step: step.uuid, output: last } }),
    );
  }
  return last;
}

export const mockApi: Api = {
  listActions: async () => catalog as unknown as ActionMeta[],
  runWorkflow: async (runId, workflow, input) => {
    try {
      return await runSteps(runId, workflow.actions, input ?? { type: "nothing" });
    } catch (e) {
      if (e instanceof Stop) return e.value;
      throw e;
    } finally {
      cancelled.delete(runId);
    }
  },
  cancelRun: async (runId) => {
    cancelled.add(runId);
  },
  hostReply: async (id, answer) => {
    pending.get(id)?.(answer);
    pending.delete(id);
  },
  listShortcuts: async () => readLibrary(),
  saveShortcut: async (wf) => {
    const list = readLibrary().filter((w) => w.id !== wf.id);
    writeLibrary([...list, wf]);
  },
  deleteShortcut: async (id) => writeLibrary(readLibrary().filter((w) => w.id !== id)),
  parseShortcut: async (raw) => {
    const wf = JSON.parse(raw) as Workflow;
    if (typeof wf.schemaVersion !== "number") throw new Error("missing schemaVersion");
    return wf;
  },
  exportShortcut: async (wf) => JSON.stringify(wf, null, 2),
  onBattery: async () => false,
  readTextFile: async () => {
    throw new Error("no file access in the browser");
  },
  onRunEvent: async (cb) => {
    runListeners.add(cb);
    return () => runListeners.delete(cb);
  },
  onHostRequest: async (cb) => {
    hostListeners.add(cb);
    return () => hostListeners.delete(cb);
  },
};
