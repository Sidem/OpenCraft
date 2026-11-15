// The words on the Home page's numbers (home.ts), kept pure so web/test/home-text.test.mjs can check them.

/** Game time played, in the largest two units that matter: "Under a minute", "42 min", "5 h 12 min", "31 h". */
export function playTimeText(seconds: number): string {
  const minutes = Math.floor(Math.max(0, seconds) / 60);
  if (minutes < 1) return 'Under a minute';
  if (minutes < 60) return `${minutes} min`;
  const hours = Math.floor(minutes / 60);
  const rest = minutes % 60;
  return hours >= 10 || rest === 0 ? `${hours} h` : `${hours} h ${rest} min`;
}

/** How far the current research is: "3 of 5" for a tech with units, "level 4" for an endless one. */
export function researchProgress(done: number, units: number, endless: boolean): string {
  return endless ? `level ${done + 1}` : `${Math.min(done, units)} of ${units}`;
}

/** "Nothing waiting behind it", "1 more waiting", "4 more waiting". */
export function waitingText(waiting: number): string {
  return waiting <= 0 ? '' : `${waiting} more waiting`;
}
