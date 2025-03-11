<script lang="ts">
  import { get, writable } from "svelte/store";
  import { type Writable } from "svelte/store";

  import { SvelteFlow, Controls } from "@xyflow/svelte";
  import type { Node, NodeTypes } from "@xyflow/svelte";
  // 👇 this is important! You need to import the styles for Svelte Flow to work
  import "@xyflow/svelte/dist/style.css";
  import { Button, Drawer, GradientButton } from "flowbite-svelte";
  import { nanoid } from "nanoid";

  import type { AgentFlow, AgentFlowNodeDataType } from "@/lib/types";
  import { save_agent_flow } from "@/lib/utils";

  import AgentNode from "./AgentNode.svelte";

  const { data } = $props();

  const catalog = data.catalog;
  const settings = data.settings;
  const properties = data.properties;

  const nodes: Writable<Node[]> = writable([]);
  const edges = writable([]);
  const nodeTypes: NodeTypes = {
    agent: AgentNode,
  };

  $effect(() => {
    let new_nodes: Node[] = [];
    for (const node of data.agent_flows[0].nodes) {
      const name = node.name;
      new_nodes.push({
        id: node.id,
        type: "agent",
        data: {
          name,
          enabled: node.enabled,
          config: node.config,
          schema: settings[name].schema,
          properties: properties.get(name),
        },
        position: {
          x: node.x,
          y: node.y,
        },
        width: node.width,
        height: node.height,
      });
    }
    nodes.set(new_nodes);
  });

  function addNode(agent_name: string) {
    // TODO: support multiple flows

    nodes.update((nodes) => {
      nodes.push({
        id: nanoid(),
        type: "agent",
        data: {
          name: agent_name,
          enabled: false,
          config: settings[agent_name].default_config,
          schema: settings[agent_name].schema,
          properties: properties.get(agent_name), // can we put the whole properties into a Context?
        },
        position: {
          x: Math.random() * 1000,
          y: Math.random() * 1000,
        },
      });
      return nodes;
    });
  }

  async function saveFlow() {
    // await save_agent_flows(flow_nodes[flow_index], flow_index);
    const flow: AgentFlow = { nodes: [] };
    get(nodes).forEach((node) => {
      const data = node.data as AgentFlowNodeDataType;
      flow.nodes.push({
        id: node.id,
        name: data.name as string,
        enabled: data.enabled,
        config: data.config,
        x: node.position.x,
        y: node.position.y,
        width: node.width,
        height: node.height,
      });
    });
    await save_agent_flow(flow, 0);
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
    <GradientButton color="pinkToOrange" class="w-full mb-4" onclick={saveFlow}>
      Save
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
