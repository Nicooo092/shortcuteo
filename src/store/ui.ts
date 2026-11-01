import { create } from "zustand";

export type Screen = "library" | "favorites" | "gallery" | "automations" | "settings" | "editor";

interface UiState {
  screen: Screen;
  folder: string | null;
  editingId: string | null;
  pickerOpen: boolean;
  paletteOpen: boolean;
  go(screen: Screen, folder?: string | null): void;
  edit(id: string): void;
  setPicker(open: boolean): void;
  setPalette(open: boolean): void;
}

export const useUi = create<UiState>((set) => ({
  screen: "library",
  folder: null,
  editingId: null,
  pickerOpen: false,
  paletteOpen: false,
  go: (screen, folder = null) => set({ screen, folder, pickerOpen: false }),
  edit: (id) => set({ screen: "editor", editingId: id, pickerOpen: false, paletteOpen: false }),
  setPicker: (pickerOpen) => set({ pickerOpen }),
  setPalette: (paletteOpen) => set({ paletteOpen }),
}));
