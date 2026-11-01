import { create } from "zustand";

import type { HostRequestEvent } from "../bindings/HostRequestEvent";
import type { RunError } from "../bindings/RunError";
import type { Value } from "../bindings/Value";
import type { Workflow } from "../bindings/Workflow";
import { countSteps, newUuid } from "../features/editor/tree";
import { api } from "../ipc";

export interface RunState {
  runId: string;
  shortcutId: string;
  activeStep: string | null;
  finished: string[];
  failedStep: string | null;
  total: number;
  status: "running" | "done" | "failed" | "cancelled";
}

export interface Toast {
  id: string;
  title: string;
  body: string;
  tone: "success" | "error" | "info";
}

interface RunStore {
  runs: Record<string, RunState>;
  dialogs: HostRequestEvent[];
  toasts: Toast[];
  start(wf: Workflow): Promise<Value | undefined>;
  stop(shortcutId: string): void;
  reply(id: number, answer: unknown): void;
  toast(t: Omit<Toast, "id">): void;
  dismissToast(id: string): void;
  listen(): Promise<() => void>;
}

function shortcutOf(runs: Record<string, RunState>, runId: string) {
  return Object.values(runs).find((r) => r.runId === runId);
}

export const useRuns = create<RunStore>((set, get) => ({
  runs: {},
  dialogs: [],
  toasts: [],

  start: async (wf) => {
    if (get().runs[wf.id]?.status === "running") return undefined;
    const runId = newUuid();
    const state: RunState = {
      runId,
      shortcutId: wf.id,
      activeStep: null,
      finished: [],
      failedStep: null,
      total: Math.max(1, countSteps(wf.actions)),
      status: "running",
    };
    set({ runs: { ...get().runs, [wf.id]: state } });
    const patch = (p: Partial<RunState>) => {
      const current = get().runs[wf.id] ?? state;
      set({ runs: { ...get().runs, [wf.id]: { ...current, ...p, activeStep: null } } });
    };
    try {
      const value = await api.runWorkflow(runId, wf);
      patch({ status: "done" });
      return value;
    } catch (e) {
      const err = e as RunError;
      patch({ status: err.cancelled ? "cancelled" : "failed", failedStep: err.step ?? null });
      throw err;
    }
  },

  stop: (shortcutId) => {
    const run = get().runs[shortcutId];
    if (run?.status === "running") void api.cancelRun(run.runId);
  },

  reply: (id, answer) => {
    set({ dialogs: get().dialogs.filter((d) => d.id !== id) });
    void api.hostReply(id, answer);
  },

  toast: (t) => {
    const id = newUuid();
    set({ toasts: [...get().toasts.slice(-2), { ...t, id }] });
    setTimeout(() => get().dismissToast(id), 4200);
  },

  dismissToast: (id) => set({ toasts: get().toasts.filter((t) => t.id !== id) }),

  listen: async () => {
    const offRun = await api.onRunEvent(({ runId, event }) => {
      const run = shortcutOf(get().runs, runId);
      if (!run) return;
      const next = { ...run };
      if (event.type === "stepStarted") next.activeStep = event.step;
      if (event.type === "stepFinished") {
        next.finished = [...run.finished, event.step];
        if (run.activeStep === event.step) next.activeStep = null;
      }
      if (event.type === "stepFailed") next.failedStep = event.step;
      set({ runs: { ...get().runs, [run.shortcutId]: next } });
    });
    const offHost = await api.onHostRequest((req) => set({ dialogs: [...get().dialogs, req] }));
    return () => {
      offRun();
      offHost();
    };
  },
}));
