import { AnimatePresence, motion } from "motion/react";

import { useRuns } from "../store/run";
import { motion as m } from "../theme/tokens";
import { Glass } from "./glass/Glass";

const MotionGlass = motion.create(Glass);

export function Toasts() {
  const toasts = useRuns((s) => s.toasts);
  const dismiss = useRuns((s) => s.dismissToast);

  return (
    <div className="toasts" role="status" aria-live="polite">
      <AnimatePresence initial={false}>
        {toasts.map((t) => (
          <MotionGlass
            key={t.id}
            layout
            variant="thick"
            refract
            className={`toast toast--${t.tone}`}
            initial={{ opacity: 0, y: 40, scale: 0.92 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: 16, scale: 0.96 }}
            transition={m.spring}
            onClick={() => dismiss(t.id)}
          >
            <span className="toast__dot" aria-hidden />
            <div>
              <div className="toast__title">{t.title}</div>
              {t.body && <div className="toast__body secondary">{t.body}</div>}
            </div>
          </MotionGlass>
        ))}
      </AnimatePresence>
    </div>
  );
}
