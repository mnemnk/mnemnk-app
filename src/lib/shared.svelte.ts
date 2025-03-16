import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import { SvelteMap } from "svelte/reactivity";
import { writable, type Writable } from "svelte/store";

import type { Node } from "@xyflow/svelte";

import type { BoardMessage, WriteBoardEmit } from "@/lib/types";

export const boards = $state(new SvelteMap<string, BoardMessage[]>());

export const nodes: Writable<Node[]> = writable([]);

// same for edges
export const edges = writable([]);

export const MESSAGES_TIMEOUT = 10 * 60 * 1000;

function addMessage(agent: string, kind: string, value: any, time: number) {
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
  const timeout_threshold = Date.now() - MESSAGES_TIMEOUT;
  messages = messages.filter((m) => m.time > timeout_threshold);
  messages.push({ agent, kind, value, time });
  boards.set(kind, messages);

  nodes.update((nodes) => {
    return [
      ...nodes.filter((n) => !n.data.time || ((n.data.time as number) ?? 0) > timeout_threshold),
      {
        id: Math.random().toString(),
        type: "message",
        data: { agent, value, time },
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
    const { agent, kind, value } = event.payload;
    let time = value.t ?? Date.now();
    addMessage(agent, kind, value, time);
  }).then((unlistenFn) => {
    unlisten = unlistenFn;
  });
  return () => {
    unlisten?.();
    boards.clear();
  };
});
