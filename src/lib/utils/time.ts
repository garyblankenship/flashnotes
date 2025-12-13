const SECOND = 1000;
const MINUTE = 60 * SECOND;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;
const WEEK = 7 * DAY;
const MONTH = 30 * DAY;
const YEAR = 365 * DAY;

export function formatRelativeTime(timestamp: number): string {
  const now = Date.now();
  const diff = now - timestamp * 1000; // Convert from Unix timestamp

  if (diff < MINUTE) {
    return 'just now';
  } else if (diff < HOUR) {
    const mins = Math.floor(diff / MINUTE);
    return `${mins}m`;
  } else if (diff < DAY) {
    const hours = Math.floor(diff / HOUR);
    return `${hours}h`;
  } else if (diff < WEEK) {
    const days = Math.floor(diff / DAY);
    return `${days}d`;
  } else if (diff < MONTH) {
    const weeks = Math.floor(diff / WEEK);
    return `${weeks}w`;
  } else if (diff < YEAR) {
    const months = Math.floor(diff / MONTH);
    return `${months}mo`;
  } else {
    const years = Math.floor(diff / YEAR);
    return `${years}y`;
  }
}
