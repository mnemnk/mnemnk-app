<script lang="ts">
  import Agent from "./Agent.svelte";
  import Core from "./Core.svelte";
  import Database from "./Database.svelte";

  const { data } = $props();

  const settings = $derived(data.coreSettings);
  const agentGlobalConfigs = $derived(data.agentGlobalConfigs);
  const agentDefs = $derived(data.agentDefs);
</script>

<main class="container mx-auto p-8 space-y-8 mt-20">
  <h1 class="text-xl font-semibold sm:text-2xl">Settings</h1>
  <Core {settings} />

  <Database />

  <h2 class="text-xl font-semibold sm:text-2xl">Agents</h2>
  {#each Object.entries(agentGlobalConfigs) as [agentName, agentConfig]}
    <Agent {agentName} {agentConfig} agentDef={agentDefs[agentName]} />
  {/each}
</main>
