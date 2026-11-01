import {
  closestCenter,
  DndContext,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
  type DragEndEvent,
} from "@dnd-kit/core";
import {
  SortableContext,
  sortableKeyboardCoordinates,
  verticalListSortingStrategy,
} from "@dnd-kit/sortable";
import { ArrowLeft, Play, Plus, Square } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";

import type { ActionMeta } from "../../bindings/ActionMeta";
import type { Step } from "../../bindings/Step";
import type { Workflow } from "../../bindings/Workflow";
import { ActionTile } from "../../components/ActionTile";
import { useCatalog } from "../../store/catalog";
import { useLibrary } from "../../store/library";
import { useRuns } from "../../store/run";
import { useUi } from "../../store/ui";
import { motion as m } from "../../theme/tokens";
import { ActionSheet } from "../picker/ActionSheet";
import { runWithFeedback } from "../run/feedback";
import { Block, type BlockState } from "./Block";
import { Inspector } from "./Inspector";
import { magicFor, variableNames } from "./magic";
import { Marker } from "./Marker";
import { buildRows } from "./rows";
import {
  childrenOf,
  containerOf,
  duplicateStep,
  findStep,
  insertStep,
  moveStep,
  newUuid,
  removeStep,
  updateStep,
  withParam,
  type ContainerId,
} from "./tree";
import "./editor.css";

function newStep(meta: ActionMeta): Step {
  const step: Step = { uuid: newUuid(), action: meta.id, params: {} };
  if (meta.block === "if") return { ...step, body: [], otherwise: [] };
  if (meta.block === "repeat" || meta.block === "repeatEach") return { ...step, body: [] };
  if (meta.block === "menu") {
    return {
      ...step,
      cases: [
        { label: "1", body: [] },
        { label: "2", body: [] },
      ],
    };
  }
  return step;
}

