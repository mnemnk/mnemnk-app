import type { Writable } from "svelte/store";

import type { Edge, Node } from "@xyflow/svelte";

export type AgentCatalog = AgentCatalogEntry[];

export type AgentCatalogEntry = {
  name: string;
  path: string;
};

export type AgentDefaultConfig = Record<string, any>;
export type AgentSchema = Record<string, any>;

export type AgentSetting = {
  // enabled: boolean | null;
  default_config: AgentDefaultConfig | null;
  schema: AgentSchema | null;
};

export type AgentSettings = Record<string, AgentSetting>;

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
  target: string;
};

// agent name -> key -> schema
// export type AgentProperties = Record<string, Record<string, SAgentConfig>>;

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
  config: AgentConfig | null;
};

export type AgentConfig = Record<string, AgentConfigEntry>;

export type AgentConfigEntry = {
  value: Writable<any>;
  type: string | null;
  title: string | null;
  description: string | null;
};

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
  data_dir: string;
  shortcut_key: string | null;
  shortcut_keys: Record<string, string>;
  thumbnail_width: number | null;
  thumbnail_height: number | null;
  day_start_hour: number | null;
};

export type Settings = {
  core: CoreSettings;
  agents: Record<string, AgentSetting>;
  agent_flows: SAgentFlow[];
};

// emit

export type WriteBoardEmit = {
  agent: string;
  kind: string;
  value: any;
};
