import { Plus } from "lucide-react";
import { motion } from "motion/react";
import { useTranslation } from "react-i18next";

import { ActionTile } from "../../components/ActionTile";
import { useLibrary } from "../../store/library";
import { useRuns } from "../../store/run";
import { motion as m } from "../../theme/tokens";
import { countSteps, newUuid } from "../editor/tree";
import { gallery } from "./examples";

export function GalleryScreen() {
  const { t } = useTranslation();
  const save = useLibrary((s) => s.save);
  const toast = useRuns((s) => s.toast);

  const add = async (index: number) => {
    const wf = gallery[index];
    if (!wf) return;
    await save({ ...wf, id: newUuid() });
    toast({ tone: "success", title: t("gallery.added"), body: wf.name });
  };

  return (
    <section className="screen">
      <div className="screen__head">
        <h1 className="screen__title">{t("gallery.title")}</h1>
      </div>
      <p className="secondary gallery__subtitle">{t("gallery.subtitle")}</p>
      <div className="grid">
        {gallery.map((wf, i) => (
          <motion.div
            key={wf.id}
            className="card glass glass--regular"
            whileHover={{ scale: m.hoverScale, y: m.hoverLift }}
            transition={m.spring}
          >
            <ActionTile tile={wf.color} icon={wf.icon} size={44} />
            <h3 className="card__name">{wf.name}</h3>
            <p className="card__desc secondary">{wf.description}</p>
            <div className="card__meta">
              <span className="kbd">{t("library.actions", { count: countSteps(wf.actions) })}</span>
            </div>
            <motion.button
              className="pill-button primary gallery-card__add"
              whileTap={{ scale: m.press }}
              onClick={() => void add(i)}
            >
              <Plus size={15} />
              {t("gallery.add")}
            </motion.button>
          </motion.div>
        ))}
      </div>
    </section>
  );
}
