<script lang="ts">
  import type { DailyStats } from "$lib/types";

  interface Props {
    year: number;
    daily_stats: DailyStats[];
    onDateChange?: (date: string) => void;
  }

  let { year, daily_stats, onDateChange }: Props = $props();

  // TODO: use style?
  const colors = {
    0: "#101a10",
    1: "#216e39",
    2: "#30a14e",
    3: "#40c463",
    4: "#9be9a8",
  };

  const weekdays = ["", "Mon", "", "Wed", "", "Fri", ""];

  function getColor(count: number) {
    if (count === 0) return colors[0];
    if (count <= 3) return colors[1];
    if (count <= 6) return colors[2];
    if (count <= 9) return colors[3];
    return colors[4];
  }

  function getMonthLabel(weekIndex: number, year: number) {
    const date = new Date(year, 0, 1);
    date.setDate(date.getDate() + weekIndex * 7);

    const currentMonth = date.getMonth();
    const prevDate = new Date(date);
    prevDate.setDate(prevDate.getDate() - 7);

    if (currentMonth !== prevDate.getMonth()) {
      const months = [
        "Jan",
        "Feb",
        "Mar",
        "Apr",
        "May",
        "Jun",
        "Jul",
        "Aug",
        "Sep",
        "Oct",
        "Nov",
        "Dec",
      ];
      return months[currentMonth];
    }
    return null;
  }

  function getPosition(dateNum: number) {
    const year = Math.floor(dateNum / 10000);
    const month = Math.floor((dateNum % 10000) / 100);
    const day = dateNum % 100;
    const date = new Date(year, month - 1, day);
    const startOfYear = new Date(year, 0, 1);
    const startDayOfYear = startOfYear.getDay() * 25 * 60 * 60 * 1000;
    const weekIndex = Math.floor((date - startOfYear + startDayOfYear) / (7 * 24 * 60 * 60 * 1000));
    const dayIndex = date.getDay();
    return { weekIndex, dayIndex, date: `${year}/${month}/${day}` };
  }

  function handleDateChange(date: string) {
    if (date) {
      onDateChange?.(date);
    }
  }

  let grid = $derived.by(() => {
    let grid = Array(53)
      .fill(null)
      .map(() => Array(7).fill(null));
    if (daily_stats) {
      daily_stats.forEach((stats) => {
        const { weekIndex, dayIndex, date } = getPosition(stats.date);
        if (weekIndex < 53) {
          grid[weekIndex][dayIndex] = { date, count: stats.count };
        }
      });
    }
    return grid;
  });

  let monthLabels = $derived(
    Array(53)
      .fill(null)
      .map((_, index) => getMonthLabel(index, year)),
  );
</script>

<div>
  <h1 class="text-3xl text-bold text-center mb-8">{year}</h1>
  <div class="flex justify-center">
    <div class="w-[776px]">
      <div class="flex flex-row gap-[2px] pl-[34px] mb-1">
        {#each monthLabels as month}
          <div class="flex-none w-[12px] text-sm text-gray-200">
            {#if month}
              {month}
            {/if}
          </div>
        {/each}
      </div>

      <div class="flex flex-row gap-[4px]">
        <div class="flex flex-col gap-[2px]">
          {#each weekdays as day}
            <div class="text-sm/[10px] text-right text-gray-200 h-[12px] w-[30px] pr-[4px]">
              {day}
            </div>
          {/each}
        </div>

        <div class="flex flex-row gap-[2px]">
          {#each grid as week, weekIndex}
            <div class="flex flex-col gap-[2px]">
              {#each week as day, dayIndex}
                <div
                  class="w-[12px] h-[12px] rounded-[2px] cursor-pointer"
                  style:background-color={day ? getColor(day.count) : colors[0]}
                  title={day ? `${day.date}: ${day.count} events` : "No events"}
                  onclick={() => handleDateChange(day.date)}
                  role="button"
                  tabindex="0"
                  onkeydown={(e) => e.key === "Enter" && handleDateChange(day.date)}
                ></div>
              {/each}
            </div>
          {/each}
        </div>
      </div>
    </div>
  </div>
</div>
