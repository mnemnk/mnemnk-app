<script lang="ts">
  import { writable } from "svelte/store";
  import type { Writable } from "svelte/store";

  import { SvelteFlow, Controls, type NodeTypes } from "@xyflow/svelte";
  // 👇 this is important! You need to import the styles for Svelte Flow to work
  import "@xyflow/svelte/dist/style.css";

  import { deserializeAgentFlow, setAgentSettingsContext } from "@/lib/agent";
  import type { AgentFlowNode, AgentFlowEdge } from "@/lib/types";

  import AgentDrawer from "./AgentDrawer.svelte";
  import AgentNode from "./AgentNode.svelte";
  import BoardNode from "./BoardNode.svelte";
  import DatabaseNode from "./DatabaseNode.svelte";

  const { data } = $props();

  const catalog = data.catalog;
  setAgentSettingsContext(data.settings);

  const nodes: Writable<AgentFlowNode[]> = writable([]);
  const edges: Writable<AgentFlowEdge[]> = writable([]);
  const nodeTypes: NodeTypes = {
    agent: AgentNode,
    board: BoardNode,
    database: DatabaseNode,
  };

  const flow_index = $state(0);

  $effect(() => {
    const flow = deserializeAgentFlow(data.agent_flows[flow_index], data.settings);
    nodes.set(flow.nodes);
    edges.set(flow.edges);
  });
</script>

<main class="container static min-w-[100vw]">
  <SvelteFlow
    {nodes}
    {nodeTypes}
    {edges}
    colorMode="dark"
    fitView
    maxZoom={2}
    minZoom={0.2}
    class="relative w-full min-h-screen !text-black !dark:text-white !bg-gray-100 dark:!bg-black"
  >
    <Controls />
  </SvelteFlow>

  <AgentDrawer {catalog} {flow_index} {nodes} {edges} settings={data.settings} />
</main>

<style>
  :root {
    --xy-node-background-color: rgb(243, 244, 246);
    --xy-node-color-default: rgb(17, 17, 17);
    --xy-node-border-radius: 10px;
    --xy-node-box-shadow: 10px 0 15px rgba(42, 138, 246, 0.3), -10px 0 15px rgba(233, 42, 103, 0.3);
  }
</style>
