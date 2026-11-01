import { Braces } from "lucide-react";
import { useId, useRef, useState } from "react";
import { useTranslation } from "react-i18next";

import type { ParamSpec } from "../../bindings/ParamSpec";
import { useLocalized } from "../../i18n";
import type { MagicVar } from "./magic";
import { MagicPill } from "./MagicPill";

interface Props {
  spec: ParamSpec;
  value: unknown;
  onChange(value: unknown): void;
  magic: MagicVar[];
  variables: string[];
  autoFocus?: boolean;
}

export function asText(value: unknown): string {
  if (value === undefined || value === null) return "";
  if (typeof value === "string") return value;
  if (typeof value === "number" || typeof value === "boolean") return String(value);
  return JSON.stringify(value);
}

export function ParamEditor({ spec, value, onChange, magic, variables, autoFocus }: Props) {
  const { t } = useTranslation();
  const loc = useLocalized();
  const id = useId();
  const field = useRef<HTMLInputElement & HTMLTextAreaElement>(null);
  const [menu, setMenu] = useState(false);
  const kind = spec.kind;
  const effective = value ?? spec.default;

  // keeps the caret where it was, so a token lands where the user is typing
  const insert = (token: string) => {
    const el = field.current;
    const text = asText(value);
    const raw = `{{${token}}}`;
    if (!el) return onChange(text + raw);
    const start = el.selectionStart ?? text.length;
    const end = el.selectionEnd ?? text.length;
    onChange(text.slice(0, start) + raw + text.slice(end));
    setMenu(false);
    requestAnimationFrame(() => {
      el.focus();
      el.setSelectionRange(start + raw.length, start + raw.length);
    });
  };

  if (kind.kind === "bool") {
    const on = effective === true || effective === "true";
    return (
      <button
        role="switch"
        className="switch"
        aria-checked={on}
        aria-label={loc(spec.label)}
        autoFocus={autoFocus}
        onClick={() => onChange(!on)}
      />
    );
  }

  if (kind.kind === "choice") {
    return (
      <div className="choice" role="radiogroup" aria-label={loc(spec.label)}>
        {kind.options.map((o, i) => (
          <button
            key={o.value}
            role="radio"
            aria-checked={effective === o.value}
            className="choice__option"
            autoFocus={autoFocus && (effective === o.value || (i === 0 && effective === undefined))}
            onClick={() => onChange(o.value)}
          >
            {loc(o.label)}
          </button>
        ))}
      </div>
    );
  }

  const text = asText(value);
  const commit = (raw: string) => {
    // plain numbers stay numbers in the file; anything with a token stays text
    if (
      kind.kind === "number" &&
      raw.trim() !== "" &&
      !raw.includes("{{") &&
      !Number.isNaN(Number(raw))
    ) {
      onChange(Number(raw));
    } else {
      onChange(raw === "" ? undefined : raw);
    }
  };

  const placeholder =
    spec.fromInput && magic.length
      ? t("editor.previousOutput")
      : asText(spec.default) || loc(spec.label);
  const common = {
    id,
    ref: field,
    className: "field",
    value: text,
    placeholder,
    autoFocus,
    "aria-label": loc(spec.label),
    onChange: (e: { target: { value: string } }) => commit(e.target.value),
  };

  return (
    <div className="param-editor">
      <div className="param-editor__row">
        {kind.kind === "text" && kind.multiline ? (
          <textarea {...common} rows={4} />
        ) : (
          <input
            {...common}
            inputMode={kind.kind === "number" ? "decimal" : undefined}
            list={kind.kind === "variable" ? `${id}-vars` : undefined}
          />
        )}
        {kind.kind === "number" && kind.unit && (
          <span className="param-editor__unit mono">{kind.unit}</span>
        )}
        {kind.kind !== "variable" && (
          <button
            className="icon-button"
            aria-label={t("editor.variables")}
            aria-expanded={menu}
            onClick={() => setMenu((m) => !m)}
          >
            <Braces size={15} />
          </button>
        )}
      </div>
      {kind.kind === "variable" && (
        <datalist id={`${id}-vars`}>
          {variables.map((v) => (
            <option key={v} value={v} />
          ))}
        </datalist>
      )}
      {menu && (
        <div className="magic-menu" role="menu" aria-label={t("editor.variables")}>
          {magic.map((v) => (
            <button
              key={v.token}
              role="menuitem"
              className="magic-menu__item"
              onClick={() => insert(v.token)}
            >
              <MagicPill v={v} />
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
