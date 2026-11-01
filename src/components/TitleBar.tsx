import { Copy, Minus, Square, X } from "lucide-react";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import { isNative } from "../ipc";
import "./chrome.css";

async function win() {
  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  return getCurrentWindow();
}

export function TitleBar() {
  const { t } = useTranslation();
  const [maximized, setMaximized] = useState(false);

  useEffect(() => {
    if (!isNative) return;
    let off: (() => void) | undefined;
    void win().then(async (w) => {
      setMaximized(await w.isMaximized());
      off = await w.onResized(async () => setMaximized(await w.isMaximized()));
    });
    return () => off?.();
  }, []);

  const act = (fn: (w: Awaited<ReturnType<typeof win>>) => Promise<void>) => () => {
    if (isNative) void win().then(fn);
  };

  return (
    <header className="titlebar" data-tauri-drag-region>
      <div className="titlebar__brand" data-tauri-drag-region>
        <span className="titlebar__logo" aria-hidden />
        <span data-tauri-drag-region>Shortcuteo</span>
      </div>
      <div className="titlebar__controls">
        <button
          className="titlebar__btn"
          aria-label={t("app.minimize")}
          onClick={act((w) => w.minimize())}
        >
          <Minus size={16} strokeWidth={1.6} />
        </button>
        <button
          className="titlebar__btn"
          aria-label={t("app.maximize")}
          onClick={act((w) => w.toggleMaximize())}
        >
          {maximized ? (
            <Copy size={13} strokeWidth={1.6} />
          ) : (
            <Square size={12} strokeWidth={1.6} />
          )}
        </button>
        <button
          className="titlebar__btn titlebar__btn--close"
          aria-label={t("app.close")}
          onClick={act((w) => w.close())}
        >
          <X size={16} strokeWidth={1.6} />
        </button>
      </div>
    </header>
  );
}
