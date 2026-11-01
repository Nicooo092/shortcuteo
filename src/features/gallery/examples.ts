import type { Workflow } from "../../bindings/Workflow";

const files = import.meta.glob<string>("../../../examples/*.shortcuteo", {
  query: "?raw",
  import: "default",
  eager: true,
});

export const gallery: Workflow[] = Object.values(files)
  .map((raw) => JSON.parse(raw) as Workflow)
  .sort((a, b) => a.name.localeCompare(b.name));
