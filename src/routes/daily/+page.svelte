<script lang="ts">
  import HourlyEvents from "./HourlyEvents.svelte";

  const { data } = $props();
</script>

<main id="main" class="container p-0">
  <HourlyEvents date={data.date} events={data.events} />
</main>

<style>
  :global(body) {
    overflow-x: hidden;
  }
</style>
