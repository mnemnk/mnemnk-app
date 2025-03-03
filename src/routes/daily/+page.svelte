<script lang="ts">
  import HourlyEvents from "./HourlyEvents.svelte";

  const { data } = $props();
  const date = data.date;
  const events = data.events;
  const shortcut_keys = data.settings.shortcut_keys;
  const day_start_hour = data.settings.day_start_hour;
</script>

<main id="main" class="container p-0">
  <HourlyEvents {date} {events} {shortcut_keys} {day_start_hour} />
</main>

<style>
  :global(body) {
    overflow-x: hidden;
  }
</style>
