import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import { writable, type Writable } from "svelte/store";

import type { DisplayMessage, ErrorMessage, InputMessage } from "@/lib/types";

// Display Message

let displayMessageStore: Map<string, Map<string, Writable<any>>> = new Map<
  string,
  Map<string, Writable<any>>
>();

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

// Error Message
let errorMessageStore: Map<string, Writable<string>> = new Map<string, Writable<string>>();

export function subscribeErrorMessage(
  agentId: string,
  callback: (message: string) => void,
): () => void {
  let errorStore = errorMessageStore.get(agentId);
  if (!errorStore) {
    errorStore = writable("");
    errorMessageStore.set(agentId, errorStore);
  }
  return errorStore.subscribe(callback);
}

let unlistenError: UnlistenFn | null = null;

// Input Message
let inputMessageStore: Map<string, Writable<{ ch: string; t: number }>> = new Map<
  string,
  Writable<{ ch: string; t: number }>
>();

export function subscribeInputMessage(
  agentId: string,
  callback: (message: { ch: string; t: number }) => void,
): () => void {
  let inputStore = inputMessageStore.get(agentId);
  if (!inputStore) {
    inputStore = writable({ ch: "", t: 0 });
    inputMessageStore.set(agentId, inputStore);
  }
  return inputStore.subscribe(callback);
}

let unlistenInput: UnlistenFn | null = null;

//

$effect.root(() => {
  listen<DisplayMessage>("mnemnk:display", (event) => {
    const { agent_id, key, data } = event.payload;
    let store = displayMessageStore.get(agent_id);
    if (!store) {
      return;
    }
    let v = store.get(key);
    if (!v) {
      return;
    }
    v.set(data);
  }).then((unlistenFn) => {
    unlistenDisplay = unlistenFn;
  });

  // Listen for error messages
  listen<ErrorMessage>("mnemnk:error", (event) => {
    const { agent_id, message } = event.payload;
    let errorStore = errorMessageStore.get(agent_id);
    if (!errorStore) {
      return;
    }
    errorStore.set(message);
  }).then((unlistenFn) => {
    unlistenError = unlistenFn;
  });

  // Listen for input messages
  listen<InputMessage>("mnemnk:input", (event) => {
    const { agent_id, ch } = event.payload;
    let inputStore = inputMessageStore.get(agent_id);
    if (!inputStore) {
      return;
    }
    inputStore.set({ ch, t: Date.now() });
  }).then((unlistenFn) => {
    unlistenInput = unlistenFn;
  });

  return () => {
    unlistenDisplay?.();
    unlistenError?.();
    unlistenInput?.();
  };
});

// Agent Flow

export const flowNameState = $state({ name: "main" });
