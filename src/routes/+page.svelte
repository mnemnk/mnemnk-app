<script lang="ts">
  import { goto } from "$app/navigation";

  import EventCalendar from "@/components/EventCalendar.svelte";
  import { dateString } from "@/lib/utils";

  let { data } = $props();
  let daily_stats = $derived(data.daily_stats);

  let year = new Date().getFullYear();

  function onDateChange(date: string) {
    let d = new Date(date);
    goto(`/daily/${dateString(d)}/`);
  }
</script>

<main class="container mx-auto p-8 space-y-8 mt-20">
  <div class="mx-auto">
    <EventCalendar {year} {daily_stats} {onDateChange} />
  </div>
</main>
