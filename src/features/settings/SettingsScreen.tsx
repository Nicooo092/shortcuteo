import { useTranslation } from "react-i18next";

import { Glass } from "../../components/glass/Glass";
import i18n, { type Lang } from "../../i18n";
import { useSettings } from "../../store/settings";
import { color, tiles } from "../../theme/tokens";

const ACCENTS = [
  color.accent,
  tiles.orchid.to,
  tiles.ember.from,
  tiles.lagoon.from,
  tiles.amber.from,
];

export default function SettingsScreen() {
  const { t } = useTranslation();
  const s = useSettings();

  const setLang = (lang: Lang) => {
    s.set({ lang });
    void i18n.changeLanguage(lang);
  };

  return (
    <section className="screen">
      <div className="screen__head">
        <h1 className="screen__title">{t("settings.title")}</h1>
      </div>
      <div className="settings">
        <Glass variant="regular" className="settings__row">
          <span>{t("settings.language")}</span>
          <div className="segmented" role="group" aria-label={t("settings.language")}>
            {(["fr", "en"] as const).map((l) => (
              <button key={l} aria-pressed={s.lang === l} onClick={() => setLang(l)}>
                {l === "fr" ? "Français" : "English"}
              </button>
            ))}
          </div>
        </Glass>
        <Glass variant="regular" className="settings__row">
          <span>{t("settings.accent")}</span>
          <div className="settings__swatches" role="group" aria-label={t("settings.accent")}>
            {ACCENTS.map((c) => (
              <button
                key={c}
                className="swatch"
                style={{ background: c }}
                aria-label={c}
                aria-pressed={s.accent === c}
                onClick={() => s.set({ accent: c })}
              />
            ))}
          </div>
        </Glass>
        <Glass variant="regular" className="settings__row">
          <label htmlFor="blur">{t("settings.blur")}</label>
          <input
            id="blur"
            type="range"
            min={0}
            max={1.5}
            step={0.1}
            value={s.blur}
            onChange={(e) => s.set({ blur: Number(e.target.value) })}
          />
        </Glass>
        <Glass variant="regular" className="settings__row">
          <span>{t("settings.batteryBlur")}</span>
          <Switch
            on={s.batteryBlur}
            label={t("settings.batteryBlur")}
            onChange={(v) => s.set({ batteryBlur: v })}
          />
        </Glass>
        <Glass variant="regular" className="settings__row">
          <div>
            <div>{t("settings.animations")}</div>
            <div className="secondary">{t("settings.animationsHint")}</div>
          </div>
          <Switch
            on={s.animations}
            label={t("settings.animations")}
            onChange={(v) => s.set({ animations: v })}
          />
        </Glass>
      </div>
    </section>
  );
}

function Switch({
  on,
  label,
  onChange,
}: {
  on: boolean;
  label: string;
  onChange: (v: boolean) => void;
}) {
  return (
    <button
      role="switch"
      className="switch"
      aria-checked={on}
      aria-label={label}
      onClick={() => onChange(!on)}
    />
  );
}
