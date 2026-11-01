import type { RunError } from "../../bindings/RunError";
import type { Workflow } from "../../bindings/Workflow";
import i18n from "../../i18n";
import { useCatalog } from "../../store/catalog";
import { useRuns } from "../../store/run";
import { findStep } from "../editor/tree";
import { preview } from "../editor/values";

// runs a shortcut and reports the outcome as a toast, shared by the library and the editor
export async function runWithFeedback(wf: Workflow) {
  const { start, toast } = useRuns.getState();
  try {
    const value = await start(wf);
    if (value === undefined) return;
    toast({ tone: "success", title: wf.name, body: preview(value, 140) || i18n.t("editor.done") });
  } catch (e) {
    const err = e as RunError;
    if (err.cancelled) return;
    const step = err.step ? findStep(wf.actions, err.step) : undefined;
    const meta = step ? useCatalog.getState().byId[step.action] : undefined;
    const lang = i18n.language as "fr" | "en";
    const stepName = meta ? meta.name[lang] : (err.action ?? "?");
    toast({
      tone: "error",
      title: i18n.t("editor.failed", { step: stepName }),
      body: err.message ?? String(e),
    });
  }
}
