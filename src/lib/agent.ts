import { invoke } from "@tauri-apps/api/core";

import { getContext, setContext } from "svelte";
import { get, writable } from "svelte/store";

import { nanoid } from "nanoid";

import type {
  AgentCatalog,
  AgentConfig,
  AgentConfigEntry,
  AgentDefaultConfig,
  SAgentFlow,
  SAgentFlowEdge,
  SAgentFlowNode,
  AgentSchema,
  AgentSetting,
  AgentSettings,
  AgentFlow,
  AgentFlowEdge,
  AgentFlowNode,
} from "./types";

export async function get_agent_catalog(): Promise<AgentCatalog> {
  return await invoke("get_agent_catalog_cmd");
}

export async function start_agent(agent_id: string): Promise<void> {
  await invoke("start_agent_cmd", { agent_id });
}

export async function stop_agent(agent_id: string): Promise<void> {
  await invoke("stop_agent_cmd", { agent_id });
}

export async function save_agent_config(
  agent_id: string,
  config: Record<string, any>,
): Promise<void> {
  await invoke("save_agent_config_cmd", { agent_id, config });
}

export async function set_agent_enabled(agent_id: string, enabled: boolean): Promise<void> {
  await invoke("set_agent_enabled_cmd", { agent_id, enabled });
}

export async function get_agent_settings(): Promise<Record<string, AgentSetting>> {
  return await invoke("get_agent_settings_cmd");
}

export async function get_agent_flows(): Promise<SAgentFlow[]> {
  return await invoke("get_agent_flows_cmd");
}

export async function save_agent_flow(agent_flow: SAgentFlow, idx: number): Promise<void> {
  await invoke("save_agent_flow_cmd", { agent_flow, idx });
}

const agentSettingsKey = Symbol("agentSettings");

export function setAgentSettingsContext(settings: Record<string, AgentSetting>): void {
  setContext(agentSettingsKey, settings);
}

export function getAgentSettingsContext(): Record<string, AgentSetting> {
  return getContext(agentSettingsKey);
}

// Agent Flow

// deserialize: SAgentFlow -> AgentFlow

export function deserializeAgentFlow(
  flow: SAgentFlow,
  agent_settings: Record<string, AgentSetting>,
): AgentFlow {
  return {
    nodes: flow.nodes.map((node) => deserializeAgentFlowNode(node, agent_settings)),
    edges: flow.edges.map((edge) => deserializeAgentFlowEdge(edge)),
  };
}

export function deserializeAgentFlowNode(
  node: SAgentFlowNode,
  agent_settings: Record<string, AgentSetting>,
): AgentFlowNode {
  if (node.name === "$board") {
    return {
      id: node.id,
      type: "board",
      data: {
        name: node.name,
        enabled: writable(node.enabled),
        config: {
          board_name: {
            value: writable(""),
            type: "string",
            title: "Board Name",
            description: null,
          },
          persistent: {
            value: writable(false),
            type: "boolean",
            title: "Persistent",
            description: "Store messages into DB if true",
          },
        },
      },
      position: {
        x: node.x,
        y: node.y,
      },
      width: node.width,
      height: node.height,
    };
  }

  const settings = agent_settings[node.name];
  const default_config = settings?.default_config;
  const schema_properties = settings?.schema?.properties;
  return {
    id: node.id,
    type: "agent",
    data: {
      name: node.name,
      enabled: writable(node.enabled),
      config: deserializeAgentConfig(node.config, default_config, schema_properties),
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
  default_config: AgentDefaultConfig | null,
  schema_properties: AgentSchema | null,
): AgentConfig {
  if (!node_config) {
    node_config = {};
  }
  if (default_config) {
    for (const key of Object.keys(default_config)) {
      if (node_config[key] === undefined) {
        node_config[key] = default_config[key];
      }
    }
  }
  const agent_config: AgentConfig = {};
  for (const key of Object.keys(node_config)) {
    const entry: AgentConfigEntry = {
      value: writable(null),
      type: null,
      title: null,
      description: null,
    };
    const property = schema_properties?.[key];
    if (property) {
      const t = property.type;
      if (t === "boolean") {
        entry.type = "boolean";
        entry.value = writable(node_config[key]);
      } else if (t === "integer") {
        entry.type = "integer";
        entry.value = writable(node_config[key]);
      } else if (t === "number") {
        entry.type = "number";
        entry.value = writable(node_config[key].toString());
      } else if (t === "string") {
        entry.type = "string";
        entry.value = writable(node_config[key]);
      } else if (isOptionString(t)) {
        entry.type = "string?";
        entry.value = writable(node_config[key] === null ? "" : node_config[key]);
      } else if (isArrayString(t, property.items?.type)) {
        entry.type = "string[]";
        entry.value = writable((node_config[key] as string[]).join("\n"));
      } else {
        entry.type = `unknown (${t})`;
        entry.value = writable(node_config[key]);
      }
      if (property.title) {
        entry.title = property.title;
      }
      if (property.description) {
        entry.description = property.description;
      }
    } else {
      entry.type = "unknown";
      entry.value = writable(node_config[key]);
    }
    agent_config[key] = entry;
  }
  return agent_config;
}

function isOptionString(t: any): boolean {
  return Array.isArray(t) && t.length === 2 && t.includes("string") && t.includes("null");
}

function isArrayString(t: any, items_t: any): boolean {
  return t === "array" && items_t === "string";
}

function deserializeAgentFlowEdge(edge: SAgentFlowEdge): AgentFlowEdge {
  return {
    id: edge.id,
    source: edge.source,
    target: edge.target,
  };
}

// serialize: AgentFlow -> SAgentFlow

export function serializeAgentFlow(nodes: AgentFlowNode[], edges: AgentFlowEdge[]): SAgentFlow {
  return {
    nodes: nodes.map((node) => serializeAgentFlowNode(node)),
    edges: edges.map((edge) => serializeAgentFlowEdge(edge)),
  };
}

export function serializeAgentFlowNode(node: AgentFlowNode): SAgentFlowNode {
  return {
    id: node.id,
    name: node.data.name,
    config: serializeAgentFlowNodeConfig(node.data.config),
    enabled: get(node.data.enabled),
    x: node.position.x,
    y: node.position.y,
    width: node.width,
    height: node.height,
  };
}

function serializeAgentFlowNodeConfig(node_config: AgentConfig | null): Record<string, any> | null {
  if (node_config === null) {
    return null;
  }
  const config: Record<string, any> = {};
  for (const key of Object.keys(node_config)) {
    const t = node_config[key].type;
    const value = get(node_config[key].value);
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
    target: edge.target,
  };
}

export function newAgentFlowNode(agent_name: string, settings: AgentSettings): AgentFlowNode {
  if (agent_name === "$board") {
    return newAgentFlowBoardNode(settings);
  }

  const id = newNodeId(agent_name);
  const default_config = settings[agent_name].default_config ?? {};
  const node_data = {
    id,
    name: agent_name,
    enabled: true,
    config: { ...default_config },
    x: Math.random() * 1000,
    y: Math.random() * 1000,
  };
  return deserializeAgentFlowNode(node_data, settings);
}

export function newAgentFlowBoardNode(settings: AgentSettings): AgentFlowNode {
  const id = newNodeId("$board");
  const node_data = {
    id,
    name: "$board",
    enabled: true,
    config: {
      board_name: "",
      persistent: false,
    },
    x: Math.random() * 1000,
    y: Math.random() * 1000,
  };
  return deserializeAgentFlowNode(node_data, settings);
}

function newNodeId(prefix: string) {
  return `${prefix}_${nanoid()}`;
}
