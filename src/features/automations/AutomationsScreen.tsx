import { Zap } from "lucide-react";
import { useTranslation } from "react-i18next";

// triggers land in M3; the screen exists so navigation is complete
export default function AutomationsScreen() {
  const { t } = useTranslation();
  return (
    <section className="screen">
      <div className="screen__head">
        <h1 className="screen__title">{t("automations.title")}</h1>
      </div>
      <div className="empty">
        <Zap size={28} aria-hidden />
        <p>{t("automations.empty")}</p>
      </div>
    </section>
  );
}
