import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import { SvelteMap } from "svelte/reactivity";
import { writable, type Writable } from "svelte/store";

import type { Node } from "@xyflow/svelte";

import type { WriteBoardEmit } from "@/lib/types";

export const boards = $state(new SvelteMap<string, WriteBoardEmit[]>());

export const nodes: Writable<Node[]> = writable([]);

// same for edges
export const edges = writable([]);

// const MAX_MESSAGES = 10;

function addMessage(agent: string, kind: string, value: any) {
  if (!boards.has(kind)) {
    nodes.update((nodes) => {
      return [
        ...nodes,
        {
          id: kind,
          type: "board",
          data: { kind },
          position: { x: Math.random() * 1800, y: Math.random() * 1000 },
        },
      ];
    });
  }
  let messages = boards.get(kind) ?? [];
  messages.push({ agent, kind, value });
  //   messages = messages.slice(-MAX_MESSAGES);
  boards.set(kind, messages);

  nodes.update((nodes) => {
    return [
      //   ...nodes.slice(-MAX_MESSAGES + 1),
      ...nodes,
      {
        id: Math.random().toString(),
        type: "message",
        data: { agent, value },
        position: { x: Math.random() * 800 - 400, y: Math.random() * 800 - 400 },
        parentId: kind,
      },
    ];
  });
}

let unlisten: UnlistenFn | null = null;

$effect.root(() => {
  listen<WriteBoardEmit>("mnemnk:write_board", (event) => {
    const { agent, kind, value } = event.payload;
    // console.log("write_board", agent, kind, value);
    addMessage(agent, kind, value);
  }).then((unlistenFn) => {
    unlisten = unlistenFn;
  });
  return () => {
    unlisten?.();
    boards.clear();
  };
});
