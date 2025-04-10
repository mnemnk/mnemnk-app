<script lang="ts">
  import type { MnemnkEvent } from "@/lib/types";

  interface Props {
    events: MnemnkEvent[];
    day_start_hour?: number | null;
  }

  const { events, day_start_hour }: Props = $props();

  const hours: (string[] | null)[] = $derived.by(() => {
    const hours = new Array(24).fill(null);
    events.forEach((ev) => {
      const d = new Date(ev.time);
      const h = d.getHours();
      let i = h - (day_start_hour ?? 0);
      if (i < 0) {
        i += 24;
      }
      if (hours[i] === null) {
        hours[i] = [
          h.toString().padStart(2, "0"),
          `${d.getFullYear()}${(d.getMonth() + 1).toString().padStart(2, "0")}${d.getDate().toString().padStart(2, "0")}${h.toString().padStart(2, "0")}${d.getMinutes().toString().padStart(2, "0")}`,
        ];
      }
    });
    return hours;
  });
</script>

<div class="grid grid-rows-24 pt-10 p-4 bg-transparent/40">
  {#each hours as hour}
    <div class="text-[3dvh]/[3.5dvh] overflow-visible">
      {#if hour !== null}
        <a href="#t{hour[1]}" class="block">
          {hour[0]}
        </a>
      {:else}
        <span>&nbsp;</span>
      {/if}
    </div>
  {/each}
</div>
