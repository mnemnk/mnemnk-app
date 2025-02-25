<script lang="ts">
  import HourlyEvents from "./HourlyEvents.svelte";

  let { data } = $props();
  let date = data.date;
  let events = data.events;
</script>

<main id="main" class="container">
  <div class="p-0">
    <HourlyEvents {date} {events} />
  </div>
</main>

<style>
  :global(body) {
    overflow-x: hidden;
  }
</style>
