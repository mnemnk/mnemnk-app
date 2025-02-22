<script lang="ts">
  import type { MnemnkEvent } from "@/lib/types";

  interface Props {
    events: MnemnkEvent[];
  }

  let { events }: Props = $props();

  let hours: (string[] | null)[] = $derived.by(() => {
    let hours = new Array(24).fill(null);
    let last_h = -1;
    events.forEach((ev) => {
      let d = new Date(ev.time);
      let h = d.getHours();
      if (h > last_h) {
        hours[h] = [
          h.toString().padStart(2, "0"),
          `${d.getFullYear()}${(d.getMonth() + 1).toString().padStart(2, "0")}${d.getDate().toString().padStart(2, "0")}${h.toString().padStart(2, "0")}${d.getMinutes().toString().padStart(2, "0")}`,
        ];
        last_h = h;
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
