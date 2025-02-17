<script lang="ts">
  import { goto } from "$app/navigation";

  import EventCalendar from "@/components/EventCalendar.svelte";
  // import SearchBox from "@/components/SearchBox.svelte";
  import { dateString } from "@/lib/utils";

  let { data } = $props();
  let year = $derived(data.year);
  let daily_counts = $derived(data.daily_counts);

  function onDateChange(date: string) {
    let d = new Date(date);
    // date string YYYYMMDD
    // let date_str = `${d.getFullYear()}${(d.getMonth() + 1).toString().padStart(2, '0')}${d.getDate().toString().padStart(2, '0')}`;
    goto(`/daily?d=${dateString(d)}`);
  }
</script>

<main class="container mx-auto p-8 space-y-8 mt-20">
  <div class="mx-auto">
    <EventCalendar {year} {daily_counts} {onDateChange} />
  </div>
</main>
