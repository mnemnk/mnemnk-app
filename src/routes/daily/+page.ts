import type { MnemnkEvent } from "@/lib/types.js";
import { find_events_by_ymd } from "@/lib/utils";

export async function load({ url }): Promise<{ date: Date; events: MnemnkEvent[] }> {
  const d = url.searchParams.get("d") || "";
  // check if params.date is a valid date string
  if (!/^\d{8}$/.test(d)) {
    return {
      date: new Date(0),
      events: [],
    };
  }
  let year = parseInt(d.substring(0, 4));
  let month = parseInt(d.substring(4, 6));
  let day = parseInt(d.substring(6, 8));
  let date = new Date(year, month - 1, day);
  let events = await find_events_by_ymd(year, month, day);
  return {
    date,
    events,
  };
}
