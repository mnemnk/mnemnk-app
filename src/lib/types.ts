// agent

export type AgentCatalogEntry = {
  name: string;
  path: string;
};

// events

export type MnemnkEvent = {
  id: string;
  kind: string;
  time: number;
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
