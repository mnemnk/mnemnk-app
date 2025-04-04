import type { Edge, Node } from "@xyflow/svelte";

export type SAgentDefinitions = Record<string, SAgentDefinition>;
export type SAgentGlobalConfigs = Record<string, SAgentConfigs>;

export type SAgentDefinition = {
  kind: string;
  name: string;
  title: string | null;
  description: string | null;
  category: string | null;
  path: string;
  inputs: string[] | null;
  outputs: string[] | null;
  default_config: SAgentDefaultConfig | null;
  global_config: SAgentGlobalConfig | null;
  display_config: SAgentDisplayConfig | null;
};

export type SAgentDefaultConfig = [string, SAgentConfigEntry][];
export type SAgentGlobalConfig = [string, SAgentConfigEntry][];

export type SAgentConfigEntry = {
  value: any;
  type: string | null;
  title?: string | null;
  description?: string | null;
};

export type SAgentDisplayConfig = [string, SAgentDisplayConfigEntry][];

export type SAgentDisplayConfigEntry = {
  type: string | null;
  title?: string | null;
  description?: string | null;
};

export type SAgentFlows = Record<string, SAgentFlow>;

export type SAgentFlow = {
  nodes: SAgentFlowNode[];
  edges: SAgentFlowEdge[];
  name: string;
};

export type SAgentConfigs = Record<string, SAgentConfig>;
export type SAgentConfig = Record<string, any>;

export type SAgentFlowNode = {
  id: string;
  name: string;
  config: SAgentConfig | null;
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
  name: string;
};

export type AgentFlowNode = Node & {
  data: AgentFlowNodeData;
};

export type AgentFlowNodeData = {
  name: string;
  enabled: boolean;
  config: AgentFlowNodeConfig | null;
  display: AgentFlowNodeDisplay | null;
};

export type AgentFlowNodeConfig = Record<string, any>;
export type AgentFlowNodeDisplay = Record<string, any>;

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

export type DisplayMessage = {
  agent_id: string;
  key: string;
  value: any;
};
