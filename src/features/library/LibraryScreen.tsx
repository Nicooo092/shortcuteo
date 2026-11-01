import { Plus, Search } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import { Glass } from "../../components/glass/Glass";
import { useLibrary } from "../../store/library";
import { useUi } from "../../store/ui";
import { motion as m } from "../../theme/tokens";
import { fuzzyFilter } from "../picker/fuzzy";
import { ShortcutCard } from "./ShortcutCard";
import "./library.css";

export function LibraryScreen({ favorites = false }: { favorites?: boolean }) {
  const { t } = useTranslation();
  const { shortcuts, create, save } = useLibrary();
  const { folder, edit, go } = useUi();
  const [query, setQuery] = useState("");

  const visible = useMemo(() => {
    const base = shortcuts.filter((s) =>
      favorites ? s.favorite : folder ? s.folder === folder : true,
    );
    return fuzzyFilter(
      base,
      query,
      (s) => [s.name],
      (s) => [s.description],
    );
  }, [shortcuts, favorites, folder, query]);

  const add = async () => {
    const wf = create(t("editor.untitled"));
    if (folder) wf.folder = folder;
    await save(wf);
    edit(wf.id);
  };

  const title = favorites ? t("library.favoritesTitle") : (folder ?? t("library.title"));

  return (
    <section className="screen library">
      <div className="screen__head">
        <h1 className="screen__title">{title}</h1>
        <Glass variant="thin" refract className="search">
          <Search size={15} className="search__icon" aria-hidden />
          <input
            className="search__input"
            placeholder={t("library.search")}
            aria-label={t("library.search")}
            value={query}
            onChange={(e) => setQuery(e.target.value)}
          />
          <span className="kbd">Ctrl K</span>
        </Glass>
        <motion.button
          className="pill-button primary glass--refract"
          onClick={() => void add()}
          whileTap={{ scale: m.press }}
          transition={m.spring}
        >
          <Plus size={16} />
          {t("library.new")}
        </motion.button>
      </div>

      {visible.length === 0 ? (
        <div className="empty">
          <p>{favorites ? t("library.emptyFavorites") : t("library.empty")}</p>
          {!favorites && (
            <button className="pill-button" onClick={() => go("gallery")}>
              {t("library.emptyHint")}
            </button>
          )}
        </div>
      ) : (
        <motion.div className="grid" layout transition={m.spring}>
          <AnimatePresence mode="popLayout">
            {visible.map((wf) => (
              <ShortcutCard key={wf.id} wf={wf} />
            ))}
          </AnimatePresence>
        </motion.div>
      )}
    </section>
  );
}
