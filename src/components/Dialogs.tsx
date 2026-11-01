import { AnimatePresence, motion } from "motion/react";
import { useEffect, useRef, useState, type FormEvent } from "react";
import { useTranslation } from "react-i18next";

import type { HostRequestEvent } from "../bindings/HostRequestEvent";
import { valueToText } from "../features/editor/values";
import { useRuns } from "../store/run";
import { motion as m } from "../theme/tokens";
import { Glass } from "./glass/Glass";

const MotionGlass = motion.create(Glass);

// dialogs the running shortcut asks for (alert, ask, menu, show), answered back to rust
export function Dialogs() {
  const dialogs = useRuns((s) => s.dialogs);
  const current = dialogs[0];
  return (
    <AnimatePresence>
      {current && (
        <motion.div
          key={current.id}
          className="overlay"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
        >
          <MotionGlass
            variant="thick"
            className="dialog"
            role="dialog"
            aria-modal
            initial={{ opacity: 0, scale: 0.9, y: 20 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.95 }}
            transition={m.spring}
          >
            <DialogBody req={current} />
          </MotionGlass>
        </motion.div>
      )}
    </AnimatePresence>
  );
}

function DialogBody({ req }: { req: HostRequestEvent }) {
  const { t } = useTranslation();
  const reply = useRuns((s) => s.reply);
  const r = req.request;
  const answer = (value: unknown) => reply(req.id, value);
  const [text, setText] = useState(r.kind === "ask" ? r.default : "");
  const first = useRef<HTMLButtonElement & HTMLInputElement>(null);

  useEffect(() => first.current?.focus(), []);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") answer(r.kind === "show" ? true : null);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });

  if (r.kind === "alert") {
    return (
      <>
        {r.title && <h2 className="dialog__title">{r.title}</h2>}
        <p style={{ margin: 0, whiteSpace: "pre-wrap" }}>{r.message}</p>
        <div className="dialog__actions">
          <button className="pill-button" onClick={() => answer(false)}>
            {t("dialog.cancel")}
          </button>
          <button ref={first} className="pill-button primary" onClick={() => answer(true)}>
            {t("dialog.ok")}
          </button>
        </div>
      </>
    );
  }

  if (r.kind === "ask") {
    const submit = (e: FormEvent) => {
      e.preventDefault();
      answer(text);
    };
    return (
      <form onSubmit={submit} style={{ display: "contents" }}>
        <h2 className="dialog__title">{r.prompt || " "}</h2>
        <input
          ref={first}
          className="field"
          value={text}
          onChange={(e) => setText(e.target.value)}
        />
        <div className="dialog__actions">
          <button type="button" className="pill-button" onClick={() => answer(null)}>
            {t("dialog.cancel")}
          </button>
          <button type="submit" className="pill-button primary">
            {t("dialog.ok")}
          </button>
        </div>
      </form>
    );
  }

  if (r.kind === "choose") {
    return (
      <>
        {r.prompt && <h2 className="dialog__title">{r.prompt}</h2>}
        <div className="dialog__options">
          {r.options.map((o, i) => (
            <button
              key={i}
              ref={i === 0 ? first : undefined}
              className="dialog__option"
              onClick={() => answer([i])}
            >
              {o}
            </button>
          ))}
        </div>
        <div className="dialog__actions">
          <button className="pill-button" onClick={() => answer(null)}>
            {t("dialog.cancel")}
          </button>
        </div>
      </>
    );
  }

  return (
    <>
      <h2 className="dialog__title">{t("editor.result")}</h2>
      <div className="dialog__content">{valueToText(r.value)}</div>
      <div className="dialog__actions">
        <button ref={first} className="pill-button primary" onClick={() => answer(true)}>
          {t("dialog.done")}
        </button>
      </div>
    </>
  );
}
