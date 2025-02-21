<script lang="ts">
  import HourlyEvents from "./HourlyEvents.svelte";

  let { data } = $props();
  let date = $derived(data.date);
  let events = $derived(data.events);
</script>

<main id="main" class="container">
  <div class="w-[100vw] p-0">
    <HourlyEvents {date} {events} />
  </div>
</main>

<style>
  :global(body) {
    overflow-x: hidden;
  }
</style>
