import { useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { CircleOff, Copy, GripVertical, Trash2 } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { useState, type CSSProperties } from "react";
import { useTranslation } from "react-i18next";

import type { ActionMeta } from "../../bindings/ActionMeta";
import type { ParamSpec } from "../../bindings/ParamSpec";
import type { Step } from "../../bindings/Step";
import { ActionTile } from "../../components/ActionTile";
import { Glass } from "../../components/glass/Glass";
import { useLocalized } from "../../i18n";
import { motion as m } from "../../theme/tokens";
import { describeToken, type MagicVar } from "./magic";
import { MagicPill } from "./MagicPill";
import { asText, ParamEditor } from "./ParamEditor";
import { parseTokens } from "./tokens";

export type BlockState = "idle" | "active" | "done" | "failed";

interface Props {
  step: Step;
  meta: ActionMeta | undefined;
  depth: number;
  selected: boolean;
  state: BlockState;
  steps: Step[];
  byId: Record<string, ActionMeta>;
  magic: MagicVar[];
  variables: string[];
  onSelect(): void;
  onParam(key: string, value: unknown): void;
  onDelete(): void;
  onDuplicate(): void;
  onToggle(): void;
}

const INDENT = 28;

export function Block(props: Props) {
  const { step, meta, depth, selected, state } = props;
  const { t } = useTranslation();
  const loc = useLocalized();
  const { setNodeRef, transform, transition, isDragging, attributes, listeners } = useSortable({
    id: step.uuid,
  });
  const style: CSSProperties = {
    transform: CSS.Translate.toString(transform),
    transition: transition,
    marginLeft: depth * INDENT,
    zIndex: isDragging ? 5 : undefined,
  };

  if (!meta) {
    return (
      <div ref={setNodeRef} style={style} className="block block--unknown">
        <span className="secondary">{step.action}</span>
      </div>
    );
  }

  return (
    <div ref={setNodeRef} style={style} className="block-row">
      <motion.div
        layout="position"
        transition={m.spring}
        initial={{ opacity: 0, scale: 0.96 }}
        animate={{ opacity: step.disabled ? 0.45 : 1, scale: isDragging ? 1.02 : 1 }}
      >
        <Glass
          variant="regular"
          className={`block is-${state}${selected ? " is-selected" : ""}`}
          onClick={props.onSelect}
          role="group"
          aria-label={loc(meta.name)}
          aria-current={state === "active" ? "step" : undefined}
        >
          <button
            className="block__grip"
            aria-label={t("editor.drag")}
            {...attributes}
            {...listeners}
          >
            <GripVertical size={15} />
          </button>
          <ActionTile tile={meta.tile} icon={meta.icon} size={30} />
          <Sentence {...props} meta={meta} />
          <div className="block__tools">
            <button
              className="icon-button"
              aria-label={step.disabled ? t("editor.enable") : t("editor.disable")}
              onClick={(e) => {
                e.stopPropagation();
                props.onToggle();
              }}
            >
              <CircleOff size={14} />
            </button>
            <button
              className="icon-button"
              aria-label={t("editor.duplicate")}
              onClick={(e) => {
                e.stopPropagation();
                props.onDuplicate();
              }}
            >
              <Copy size={14} />
            </button>
            <button
              className="icon-button"
              aria-label={t("editor.delete")}
              onClick={(e) => {
                e.stopPropagation();
                props.onDelete();
              }}
            >
              <Trash2 size={14} />
            </button>
          </div>
        </Glass>
      </motion.div>
    </div>
  );
}

// "Wait {seconds}" becomes text with clickable parameter pills, like ios
function Sentence(props: Props & { meta: ActionMeta }) {
  const loc = useLocalized();
  const sentence = loc(props.meta.sentence);
  const parts = sentence.split(/(\{[a-z_]+\})/g).filter(Boolean);
  const specs = Object.fromEntries(props.meta.params.map((p) => [p.key, p]));
  return (
    <div className="block__sentence">
      {parts.map((part, i) => {
        const key = /^\{([a-z_]+)\}$/.exec(part)?.[1];
        const spec = key ? specs[key] : undefined;
        if (!spec) return <span key={i}>{part}</span>;
        return <ParamPill key={i} spec={spec} {...props} />;
      })}
    </div>
  );
}

function ParamPill({
  spec,
  step,
  steps,
  byId,
  magic,
  variables,
  onParam,
}: Props & { spec: ParamSpec }) {
  const loc = useLocalized();
  const [open, setOpen] = useState(false);
  const raw = step.params?.[spec.key];
  const value = raw ?? spec.default;

  let content: React.ReactNode;
  if (raw === undefined && spec.fromInput && magic.length > 0) {
    content = (
      <span className="pill__placeholder is-magic">
        {loc({ en: "previous output", fr: "sortie précédente" })}
      </span>
    );
  } else if (spec.kind.kind === "choice") {
    const opt = spec.kind.options.find((o) => o.value === value);
    content = opt ? loc(opt.label) : loc(spec.label);
  } else if (spec.kind.kind === "bool") {
    content = loc(value === true ? { en: "on", fr: "oui" } : { en: "off", fr: "non" });
  } else {
    const text = asText(value);
    content = text ? (
      parseTokens(text).map((seg, i) =>
        seg.kind === "token" ? (
          <MagicPill key={i} v={describeToken(seg.token, steps, byId)} />
        ) : (
          <span key={i}>{seg.text.length > 40 ? `${seg.text.slice(0, 39)}...` : seg.text}</span>
        ),
      )
    ) : (
      <span className="pill__placeholder">{loc(spec.label)}</span>
    );
  }

  return (
    <span className="pill-wrap">
      <button
        className={`pill${open ? " is-open" : ""}`}
        aria-label={loc(spec.label)}
        aria-expanded={open}
        onClick={(e) => {
          e.stopPropagation();
          setOpen((o) => !o);
        }}
      >
        {content}
      </button>
      <AnimatePresence>
        {open && (
          <motion.div
            className="pill-pop"
            initial={{ opacity: 0, y: -6, scale: 0.96 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: -4, scale: 0.98 }}
            transition={m.spring}
            onClick={(e) => e.stopPropagation()}
            onKeyDown={(e) => {
              if (e.key === "Escape" || (e.key === "Enter" && !e.shiftKey)) setOpen(false);
            }}
          >
            <Glass variant="thick" className="pill-pop__panel">
              <div className="pill-pop__label secondary">{loc(spec.label)}</div>
              <ParamEditor
                spec={spec}
                value={raw}
                onChange={(v) => onParam(spec.key, v)}
                magic={magic}
                variables={variables}
                autoFocus
              />
            </Glass>
          </motion.div>
        )}
      </AnimatePresence>
      {open && (
        <span
          className="pill-pop__backdrop"
          onClick={(e) => (e.stopPropagation(), setOpen(false))}
        />
      )}
    </span>
  );
}
