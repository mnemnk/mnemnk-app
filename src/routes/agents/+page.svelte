<script lang="ts">
  import { writable } from "svelte/store";
  import type { Writable } from "svelte/store";

  import { SvelteFlow, Controls, type NodeTypes } from "@xyflow/svelte";
  // 👇 this is important! You need to import the styles for Svelte Flow to work
  import "@xyflow/svelte/dist/style.css";

  import { deserializeAgentFlow, setAgentConfigsContext } from "@/lib/agent";
  import type { AgentFlowNode, AgentFlowEdge } from "@/lib/types";

  import AgentDrawer from "./AgentDrawer.svelte";
  import AgentNode from "./AgentNode.svelte";

  const { data } = $props();

  setAgentConfigsContext(data.agent_configs);

  const nodes: Writable<AgentFlowNode[]> = writable([]);
  const edges: Writable<AgentFlowEdge[]> = writable([]);
  const nodeTypes: NodeTypes = {
    agent: AgentNode,
  };

  const flow_index = $state(0);

  $effect(() => {
    const flow = deserializeAgentFlow(data.agent_flows[flow_index], data.agent_configs);
    nodes.set(flow.nodes);
    edges.set(flow.edges);
  });
</script>

<main class="container static min-w-[100vw]">
  <SvelteFlow
    {nodes}
    {nodeTypes}
    {edges}
    deleteKey={["Delete", "Backspace"]}
    connectionRadius={38}
    colorMode="dark"
    fitView
    maxZoom={2}
    minZoom={0.2}
    attributionPosition="bottom-left"
    class="relative w-full min-h-screen !text-black !dark:text-white !bg-gray-100 dark:!bg-black"
  >
    <Controls />
  </SvelteFlow>

  <AgentDrawer {flow_index} {nodes} {edges} agent_configs={data.agent_configs} />
</main>

<style>
  :global(.svelte-flow__edge .svelte-flow__edge-path) {
    stroke-width: 3;
    stroke-opacity: 0.75;
  }
</style>
