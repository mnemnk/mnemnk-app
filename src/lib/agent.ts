import { invoke } from "@tauri-apps/api/core";

import { getContext, setContext } from "svelte";

import { nanoid } from "nanoid";

import type {
  AgentFlowNodeConfig,
  SAgentFlow,
  SAgentFlowEdge,
  SAgentFlowNode,
  SAgentDefinitions,
  SAgentDefaultConfig,
  AgentFlow,
  AgentFlowEdge,
  AgentFlowNode,
} from "./types";

export async function getAgentDefs(): Promise<SAgentDefinitions> {
  return await invoke("get_agent_defs_cmd");
}

export async function getAgentFlows(): Promise<SAgentFlow[]> {
  return await invoke("get_agent_flows_cmd");
}

export async function importAgentFlow(path: string): Promise<SAgentFlow> {
  return await invoke("import_agent_flow_cmd", { path });
}

export async function newAgentFlow(name: string): Promise<SAgentFlow> {
  return await invoke("new_agent_flow_cmd", { name });
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

export async function addAgentNode(flowName: string, node: SAgentFlowNode): Promise<void> {
  await invoke("add_agent_node_cmd", { flowName, node });
}

export async function deleteAgentNode(flowName: string, nodeId: string): Promise<void> {
  await invoke("delete_agent_node_cmd", { flowName, nodeId });
}

export async function addAgentEdge(flowName: string, edge: SAgentFlowEdge): Promise<void> {
  await invoke("add_agent_edge_cmd", { flowName, edge });
}

export async function deleteAgentEdge(flowName: string, edgeId: string): Promise<void> {
  await invoke("delete_agent_edge_cmd", { flowName, edgeId });
}

// Agent Flow

// export function addAgentNode(
//   agent_name: string,
//   nodes: Writable<AgentFlowNode[]>,
//   settings: SAgentDefinitions,
// ) {
//   const new_node = newAgentFlowNode(agent_name, settings);
//   nodes.update((nodes) => {
//     return [...nodes, new_node];
//   });
// }

// export async function updateAgentFlow(
//   nodes: Writable<AgentFlowNode[]>,
//   edges: Writable<AgentFlowEdge[]>,
//   flow_index: number,
//   agent_configs: SAgentDefinitions,
// ) {
//   const flow = serializeAgentFlow(get(nodes), get(edges), agent_configs);
//   await saveAgentFlow(flow, flow_index);
// }

// deserialize: SAgentFlow -> AgentFlow

export function deserializeAgentFlow(
  flow: SAgentFlow,
  agent_settings: SAgentDefinitions,
): AgentFlow {
  return {
    nodes: flow.nodes.map((node) => deserializeAgentFlowNode(node, agent_settings)),
    edges: flow.edges.map((edge) => deserializeAgentFlowEdge(edge)),
    name: flow.name,
  };
}

export function deserializeAgentFlowNode(
  node: SAgentFlowNode,
  agent_configs: SAgentDefinitions,
): AgentFlowNode {
  const configs = agent_configs[node.name];
  const default_config = configs?.default_config;
  return {
    id: node.id,
    type: "agent",
    data: {
      name: node.name,
      title: configs?.title ?? null,
      description: configs?.description ?? null,
      enabled: node.enabled,
      inputs: configs?.inputs ?? [],
      outputs: configs?.outputs ?? [],
      config: deserializeAgentConfig(node.config, default_config),
    },
    position: {
      x: node.x,
      y: node.y,
    },
    width: node.width,
    height: node.height,
  };
}

function deserializeAgentConfig(
  node_config: Record<string, any> | null,
  default_config: SAgentDefaultConfig | null,
): AgentFlowNodeConfig {
  if (!node_config) {
    node_config = {};
  }
  if (default_config) {
    for (const key of Object.keys(default_config)) {
      if (node_config[key] === undefined) {
        node_config[key] = default_config[key].value;
      }
    }
  }
  const agent_config: AgentFlowNodeConfig = {};
  for (const key of Object.keys(node_config)) {
    const t = default_config?.[key].type;
    if (t === null) {
      agent_config[key] = node_config[key];
    } else if (t === "boolean") {
      agent_config[key] = node_config[key] === true ? "true" : "false";
    } else if (t === "integer") {
      agent_config[key] = node_config[key].toString();
    } else if (t === "number") {
      agent_config[key] = node_config[key].toString();
    } else if (t === "string") {
      agent_config[key] = node_config[key];
    } else if (t === "string?") {
      agent_config[key] = node_config[key] ?? "";
    } else if (t === "string[]") {
      agent_config[key] = node_config[key].join("\n");
    } else if (t === "object") {
      agent_config[key] = JSON.stringify(node_config[key], null, 2);
    } else {
      agent_config[key] = node_config[key];
    }
  }
  return agent_config;
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
    config: serializeAgentFlowNodeConfig(
      node.data.config,
      agent_configs[node.data.name].default_config,
    ),
    enabled: node.data.enabled,
    x: node.position.x,
    y: node.position.y,
    width: node.width,
    height: node.height,
  };
}

function serializeAgentFlowNodeConfig(
  node_config: AgentFlowNodeConfig | null,
  default_config: SAgentDefaultConfig | null,
): Record<string, any> | null {
  if (node_config === null) {
    return null;
  }

  if (default_config === null) {
    const config: Record<string, any> = {};
    for (const key of Object.keys(node_config)) {
      config[key] = node_config[key];
    }
    return config;
  }

  const config: Record<string, any> = {};
  for (const key of Object.keys(node_config)) {
    const t = default_config[key].type;
    const value = node_config[key];
    if (t === "boolean") {
      config[key] = value === "true";
    } else if (t === "integer") {
      config[key] = parseInt(value);
    } else if (t === "number") {
      config[key] = parseFloat(value);
    } else if (t === "string") {
      config[key] = value;
    } else if (t === "string?") {
      config[key] = value === "" ? null : value;
    } else if (t === "string[]") {
      config[key] = value.split("\n");
    } else if (t === "object") {
      config[key] = JSON.parse(value);
    } else {
      config[key] = value;
    }
  }
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
  const default_config = agent_defs[def_name].default_config ?? {};
  const config: Record<string, any> = {};
  for (const key of Object.keys(default_config)) {
    config[key] = default_config[key].value;
  }
  const node_data = {
    id,
    name: def_name,
    enabled: false,
    config,
    x: 0,
    y: 0,
  };
  return node_data;
}

function newNodeId(prefix: string) {
  return `${prefix}_${nanoid()}`;
}
