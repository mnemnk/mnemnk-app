<script lang="ts">
  import { goto } from "$app/navigation";

  import EventCalendar from "@/components/EventCalendar.svelte";
  import { dateString } from "@/lib/utils";

  const { data } = $props();

  let year = new Date().getFullYear();

  function onDateChange(date: string) {
    let d = new Date(date);
    goto(`/daily?d=${dateString(d)}`);
  }

  $effect(() => {
    if (!data.coreSettings.mnemnk_dir) {
      goto("/settings");
    }
  });
</script>

<main class="container mx-auto p-8 space-y-8 mt-20">
  <div class="mx-auto">
    <EventCalendar {year} dailyStats={data.dailyStats} {onDateChange} />
  </div>
</main>
