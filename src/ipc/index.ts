import type { ActionMeta } from "../bindings/ActionMeta";
import type { HostRequestEvent } from "../bindings/HostRequestEvent";
import type { RunEventPayload } from "../bindings/RunEventPayload";
import type { Value } from "../bindings/Value";
import type { Workflow } from "../bindings/Workflow";

export type Unlisten = () => void;

export interface Api {
  listActions(): Promise<ActionMeta[]>;
  runWorkflow(runId: string, workflow: Workflow, input?: Value): Promise<Value>;
  cancelRun(runId: string): Promise<void>;
  hostReply(id: number, answer: unknown): Promise<void>;
  listShortcuts(): Promise<Workflow[]>;
  saveShortcut(workflow: Workflow): Promise<void>;
  deleteShortcut(id: string): Promise<void>;
  parseShortcut(text: string): Promise<Workflow>;
  exportShortcut(workflow: Workflow): Promise<string>;
  onBattery(): Promise<boolean>;
  readTextFile(path: string): Promise<string>;
  onRunEvent(cb: (e: RunEventPayload) => void): Promise<Unlisten>;
  onHostRequest(cb: (e: HostRequestEvent) => void): Promise<Unlisten>;
}

export const isNative = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

let loaded: Promise<Api> | undefined;

// the mock and its catalog snapshot stay out of the native bundle
function load(): Promise<Api> {
  loaded ??= isNative
    ? import("./native").then((m) => m.nativeApi)
    : import("./mock").then((m) => m.mockApi);
  return loaded;
}

export const api: Api = new Proxy({} as Api, {
  get:
    (_, key: keyof Api) =>
    async (...args: unknown[]) => {
      const impl = await load();
      return (impl[key] as (...a: unknown[]) => unknown)(...args);
    },
});
