import { create } from "zustand";

import type { Workflow } from "../bindings/Workflow";
import { newUuid } from "../features/editor/tree";
import { api } from "../ipc";

interface LibraryState {
  shortcuts: Workflow[];
  loaded: boolean;
  load(): Promise<void>;
  save(wf: Workflow): Promise<void>;
  remove(id: string): Promise<void>;
  create(name: string): Workflow;
  importText(text: string): Promise<Workflow>;
}

const sortByName = (list: Workflow[]) => [...list].sort((a, b) => a.name.localeCompare(b.name));

export const useLibrary = create<LibraryState>((set, get) => ({
  shortcuts: [],
  loaded: false,
  load: async () => set({ shortcuts: sortByName(await api.listShortcuts()), loaded: true }),
  save: async (wf) => {
    // optimistic: the editor saves on every change and must not wait on disk
    set({ shortcuts: sortByName([...get().shortcuts.filter((w) => w.id !== wf.id), wf]) });
    await api.saveShortcut(wf);
  },
  remove: async (id) => {
    set({ shortcuts: get().shortcuts.filter((w) => w.id !== id) });
    await api.deleteShortcut(id);
  },
  create: (name) => ({
    schemaVersion: 1,
    id: newUuid(),
    name,
    description: "",
    icon: "sparkles",
    color: "iris",
    favorite: false,
    actions: [],
  }),
  importText: async (text) => {
    const parsed = await api.parseShortcut(text);
    // imported files get a new id so they never overwrite a local shortcut
    const wf: Workflow = { ...parsed, id: newUuid(), favorite: false };
    await get().save(wf);
    return wf;
  },
}));
