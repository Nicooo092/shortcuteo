import { create } from "zustand";

import type { ActionMeta } from "../bindings/ActionMeta";
import { api } from "../ipc";

interface CatalogState {
  actions: ActionMeta[];
  byId: Record<string, ActionMeta>;
  load(): Promise<void>;
}

export const useCatalog = create<CatalogState>((set) => ({
  actions: [],
  byId: {},
  load: async () => {
    const actions = await api.listActions();
    set({ actions, byId: Object.fromEntries(actions.map((a) => [a.id, a])) });
  },
}));
