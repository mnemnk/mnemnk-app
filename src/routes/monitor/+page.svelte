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
    class="w-full min-h-screen !text-black !dark:text-white !bg-gray-100 dark:!bg-black"
  >
    <Controls />
    <!-- <Background variant={BackgroundVariant.Dots} /> -->
    <!-- <MiniMap /> -->
  </SvelteFlow>
</main>

<style>
  :root {
    --xy-node-background-color: rgb(243, 244, 246);
    --xy-node-color-default: rgb(17, 17, 17);
    --xy-node-border-radius: 10px;
    --xy-node-box-shadow: 10px 0 15px rgba(42, 138, 246, 0.3), -10px 0 15px rgba(233, 42, 103, 0.3);
  }
</style>
