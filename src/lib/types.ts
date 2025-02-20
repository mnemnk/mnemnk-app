// agent

export type AgentCatalogEntry = {
  name: string;
  path: string;
};

// settings

export type CoreSettings = {
  autostart: boolean;
  data_dir: string;
  shortcut_key: string | null;
  thumbnail_width: number | null;
  thumbnail_height: number | null;
};

export type AgentSettings = {
  enabled: boolean | null;
  config: Record<string, any> | null;
};

export type Settings = {
  core: CoreSettings;
  agents: Record<string, AgentSettings>;
};
