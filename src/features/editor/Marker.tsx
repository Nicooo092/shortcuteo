import { useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { useTranslation } from "react-i18next";

import type { Row } from "./rows";

const INDENT = 28;

// "Otherwise", "End if", menu case headers: not draggable, but valid drop targets
export function Marker({ row }: { row: Extract<Row, { kind: "marker" }> }) {
  const { t } = useTranslation();
  const { setNodeRef, transform, transition, isOver } = useSortable({
    id: row.id,
    disabled: { draggable: true },
  });
  const label =
    row.label === "case"
      ? row.owner.cases?.[row.caseIndex ?? 0]?.label || t("editor.caseLabel")
      : t(`editor.${row.label}`);
  return (
    <div
      ref={setNodeRef}
      style={{
        transform: CSS.Translate.toString(transform),
        transition: transition,
        marginLeft: row.depth * INDENT,
      }}
      className={`marker marker--${row.label}${isOver ? " is-over" : ""}`}
    >
      <span className="marker__line" aria-hidden />
      <span className="marker__label">{label}</span>
      <span className="marker__line" aria-hidden />
    </div>
  );
}
