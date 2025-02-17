<script lang="ts">
  import HourlyEvents from "@/components/HourlyEvents.svelte";

  let { data } = $props();
  let date = $derived(data.date);
  let events = $derived(data.events);

  let date_str = $derived(
    `${date.getFullYear()} / ${(date.getMonth() + 1).toString().padStart(2, "0")} / ${date.getDate().toString().padStart(2, "0")}`,
  );
</script>

<main id="main" class="container">
  <div class="w-[100vw] p-0">
    <HourlyEvents date={date_str} {events} />
  </div>
</main>

<style>
  :global(body) {
    overflow-x: hidden;
  }
</style>
