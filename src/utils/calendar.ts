export type CalendarDate = { year: number; month: number; day: number };
export function todayInTimezone(timezone: string, instant = new Date()): CalendarDate {
  const parts = new Intl.DateTimeFormat('en-CA', { timeZone: timezone, year: 'numeric', month: 'numeric', day: 'numeric' }).formatToParts(instant);
  const part = (type: string) => Number(parts.find((item) => item.type === type)?.value);
  return { year: part('year'), month: part('month') - 1, day: part('day') };
}
export function monthDays(year: number, month: number): CalendarDate[] {
  const start = new Date(Date.UTC(year, month, 1));
  const offset = start.getUTCDay();
  return Array.from({ length: 42 }, (_, index) => {
    const date = new Date(Date.UTC(year, month, index - offset + 1));
    return { year: date.getUTCFullYear(), month: date.getUTCMonth(), day: date.getUTCDate() };
  });
}
export function monthTitle(year: number, month: number) { return new Intl.DateTimeFormat('pt-BR', { month: 'long', year: 'numeric', timeZone: 'UTC' }).format(new Date(Date.UTC(year, month, 1))); }
export function dateKey(date: CalendarDate) { return `${date.year}-${String(date.month + 1).padStart(2, '0')}-${String(date.day).padStart(2, '0')}`; }
