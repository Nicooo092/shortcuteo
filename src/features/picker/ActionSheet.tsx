import { Info, Search, X } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { useEffect, useMemo, useRef, useState, type KeyboardEvent } from "react";
import { useTranslation } from "react-i18next";

import type { ActionMeta } from "../../bindings/ActionMeta";
import type { Category } from "../../bindings/Category";
import { ActionTile } from "../../components/ActionTile";
import { Glass } from "../../components/glass/Glass";
import { useLocalized } from "../../i18n";
import { useCatalog } from "../../store/catalog";
import { motion as m } from "../../theme/tokens";
import { fuzzyFilter } from "./fuzzy";
import "./picker.css";

const ORDER: Category[] = [
  "controlFlow",
  "scripting",
  "variables",
  "lists",
  "text",
  "math",
  "dates",
  "clipboard",
  "web",
  "files",
  "navigation",
  "system",
  "display",
  "audio",
  "capture",
  "device",
  "wallpaper",
  "windows",
];

interface Props {
  onPick(meta: ActionMeta): void;
  onClose(): void;
}

export function ActionSheet({ onPick, onClose }: Props) {
  const { t } = useTranslation();
  const loc = useLocalized();
  const actions = useCatalog((s) => s.actions);
  const [query, setQuery] = useState("");
  const [category, setCategory] = useState<Category | null>(null);
  const [cursor, setCursor] = useState(0);
  const [info, setInfo] = useState<string | null>(null);
  const list = useRef<HTMLDivElement>(null);

  const results = useMemo(() => {
    const pool = category ? actions.filter((a) => a.category === category) : actions;
    const found = fuzzyFilter(
      pool,
      query,
      (a) => [loc(a.name), a.name.en],
      (a) => [loc(a.description)],
    );
    // with a query, relevance wins; without one, group by category
    return query.trim()
      ? found
      : [...found].sort(
          (a, b) =>
            ORDER.indexOf(a.category) - ORDER.indexOf(b.category) ||
            loc(a.name).localeCompare(loc(b.name)),
        );
  }, [actions, category, query, loc]);

  useEffect(() => {
    list.current?.querySelector(`[data-index="${cursor}"]`)?.scrollIntoView({ block: "nearest" });
  }, [cursor]);

  const onKey = (e: KeyboardEvent) => {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setCursor((c) => Math.min(c + 1, results.length - 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setCursor((c) => Math.max(c - 1, 0));
    } else if (e.key === "Enter") {
      const hit = results[cursor];
      if (hit) onPick(hit);
    } else if (e.key === "Escape") {
      onClose();
    } else if (e.key === "Backspace" && !query && category) {
      setCategory(null);
      setCursor(0);
    }
  };

  return (
    <motion.div
      className="sheet-overlay"
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      onClick={onClose}
    >
      <motion.div
        layoutId="add-sheet"
        className="sheet-wrap"
        transition={m.spring}
        onClick={(e) => e.stopPropagation()}
      >
        <Glass
          variant="thick"
          className="sheet"
          role="dialog"
          aria-modal
          aria-label={t("picker.title")}
        >
          <div className="sheet__head">
            <Glass variant="thin" refract className="search sheet__search">
              <Search size={15} className="search__icon" aria-hidden />
              {category && (
                <button
                  className="chip"
                  aria-label={t("picker.removeFilter", { name: t(`categories.${category}`) })}
                  onClick={() => {
                    setCategory(null);
                    setCursor(0);
                  }}
                >
                  {t(`categories.${category}`)}
                  <X size={12} />
                </button>
              )}
              <input
                autoFocus
                className="search__input"
                placeholder={t("picker.search")}
                aria-label={t("picker.search")}
                role="combobox"
                aria-expanded
                aria-controls="sheet-list"
                aria-activedescendant={results[cursor] ? `opt-${results[cursor].id}` : undefined}
                value={query}
                onChange={(e) => {
                  setQuery(e.target.value);
                  setCursor(0);
                }}
                onKeyDown={onKey}
              />
            </Glass>
            <button className="icon-button" aria-label={t("picker.close")} onClick={onClose}>
              <X size={18} />
            </button>
          </div>
          <div className="sheet__list" id="sheet-list" role="listbox" ref={list}>
            {results.length === 0 && (
              <p className="secondary sheet__none">{t("picker.noResults")}</p>
            )}
            {results.map((a, i) => {
              const header = !query.trim() && a.category !== results[i - 1]?.category;
              return (
                <div key={a.id}>
                  {header && (
                    <button
                      className="sheet__section"
                      onClick={() => {
                        setCategory(a.category);
                        setCursor(0);
                      }}
                    >
                      {t(`categories.${a.category}`)}
                    </button>
                  )}
                  <div
                    id={`opt-${a.id}`}
                    data-index={i}
                    role="option"
                    aria-selected={i === cursor}
                    className={`sheet__row${i === cursor ? " is-cursor" : ""}`}
                    onMouseMove={() => setCursor(i)}
                    onClick={() => onPick(a)}
                  >
                    <ActionTile tile={a.tile} icon={a.icon} size={30} />
                    <span className="sheet__name">{loc(a.name)}</span>
                    <button
                      className="icon-button"
                      aria-label={t("picker.info", { name: loc(a.name) })}
                      aria-expanded={info === a.id}
                      onClick={(e) => {
                        e.stopPropagation();
                        setInfo((x) => (x === a.id ? null : a.id));
                      }}
                    >
                      <Info size={16} />
                    </button>
                  </div>
                  <AnimatePresence initial={false}>
                    {info === a.id && (
                      <motion.div
                        className="sheet__info"
                        initial={{ opacity: 0, y: -4 }}
                        animate={{ opacity: 1, y: 0 }}
                        exit={{ opacity: 0 }}
                      >
                        <p>{loc(a.description)}</p>
                        {a.params.length > 0 && (
                          <p className="secondary">
                            {t("picker.inputs")} : {a.params.map((p) => loc(p.label)).join(", ")}
                          </p>
                        )}
                        <p className="secondary">
                          {t("picker.outputs")} : {t(`types.${a.output}`)}
                        </p>
                        {a.sensitive && <p className="secondary">{t("picker.sensitive")}</p>}
                      </motion.div>
                    )}
                  </AnimatePresence>
                </div>
              );
            })}
          </div>
        </Glass>
      </motion.div>
    </motion.div>
  );
}
