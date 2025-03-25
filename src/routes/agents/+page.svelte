<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";

  import { writable } from "svelte/store";
  import type { Writable } from "svelte/store";
  import { get } from "svelte/store";

  import { SvelteFlow, Controls, type NodeTypes } from "@xyflow/svelte";
  // 👇 this is important! You need to import the styles for Svelte Flow to work
  import "@xyflow/svelte/dist/style.css";
  import { GradientButton } from "flowbite-svelte";
  import hotkeys from "hotkeys-js";

  import {
    deserializeAgentFlow,
    readAgentFlow,
    serializeAgentFlow,
    setAgentDefinitionsContext,
    updateAgentFlow,
  } from "@/lib/agent";
  import type { AgentFlowNode, AgentFlowEdge } from "@/lib/types";

  import AgentDrawer from "./AgentDrawer.svelte";
  import AgentNode from "./AgentNode.svelte";

  const { data } = $props();

  setAgentDefinitionsContext(data.agent_defs);

  const nodes: Writable<AgentFlowNode[]> = writable([]);
  const edges: Writable<AgentFlowEdge[]> = writable([]);
  const nodeTypes: NodeTypes = {
    agent: AgentNode,
  };

  const agent_defs = data.agent_defs;

  const flow_index = $state(0);

  $effect(() => {
    const flow = deserializeAgentFlow(data.agent_flows[flow_index], data.agent_defs);
    nodes.set(flow.nodes);
    edges.set(flow.edges);
  });

  let agent_drawer = $state(false);

  const key_agent_drawer = "a";

  $effect(() => {
    hotkeys(key_agent_drawer, () => {
      agent_drawer = !agent_drawer;
    });

    return () => {
      hotkeys.unbind(key_agent_drawer);
    };
  });

  async function updateFlow() {
    await updateAgentFlow(nodes, edges, flow_index, agent_defs);
  }

  function exportFlow() {
    const flow = serializeAgentFlow(get(nodes), get(edges), agent_defs);
    const jsonStr = JSON.stringify(flow, null, 2);
    const blob = new Blob([jsonStr], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "flow.json";
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }

  async function importFlow() {
    const file = await open({ multiple: false, filter: "json" });
    if (!file) return;
    const sflow = await readAgentFlow(file);
    if (!sflow.nodes || !sflow.edges) return;
    const flow = deserializeAgentFlow(sflow, agent_defs);
    nodes.set(flow.nodes);
    edges.set(flow.edges);
  }
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

  <div class="fixed top-4 left-4 z-30 w-20">
    <GradientButton color="pinkToOrange" class="w-full mb-4" onclick={updateFlow}
      >Update</GradientButton
    >
    <GradientButton color="purpleToBlue" class="w-full mb-4" onclick={exportFlow}
      >Export</GradientButton
    >
    <GradientButton color="purpleToPink" class="w-full mb-4" onclick={importFlow}
      >Import</GradientButton
    >
  </div>

  {#if agent_drawer}
    <AgentDrawer {nodes} agent_defs={data.agent_defs} />
  {:else}
    <GradientButton
      shadow
      color="cyan"
      class="fixed top-4 right-4 z-30"
      onclick={() => (agent_drawer = true)}
    >
      Agents
    </GradientButton>
  {/if}
</main>

<style>
  :global(.svelte-flow__edge .svelte-flow__edge-path) {
    stroke-width: 3;
    stroke-opacity: 0.75;
  }
</style>
