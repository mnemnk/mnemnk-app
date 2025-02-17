<script lang="ts">
  import { AppBar } from "@skeletonlabs/skeleton";
  import IconsMdiSettings from "~icons/mdi/settings";

  import { goto } from "$app/navigation";

  import EventCalendar from "@/components/EventCalendar.svelte";
  import SearchBox from "@/components/SearchBox.svelte";
  import { dateString } from "@/lib/utils";

  let { data } = $props();
  let year = $derived(data.year);
  let daily_counts = $derived(data.daily_counts);

  function onsearch(query: string) {
    if (!query) {
      return;
    }
    goto(`/search?q=${query}`);
  }

  function onDateChange(date: string) {
    let d = new Date(date);
    // date string YYYYMMDD
    // let date_str = `${d.getFullYear()}${(d.getMonth() + 1).toString().padStart(2, '0')}${d.getDate().toString().padStart(2, '0')}`;
    goto(`/daily?d=${dateString(d)}`);
  }
</script>

<div>
  <AppBar class="!bg-transparent">
    {#snippet lead()}
      &nbsp;
    {/snippet}
    {#snippet trail()}
      <a href="/settings"><IconsMdiSettings /></a>
    {/snippet}
  </AppBar>
  <main class="container mx-auto p-8 space-y-8">
    <SearchBox {onsearch} />
    <EventCalendar {year} {daily_counts} {onDateChange} />
  </main>
</div>
