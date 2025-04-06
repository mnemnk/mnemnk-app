import { invoke } from "@tauri-apps/api/core";

import { getContext, setContext } from "svelte";

import { nanoid } from "nanoid";

import type {
  AgentFlow,
  AgentFlowEdge,
  AgentFlowNode,
  AgentFlowNodeConfig,
  AgentFlowNodeDisplay,
  SAgentConfig,
  SAgentDefaultConfig,
  SAgentDefinitions,
  SAgentDisplayConfig,
  SAgentFlow,
  SAgentFlowEdge,
  SAgentFlowNode,
  SAgentFlows,
} from "./types";

export async function startAgent(agentId: string): Promise<void> {
  await invoke("start_agent_cmd", { agentId });
}

export async function stopAgent(agentId: string): Promise<void> {
  await invoke("stop_agent_cmd", { agentId });
}

export async function setAgentConfig(agentId: string, config: SAgentConfig): Promise<void> {
  await invoke("set_agent_config_cmd", { agentId, config });
}

export async function getAgentDefs(): Promise<SAgentDefinitions> {
  return await invoke("get_agent_defs_cmd");
}

export async function getAgentFlows(): Promise<SAgentFlows> {
  return await invoke("get_agent_flows_cmd");
}

export async function importAgentFlow(path: string): Promise<SAgentFlow> {
  return await invoke("import_agent_flow_cmd", { path });
}

export async function newAgentFlow(name: string): Promise<SAgentFlow> {
  return await invoke("new_agent_flow_cmd", { name });
}

export async function renameAgentFlow(oldName: string, newName: string): Promise<string> {
  return await invoke("rename_agent_flow_cmd", { oldName, newName });
}

export async function deleteAgentFlow(name: string): Promise<void> {
  await invoke("delete_agent_flow_cmd", { name });
}

export async function saveAgentFlow(agentFlow: SAgentFlow): Promise<void> {
  await invoke("save_agent_flow_cmd", { agentFlow });
}

const agentDefinitionsKey = Symbol("agentDefinitions");

export function setAgentDefinitionsContext(defs: SAgentDefinitions): void {
  setContext(agentDefinitionsKey, defs);
}

export function getAgentDefinitionsContext(): SAgentDefinitions {
  return getContext(agentDefinitionsKey);
}

export async function addAgentFlowNode(flowName: string, node: SAgentFlowNode): Promise<void> {
  await invoke("add_agent_flow_node_cmd", { flowName, node });
}

export async function removeAgentFlowNode(flowName: string, nodeId: string): Promise<void> {
  await invoke("remove_agent_flow_node_cmd", { flowName, nodeId });
}

export async function addAgentFlowEdge(flowName: string, edge: SAgentFlowEdge): Promise<void> {
  await invoke("add_agent_flow_edge_cmd", { flowName, edge });
}

export async function removeAgentFlowEdge(flowName: string, edgeId: string): Promise<void> {
  await invoke("remove_agent_flow_edge_cmd", { flowName, edgeId });
}

// Agent Flow

// deserialize: SAgentFlow -> AgentFlow

export function deserializeAgentFlow(
  flow: SAgentFlow,
  agent_settings: SAgentDefinitions,
): AgentFlow {
  // Deserialize nodes first
  const nodes = flow.nodes.map((node) => deserializeAgentFlowNode(node, agent_settings));

  // Create a map to retrieve available handles from node IDs
  const nodeHandles = new Map<string, { inputs: string[]; outputs: string[] }>();

  nodes.forEach((node) => {
    const def = agent_settings[node.data.name];
    if (def) {
      nodeHandles.set(node.id, {
        inputs: def.inputs || [],
        outputs: def.outputs || [],
      });
    }
  });

  // Filter only valid edges
  const validEdges = flow.edges.filter((edge) => {
    const sourceNode = nodeHandles.get(edge.source);
    const targetNode = nodeHandles.get(edge.target);

    if (!sourceNode || !targetNode) return false;

    // Ensure that the source and target handles actually exist
    const isSourceValid = sourceNode.outputs.includes(edge.source_handle ?? "");
    const isTargetValid = targetNode.inputs.includes(edge.target_handle ?? "");

    return isSourceValid && isTargetValid;
  });

  return {
    nodes: nodes,
    edges: validEdges.map((edge) => deserializeAgentFlowEdge(edge)),
    name: flow.name,
  };
}

export function deserializeAgentFlowNode(
  node: SAgentFlowNode,
  agentDefs: SAgentDefinitions,
): AgentFlowNode {
  const agentDef = agentDefs[node.name];
  const default_config = agentDef?.default_config;
  const display_config = agentDef?.display_config;
  return {
    id: node.id,
    type: "agent",
    data: {
      name: node.name,
      enabled: node.enabled,
      title: node.title,
      config: deserializeAgentConfig(node.config, default_config),
      display: deserializeAgentDisplayConfig(display_config),
    },
    position: {
      x: node.x,
      y: node.y,
    },
    width: node.width,
    height: node.height,
  };
}

