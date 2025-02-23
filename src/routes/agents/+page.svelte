<script lang="ts">
  import { open } from "@tauri-apps/plugin-shell";

  import { Button, Input, Label, NumberInput, Toggle } from "flowbite-svelte";

  import Card from "@/components/Card.svelte";
  import { get_settings_filepath, set_agent_enabled, start_agent, stop_agent } from "@/lib/utils";

  let { data } = $props();

  let agents = $state(data.settings.agents);
  let catalog = $state(data.catalog);

  async function open_settings_file() {
    let path = await get_settings_filepath();
    await open(path);
  }

  async function toggle_agent(agent_name: string) {
    if (agents[agent_name].enabled) {
      await start_agent(agent_name);
      await set_agent_enabled(agent_name, true);
    } else {
      await set_agent_enabled(agent_name, false);
      await stop_agent(agent_name);
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
    {#if agents[name].schema && agents[name].config}
      {@const schema = agents[name].schema as Record<string, any>}
      {@const properties = schema["properties"] as Record<string, any>}
      <Card
        title={schema["title"] || name}
        subtitle={schema["description"] || ""}
        tooltip={agent.path}
      >
        <form class="grid grid-cols-6 gap-6">
          <Toggle
            bind:checked={agents[name].enabled as boolean}
            onchange={() => toggle_agent(name)}
            class="col-span-6"
          ></Toggle>
          {#each Object.keys(properties) as key}
            {@const prop = properties[key]}
            <Label class="col-span-6 space-y-2">
              <h3>{prop["title"] || key}</h3>
              <p class="text-xs text-gray-500">{prop["description"]}</p>
              {#if prop["type"] === "boolean"}
                <Toggle bind:checked={agents[name].config[key]} />
              {:else if prop["type"] === "integer"}
                <NumberInput bind:value={agents[name].config[key]} />
              {:else if prop["type"] === "number"}
                <Input type="text" bind:value={agents[name].config[key]} />
              {:else}
                <Input type="text" bind:value={agents[name].config[key]} />
              {/if}
            </Label>
          {/each}
          <Button class="w-fit" outline disabled>Save</Button>
        </form>
      </Card>
    {:else}
      <Card title={name} tooltip={agent.path}>
        <form class="grid grid-cols-6 gap-6">
          <Toggle
            bind:checked={agents[name].enabled as boolean}
            onchange={() => toggle_agent(name)}
            class="col-span-6"
          ></Toggle>
          {#if agents[name].config}
            {@const config = agents[name].config as Record<string, any>}
            {#each Object.keys(config) as key}
              <Label class="col-span-6 space-y-2">
                <span>{key}</span>
                {#if typeof config[key] === "boolean"}
                  <Toggle bind:checked={config[key]} disabled />
                {:else}
                  <Input type="text" bind:value={config[key]} disabled />
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
