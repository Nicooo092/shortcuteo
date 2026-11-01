import { Folder, LayoutGrid, Settings, Sparkles, Star, Zap } from "lucide-react";
import { motion } from "motion/react";
import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";

import { Glass } from "../../components/glass/Glass";
import { useLibrary } from "../../store/library";
import { useUi, type Screen } from "../../store/ui";
import { motion as m } from "../../theme/tokens";

const FOLDER_TILES = ["iris", "ember", "lagoon", "amber", "orchid", "glacier"] as const;

export function Sidebar() {
  const { t } = useTranslation();
  const { screen, folder, go } = useUi();
  const shortcuts = useLibrary((s) => s.shortcuts);
  const folders = [
    ...new Set(shortcuts.map((s) => s.folder).filter((f): f is string => !!f)),
  ].sort();

  const item = (
    target: Screen,
    label: string,
    icon: ReactNode,
    count?: number,
    f: string | null = null,
  ) => {
    const active = screen === target && folder === f;
    return (
      <button
        key={`${target}-${f ?? ""}`}
        className={`side__item${active ? " is-active" : ""}`}
        aria-current={active ? "page" : undefined}
        onClick={() => go(target, f)}
      >
        {active && (
          <motion.span layoutId="side-active" className="side__active" transition={m.spring} />
        )}
        <span className="side__icon">{icon}</span>
        <span className="side__label">{label}</span>
        {count !== undefined && <span className="side__count">{count}</span>}
      </button>
    );
  };

  return (
    <Glass variant="regular" className="side" role="navigation">
      <div className="side__group">
        {item("library", t("nav.all"), <LayoutGrid size={17} />, shortcuts.length)}
        {item(
          "favorites",
          t("nav.favorites"),
          <Star size={17} />,
          shortcuts.filter((s) => s.favorite).length,
        )}
        {item("automations", t("nav.automations"), <Zap size={17} />)}
        {item("gallery", t("nav.gallery"), <Sparkles size={17} />)}
      </div>
      {folders.length > 0 && (
        <div className="side__group">
          <div className="side__heading">{t("nav.folders")}</div>
          {folders.map((f, i) =>
            item(
              "library",
              f,
              <Folder
                size={17}
                className={`side__folder side__folder--${FOLDER_TILES[i % FOLDER_TILES.length]}`}
              />,
              shortcuts.filter((s) => s.folder === f).length,
              f,
            ),
          )}
        </div>
      )}
      <div className="side__spacer" />
      <div className="side__group">
        {item("settings", t("nav.settings"), <Settings size={17} />)}
      </div>
    </Glass>
  );
}