export function deserializeAgentConfig(
  node_config: SAgentConfig | null,
  default_config: SAgentDefaultConfig | null,
): AgentFlowNodeConfig {
  let agent_config: AgentFlowNodeConfig = {};
  let config_types: Record<string, string | null> = {};

  if (default_config) {
    default_config.forEach(([key, entry]) => {
      agent_config[key] = entry.value;
      config_types[key] = entry.type;
    });
  }

  if (node_config) {
    for (const [key, value] of Object.entries(node_config)) {
      agent_config[key] = value;
    }
  }

  for (const [key, value] of Object.entries(agent_config)) {
    const t = config_types[key];
    if (t === null) {
      continue;
    } else if (t === "boolean") {
      agent_config[key] = value;
    } else if (t === "integer") {
      agent_config[key] = value.toString();
    } else if (t === "number") {
      agent_config[key] = value.toString();
    } else if (t === "string") {
      agent_config[key] = value;
    } else if (t === "text") {
      agent_config[key] = Array.isArray(value) ? value.join("\n") : "";
    } else if (t === "object") {
      agent_config[key] = JSON.stringify(value, null, 2);
    }
  }

  return agent_config;
}

export function deserializeAgentDisplayConfig(
  display_config: SAgentDisplayConfig | null,
): AgentFlowNodeDisplay | null {
  if (!display_config) {
    return null;
  }
  let display: AgentFlowNodeDisplay = {};
  display_config.forEach(([key, _entry]) => {
    display[key] = null;
  });
  return display;
}

function deserializeAgentFlowEdge(edge: SAgentFlowEdge): AgentFlowEdge {
  return {
    id: edge.id,
    source: edge.source,
    sourceHandle: edge.source_handle,
    target: edge.target,
    targetHandle: edge.target_handle,
  };
}

// serialize: AgentFlow -> SAgentFlow

export function serializeAgentFlow(
  nodes: AgentFlowNode[],
  edges: AgentFlowEdge[],
  name: string,
  agent_configs: SAgentDefinitions,
): SAgentFlow {
  return {
    nodes: nodes.map((node) => serializeAgentFlowNode(node, agent_configs)),
    edges: edges.map((edge) => serializeAgentFlowEdge(edge)),
    name,
  };
}

export function serializeAgentFlowNode(
  node: AgentFlowNode,
  agent_configs: SAgentDefinitions,
): SAgentFlowNode {
  return {
    id: node.id,
    name: node.data.name,
    enabled: node.data.enabled,
    config: serializeAgentFlowNodeConfig(
      node.data.config,
      agent_configs[node.data.name].default_config,
    ),
    title: node.data.title,
    x: node.position.x,
    y: node.position.y,
    width: node.width,
    height: node.height,
  };
}

export function serializeAgentFlowNodeConfig(
  node_config: AgentFlowNodeConfig | null,
  default_config: SAgentDefaultConfig | null,
): SAgentConfig | null {
  if (node_config === null) {
    return null;
  }

  let config: SAgentConfig = {};

  if (default_config === null) {
    // if no default config, just return the node_config as is
    for (const [key, value] of Object.entries(node_config)) {
      config[key] = value;
    }
    return config;
  }

  default_config.forEach(([key, entry]) => {
    const t = entry.type;
    const value = node_config[key];
    if (t === "boolean") {
      config[key] = value;
    } else if (t === "integer") {
      config[key] = parseInt(value);
    } else if (t === "number") {
      config[key] = parseFloat(value);
    } else if (t === "string") {
      config[key] = value;
    } else if (t === "text") {
      config[key] = value.split("\n");
    } else if (t === "object") {
      config[key] = JSON.parse(value);
    } else {
      config[key] = value;
    }
  });

  return config;
}

export function serializeAgentFlowEdge(edge: AgentFlowEdge): SAgentFlowEdge {
  return {
    id: edge.id,
    source: edge.source,
    source_handle: edge.sourceHandle ?? null,
    target: edge.target,
    target_handle: edge.targetHandle ?? null,
  };
}

export function newAgentFlowNode(def_name: string, agent_defs: SAgentDefinitions): SAgentFlowNode {
  const id = newNodeId(def_name);
  const default_config = agent_defs[def_name].default_config ?? [];
  const config: SAgentConfig = Object.fromEntries(
    default_config.map(([key, entry]) => [key, entry.value]),
  );
  const node_data = {
    id,
    name: def_name,
    enabled: false,
    config,
    title: null,
    x: 0,
    y: 0,
  };
  return node_data;
}

function newNodeId(prefix: string) {
  return `${prefix}_${nanoid()}`;
}
