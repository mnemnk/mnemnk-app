<script lang="ts">
  import { SvelteFlow, Controls, type NodeTypes } from "@xyflow/svelte";
  // 👇 this is important! You need to import the styles for Svelte Flow to work
  import "@xyflow/svelte/dist/style.css";

  import { edges, nodes } from "@/lib/shared.svelte";

  import BoardNode from "./BoardNode.svelte";
  import MessageNode from "./MessageNode.svelte";

  const nodeTypes: NodeTypes = {
    board: BoardNode,
    message: MessageNode,
  };
</script>

<main class="container min-w-[100vw]">
  <SvelteFlow
    {nodes}
    {nodeTypes}
    {edges}
    fitView
    maxZoom={2}
    minZoom={0.2}
    attributionPosition="bottom-left"
    class="w-full min-h-screen !text-black !dark:text-white !bg-gray-100 dark:!bg-black"
  >
    <Controls />
    <!-- <Background variant={BackgroundVariant.Dots} /> -->
    <!-- <MiniMap /> -->
  </SvelteFlow>
</main>
