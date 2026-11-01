import i18n from "i18next";
import { initReactI18next, useTranslation } from "react-i18next";

import type { Localized } from "../bindings/Localized";
import en from "./en.json";
import fr from "./fr.json";

export type Lang = "fr" | "en";

export function detectLang(): Lang {
  return navigator.language.toLowerCase().startsWith("fr") ? "fr" : "en";
}

void i18n.use(initReactI18next).init({
  resources: { fr: { translation: fr }, en: { translation: en } },
  lng: detectLang(),
  fallbackLng: "en",
  interpolation: { escapeValue: false },
});

// action texts come from rust metadata, not from these json files
export function useLocalized() {
  const { i18n: instance } = useTranslation();
  const lang = instance.language as Lang;
  return (text: Localized) => text[lang] || text.en;
}

export default i18n;
