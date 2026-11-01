import { AnimatePresence, LayoutGroup, motion, MotionConfig } from "motion/react";
import { lazy, Suspense, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import { Dialogs } from "./components/Dialogs";
import { Ambient } from "./components/glass/Ambient";
import { RefractionFilter } from "./components/glass/RefractionFilter";
import { TitleBar } from "./components/TitleBar";
import { Toasts } from "./components/Toasts";
import { GalleryScreen } from "./features/gallery/GalleryScreen";
import { LibraryScreen } from "./features/library/LibraryScreen";
import { Sidebar } from "./features/library/Sidebar";
import { CommandPalette } from "./features/picker/CommandPalette";
import i18n from "./i18n";
import { api, isNative } from "./ipc";
import { useCatalog } from "./store/catalog";
import { useLibrary } from "./store/library";
import { useRuns } from "./store/run";
import { useSettings } from "./store/settings";
import { useUi } from "./store/ui";
import { applyTokens } from "./theme/tokens";
import "./components/glass/glass.css";

const EditorScreen = lazy(() => import("./features/editor/EditorScreen"));
const AutomationsScreen = lazy(() => import("./features/automations/AutomationsScreen"));
const SettingsScreen = lazy(() => import("./features/settings/SettingsScreen"));

const BATTERY_BLUR = 0.5;

export function App() {
  const { t } = useTranslation();
  const screen = useUi((s) => s.screen);
  const editingId = useUi((s) => s.editingId);
  const { accent, blur, batteryBlur, animations } = useSettings();
  const [onBattery, setOnBattery] = useState(false);

  useEffect(() => {
    document.documentElement.classList.toggle("native", isNative);
    void useCatalog.getState().load();
    void useLibrary.getState().load();
    const off = useRuns.getState().listen();
    const check = () => void api.onBattery().then(setOnBattery);
    check();
    // checked on focus rather than polled: cheap and good enough for blur level
    window.addEventListener("focus", check);
    return () => {
      void off.then((f) => f());
      window.removeEventListener("focus", check);
    };
  }, []);

  useEffect(() => {
    applyTokens(document.documentElement, {
      accent,
      blurScale: blur * (batteryBlur && onBattery ? BATTERY_BLUR : 1),
    });
  }, [accent, blur, batteryBlur, onBattery]);

  useEffect(() => {
    void i18n.changeLanguage(useSettings.getState().lang);
  }, []);

  useEffect(() => {
    document.documentElement.classList.toggle("reduce-motion", !animations);
  }, [animations]);

  useDropImport(t("library.imported"), t("library.importFailed"));

  return (
    <MotionConfig reducedMotion={animations ? "user" : "always"}>
      <RefractionFilter />
      <Ambient animate={animations} />
      <div className="app">
        <TitleBar />
        <LayoutGroup>
          <div className="app-body">
            <AnimatePresence initial={false}>
              {screen !== "editor" && (
                <motion.div
                  key="sidebar"
                  style={{ display: "flex" }}
                  initial={{ opacity: 0, x: -20 }}
                  animate={{ opacity: 1, x: 0 }}
                  exit={{ opacity: 0, x: -20 }}
                >
                  <Sidebar />
                </motion.div>
              )}
            </AnimatePresence>
            <Suspense fallback={<div className="screen" />}>
              {screen === "library" && <LibraryScreen />}
              {screen === "favorites" && <LibraryScreen favorites />}
              {screen === "gallery" && <GalleryScreen />}
              {screen === "automations" && <AutomationsScreen />}
              {screen === "settings" && <SettingsScreen />}
              {screen === "editor" && <EditorScreen key={editingId ?? ""} />}
            </Suspense>
          </div>
        </LayoutGroup>
      </div>
      <CommandPalette />
      <Dialogs />
      <Toasts />
    </MotionConfig>
  );
}

// dropping a .shortcuteo file anywhere imports it; tauri hands us paths, the browser hands us files
function useDropImport(okTitle: string, failTitle: string) {
  useEffect(() => {
    const { toast } = useRuns.getState();
    const importText = async (text: string) => {
      try {
        const wf = await useLibrary.getState().importText(text);
        toast({ tone: "success", title: okTitle, body: wf.name });
      } catch (e) {
        toast({ tone: "error", title: failTitle, body: String(e) });
      }
    };

    if (isNative) {
      let off: (() => void) | undefined;
      void import("@tauri-apps/api/webview").then(async ({ getCurrentWebview }) => {
        off = await getCurrentWebview().onDragDropEvent(async (e) => {
          if (e.payload.type !== "drop") return;
          for (const path of e.payload.paths.filter((p) => p.endsWith(".shortcuteo"))) {
            try {
              await importText(await api.readTextFile(path));
            } catch (err) {
              toast({ tone: "error", title: failTitle, body: String(err) });
            }
          }
        });
      });
      return () => off?.();
    }

    const over = (e: DragEvent) => e.preventDefault();
    const drop = (e: DragEvent) => {
      e.preventDefault();
      for (const file of Array.from(e.dataTransfer?.files ?? [])) {
        if (file.name.endsWith(".shortcuteo")) void file.text().then(importText);
      }
    };
    window.addEventListener("dragover", over);
    window.addEventListener("drop", drop);
    return () => {
      window.removeEventListener("dragover", over);
      window.removeEventListener("drop", drop);
    };
  }, [okTitle, failTitle]);
}
