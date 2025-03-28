import type { Writable } from "svelte/store";

import type { Edge, Node } from "@xyflow/svelte";

export type SAgentDefinitions = Record<string, SAgentDefinition>;

export type SAgentDefinition = {
  name: string;
  title: string | null;
  description: string | null;
  path: string;
  inputs: string[] | null;
  outputs: string[] | null;
  default_config: SAgentDefaultConfig | null;
};

export type SAgentDefaultConfig = Record<string, SAgentDefaultConfigEntry>;

export type SAgentDefaultConfigEntry = {
  value: any;
  type: string | null;
  title?: string | null;
  description?: string | null;
  scope?: string | null;
};

export type SAgentFlow = {
  nodes: SAgentFlowNode[];
  edges: SAgentFlowEdge[];
};

export type SAgentFlowNode = {
  id: string;
  name: string;
  config: Record<string, any> | null;
  enabled: boolean;
  x: number;
  y: number;
  width?: number;
  height?: number;
};

export type SAgentFlowEdge = {
  id: string;
  source: string;
  source_handle: string | null;
  target: string;
  target_handle: string | null;
};

export type AgentFlow = {
  nodes: AgentFlowNode[];
  edges: AgentFlowEdge[];
};

export type AgentFlowNode = Node & {
  data: AgentFlowNodeData;
};

export type AgentFlowNodeData = {
  name: string;
  enabled: Writable<boolean>;
  config: AgentFlowNodeConfig | null;
};

export type AgentFlowNodeConfig = Record<string, Writable<any>>;

export type AgentFlowEdge = Edge;

// events

export type MnemnkEvent = {
  id: string;
  kind: string;
  time: number;
  local_offset: number;
  local_ymd: number;
  value: any;
};

export type ScreenshotEvent = MnemnkEvent & {
  value: {
    image_id: string;
  };
};

export type DailyStats = {
  date: number;
  count: number;
};

// settings

export type CoreSettings = {
  autostart: boolean;
  mnemnk_dir: string;
  shortcut_keys: Record<string, string>;
  thumbnail_width: number | null;
  thumbnail_height: number | null;
  day_start_hour: number | null;
};

export type Settings = {
  core: CoreSettings;
  agents: Record<string, SAgentDefinition>;
  agent_flows: SAgentFlow[];
};

// emit

export type WriteBoardEmit = {
  kind: string;
  value: any;
};

export type BoardMessage = {
  kind: string;
  value: any;
  time: number;
};
