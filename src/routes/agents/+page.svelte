<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";

  import { writable } from "svelte/store";
  import type { Writable } from "svelte/store";
  import { get } from "svelte/store";

  import { SvelteFlow, Controls, type NodeTypes, useSvelteFlow } from "@xyflow/svelte";
  // 👇 this is important! You need to import the styles for Svelte Flow to work
  import "@xyflow/svelte/dist/style.css";
  import { GradientButton, Modal } from "flowbite-svelte";
  import hotkeys from "hotkeys-js";

  import {
    addAgentEdge,
    addAgentNode,
    deleteAgentEdge,
    deleteAgentNode,
    deserializeAgentFlow,
    deserializeAgentFlowNode,
    importAgentFlow,
    newAgentFlow,
    newAgentFlowNode,
    saveAgentFlow,
    serializeAgentFlow,
    serializeAgentFlowEdge,
    setAgentDefinitionsContext,
  } from "@/lib/agent";
  import type { AgentFlowNode, AgentFlowEdge } from "@/lib/types";

  import AgentDrawer from "./AgentDrawer.svelte";
  import AgentNode from "./AgentNode.svelte";
  import FlowDrawer from "./FlowDrawer.svelte";

  const { data } = $props();

  const { screenToFlowPosition } = useSvelteFlow();
  setAgentDefinitionsContext(data.agent_defs);

  const nodes: Writable<AgentFlowNode[]> = writable([]);
  const edges: Writable<AgentFlowEdge[]> = writable([]);
  const nodeTypes: NodeTypes = {
    agent: AgentNode,
  };

  const agent_defs = data.agent_defs;

  // const flows = data.agent_flows.map((flow) => deserializeAgentFlow(flow, data.agent_defs));
  let flows = $state(data.agent_flows.map((flow) => deserializeAgentFlow(flow, data.agent_defs)));
  let flowIndex = $state(Math.min(0, data.agent_flows.length - 1));

  $effect(() => {
    if (flowIndex < 0) {
      return;
    }
    nodes.set(flows[flowIndex].nodes);
    edges.set(flows[flowIndex].edges);
  });

  async function checkDeletedNodes(nodes: AgentFlowNode[]) {
    const nodeIds = new Set(nodes.map((node) => node.id));
    const deletedNodes = flows[flowIndex].nodes.filter((node) => !nodeIds.has(node.id));
    for (const node of deletedNodes) {
      await deleteAgentNode(flows[flowIndex].name, node.id);
      flows[flowIndex].nodes = flows[flowIndex].nodes.filter((n) => n.id !== node.id);
    }
  }

  async function checkDeletedEdges(edges: AgentFlowEdge[]) {
    const edgeIds = new Set(edges.map((edge) => edge.id));

    const deletedEdges = flows[flowIndex].edges.filter((edge) => !edgeIds.has(edge.id));
    for (const edge of deletedEdges) {
      await deleteAgentEdge(flows[flowIndex].name, edge.id);
      flows[flowIndex].edges = flows[flowIndex].edges.filter((e) => e.id !== edge.id);
    }

    const addedEdges = edges.filter(
      (edge) => !flows[flowIndex].edges.some((e) => e.id === edge.id),
    );
    for (const edge of addedEdges) {
      await addAgentEdge(flows[flowIndex].name, serializeAgentFlowEdge(edge));
      flows[flowIndex].edges.push(edge);
    }
  }

  $effect(() => {
    const unsubscribeNodes = nodes.subscribe(async (nodes) => {
      checkDeletedNodes(nodes);
    });
    const unsubscribeEdges = edges.subscribe(async (edges) => {
      checkDeletedEdges(edges);
    });
    return () => {
      unsubscribeNodes();
      unsubscribeEdges();
    };
  });

  // AgentDrawer

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

  // FlowDrawer

  let flow_drawer = $state(false);

  const key_flow_drawer = "f";

  $effect(() => {
    hotkeys(key_flow_drawer, () => {
      flow_drawer = !flow_drawer;
    });

    return () => {
      hotkeys.unbind(key_flow_drawer);
    };
  });

  // New Flow

  let new_flow_modal = $state(false);
  let new_flow_name = $state("");

  async function handleNewFlow() {
    new_flow_name = "";
    new_flow_modal = true;
  }

  async function createNewFlow() {
    new_flow_modal = false;
    if (!new_flow_name) return;
    const flow = await newAgentFlow(new_flow_name);
    if (!flow) return;
    flows.push(deserializeAgentFlow(flow, agent_defs));
    flowIndex = flows.length - 1;
  }

  // async function updateFlow() {
  //   await updateAgentFlow(nodes, edges, flow_index, agent_defs);
  // }

  async function onSaveFlow() {
    if (flowIndex < 0) return;
    const flow = serializeAgentFlow(get(nodes), get(edges), flows[flowIndex].name, data.agent_defs);
    await saveAgentFlow(flow);
  }

  function onExportFlow() {
    const flow = serializeAgentFlow(get(nodes), get(edges), flows[flowIndex].name, agent_defs);
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

  async function onImportFlow() {
    const file = await open({ multiple: false, filter: "json" });
    if (!file) return;
    const sflow = await importAgentFlow(file);
    if (!sflow.nodes || !sflow.edges) return;
    const flow = deserializeAgentFlow(sflow, agent_defs);
    nodes.set(flow.nodes);
    edges.set(flow.edges);
  }

  async function onAddAgent(agent_name: string) {
    const snode = newAgentFlowNode(agent_name, agent_defs);
    const xy = screenToFlowPosition({
      x: window.innerWidth * 0.45,
      y: window.innerHeight * 0.3,
    });
    snode.x = xy.x;
    snode.y = xy.y;
    await addAgentNode(flows[flowIndex].name, snode);
    const new_node = deserializeAgentFlowNode(snode, agent_defs);
    flows[flowIndex].nodes.push(new_node);
    nodes.update((nodes) => {
      return [...nodes, new_node];
    });
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
    <div class="fixed top-4 left-4 z-30 w-20">
      <GradientButton color="pinkToOrange" class="w-full mb-4" onclick={handleNewFlow}
        >New Flow</GradientButton
      >
      <GradientButton color="pinkToOrange" class="w-full mb-4" onclick={onSaveFlow}
        >Save</GradientButton
      >
      <!-- <GradientButton color="pinkToOrange" class="w-full mb-4" onclick={updateFlow}
        >Update</GradientButton
      > -->
      <GradientButton color="purpleToBlue" class="w-full mb-4" onclick={onExportFlow}
        >Export</GradientButton
      >
      <GradientButton color="purpleToPink" class="w-full mb-4" onclick={onImportFlow}
        >Import</GradientButton
      >
    </div>

    {#if agent_drawer}
      <AgentDrawer agent_defs={data.agent_defs} {onAddAgent} />
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

    {#if flow_drawer}
      <FlowDrawer {flows} bind:flowIndex />
    {:else}
      <GradientButton
        shadow
        color="cyan"
        class="fixed top-24 right-4 z-30"
        onclick={() => (flow_drawer = true)}
      >
        Flows
      </GradientButton>
    {/if}

    {#if new_flow_modal}
      <Modal title="New Flow" bind:open={new_flow_modal}>
        <div class="flex flex-col">
          <label for="flow_name" class="mb-2 text-sm font-medium text-gray-900 dark:text-white"
            >Flow Name</label
          >
          <input
            type="text"
            id="flow_name"
            bind:value={new_flow_name}
            class="block p-2 w-full text-sm text-gray-900 bg-gray-50 rounded-lg border border-gray-300 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500"
            placeholder="Flow Name"
          />
        </div>
        <div class="flex justify-end mt-4">
          <GradientButton color="pinkToOrange" onclick={createNewFlow}>Create</GradientButton>
        </div>
      </Modal>
    {/if}
  </SvelteFlow>
</main>

<style>
  :global(.svelte-flow__edge .svelte-flow__edge-path) {
    stroke-width: 3;
    stroke-opacity: 0.75;
  }
</style>
