import { find_events_by_ymd, index_year } from "@/lib/utils";

export async function load({ params }) {
  let date = new Date();
  let year = date.getFullYear();
  // let events = await find_events_by_ymd(year, date.getMonth() + 1, date.getDate());
  let daily_counts = await index_year(year);
  return {
    date,
    year,
    // events,
    daily_counts,
  };
}
