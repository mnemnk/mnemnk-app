<script lang="ts">
  import { run } from "svelte/legacy";

  interface Props {
    year?: any;
    daily_counts?: any;
    onDateChange?: (date: string) => void;
  }

  let { year = new Date().getFullYear(), daily_counts = [], onDateChange }: Props = $props();

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

  let grid = $state();
  run(() => {
    grid = Array(53)
      .fill()
      .map(() => Array(7).fill(null));
    if (daily_counts) {
      daily_counts.forEach((count) => {
        const { weekIndex, dayIndex, date } = getPosition(count[0]);
        if (weekIndex < 53) {
          grid[weekIndex][dayIndex] = { date, count: count[1] };
        }
      });
    }
  });

  let monthLabels = $derived(
    Array(53)
      .fill(null)
      .map((_, index) => getMonthLabel(index, year)),
  );
</script>

<div class="event-calendar">
  <div class="year-label">
    {year}
  </div>
  <div class="month-labels">
    <div class="weekday-spacer"></div>
    {#each monthLabels as month}
      <div class="month-label">
        {#if month}
          {month}
        {/if}
      </div>
    {/each}
  </div>

  <div class="graph-container">
    <div class="weekday-labels">
      {#each weekdays as day}
        <div class="weekday-label">
          {day}
        </div>
      {/each}
    </div>

    <div class="grid">
      {#each grid as week, weekIndex}
        <div class="week">
          {#each week as day, dayIndex}
            <div
              class="day"
              style="background-color: {day ? getColor(day.count) : colors[0]}"
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

<style>
  .event-calendar {
    padding: 1rem;
  }

  .month-labels {
    display: flex;
    gap: 2px;
    margin-bottom: 8px;
  }

  .weekday-spacer {
    width: 30px;
  }

  .month-label {
    width: 10px;
    font-size: 12px;
    color: #666;
  }

  .graph-container {
    display: flex;
    gap: 4px;
  }

  .weekday-labels {
    display: flex;
    flex-direction: column;
    gap: 2px;
    font-size: 12px;
    color: #666;
  }

  .weekday-label {
    height: 10px;
    line-height: 10px;
    text-align: right;
    padding-right: 4px;
    width: 26px;
  }

  .grid {
    display: flex;
    gap: 2px;
  }

  .week {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .day {
    width: 10px;
    height: 10px;
    border-radius: 2px;
    cursor: pointer;
    transition: transform 0.1s ease;
  }

  .day:hover {
    transform: scale(1.1);
  }
</style>
