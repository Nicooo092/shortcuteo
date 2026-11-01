import { ArrowRight, Plus, Star, Trash2, X } from "lucide-react";
import { useTranslation } from "react-i18next";

import type { ActionMeta } from "../../bindings/ActionMeta";
import type { Step } from "../../bindings/Step";
import type { Tile } from "../../bindings/Tile";
import type { Workflow } from "../../bindings/Workflow";
import { ActionTile } from "../../components/ActionTile";
import { Glass } from "../../components/glass/Glass";
import { useLocalized } from "../../i18n";
import type { MagicVar } from "./magic";
import { ParamEditor } from "./ParamEditor";
import { withParam } from "./tree";

const TILES: Tile[] = ["iris", "ember", "lagoon", "amber", "orchid", "glacier", "slate"];
const ICONS = [
  "sparkles",
  "rocket",
  "zap",
  "hash",
  "clipboard",
  "terminal",
  "globe",
  "timer",
  "music",
  "moon",
  "sun",
  "bell",
];

interface Props {
  wf: Workflow;
  step: Step | undefined;
  meta: ActionMeta | undefined;
  magic: MagicVar[];
  variables: string[];
  onWorkflow(patch: Partial<Workflow>): void;
  onStep(patch: (s: Step) => Step): void;
  onDeleteShortcut(): void;
}

export function Inspector({
  wf,
  step,
  meta,
  magic,
  variables,
  onWorkflow,
  onStep,
  onDeleteShortcut,
}: Props) {
  const { t } = useTranslation();
  const loc = useLocalized();

  if (step && meta) {
    const setParam = (key: string, value: unknown) => onStep((s) => withParam(s, key, value));

    return (
      <Glass variant="regular" className="inspector" aria-label={t("editor.inspector")}>
        <div className="inspector__head">
          <ActionTile tile={meta.tile} icon={meta.icon} size={36} />
          <div>
            <h2 className="inspector__title">{loc(meta.name)}</h2>
            <div className="secondary">
              {t(`types.${meta.input}`)} <ArrowRight size={12} aria-hidden />{" "}
              {t(`types.${meta.output}`)}
            </div>
          </div>
        </div>
        {loc(meta.description) && (
          <p className="secondary inspector__desc">{loc(meta.description)}</p>
        )}
        {meta.params.map((p) => (
          <div key={p.key} className="inspector__field">
            <label className="inspector__label">{loc(p.label)}</label>
            <ParamEditor
              spec={p}
              value={step.params?.[p.key]}
              onChange={(v) => setParam(p.key, v)}
              magic={magic}
              variables={variables}
            />
          </div>
        ))}
        {meta.block === "menu" && (
          <div className="inspector__field">
            <label className="inspector__label">{t("editor.caseLabel")}</label>
            {(step.cases ?? []).map((c, i) => (
              <div key={i} className="param-editor__row">
                <input
                  className="field"
                  value={c.label}
                  aria-label={`${t("editor.caseLabel")} ${i + 1}`}
                  onChange={(e) =>
                    onStep((s) => ({
                      ...s,
                      cases: (s.cases ?? []).map((x, j) =>
                        j === i ? { ...x, label: e.target.value } : x,
                      ),
                    }))
                  }
                />
                <button
                  className="icon-button"
                  aria-label={t("editor.delete")}
                  onClick={() =>
                    onStep((s) => ({ ...s, cases: (s.cases ?? []).filter((_, j) => j !== i) }))
                  }
                >
                  <X size={14} />
                </button>
              </div>
            ))}
            <button
              className="pill-button"
              onClick={() =>
                onStep((s) => ({
                  ...s,
                  cases: [
                    ...(s.cases ?? []),
                    { label: `${t("editor.caseLabel")} ${(s.cases?.length ?? 0) + 1}`, body: [] },
                  ],
                }))
              }
            >
              <Plus size={14} />
              {t("editor.addCase")}
            </button>
          </div>
        )}
        {meta.sensitive && <p className="secondary inspector__warn">{t("picker.sensitive")}</p>}
      </Glass>
    );
  }

  return (
    <Glass variant="regular" className="inspector" aria-label={t("editor.details")}>
      <h2 className="inspector__title">{t("editor.details")}</h2>
      <div className="inspector__field">
        <label className="inspector__label" htmlFor="wf-desc">
          {t("editor.description")}
        </label>
        <textarea
          id="wf-desc"
          className="field"
          rows={3}
          value={wf.description}
          onChange={(e) => onWorkflow({ description: e.target.value })}
        />
      </div>
      <div className="inspector__field">
        <span className="inspector__label">{t("editor.color")}</span>
        <div className="inspector__tiles" role="radiogroup" aria-label={t("editor.color")}>
          {TILES.map((tile) => (
            <button
              key={tile}
              role="radio"
              aria-checked={wf.color === tile}
              aria-label={tile}
              className="inspector__tile-btn"
              onClick={() => onWorkflow({ color: tile })}
            >
              <ActionTile tile={tile} icon={wf.icon} size={28} />
            </button>
          ))}
        </div>
      </div>
      <div className="inspector__field">
        <span className="inspector__label">{t("editor.icon")}</span>
        <div className="inspector__tiles" role="radiogroup" aria-label={t("editor.icon")}>
          {ICONS.map((icon) => (
            <button
              key={icon}
              role="radio"
              aria-checked={wf.icon === icon}
              aria-label={icon}
              className="inspector__tile-btn"
              onClick={() => onWorkflow({ icon })}
            >
              <ActionTile tile={wf.color} icon={icon} size={28} />
            </button>
          ))}
        </div>
      </div>
      <div className="inspector__field">
        <label className="inspector__label" htmlFor="wf-folder">
          {t("editor.folder")}
        </label>
        <input
          id="wf-folder"
          className="field"
          value={wf.folder ?? ""}
          onChange={(e) => onWorkflow({ folder: e.target.value || undefined })}
        />
      </div>
      <div className="inspector__actions">
        <button
          className="pill-button"
          aria-pressed={wf.favorite}
          onClick={() => onWorkflow({ favorite: !wf.favorite })}
        >
          <Star size={14} fill={wf.favorite ? "currentColor" : "none"} />
          {t("editor.favorite")}
        </button>
        <button className="pill-button danger" onClick={onDeleteShortcut}>
          <Trash2 size={14} />
          {t("editor.deleteShortcut")}
        </button>
      </div>
      <p className="secondary">{t("editor.noSelection")}</p>
    </Glass>
  );
}
