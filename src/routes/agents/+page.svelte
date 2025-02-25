<script lang="ts">
  import { open } from "@tauri-apps/plugin-shell";

  import { Button, Input, Label, NumberInput, Textarea, Toggle } from "flowbite-svelte";

  import Card from "@/components/Card.svelte";
  import {
    get_settings_filepath,
    save_agent_config,
    set_agent_enabled,
    start_agent,
    stop_agent,
  } from "@/lib/utils";

  let { data } = $props();

  let catalog = data.catalog;
  let settings = data.settings;
  let properties = $state(data.properties);

  async function open_settings_file() {
    let path = await get_settings_filepath();
    await open(path);
  }

  async function toggle_agent(agent_name: string) {
    if (settings[agent_name].enabled) {
      settings[agent_name].enabled = false;
      await set_agent_enabled(agent_name, false);
      await stop_agent(agent_name);
    } else {
      settings[agent_name].enabled = true;
      await start_agent(agent_name);
      await set_agent_enabled(agent_name, true);
    }
  }

  async function save(agent_name: string) {
    if (properties.has(agent_name)) {
      const props = properties.get(agent_name);
      let config = { ...settings[agent_name].config };
      if (props && config) {
        for (const [key, value] of props.entries()) {
          if (value.type === "boolean") {
            config = {
              ...config,
              [key]: value.value === "true",
            };
          } else if (value.type === "integer") {
            config = {
              ...config,
              [key]: parseInt(value.value),
            };
          } else if (value.type === "number") {
            config = {
              ...config,
              [key]: parseFloat(value.value),
            };
          } else if (value.type === "string") {
            console.log(config);
            config = {
              ...config,
              [key]: value.value,
            };
            console.log(config);
          } else if (value.type === "string[]") {
            config = {
              ...config,
              [key]: value.value.split("\n"),
            };
          } else if (value.type === "string?") {
            config = {
              ...config,
              [key]: value.value === "" ? null : value.value,
            };
          }
        }
        settings[agent_name].config = config;
        save_agent_config(agent_name, config);
      }
    }
  }
</script>

<main class="container mx-auto p-8 space-y-8 mt-20">
  <div class="flex">
    <h1 class="text-xl font-semibold sm:text-2xl">Agents</h1>
    <Button onclick={open_settings_file} class="ml-auto">config.yml</Button>
  </div>
  {#each catalog as agent}
    {@const name = agent.name as string}
    {@const props = properties.get(name)}
    {#if props}
      <Card
        title={settings[name].schema?.["title"] || name}
        subtitle={settings[name].schema?.["description"] || ""}
        tooltip={agent.path}
      >
        <form class="grid grid-cols-6 gap-6">
          <Toggle
            checked={settings[name].enabled as boolean}
            onchange={() => toggle_agent(name)}
            class="col-span-6"
          ></Toggle>
          {#each props.keys() as key}
            {@const prop = props.get(key)}
            {#if prop}
              <Label class="col-span-6 space-y-2">
                <h3>{prop.title || key}</h3>
                <p class="text-xs text-gray-500">{prop["description"]}</p>
                {#if prop.type === "boolean"}
                  <Toggle
                    bind:checked={() => prop.value, (value) => props.set(key, { ...prop, value })}
                  />
                {:else if prop.type === "integer"}
                  <NumberInput
                    bind:value={() => prop.value, (value) => props.set(key, { ...prop, value })}
                  />
                {:else if prop.type === "number"}
                  <Input
                    type="text"
                    bind:value={() => prop.value, (value) => props.set(key, { ...prop, value })}
                  />
                {:else if prop.type === "string" || prop.type === "string?"}
                  <Input
                    type="text"
                    bind:value={() => prop.value, (value) => props.set(key, { ...prop, value })}
                  />
                {:else if prop.type === "string[]"}
                  <Textarea
                    bind:value={() => prop.value, (value) => props.set(key, { ...prop, value })}
                    rows={4}
                  />
                {:else}
                  <Input type="text" value={prop.value} disabled />
                {/if}
              </Label>
            {/if}
          {/each}
          <Button onclick={() => save(name)} class="w-fit" outline>Save</Button>
        </form>
      </Card>
    {:else}
      <Card title={name} tooltip={agent.path}>
        <form class="grid grid-cols-6 gap-6">
          <Toggle
            checked={settings[name].enabled as boolean}
            onchange={() => toggle_agent(name)}
            class="col-span-6"
          ></Toggle>
          {#if settings[name].config}
            {@const config = settings[name].config as Record<string, any>}
            {#each Object.keys(config) as key}
              <Label class="col-span-6 space-y-2">
                <span>{key}</span>
                {#if typeof config[key] === "boolean"}
                  <Toggle checked={config[key]} disabled />
                {:else}
                  <Input type="text" value={config[key]} disabled />
                {/if}
              </Label>
            {/each}
            <Button class="w-fit" outline disabled>Save</Button>
          {/if}
        </form>
      </Card>
    {/if}
  {/each}
</main>
