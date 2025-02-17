<script lang="ts">
  import { AppBar } from "@skeletonlabs/skeleton";
  import IconsMdiChevronLeft from "~icons/mdi/chevron-left";

  import HourlyEvents from "@/components/HourlyEvents.svelte";

  let { data } = $props();
  let date = $derived(data.date);
  let events = $derived(data.events);

  let date_str = $derived(
    `${date.getFullYear()} / ${(date.getMonth() + 1).toString().padStart(2, "0")} / ${date.getDate().toString().padStart(2, "0")}`,
  );
</script>

<div>
  <AppBar class="!bg-transparent/60">
    {#snippet lead()}
      <a href="/"><IconsMdiChevronLeft /></a>
    {/snippet}
    <div class="text-xl">{date_str}</div>
  </AppBar>
  <main id="main" class="container">
    <div class="w-screen bg-transparent/0 p-0">
      <HourlyEvents date={date_str} {events} />
    </div>
  </main>
</div>
