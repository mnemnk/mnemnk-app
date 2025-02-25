import type { AgentCatalogEntry, AgentSettings } from "$lib/types";

import { get_agent_catalog, get_agent_settings } from "@/lib/utils";

export interface AgentProperty {
  value: any;
  type: string | null;
  title: string | null;
  description: string | null;
}

// agent name -> key -> schema
type Properties = Map<string, Map<string, AgentProperty>>;

function isOptionString(t: any): boolean {
  return Array.isArray(t) && t.length === 2 && t.includes("string") && t.includes("null");
}

function isArrayString(t: any, items_t: any): boolean {
  return t === "array" && items_t === "string";
}

export async function load() {
  let catalog: AgentCatalogEntry[] = await get_agent_catalog();
  catalog = catalog.sort((a, b) => a.name.localeCompare(b.name));

  let settings: Record<string, AgentSettings> = await get_agent_settings();
  let properties: Properties = new Map();

  for (let agent of catalog) {
    if (settings[agent.name]) {
      if (settings[agent.name].enabled === null) {
        settings[agent.name].enabled = false;
      }
      if (settings[agent.name].config && settings[agent.name].schema) {
        const c = settings[agent.name].config as Record<string, any>;
        const s = settings[agent.name].schema as Record<string, any>;
        properties.set(agent.name, new Map());
        if (s["properties"]) {
          const p = s["properties"] as Record<string, any>;
          for (let key in c) {
            let prop: AgentProperty = {
              value: null,
              type: null,
              title: null,
              description: null,
            };
            if (p[key]) {
              const t = p[key].type;
              if (t === "boolean") {
                prop.type = "boolean";
                prop.value = c[key];
              } else if (t === "integer") {
                prop.type = "integer";
                prop.value = c[key];
              } else if (t === "number") {
                prop.type = "number";
                prop.value = c[key].toString();
              } else if (t === "string") {
                prop.type = "string";
                prop.value = c[key];
              } else if (isOptionString(t)) {
                prop.type = "string?";
                prop.value = c[key] === null ? "" : c[key];
              } else if (isArrayString(t, p[key].items?.type)) {
                prop.type = "string[]";
                prop.value = (c[key] as string[]).join("\n");
              } else {
                prop.type = `unknown (${t})`;
                prop.value = c[key];
              }
            }
            if (p[key].title) {
              prop.title = p[key].title;
            }
            if (p[key].description) {
              prop.description = p[key].description;
            }
            properties.get(agent.name)?.set(key, prop);
          }
        }
      }
    } else {
      // Add new agent for enabling it
      settings[agent.name] = {
        enabled: false,
        config: null,
        schema: null,
      };
    }
  }

  return {
    catalog,
    settings,
    properties,
  };
}
