import { index_year } from "@/lib/utils";

export async function load() {
  let date = new Date();
  let year = date.getFullYear();
  let daily_counts = await index_year(year);
  return {
    date,
    year,
    daily_counts,
  };
}
