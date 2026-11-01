import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import type { HostRequestEvent } from "../bindings/HostRequestEvent";
import type { RunEventPayload } from "../bindings/RunEventPayload";
import type { Api } from "./index";

export const nativeApi: Api = {
  listActions: () => invoke("list_actions"),
  runWorkflow: (runId, workflow, input) => invoke("run_workflow", { runId, workflow, input }),
  cancelRun: (runId) => invoke("cancel_run", { runId }),
  hostReply: (id, answer) => invoke("host_reply", { id, answer }),
  listShortcuts: () => invoke("list_shortcuts"),
  saveShortcut: (workflow) => invoke("save_shortcut", { workflow }),
  deleteShortcut: (id) => invoke("delete_shortcut", { id }),
  parseShortcut: (text) => invoke("parse_shortcut", { text }),
  exportShortcut: (workflow) => invoke("export_shortcut", { workflow }),
  onBattery: () => invoke("on_battery"),
  readTextFile: (path) => invoke("read_shortcut_file", { path }),
  onRunEvent: (cb) => listen<RunEventPayload>("run-event", (e) => cb(e.payload)),
  onHostRequest: (cb) => listen<HostRequestEvent>("host-request", (e) => cb(e.payload)),
};
