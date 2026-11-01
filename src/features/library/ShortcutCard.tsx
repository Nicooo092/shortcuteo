import { Pencil } from "lucide-react";
import { motion } from "motion/react";
import { forwardRef, useRef } from "react";
import { useTranslation } from "react-i18next";

import type { Workflow } from "../../bindings/Workflow";
import { ActionTile } from "../../components/ActionTile";
import { useRuns } from "../../store/run";
import { useUi } from "../../store/ui";
import { motion as m } from "../../theme/tokens";
import { countSteps } from "../editor/tree";
import { runWithFeedback } from "../run/feedback";

// single click runs, double click edits: wait a beat so a double click does not also run
const CLICK_DELAY = 220;

// forwardRef: AnimatePresence popLayout measures the card through a ref
export const ShortcutCard = forwardRef<HTMLDivElement, { wf: Workflow }>(function ShortcutCard(
  { wf },
  ref,
) {
  const { t } = useTranslation();
  const edit = useUi((s) => s.edit);
  const run = useRuns((s) => s.runs[wf.id]);
  const timer = useRef<number | undefined>(undefined);
  const count = countSteps(wf.actions);
  const running = run?.status === "running";
  const progress = run ? Math.min(1, run.finished.length / run.total) : 0;

  const click = () => {
    window.clearTimeout(timer.current);
    timer.current = window.setTimeout(() => void runWithFeedback(wf), CLICK_DELAY);
  };
  const doubleClick = () => {
    window.clearTimeout(timer.current);
    edit(wf.id);
  };

  return (
    <motion.div
      ref={ref}
      layout
      layoutId={`card-${wf.id}`}
      className={`card glass glass--regular${running ? " is-running" : ""}`}
      whileHover={{ scale: m.hoverScale, y: m.hoverLift }}
      whileTap={{ scale: m.press }}
      transition={m.spring}
      initial={{ opacity: 0, y: 12 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, scale: 0.9 }}
    >
      <button
        className="card__hit"
        onClick={click}
        onDoubleClick={doubleClick}
        onKeyDown={(e) => {
          if (e.key === "F2" || (e.key === "Enter" && e.ctrlKey)) doubleClick();
        }}
        aria-label={t("library.run", { name: wf.name })}
        title={t("library.hint")}
      />
      <div className="card__top">
        <motion.div layoutId={`tile-${wf.id}`} transition={m.spring}>
          <ActionTile tile={wf.color} icon={wf.icon} size={44} />
        </motion.div>
        <button
          className="icon-button card__edit"
          aria-label={t("editor.details")}
          onClick={doubleClick}
        >
          <Pencil size={15} />
        </button>
      </div>
      <motion.h3 layoutId={`title-${wf.id}`} className="card__name" transition={m.spring}>
        {wf.name}
      </motion.h3>
      <p className="card__desc secondary">{wf.description}</p>
      <div className="card__meta">
        <span className="kbd">{t("library.actions", { count })}</span>
      </div>
      <div className="card__progress" aria-hidden>
        <motion.span
          className="card__progress-bar"
          initial={false}
          animate={{ scaleX: running ? Math.max(progress, 0.06) : 0, opacity: running ? 1 : 0 }}
          transition={m.springSoft}
        />
      </div>
    </motion.div>
  );
});
