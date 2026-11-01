import { create } from "zustand";

import { color } from "../theme/tokens";
import { detectLang, type Lang } from "../i18n";

export interface Settings {
  lang: Lang;
  accent: string;
  blur: number;
  batteryBlur: boolean;
  animations: boolean;
}

interface SettingsState extends Settings {
  set(patch: Partial<Settings>): void;
}

const KEY = "shortcuteo.settings";

function read(): Partial<Settings> {
  try {
    return JSON.parse(localStorage.getItem(KEY) ?? "{}") as Partial<Settings>;
  } catch {
    return {};
  }
}

const defaults: Settings = {
  lang: detectLang(),
  accent: color.accent,
  blur: 1,
  batteryBlur: true,
  animations: true,
};

export const useSettings = create<SettingsState>((set, get) => ({
  ...defaults,
  ...read(),
  set: (patch) => {
    set(patch);
    const { lang, accent, blur, batteryBlur, animations } = get();
    const values: Settings = { lang, accent, blur, batteryBlur, animations };
    try {
      localStorage.setItem(KEY, JSON.stringify(values));
    } catch {
      // storage can be unavailable, settings then last for the session only
    }
  },
}));
