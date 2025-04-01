<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";

  import { SvelteFlow, Controls, type NodeTypes, useSvelteFlow } from "@xyflow/svelte";
  // 👇 this is important! You need to import the styles for Svelte Flow to work
  import "@xyflow/svelte/dist/style.css";
  import {
    Accordion,
    AccordionItem,
    Dropdown,
    DropdownItem,
    GradientButton,
    Modal,
    Navbar,
    NavLi,
    NavUl,
  } from "flowbite-svelte";
  import { ChevronDownOutline } from "flowbite-svelte-icons";
  import hotkeys from "hotkeys-js";

  import {
    addAgentFlowEdge,
    addAgentFlowNode,
    removeAgentFlowEdge,
    removeAgentFlowNode,
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

  import AgentList from "./AgentList.svelte";
  import AgentNode from "./AgentNode.svelte";
  import FlowList from "./FlowList.svelte";

  const { data } = $props();

  const { screenToFlowPosition } = $derived(useSvelteFlow());
  setAgentDefinitionsContext(data.agent_defs);

  const nodeTypes: NodeTypes = {
    agent: AgentNode,
  };

  const agent_defs = data.agent_defs;

  let nodes = $state.raw<AgentFlowNode[]>([]);
  let edges = $state.raw<AgentFlowEdge[]>([]);
  let flows = $state(data.agent_flows.map((flow) => deserializeAgentFlow(flow, data.agent_defs)));
  let flowIndex = $state(Math.min(0, data.agent_flows.length - 1));

  $effect(() => {
    if (flowIndex < 0) {
      return;
    }
    nodes = flows[flowIndex].nodes;
    edges = flows[flowIndex].edges;
  });

  async function checkDeletedNodes(nodes: AgentFlowNode[]) {
    const nodeIds = new Set(nodes.map((node) => node.id));
    const deletedNodes = flows[flowIndex].nodes.filter((node) => !nodeIds.has(node.id));
    for (const node of deletedNodes) {
      await removeAgentFlowNode(flows[flowIndex].name, node.id);
      flows[flowIndex].nodes = flows[flowIndex].nodes.filter((n) => n.id !== node.id);
    }
  }

  async function checkDeletedEdges(edges: AgentFlowEdge[]) {
    const edgeIds = new Set(edges.map((edge) => edge.id));

    const deletedEdges = flows[flowIndex].edges.filter((edge) => !edgeIds.has(edge.id));
    for (const edge of deletedEdges) {
      await removeAgentFlowEdge(flows[flowIndex].name, edge.id);
      flows[flowIndex].edges = flows[flowIndex].edges.filter((e) => e.id !== edge.id);
    }

    const addedEdges = edges.filter(
      (edge) => !flows[flowIndex].edges.some((e) => e.id === edge.id),
    );
    for (const edge of addedEdges) {
      await addAgentFlowEdge(flows[flowIndex].name, serializeAgentFlowEdge(edge));
      flows[flowIndex].edges.push(edge);
    }
  }

  $effect(() => {
    checkDeletedNodes(nodes);
    checkDeletedEdges(edges);
  });

  // AgentList

  let openAgent = $state(false);

  const key_open_agent = "a";

  $effect(() => {
    hotkeys(key_open_agent, () => {
      openAgent = !openAgent;
    });

    return () => {
      hotkeys.unbind(key_open_agent);
    };
  });

  // FlowList

  let openFlow = $state(false);

  const key_open_flow = "f";

  $effect(() => {
    hotkeys(key_open_flow, () => {
      openFlow = !openFlow;
    });

    return () => {
      hotkeys.unbind(key_open_flow);
    };
  });

  // shortcuts

  $effect(() => {
    hotkeys("ctrl+s", (event) => {
      event.preventDefault();
      onSaveFlow();
    });

    return () => {
      hotkeys.unbind("ctrl+s");
    };
  });

  // New Flow

  let new_flow_modal = $state(false);
  let new_flow_name = $state("");

  async function onNewFlow() {
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

  async function onSaveFlow() {
    if (flowIndex < 0) return;
    const flow = serializeAgentFlow(nodes, edges, flows[flowIndex].name, data.agent_defs);
    await saveAgentFlow(flow);
  }

  function onExportFlow() {
    const flow = serializeAgentFlow(nodes, edges, flows[flowIndex].name, agent_defs);
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
    nodes = flow.nodes;
    edges = flow.edges;
  }

  async function onAddAgent(agent_name: string) {
    const snode = newAgentFlowNode(agent_name, agent_defs);
    const xy = screenToFlowPosition({
      x: window.innerWidth * 0.45,
      y: window.innerHeight * 0.3,
    });
    snode.x = xy.x;
    snode.y = xy.y;
    await addAgentFlowNode(flows[flowIndex].name, snode);
    const new_node = deserializeAgentFlowNode(snode, agent_defs);
    flows[flowIndex].nodes.push(new_node);
    nodes = [...nodes, new_node];
  }
</script>

<main class="container static min-w-[100vw]">
  <SvelteFlow
    bind:nodes
    bind:edges
    {nodeTypes}
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

  <Navbar class="fixed top-4 left-0 z-10 !bg-transparent">
    <NavUl>
      <NavLi class="cursor-pointer w-40"
        >File<ChevronDownOutline class="w-6 h-6 ms-2 inline" /></NavLi
      >
      <Dropdown class="w-40 z-20">
        <DropdownItem onclick={onNewFlow}>New</DropdownItem>
        <DropdownItem onclick={onSaveFlow}>Save</DropdownItem>
        <DropdownItem onclick={onExportFlow}>Export</DropdownItem>
        <DropdownItem onclick={onImportFlow}>Import</DropdownItem>
      </Dropdown>
    </NavUl>
  </Navbar>

  <div class="fixed top-4 right-4 z-30">
    <Accordion class="w-40 bg-white dark:bg-black" classActive="bg-white dark:bg-black">
      <AccordionItem open={openAgent}>
        <div slot="header">Agents</div>
        <AgentList agent_defs={data.agent_defs} {onAddAgent} />
      </AccordionItem>
      <AccordionItem open={openFlow}>
        <span slot="header">Flows</span>
        <FlowList {flows} bind:flowIndex />
      </AccordionItem>
    </Accordion>
  </div>

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
</main>

<style>
  :global(.svelte-flow__edge .svelte-flow__edge-path) {
    stroke-width: 3;
    stroke-opacity: 0.75;
  }
</style>
