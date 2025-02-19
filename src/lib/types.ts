export type CoreSettings = {
  autostart: boolean;
  data_dir: string;
  shortcut_key: string | null;
  thumbnail_width: number | null;
  thumbnail_height: number | null;
};

export type AgentSettings = {
  enabled: boolean;
  config: Record<string, any>;
};

export type Settings = {
  core: CoreSettings;
  agents: Record<string, AgentSettings>;
};
