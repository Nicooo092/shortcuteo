import type { Value } from "../../bindings/Value";

// mirrors Value::to_text on the rust side, for previews only
export function valueToText(v: Value): string {
  switch (v.type) {
    case "nothing":
      return "";
    case "number":
    case "bool":
      return String(v.value);
    case "list":
      return v.value.map(valueToText).join("\n");
    case "dict":
      return JSON.stringify(toJson(v), null, 2);
    default:
      return v.value;
  }
}

function toJson(v: Value): unknown {
  switch (v.type) {
    case "nothing":
      return null;
    case "list":
      return v.value.map(toJson);
    case "dict":
      return Object.fromEntries(Object.entries(v.value).map(([k, x]) => [k, toJson(x)]));
    default:
      return v.value;
  }
}

export function preview(v: Value, max = 80): string {
  const s = valueToText(v).replace(/\s+/g, " ").trim();
  return s.length > max ? `${s.slice(0, max - 3)}...` : s;
}
