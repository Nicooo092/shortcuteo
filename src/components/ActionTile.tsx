import { DynamicIcon, type IconName } from "lucide-react/dynamic";

import type { Tile } from "../bindings/Tile";

interface Props {
  tile: Tile;
  icon: string;
  size?: number;
}

export function ActionTile({ tile, icon, size = 32 }: Props) {
  return (
    <span
      className={`tile tile--${tile}`}
      style={{ width: size, height: size, borderRadius: Math.round(size * 0.31) }}
      aria-hidden
    >
      <DynamicIcon
        name={icon as IconName}
        size={Math.round(size * 0.55)}
        strokeWidth={2}
        color="white"
      />
    </span>
  );
}
