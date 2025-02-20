<script lang="ts">
  import { open } from "@tauri-apps/plugin-shell";

  import { Button, Input, Label, Toggle } from "flowbite-svelte";

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
    <Card title={agent.name} tooltip={agent.path}>
      <form class="grid grid-cols-6 gap-6">
        <Toggle
          bind:checked={agents[agent.name].enabled as boolean}
          onchange={() => toggle_agent(agent.name)}
          class="col-span-6"
        ></Toggle>
        {#if agents[agent.name].config}
          {#each Object.keys(agents[agent.name].config) as key}
            <Label class="col-span-6 space-y-2">
              <span>{key}</span>
              {#if typeof agents[agent.name].config[key] === "boolean"}
                <Toggle bind:checked={agents[agent.name].config[key]} disabled />
              {:else}
                <Input type="text" bind:value={agents[agent.name].config[key]} disabled />
              {/if}
            </Label>
          {/each}
          <Button class="w-fit" outline disabled>Save</Button>
        {/if}
      </form>
    </Card>
  {/each}
</main>
