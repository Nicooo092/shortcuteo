import { useLocalized } from "../../i18n";
import { ActionTile } from "../../components/ActionTile";
import type { MagicVar } from "./magic";

export function MagicPill({ v, dim = false }: { v: MagicVar; dim?: boolean }) {
  const loc = useLocalized();
  const label = typeof v.label === "string" ? v.label : loc(v.label);
  return (
    <span className={`magic magic--${v.tile}${dim ? " is-dim" : ""}`}>
      <ActionTile tile={v.tile} icon={v.icon} size={16} />
      {label}
    </span>
  );
}
