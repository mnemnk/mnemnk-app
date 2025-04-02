import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import { SvelteMap } from "svelte/reactivity";
import { writable, type Writable } from "svelte/store";

import type { Node } from "@xyflow/svelte";

import type { BoardMessage, WriteBoardEmit, DisplayMessage } from "@/lib/types";

export const boards = $state(new SvelteMap<string, BoardMessage[]>());

export const nodes: Writable<Node[]> = writable([]);
export const edges = writable([]);

export const MESSAGES_TIMEOUT = 10 * 60 * 1000;

function addMessage(kind: string, value: any, time: number) {
  if (!boards.has(kind)) {
    nodes.update((nodes) => {
      return [
        ...nodes,
        {
          id: kind,
          type: "board",
          data: { kind },
          position: { x: Math.random() * 1800, y: Math.random() * 1000 },
          width: 1400,
          height: 1400,
        },
      ];
    });
  }
  let messages = boards.get(kind) ?? [];
  const timeoutThreshold = Date.now() - MESSAGES_TIMEOUT;
  messages = messages.filter((m) => m.time > timeoutThreshold);
  messages.push({ kind, value, time });
  boards.set(kind, messages);

  nodes.update((nodes) => {
    return [
      ...nodes.filter((n) => !n.data.time || ((n.data.time as number) ?? 0) > timeoutThreshold),
      {
        id: Math.random().toString(),
        type: "message",
        data: { value, time },
        position: { x: Math.random() * 1000, y: Math.random() * 1000 },
        parentId: kind,
        extent: "parent",
      },
    ];
  });
}

let unlisten: UnlistenFn | null = null;

$effect.root(() => {
  listen<WriteBoardEmit>("mnemnk:write_board", (event) => {
    const { kind, value } = event.payload;
    let time = value.t ?? Date.now();
    addMessage(kind, value, time);
  }).then((unlistenFn) => {
    unlisten = unlistenFn;
  });
  return () => {
    unlisten?.();
    boards.clear();
  };
});

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
