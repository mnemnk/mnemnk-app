<script lang="ts">
  import { get, writable } from "svelte/store";
  import type { Writable } from "svelte/store";

  import { SvelteFlow, Controls, type NodeTypes } from "@xyflow/svelte";
  // 👇 this is important! You need to import the styles for Svelte Flow to work
  import "@xyflow/svelte/dist/style.css";
  import { Button, Drawer, GradientButton } from "flowbite-svelte";
  import { nanoid } from "nanoid";

  import {
    deserializeAgentFlow,
    deserializeAgentFlowNode,
    save_agent_flow,
    serializeAgentFlow,
    setAgentSettingsContext,
  } from "@/lib/agent";
  import type { SAgentNode } from "@/lib/types";

  import AgentNode from "./AgentNode.svelte";

  const { data } = $props();

  const catalog = data.catalog;
  setAgentSettingsContext(data.settings);

  const nodes: Writable<SAgentNode[]> = writable([]);
  const edges = writable([]);
  const nodeTypes: NodeTypes = {
    agent: AgentNode,
  };

  const flow_index = $state(0);

  $effect(() => {
    nodes.set(deserializeAgentFlow(data.agent_flows[flow_index], data.settings));
  });

  function addNode(agent_name: string) {
    const id = nanoid();
    const default_config = data.settings[agent_name]?.default_config || {};
    const node_data = {
      id,
      name: agent_name,
      enabled: true,
      config: {
        ...default_config,
      },
      x: Math.random() * 1000,
      y: Math.random() * 1000,
    };
    nodes.update((nodes) => {
      return [...nodes, deserializeAgentFlowNode(node_data, data.settings)];
    });
  }

  async function updateAgentFlow() {
    const flow = serializeAgentFlow(get(nodes));
    await save_agent_flow(flow, flow_index);
  }
</script>

<main class="container static min-w-[100vw]">
  <SvelteFlow
    {nodes}
    {nodeTypes}
    {edges}
    fitView
    maxZoom={2}
    minZoom={0.2}
    class="relative w-full min-h-screen !text-black !dark:text-white !bg-gray-100 dark:!bg-black"
  >
    <Controls />
  </SvelteFlow>

  <Drawer
    activateClickOutside={false}
    backdrop={false}
    hidden={false}
    placement="right"
    class="w-200"
  >
    <GradientButton color="pinkToOrange" class="w-full mb-4" onclick={updateAgentFlow}>
      Update
    </GradientButton>
    {#each catalog as agent}
      <div class="mb-4">
        <Button class="w-full" outline onclick={() => addNode(agent.name)}>{agent.name}</Button>
      </div>
    {/each}
  </Drawer>
</main>

<style>
  :root {
    --xy-node-background-color: rgb(243, 244, 246);
    --xy-node-color-default: rgb(17, 17, 17);
    --xy-node-border-radius: 10px;
    --xy-node-box-shadow: 10px 0 15px rgba(42, 138, 246, 0.3), -10px 0 15px rgba(233, 42, 103, 0.3);
  }
</style>
