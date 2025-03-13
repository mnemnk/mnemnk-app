// agent

export type AgentCatalogEntry = {
  name: string;
  path: string;
};

export type AgentSettings = {
  // enabled: boolean | null;
  default_config: Record<string, any> | null;
  schema: Record<string, any> | null;
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

export type AgentFlowNodeDataType = {
  name: string;
  enabled: boolean;
  config?: Record<string, any>;
  schema?: Record<string, any>;
  props?: Map<string, any>;
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
