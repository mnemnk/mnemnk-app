import type { Writable } from "svelte/store";

import type { Node } from "@xyflow/svelte";

export type AgentCatalogEntry = {
  name: string;
  path: string;
};

export type AgentDefaultConfig = Record<string, any>;
export type AgentSchema = Record<string, any>;

export type AgentSettings = {
  // enabled: boolean | null;
  default_config: AgentDefaultConfig | null;
  schema: AgentSchema | null;
};

export type AgentFlowNode = {
  id: string;
  name: string;
  config: Record<string, any> | null;
  enabled: boolean;
  x: number;
  y: number;
  width?: number;
  height?: number;
};

export type AgentFlow = {
  nodes: AgentFlowNode[];
};

export type AgentConfig = {
  value: Writable<any>;
  type: string | null;
  title: string | null;
  description: string | null;
};

// agent name -> key -> schema
export type AgentProperties = Record<string, Record<string, AgentConfig>>;

export type SAgentNodeData = {
  name: string;
  enabled: Writable<boolean>;
  config: Record<string, AgentConfig> | null;
};

export type SAgentNode = Node & {
  data: SAgentNodeData;
};

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
  agents: Record<string, AgentSettings>;
  agent_flows: AgentFlow[];
};

// emit

export type WriteBoardEmit = {
  agent: string;
  kind: string;
  value: any;
};