export default function EditorScreen() {
  const { t } = useTranslation();
  const id = useUi((s) => s.editingId);
  const { go, pickerOpen, setPicker } = useUi();
  const stored = useLibrary((s) => s.shortcuts.find((w) => w.id === id));
  const { save, remove } = useLibrary();
  const byId = useCatalog((s) => s.byId);
  const run = useRuns((s) => (id ? s.runs[id] : undefined));
  const stop = useRuns((s) => s.stop);
  // keyed by shortcut id in App, so this initial value is enough
  const [wf, setWf] = useState<Workflow | undefined>(stored);
  const [selected, setSelected] = useState<string | null>(null);
  const [dragging, setDragging] = useState<string | null>(null);
  const saveTimer = useRef<number | undefined>(undefined);

  const update = (next: Workflow) => {
    setWf(next);
    window.clearTimeout(saveTimer.current);
    saveTimer.current = window.setTimeout(() => void save(next), 350);
  };

  useEffect(() => () => window.clearTimeout(saveTimer.current), []);

  const blocks = useMemo(
    () =>
      new Set(
        Object.values(byId)
          .filter((a) => a.block)
          .map((a) => a.id),
      ),
    [byId],
  );
  const steps = useMemo(() => wf?.actions ?? [], [wf]);
  const rows = useMemo(
    () => buildRows(steps, blocks, dragging ?? undefined),
    [steps, blocks, dragging],
  );
  const variables = useMemo(() => variableNames(steps), [steps]);

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 4 } }),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates }),
  );

  if (!wf) {
    return (
      <section className="screen">
        <button className="pill-button" onClick={() => go("library")}>
          <ArrowLeft size={16} />
          {t("editor.back")}
        </button>
      </section>
    );
  }

  const setSteps = (actions: Step[]) => update({ ...wf, actions });
  const running = run?.status === "running";

  const stateOf = (uuid: string): BlockState => {
    if (!run) return "idle";
    if (run.activeStep === uuid) return "active";
    if (run.failedStep === uuid) return "failed";
    if (run.finished.includes(uuid)) return "done";
    return "idle";
  };

  const insert = (meta: ActionMeta) => {
    const step = newStep(meta);
    let container: ContainerId = "root";
    let index = steps.length;
    if (selected) {
      const sel = findStep(steps, selected);
      // into an empty block body first, otherwise right after the selection
      if (
        sel &&
        blocks.has(sel.action) &&
        sel.action !== "control.menu" &&
        (sel.body ?? []).length === 0
      ) {
        container = `${sel.uuid}:body`;
        index = 0;
      } else {
        container = containerOf(steps, selected) ?? "root";
        index = (childrenOf(steps, container) ?? []).findIndex((s) => s.uuid === selected) + 1;
      }
    }
    setSteps(insertStep(steps, container, index, step));
    setSelected(step.uuid);
    setPicker(false);
  };

  const onDragEnd = ({ active, over }: DragEndEvent) => {
    setDragging(null);
    if (!over || active.id === over.id) return;
    const target = rows.find((r) => r.id === over.id);
    const uuid = String(active.id);
    if (!target) return;
    if (target.kind === "marker") {
      const { container, index } = target.drop;
      const len = (childrenOf(steps, container) ?? []).length;
      setSteps(moveStep(steps, uuid, container, index === "end" ? len : index));
      return;
    }
    const from = containerOf(steps, uuid);
    const current = (childrenOf(steps, target.container) ?? []).findIndex((s) => s.uuid === uuid);
    // sortable semantics: moving down in the same list lands after the hovered item
    const index =
      from === target.container && current >= 0 && current < target.index
        ? target.index + 1
        : target.index;
    setSteps(moveStep(steps, uuid, target.container, index));
  };

  const selectedStep = selected ? findStep(steps, selected) : undefined;

  return (
    <section className="screen editor">
      <div className="editor__head">
        <button className="icon-button" aria-label={t("editor.back")} onClick={() => go("library")}>
          <ArrowLeft size={18} />
        </button>
        <motion.div layoutId={`tile-${wf.id}`} transition={m.spring}>
          <ActionTile tile={wf.color} icon={wf.icon} size={38} />
        </motion.div>
        <motion.input
          layoutId={`title-${wf.id}`}
          transition={m.spring}
          className="editor__name"
          value={wf.name}
          placeholder={t("editor.namePlaceholder")}
          aria-label={t("editor.namePlaceholder")}
          onChange={(e) => update({ ...wf, name: e.target.value })}
        />
        <motion.button
          className={`pill-button ${running ? "danger" : "primary"} run-button`}
          whileTap={{ scale: m.press }}
          onClick={() => (running ? stop(wf.id) : void runWithFeedback(wf))}
        >
          {running ? <Square size={14} /> : <Play size={14} />}
          {running ? t("editor.stop") : t("editor.run")}
        </motion.button>
      </div>

      <div className="editor__body">
        <div className="editor__canvas" onClick={() => setSelected(null)}>
          {steps.length === 0 && (
            <div className="empty editor__empty">
              <p>{t("editor.empty")}</p>
              <p className="secondary">{t("editor.emptyHint")}</p>
            </div>
          )}
          <DndContext
            sensors={sensors}
            collisionDetection={closestCenter}
            onDragStart={(e) => setDragging(String(e.active.id))}
            onDragCancel={() => setDragging(null)}
            onDragEnd={onDragEnd}
          >
            <SortableContext items={rows.map((r) => r.id)} strategy={verticalListSortingStrategy}>
              <div className="blocks" role="list">
                <AnimatePresence initial={false}>
                  {rows.map((row) =>
                    row.kind === "marker" ? (
                      <Marker key={row.id} row={row} />
                    ) : (
                      <Block
                        key={row.id}
                        step={row.step}
                        meta={byId[row.step.action]}
                        depth={row.depth}
                        selected={selected === row.id}
                        state={stateOf(row.id)}
                        steps={steps}
                        byId={byId}
                        magic={magicFor(steps, row.id, byId)}
                        variables={variables}
                        onSelect={() => setSelected(row.id)}
                        onParam={(key, value) =>
                          setSteps(updateStep(steps, row.id, (s) => withParam(s, key, value)))
                        }
                        onDelete={() => {
                          setSteps(removeStep(steps, row.id));
                          if (selected === row.id) setSelected(null);
                        }}
                        onDuplicate={() => setSteps(duplicateStep(steps, row.id))}
                        onToggle={() =>
                          setSteps(
                            updateStep(steps, row.id, (s) => ({ ...s, disabled: !s.disabled })),
                          )
                        }
                      />
                    ),
                  )}
                </AnimatePresence>
              </div>
            </SortableContext>
          </DndContext>
          {!pickerOpen && (
            <motion.button
              layoutId="add-sheet"
              className="add-action glass glass--thin"
              transition={m.spring}
              whileTap={{ scale: m.press }}
              onClick={(e) => {
                e.stopPropagation();
                setPicker(true);
              }}
            >
              <Plus size={16} />
              {t("editor.addAction")}
            </motion.button>
          )}
        </div>

        <Inspector
          wf={wf}
          step={selectedStep}
          meta={selectedStep ? byId[selectedStep.action] : undefined}
          magic={selectedStep ? magicFor(steps, selectedStep.uuid, byId) : []}
          variables={variables}
          onWorkflow={(patch) => update({ ...wf, ...patch })}
          onStep={(patch) => selected && setSteps(updateStep(steps, selected, patch))}
          onDeleteShortcut={() => {
            void remove(wf.id);
            go("library");
          }}
        />
      </div>

      <AnimatePresence>
        {pickerOpen && <ActionSheet onPick={insert} onClose={() => setPicker(false)} />}
      </AnimatePresence>
    </section>
  );
}
