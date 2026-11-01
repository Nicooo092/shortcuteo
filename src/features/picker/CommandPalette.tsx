import { Command } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { useEffect, useMemo, useState, type KeyboardEvent } from "react";
import { useTranslation } from "react-i18next";

import { ActionTile } from "../../components/ActionTile";
import { Glass } from "../../components/glass/Glass";
import { useLibrary } from "../../store/library";
import { useUi, type Screen } from "../../store/ui";
import { motion as m } from "../../theme/tokens";
import { runWithFeedback } from "../run/feedback";
import { fuzzyFilter } from "./fuzzy";
import "./picker.css";

interface Item {
  id: string;
  label: string;
  hint?: string;
  tile: Parameters<typeof ActionTile>[0]["tile"];
  icon: string;
  open(): void;
  alt?(): void;
}

// ctrl+k: jump to a shortcut (enter edits, ctrl+enter runs) or a screen
export function CommandPalette() {
  const { paletteOpen, setPalette } = useUi();

  useEffect(() => {
    const onKey = (e: globalThis.KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setPalette(!useUi.getState().paletteOpen);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [setPalette]);

  return <AnimatePresence>{paletteOpen && <PaletteBody />}</AnimatePresence>;
}

// mounted only while open, so the query starts empty each time
function PaletteBody() {
  const { t } = useTranslation();
  const { setPalette, edit, go } = useUi();
  const shortcuts = useLibrary((s) => s.shortcuts);
  const [query, setQuery] = useState("");
  const [cursor, setCursor] = useState(0);

  const items = useMemo<Item[]>(() => {
    const screens: [Screen, string, string][] = [
      ["library", t("nav.all"), "layout-grid"],
      ["favorites", t("nav.favorites"), "star"],
      ["automations", t("nav.automations"), "zap"],
      ["gallery", t("nav.gallery"), "sparkles"],
      ["settings", t("nav.settings"), "settings"],
    ];
    const all: Item[] = [
      ...shortcuts.map((wf) => ({
        id: wf.id,
        label: wf.name,
        hint: wf.description,
        tile: wf.color,
        icon: wf.icon,
        open: () => edit(wf.id),
        alt: () => {
          setPalette(false);
          void runWithFeedback(wf);
        },
      })),
      ...screens.map(([screen, label, icon]) => ({
        id: `screen-${screen}`,
        label,
        tile: "slate" as const,
        icon,
        open: () => {
          go(screen);
          setPalette(false);
        },
      })),
    ];
    return fuzzyFilter(
      all,
      query,
      (i) => [i.label],
      (i) => [i.hint ?? ""],
    ).slice(0, 12);
  }, [shortcuts, query, t, edit, go, setPalette]);

  const onKey = (e: KeyboardEvent) => {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setCursor((c) => Math.min(c + 1, items.length - 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setCursor((c) => Math.max(c - 1, 0));
    } else if (e.key === "Enter") {
      const it = items[cursor];
      if (!it) return;
      if (e.ctrlKey && it.alt) it.alt();
      else it.open();
    } else if (e.key === "Escape") {
      setPalette(false);
    }
  };

  return (
    <motion.div
      className="sheet-overlay palette-overlay"
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      onClick={() => setPalette(false)}
    >
      <motion.div
        className="palette-wrap"
        initial={{ opacity: 0, y: -16, scale: 0.97 }}
        animate={{ opacity: 1, y: 0, scale: 1 }}
        exit={{ opacity: 0, y: -8, scale: 0.98 }}
        transition={m.spring}
        onClick={(e) => e.stopPropagation()}
      >
        <Glass
          variant="thick"
          className="palette"
          role="dialog"
          aria-modal
          aria-label={t("picker.palette")}
        >
          <Glass variant="thin" refract className="search palette__search">
            <Command size={15} className="search__icon" aria-hidden />
            <input
              autoFocus
              className="search__input"
              placeholder={t("picker.palette")}
              aria-label={t("picker.palette")}
              role="combobox"
              aria-expanded
              aria-controls="palette-list"
              value={query}
              onChange={(e) => {
                setQuery(e.target.value);
                setCursor(0);
              }}
              onKeyDown={onKey}
            />
          </Glass>
          <div className="sheet__list" id="palette-list" role="listbox">
            {items.length === 0 && <p className="secondary sheet__none">{t("picker.noResults")}</p>}
            {items.map((it, i) => (
              <div
                key={it.id}
                role="option"
                aria-selected={i === cursor}
                className={`sheet__row${i === cursor ? " is-cursor" : ""}`}
                onMouseMove={() => setCursor(i)}
                onClick={() => it.open()}
              >
                <ActionTile tile={it.tile} icon={it.icon} size={28} />
                <span className="sheet__name">{it.label}</span>
                {it.alt && <span className="kbd">Ctrl Enter</span>}
              </div>
            ))}
          </div>
        </Glass>
      </motion.div>
    </motion.div>
  );
}
