import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import { writable, type Writable } from "svelte/store";

import type { DisplayMessage } from "@/lib/types";

// Display Message

let displayMessageStore: Map<string, Map<string, Writable<any>>>;

export function subscribeDisplayMessage(
  agentId: string,
  key: string,
  callback: (value: any) => void,
): () => void {
  let store = displayMessageStore.get(agentId);
  if (!store) {
    store = new Map<string, Writable<any>>();
    displayMessageStore.set(agentId, store);
  }
  let v = store.get(key);
  if (!v) {
    v = writable(null);
    store.set(key, v);
  }
  return v.subscribe(callback);
}

let unlistenDisplay: UnlistenFn | null = null;

$effect.root(() => {
  displayMessageStore = new Map<string, Map<string, Writable<any>>>();

  listen<DisplayMessage>("mnemnk:display", (event) => {
    const { agent_id, key, value } = event.payload;
    let store = displayMessageStore.get(agent_id);
    if (!store) {
      return;
    }
    let v = store.get(key);
    if (!v) {
      return;
    }
    v.set(value);
  }).then((unlistenFn) => {
    unlistenDisplay = unlistenFn;
  });
  return () => {
    unlistenDisplay?.();
  };
});

// Agent Flow

export const flowNameState = $state({ name: "main" });
