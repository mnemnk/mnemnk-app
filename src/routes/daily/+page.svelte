<script lang="ts">
  import HourlyEvents from "./HourlyEvents.svelte";

  const { data } = $props();
  const date = data.date;
  const events = data.events;
  const shortcut_keys = data.settings.shortcut_keys;
</script>

<main id="main" class="container p-0">
  <HourlyEvents {date} {events} {shortcut_keys} />
</main>

<style>
  :global(body) {
    overflow-x: hidden;
  }
</style>
