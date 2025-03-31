<script lang="ts">
  import Agent from "./Agent.svelte";
  import Core from "./Core.svelte";

  const { data } = $props();

  const settings = $derived(data.settings);
  const agentConfigs = $derived(data.agentConfigs);
  const agentDefs = $derived(data.agentDefs);
</script>

<main class="container mx-auto p-8 space-y-8 mt-20">
  <h1 class="text-xl font-semibold sm:text-2xl">Settings</h1>
  <Core {settings} />

  <!-- <Card title="Search">
    <Button onclick={reindex_text} class="w-fit m-2" outline>Reindex Text</Button>
  </Card> -->

  <h2 class="text-xl font-semibold sm:text-2xl">Agents</h2>
  {#each Object.entries(agentConfigs) as [agentName, agentConfig]}
    <Agent {agentName} {agentConfig} agentDef={agentDefs[agentName]} />
  {/each}
</main>
